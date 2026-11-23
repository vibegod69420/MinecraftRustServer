// === FILE: parallel_map_transform.rs ===
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

const DEFAULT_TASKS_PER_THREAD: i32 = 16;

pub trait Executor: Send + Sync {
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>);
}

pub struct CompletableFuture<T> {
    rx: mpsc::Receiver<T>,
}

impl<T: Send + 'static> CompletableFuture<T> {
    pub fn completed_future(value: T) -> Self {
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(value);
        Self { rx }
    }

    pub fn supply_async<F, E>(f: F, executor: &E) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
        E: Executor + ?Sized,
    {
        let (tx, rx) = mpsc::channel();
        executor.execute(Box::new(move || {
            let _ = tx.send(f());
        }));
        Self { rx }
    }

    pub fn then_apply<U, F>(self, f: F) -> CompletableFuture<U>
    where
        U: Send + 'static,
        F: FnOnce(T) -> U + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let val = self.rx.recv().expect("CompletableFuture failed");
            let _ = tx.send(f(val));
        });
        CompletableFuture { rx }
    }

    pub fn join(self) -> T {
        self.rx.recv().expect("CompletableFuture failed")
    }
}

impl CompletableFuture<()> {
    pub fn run_async<F, E>(f: F, executor: &E) -> Self
    where
        F: FnOnce() + Send + 'static,
        E: Executor + ?Sized,
    {
        Self::supply_async(f, executor)
    }

    pub fn all_of(futures: Vec<CompletableFuture<()>>) -> CompletableFuture<()> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for f in futures {
                let _ = f.rx.recv();
            }
            let _ = tx.send(());
        });
        CompletableFuture { rx }
    }
}

fn max_allowed_executor_threads() -> i32 {
    thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(1)
        .max(1)
}

fn positive_ceil_div(x: i32, y: i32) -> i32 {
    let neg_x = x.wrapping_neg();
    neg_x.div_euclid(y).wrapping_neg()
}

enum SlotValue<U, V> {
    Empty,
    Input(U),
    Output(Option<V>),
}

struct Slot<K, U, V> {
    key: Option<K>,
    value: SlotValue<U, V>,
}

impl<K, U, V> Slot<K, U, V> {
    // JAVA: net.minecraft.util.thread.ParallelMapTransform$Container#key
    fn key(&self) -> &K {
        self.key.as_ref().expect("Container.key: missing key")
    }

    // JAVA: net.minecraft.util.thread.ParallelMapTransform$Container#input
    fn input(&mut self) -> U {
        match std::mem::replace(&mut self.value, SlotValue::Empty) {
            SlotValue::Input(u) => u,
            _ => panic!("Container.input: expected input"),
        }
    }

    // JAVA: net.minecraft.util.thread.ParallelMapTransform$Container#output
    fn output(&mut self) -> Option<V> {
        match std::mem::replace(&mut self.value, SlotValue::Empty) {
            SlotValue::Output(v) => v,
            _ => panic!("Container.output: expected output"),
        }
    }
}

struct Container<K, U, V> {
    operation: Arc<dyn Fn(&K, U) -> Option<V> + Send + Sync>,
    slots: Vec<Mutex<Slot<K, U, V>>>,
}

impl<K, U, V> Container<K, U, V> {
    // JAVA: net.minecraft.util.thread.ParallelMapTransform$Container#<init>
    fn new(operation: Arc<dyn Fn(&K, U) -> Option<V> + Send + Sync>, size: i32) -> Self {
        let size_usize = size as usize;
        let mut slots = Vec::with_capacity(size_usize);
        for _ in 0..size_usize {
            slots.push(Mutex::new(Slot {
                key: None,
                value: SlotValue::Empty,
            }));
        }
        Self { operation, slots }
    }

    // JAVA: net.minecraft.util.thread.ParallelMapTransform$Container#put
    fn put(&self, index: i32, key: K, input: U) {
        let mut slot = self.slots[index as usize].lock().unwrap();
        slot.key = Some(key);
        slot.value = SlotValue::Input(input);
    }

    // JAVA: net.minecraft.util.thread.ParallelMapTransform$Container#applyOperation
    fn apply_operation(&self, index: i32) {
        let mut slot = self.slots[index as usize].lock().unwrap();
        let input = slot.input();
        let key = slot.key();
        let output = (self.operation)(key, input);
        slot.value = SlotValue::Output(output);
    }

    // JAVA: net.minecraft.util.thread.ParallelMapTransform$Container#copyOut
    fn copy_out(&self, index: i32, out: &mut HashMap<K, V>)
    where
        K: Eq + Hash,
    {
        let mut slot = self.slots[index as usize].lock().unwrap();
        let output = slot.output();
        if let Some(v) = output {
            let k = slot.key.take().expect("Container.copyOut: missing key");
            out.insert(k, v);
        }
    }

