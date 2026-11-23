use std::collections::VecDeque;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub const BLOCK_TIME_NANOS: u64 = 100_000;

#[derive(Debug)]
pub struct ReportedException {
    pub cause: Option<Box<dyn std::any::Any + Send + 'static>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricCategory {
    EventLoops,
}

pub struct MetricSampler {
    pub name: String,
    pub category: MetricCategory,
    pub sampler: Box<dyn Fn() -> f64 + Send + Sync>,
}

impl MetricSampler {
    pub fn create(
        name: String,
        category: MetricCategory,
        sampler: Box<dyn Fn() -> f64 + Send + Sync>,
    ) -> Self {
        Self {
            name,
            category,
            sampler,
        }
    }
}

pub struct CompletableFuture<T> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T: Send + 'static> CompletableFuture<T> {
    pub fn completed(value: T) -> Self {
        Self {
            inner: Arc::new((Mutex::new(Some(value)), Condvar::new())),
        }
    }

    pub fn join(&self) -> T
    where
        T: Clone,
    {
        let (lock, cv) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        while guard.is_none() {
            guard = cv.wait(guard).unwrap();
        }
        guard.as_ref().unwrap().clone()
    }
}

fn complete_future<T: Send + 'static>(future: &CompletableFuture<T>, value: T) {
    let (lock, cv) = &*future.inner;
    *lock.lock().unwrap() = Some(value);
    cv.notify_all();
}

pub trait RunnableTask: Send {
    fn run(&mut self);
}