    // JAVA: net.minecraft.util.thread.ParallelMapTransform$Container#size
    fn size(&self) -> i32 {
        self.slots.len() as i32
    }
}

enum SplitterKind<K, V> {
    Single,
    Batched {
        result: Arc<Mutex<HashMap<K, V>>>,
        batch_size: i32,
        first_undersized_batch_index: i32,
    },
}

struct Splitter<K, U, V> {
    last_scheduled_index: i32,
    current_index: i32,
    tasks: Vec<CompletableFuture<()>>,
    batch_index: i32,
    container: Arc<Container<K, U, V>>,
    kind: SplitterKind<K, V>,
}

impl<K, U, V> Splitter<K, U, V> {
    // JAVA: net.minecraft.util.thread.ParallelMapTransform$SplitterBase#pendingBatchSize
    fn pending_batch_size(&self) -> i32 {
        self.current_index.wrapping_sub(self.last_scheduled_index)
    }
}

impl<K, U, V> Splitter<K, U, V>
where
    K: Eq + Hash + Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
{
    // JAVA: net.minecraft.util.thread.ParallelMapTransform$SplitterBase#scheduleTasks
    fn schedule_tasks<E: Executor + ?Sized>(
        &mut self,
        input: HashMap<K, U>,
        executor: &E,
    ) -> CompletableFuture<HashMap<K, V>> {
        for (k, u) in input {
            let idx = self.current_index;
            self.container.put(idx, k, u);
            self.current_index = self.current_index.wrapping_add(1);
            if self.pending_batch_size() == batch_size_dispatch(&self.kind, self.batch_index) {
                let task = schedule_batch_dispatch(
                    &self.kind,
                    &self.container,
                    self.last_scheduled_index,
                    self.current_index,
                    executor,
                );
                self.tasks.push(task);
                self.batch_index = self.batch_index.wrapping_add(1);
                self.last_scheduled_index = self.current_index;
            }
        }
        debug_assert_eq!(self.current_index, self.container.size());
        debug_assert_eq!(self.last_scheduled_index, self.current_index);
        debug_assert_eq!(self.batch_index as usize, self.tasks.len());
        let all = CompletableFuture::all_of(std::mem::take(&mut self.tasks));
        schedule_final_operation_dispatch(&self.kind, all, &self.container)
    }
}

fn batch_size_dispatch<K, V>(kind: &SplitterKind<K, V>, batch_index: i32) -> i32 {
    match kind {
        SplitterKind::Single => single_batch_size(batch_index),
        SplitterKind::Batched {
            batch_size,
            first_undersized_batch_index,
            ..
        } => batched_batch_size(batch_index, *batch_size, *first_undersized_batch_index),
    }
}

fn schedule_batch_dispatch<K, U, V, E: Executor + ?Sized>(
    kind: &SplitterKind<K, V>,
    container: &Arc<Container<K, U, V>>,
    start: i32,
    end: i32,
    executor: &E,
) -> CompletableFuture<()>
where
    K: Eq + Hash + Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
{
    match kind {
        SplitterKind::Single => single_schedule_batch(container, start, end, executor),
        SplitterKind::Batched {
            result, batch_size, ..
        } => batched_schedule_batch(result, *batch_size, container, start, end, executor),
    }
}

fn schedule_final_operation_dispatch<K, U, V>(
    kind: &SplitterKind<K, V>,
    all: CompletableFuture<()>,
    container: &Arc<Container<K, U, V>>,
) -> CompletableFuture<HashMap<K, V>>
where
    K: Eq + Hash + Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
{
    match kind {
        SplitterKind::Single => single_schedule_final_operation(all, container),
        SplitterKind::Batched { result, .. } => batched_schedule_final_operation(all, result),
    }
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$SingleTaskSplitter#batchSize
fn single_batch_size(_batch_index: i32) -> i32 {
    1
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$SingleTaskSplitter#scheduleBatch
fn single_schedule_batch<K, U, V, E: Executor + ?Sized>(
    container: &Arc<Container<K, U, V>>,
    start: i32,
    end: i32,
    executor: &E,
) -> CompletableFuture<()>
where
    K: Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
{
    debug_assert_eq!(start.wrapping_add(1), end);
    let container = Arc::clone(container);
    CompletableFuture::run_async(
        move || {
            container.apply_operation(start);
        },
        executor,
    )
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$SingleTaskSplitter#scheduleFinalOperation
fn single_schedule_final_operation<K, U, V>(
    all: CompletableFuture<()>,
    container: &Arc<Container<K, U, V>>,
) -> CompletableFuture<HashMap<K, V>>
where
    K: Eq + Hash + Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
{
    let container = Arc::clone(container);
    all.then_apply(move |_| {
        let mut map = HashMap::with_capacity(container.size() as usize);
        let mut i: i32 = 0;
        while i < container.size() {
            container.copy_out(i, &mut map);
            i = i.wrapping_add(1);
        }
        map
    })
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$BatchedTaskSplitter#batchSize
fn batched_batch_size(batch_index: i32, batch_size: i32, first_undersized_batch_index: i32) -> i32 {
    if batch_index < first_undersized_batch_index {
        batch_size
    } else {
        batch_size.wrapping_sub(1)
    }
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$BatchedTaskSplitter#createTask
fn batched_create_task<K, U, V>(
    result: Arc<Mutex<HashMap<K, V>>>,
    start: i32,
    end: i32,
    container: Arc<Container<K, U, V>>,
) -> impl FnOnce() + Send + 'static
where
    K: Eq + Hash + Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
{
    move || {
        let mut i = start;
        while i < end {
            container.apply_operation(i);
            i = i.wrapping_add(1);
        }
        let mut map = result.lock().unwrap();
        let mut i = start;
        while i < end {
            container.copy_out(i, &mut map);
            i = i.wrapping_add(1);
        }
    }
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$BatchedTaskSplitter#scheduleBatch
fn batched_schedule_batch<K, U, V, E: Executor + ?Sized>(
    result: &Arc<Mutex<HashMap<K, V>>>,
    batch_size: i32,
    container: &Arc<Container<K, U, V>>,
    start: i32,
    end: i32,
    executor: &E,
) -> CompletableFuture<()>
where
    K: Eq + Hash + Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
{
    let count = end.wrapping_sub(start);
    debug_assert!(count == batch_size || count == batch_size.wrapping_sub(1));
    let task = batched_create_task(Arc::clone(result), start, end, Arc::clone(container));
    CompletableFuture::run_async(task, executor)
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$BatchedTaskSplitter#scheduleFinalOperation
fn batched_schedule_final_operation<K, V>(
    all: CompletableFuture<()>,
    result: &Arc<Mutex<HashMap<K, V>>>,
) -> CompletableFuture<HashMap<K, V>>
where
    K: Eq + Hash + Send + 'static,
    V: Send + 'static,
{
    let result = Arc::clone(result);
    all.then_apply(move |_| {
        let mut guard = result.lock().unwrap();
        std::mem::take(&mut *guard)
    })
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$SplitterBase#<init>
fn splitter_base_new<K, U, V>(
    operation: Arc<dyn Fn(&K, U) -> Option<V> + Send + Sync>,
    size: i32,
    num_tasks: i32,
    kind: SplitterKind<K, V>,
) -> Splitter<K, U, V> {
    Splitter {
        last_scheduled_index: 0,
        current_index: 0,
        tasks: Vec::with_capacity(num_tasks as usize),
        batch_index: 0,
        container: Arc::new(Container::new(operation, size)),
        kind,
    }
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$SingleTaskSplitter#<init>
fn single_task_splitter_new<K, U, V>(
    operation: Arc<dyn Fn(&K, U) -> Option<V> + Send + Sync>,
    size: i32,
) -> Splitter<K, U, V> {
    splitter_base_new(operation, size, size, SplitterKind::Single)
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform$BatchedTaskSplitter#<init>
fn batched_task_splitter_new<K, U, V>(
    operation: Arc<dyn Fn(&K, U) -> Option<V> + Send + Sync>,
    size: i32,
    max_tasks: i32,
) -> Splitter<K, U, V>
where
    K: Eq + Hash + Send + 'static,
    V: Send + 'static,
{
    let batch_size = positive_ceil_div(size, max_tasks);
    let total_slots = batch_size.wrapping_mul(max_tasks);
    let excess = total_slots.wrapping_sub(size);
    let first_undersized_batch_index = max_tasks.wrapping_sub(excess);
    debug_assert!(first_undersized_batch_index > 0 && first_undersized_batch_index <= max_tasks);
    let result = Arc::new(Mutex::new(HashMap::with_capacity(size as usize)));
    splitter_base_new(
        operation,
        size,
        max_tasks,
        SplitterKind::Batched {
            result,
            batch_size,
            first_undersized_batch_index,
        },
    )
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform#schedule
pub fn schedule<K, U, V, F, E>(
    input: HashMap<K, U>,
    operation: F,
    tasks_per_thread: i32,
    executor: &E,
) -> CompletableFuture<HashMap<K, V>>
where
    K: Eq + Hash + Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
    F: Fn(&K, U) -> Option<V> + Send + Sync + 'static,
    E: Executor + ?Sized,
{
    let size = input.len() as i32;
    if size == 0 {
        return CompletableFuture::completed_future(HashMap::new());
    }
    let op: Arc<dyn Fn(&K, U) -> Option<V> + Send + Sync> = Arc::new(operation);
    if size == 1 {
        let (k, u) = input.into_iter().next().unwrap();
        let op = Arc::clone(&op);
        return CompletableFuture::supply_async(
            move || {
                let v = op(&k, u);
                if let Some(v) = v {
                    let mut m = HashMap::with_capacity(1);
                    m.insert(k, v);
                    m
                } else {
                    HashMap::new()
                }
            },
            executor,
        );
    }
    let mut splitter = if size <= tasks_per_thread {
        single_task_splitter_new(Arc::clone(&op), size)
    } else {
        batched_task_splitter_new(Arc::clone(&op), size, tasks_per_thread)
    };
    splitter.schedule_tasks(input, executor)
}

// JAVA: net.minecraft.util.thread.ParallelMapTransform#schedule
pub fn schedule_default<K, U, V, F, E>(
    input: HashMap<K, U>,
    operation: F,
    executor: &E,
) -> CompletableFuture<HashMap<K, V>>
where
    K: Eq + Hash + Send + 'static,
    U: Send + 'static,
    V: Send + 'static,
    F: Fn(&K, U) -> Option<V> + Send + Sync + 'static,
    E: Executor + ?Sized,
{
    let tasks = max_allowed_executor_threads().wrapping_mul(DEFAULT_TASKS_PER_THREAD);
    schedule(input, operation, tasks, executor)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ThreadExecutor;
    impl Executor for ThreadExecutor {
        fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
            std::thread::spawn(task);
        }
    }

    #[test]
    fn empty_map_returns_empty() {
        let ex = ThreadExecutor;
        let input: HashMap<i32, i32> = HashMap::new();
        let out = schedule(input, |_k, v| Some(v), 16, &ex).join();
        assert!(out.is_empty());
    }

    #[test]
    fn single_element_some_and_none() {
        let ex = ThreadExecutor;
        let mut input = HashMap::new();
        input.insert(42i32, 7i32);
        let out = schedule(input, |_k, v| Some(v.wrapping_mul(2)), 16, &ex).join();
        assert_eq!(out.get(&42), Some(&14));

        let mut input2 = HashMap::new();
        input2.insert(1i32, 2i32);
        let out2 = schedule(input2, |_k, _v| None::<i32>, 16, &ex).join();
        assert!(out2.is_empty());
    }

    #[test]
    fn multiple_elements_batched() {
        let ex = ThreadExecutor;
        let mut input = HashMap::new();
        for i in 0..100i32 {
            input.insert(i, i.wrapping_mul(3));
        }
        let out = schedule(input, |&k, v| Some(k.wrapping_add(v)), 8, &ex).join();
        assert_eq!(out.len(), 100);
        for i in 0..100i32 {
            let expected = i.wrapping_add(i.wrapping_mul(3));
            assert_eq!(out.get(&i), Some(&expected));
        }
    }

    #[test]
    fn negative_and_overflow_values() {
        let ex = ThreadExecutor;
        let mut input = HashMap::new();
        input.insert(i32::MIN, -1i32);
        input.insert(-1i32, i32::MAX);
        input.insert(0i32, 0i32);
        input.insert(i32::MAX, i32::MIN);
        let out = schedule(input, |_k, v| Some(v.wrapping_add(1)), 2, &ex).join();
        assert_eq!(out.get(&i32::MIN), Some(&0i32));
        assert_eq!(out.get(&-1i32), Some(&i32::MIN));
        assert_eq!(out.get(&0i32), Some(&1i32));
        assert_eq!(out.get(&i32::MAX), Some(&i32::MIN.wrapping_add(1)));
    }

    #[test]
    fn float_bits_preserved_and_default_schedule() {
        let ex = ThreadExecutor;
        let mut input = HashMap::new();
        input.insert(0u32, f32::NAN.to_bits());
        input.insert(1u32, f32::INFINITY.to_bits());
        input.insert(2u32, f32::NEG_INFINITY.to_bits());
        input.insert(3u32, (-0.0f32).to_bits());
        let out = schedule_default(input, |_k, v| Some(v), &ex).join();
        assert_eq!(out.get(&0u32), Some(&f32::NAN.to_bits()));
        assert_eq!(out.get(&1u32), Some(&f32::INFINITY.to_bits()));
        assert_eq!(out.get(&2u32), Some(&f32::NEG_INFINITY.to_bits()));
        assert_eq!(out.get(&3u32), Some(&(-0.0f32).to_bits()));
    }
}