pub struct BoxedRunnable(pub Option<Box<dyn FnOnce() + Send + 'static>>);

impl RunnableTask for BoxedRunnable {
    fn run(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

pub struct BlockableEventLoopState<R> {
    pub name: String,
    pub pending_runnables: Arc<Mutex<VecDeque<R>>>,
    pub blocking_count: Arc<AtomicI32>,
}

impl<R> BlockableEventLoopState<R> {
    // JAVA: net.minecraft.util.thread.BlockableEventLoop#<init>
    pub fn new(name: String) -> Self {
        Self {
            name,
            pending_runnables: Arc::new(Mutex::new(VecDeque::new())),
            blocking_count: Arc::new(AtomicI32::new(0)),
        }
    }
}

pub trait BlockableEventLoop<R: RunnableTask + 'static>: Send + Sync {
    fn state(&self) -> &BlockableEventLoopState<R>;

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#shouldRun
    fn should_run(&self, task: &R) -> bool;

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#getRunningThread
    fn get_running_thread(&self) -> thread::Thread;

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#wrapRunnable
    fn wrap_runnable(&self, runnable: Box<dyn FnOnce() + Send + 'static>) -> R;

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#isSameThread
    fn is_same_thread(&self) -> bool {
        thread::current().id() == self.get_running_thread().id()
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#scheduleExecutables
    fn schedule_executables(&self) -> bool {
        !self.is_same_thread()
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#getPendingTasksCount
    fn get_pending_tasks_count(&self) -> i32 {
        self.state()
            .pending_runnables
            .lock()
            .unwrap()
            .len() as i32
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#name
    fn name(&self) -> &str {
        &self.state().name
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#submit
    fn submit_supplier<V: Send + Clone + 'static>(
        &self,
        supplier: Box<dyn FnOnce() -> V + Send + 'static>,
    ) -> CompletableFuture<V> {
        if self.schedule_executables() {
            let future = CompletableFuture::<V>::completed_placeholder();
            let future_clone = CompletableFuture {
                inner: future.inner.clone(),
            };
            let task = Box::new(move || {
                let v = supplier();
                complete_future(&future_clone, v);
            });
            let r = self.wrap_runnable(task);
            self.schedule(r);
            future
        } else {
            CompletableFuture::completed(supplier())
        }
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#submitAsync
    fn submit_async(&self, runnable: Box<dyn FnOnce() + Send + 'static>) -> CompletableFuture<()> {
        let future = CompletableFuture::<()>::completed_placeholder();
        let future_clone = CompletableFuture {
            inner: future.inner.clone(),
        };
        let task = Box::new(move || {
            runnable();
            complete_future(&future_clone, ());
        });
        let r = self.wrap_runnable(task);
        self.schedule(r);
        future
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#submit
    fn submit(&self, runnable: Box<dyn FnOnce() + Send + 'static>) -> CompletableFuture<()> {
        if self.schedule_executables() {
            self.submit_async(runnable)
        } else {
            runnable();
            CompletableFuture::completed(())
        }
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#executeBlocking
    fn execute_blocking(&self, runnable: Box<dyn FnOnce() + Send + 'static>) {
        if !self.is_same_thread() {
            self.submit_async(runnable).join();
        } else {
            runnable();
        }
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#schedule
    fn schedule(&self, task: R) {
        self.state()
            .pending_runnables
            .lock()
            .unwrap()
            .push_back(task);
        self.get_running_thread().unpark();
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#execute
    fn execute(&self, runnable: Box<dyn FnOnce() + Send + 'static>) {
        let task = self.wrap_runnable(runnable);
        if self.schedule_executables() {
            self.schedule(task);
        } else {
            self.do_run_task(task);
        }
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#executeIfPossible
    fn execute_if_possible(&self, runnable: Box<dyn FnOnce() + Send + 'static>) {
        self.execute(runnable);
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#dropAllTasks
    fn drop_all_tasks(&self) {
        self.state().pending_runnables.lock().unwrap().clear();
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#runAllTasks
    fn run_all_tasks(&self) {
        while self.poll_task() {}
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#shouldRunAllTasks
    fn should_run_all_tasks(&self) -> bool {
        self.state().blocking_count.load(Ordering::Relaxed) > 0
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#pollTask
    fn poll_task(&self) -> bool {
        let mut queue = self.state().pending_runnables.lock().unwrap();
        let should_run = match queue.front() {
            None => return false,
            Some(task) => self.should_run_all_tasks() || self.should_run(task),
        };
        if !should_run {
            return false;
        }
        let task = queue.pop_front().unwrap();
        drop(queue);
        self.do_run_task(task);
        true
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#managedBlock
    fn managed_block(&self, mut is_done: impl FnMut() -> bool) {
        self.state()
            .blocking_count
            .fetch_add(1, Ordering::Relaxed);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            while !is_done() {
                if !self.poll_task() {
                    self.wait_for_tasks();
                }
            }
        }));
        self.state()
            .blocking_count
            .fetch_sub(1, Ordering::Relaxed);
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#waitForTasks
    fn wait_for_tasks(&self) {
        thread::yield_now();
        thread::park_timeout(Duration::from_nanos(BLOCK_TIME_NANOS));
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#doRunTask
    fn do_run_task(&self, mut task: R) {
        let name = self.name().to_owned();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            task.run();
        }));
        if let Err(e) = result {
            eprintln!("Error executing task on {}", name);
            if is_non_recoverable_panic(&e) {
                std::panic::resume_unwind(e);
            }
        }
    }

    // JAVA: net.minecraft.util.thread.BlockableEventLoop#profiledMetrics
    fn profiled_metrics(&self) -> Vec<MetricSampler> {
        let queue = self.state().pending_runnables.clone();
        let name = format!("{}-pending-tasks", self.name());
        vec![MetricSampler::create(
            name,
            MetricCategory::EventLoops,
            Box::new(move || queue.lock().unwrap().len() as f64),
        )]
    }
}

impl<T: Send + 'static> CompletableFuture<T> {
    fn completed_placeholder() -> Self {
        Self {
            inner: Arc::new((Mutex::new(None), Condvar::new())),
        }
    }
}

fn is_non_recoverable_panic(err: &(dyn std::any::Any + Send + 'static)) -> bool {
    is_non_recoverable(err)
}

// JAVA: net.minecraft.util.thread.BlockableEventLoop#isNonRecoverable
pub fn is_non_recoverable(err: &(dyn std::any::Any + Send + 'static)) -> bool {
    if let Some(re) = err.downcast_ref::<ReportedException>() {
        return match &re.cause {
            Some(cause) => is_non_recoverable(cause.as_ref()),
            None => true,
        };
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicI32;
    use std::sync::Arc;

    struct TestLoop {
        state: BlockableEventLoopState<BoxedRunnable>,
        running_thread: thread::Thread,
    }

    impl TestLoop {
        fn new(name: &str) -> Self {
            Self {
                state: BlockableEventLoopState::new(name.to_string()),
                running_thread: thread::current(),
            }
        }
    }

    impl BlockableEventLoop<BoxedRunnable> for TestLoop {
        fn state(&self) -> &BlockableEventLoopState<BoxedRunnable> {
            &self.state
        }

        fn should_run(&self, _task: &BoxedRunnable) -> bool {
            true
        }

        fn get_running_thread(&self) -> thread::Thread {
            self.running_thread.clone()
        }

        fn wrap_runnable(&self, runnable: Box<dyn FnOnce() + Send + 'static>) -> BoxedRunnable {
            BoxedRunnable(Some(runnable))
        }
    }

    struct SelectiveLoop {
        state: BlockableEventLoopState<BoxedRunnable>,
        running_thread: thread::Thread,
    }

    impl SelectiveLoop {
        fn new() -> Self {
            Self {
                state: BlockableEventLoopState::new("sel".to_string()),
                running_thread: thread::current(),
            }
        }
    }

    impl BlockableEventLoop<BoxedRunnable> for SelectiveLoop {
        fn state(&self) -> &BlockableEventLoopState<BoxedRunnable> {
            &self.state
        }

        fn should_run(&self, _task: &BoxedRunnable) -> bool {
            false
        }

        fn get_running_thread(&self) -> thread::Thread {
            self.running_thread.clone()
        }

        fn wrap_runnable(&self, runnable: Box<dyn FnOnce() + Send + 'static>) -> BoxedRunnable {
            BoxedRunnable(Some(runnable))
        }
    }

    #[test]
    fn poll_task_runs_and_empty_queue_returns_false() {
        let lo = TestLoop::new("test");
        assert!(!lo.poll_task());
        assert_eq!(lo.get_pending_tasks_count(), 0);
        let counter = Arc::new(AtomicI32::new(0));
        let c2 = counter.clone();
        lo.schedule(lo.wrap_runnable(Box::new(move || {
            c2.fetch_add(1, Ordering::Relaxed);
        })));
        assert_eq!(lo.get_pending_tasks_count(), 1);
        assert!(lo.is_same_thread());
        assert!(!lo.schedule_executables());
        assert!(lo.poll_task());
        assert_eq!(counter.load(Ordering::Relaxed), 1);
        assert!(!lo.poll_task());
    }

    #[test]
    fn poll_task_respects_should_run_and_blocking_count() {
        let lo = SelectiveLoop::new();
        lo.schedule(lo.wrap_runnable(Box::new(|| {})));
        assert!(!lo.poll_task());
        assert_eq!(lo.get_pending_tasks_count(), 1);

        lo.state().blocking_count.store(1, Ordering::Relaxed);
        assert!(lo.should_run_all_tasks());
        assert!(lo.poll_task());
        assert_eq!(lo.get_pending_tasks_count(), 0);

        lo.state().blocking_count.store(-1, Ordering::Relaxed);
        assert!(!lo.should_run_all_tasks());
        lo.state().blocking_count.store(i32::MAX, Ordering::Relaxed);
        assert!(lo.should_run_all_tasks());
        lo.state().blocking_count.store(0, Ordering::Relaxed);
    }

    #[test]
    fn managed_block_polls_tasks_and_restores_blocking_count() {
        let lo = TestLoop::new("mb");
        let counter = Arc::new(AtomicI32::new(0));
        let c2 = counter.clone();
        lo.schedule(lo.wrap_runnable(Box::new(move || {
            c2.fetch_add(1, Ordering::Relaxed);
        })));
        let mut ready = false;
        lo.managed_block(|| {
            let bc = lo.state().blocking_count.load(Ordering::Relaxed);
            assert_eq!(bc, 1);
            if lo.get_pending_tasks_count() == 0 {
                ready = true;
            }
            ready
        });
        assert_eq!(counter.load(Ordering::Relaxed), 1);
        assert_eq!(lo.state().blocking_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn is_non_recoverable_checks_reported_exception_chain() {
        let inner = ReportedException { cause: None };
        let outer = ReportedException {
            cause: Some(Box::new(inner)),
        };
        let any: Box<dyn std::any::Any + Send> = Box::new(outer);
        assert!(is_non_recoverable(any.as_ref()));

        let plain: Box<dyn std::any::Any + Send> = Box::new(String::from("oops"));
        assert!(!is_non_recoverable(plain.as_ref()));

        let none_cause = ReportedException { cause: None };
        let any2: Box<dyn std::any::Any + Send> = Box::new(none_cause);
        assert!(is_non_recoverable(any2.as_ref()));
    }

    #[test]
    fn execute_drop_all_tasks_and_block_time_float_bits() {
        let lo = TestLoop::new("drop");
        lo.schedule(lo.wrap_runnable(Box::new(|| {})));
        lo.schedule(lo.wrap_runnable(Box::new(|| {})));
        assert_eq!(lo.get_pending_tasks_count(), 2);
        lo.drop_all_tasks();
        assert_eq!(lo.get_pending_tasks_count(), 0);

        let counter = Arc::new(AtomicI32::new(0));
        let c2 = counter.clone();
        lo.execute(Box::new(move || {
            c2.fetch_add(2_i32.wrapping_sub(1), Ordering::Relaxed);
        }));
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        assert_eq!(BLOCK_TIME_NANOS, 100_000u64);
        let f = BLOCK_TIME_NANOS as f64;
        assert_eq!(f.to_bits(), 100_000.0f64.to_bits());
        assert_eq!((-1_i32).wrapping_add(2), 1);
    }
}
