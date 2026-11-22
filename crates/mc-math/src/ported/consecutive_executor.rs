use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct ConcurrentLinkedQueue<T> {
    inner: Mutex<VecDeque<T>>,
}

impl<T> Default for ConcurrentLinkedQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConcurrentLinkedQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
        }
    }

    pub fn add(&self, value: T) -> bool {
        self.inner.lock().expect("lock poisoned").push_back(value);
        true
    }

    pub fn poll(&self) -> Option<T> {
        self.inner.lock().expect("lock poisoned").pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().expect("lock poisoned").is_empty()
    }
}

pub trait StrictQueue<R: Send + 'static>: Send + Sync {
    fn push(&self, task: R) -> bool;
    fn pop(&self) -> Option<R>;
    fn is_empty(&self) -> bool;
}

pub struct QueueStrictQueue<T: Send + 'static> {
    queue: ConcurrentLinkedQueue<T>,
}

impl<T: Send + 'static> QueueStrictQueue<T> {
    pub fn new(queue: ConcurrentLinkedQueue<T>) -> Self {
        Self { queue }
    }
}

impl<T: Send + 'static> StrictQueue<T> for QueueStrictQueue<T> {
    fn push(&self, task: T) -> bool {
        self.queue.add(task)
    }

    fn pop(&self) -> Option<T> {
        self.queue.poll()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

pub trait StrictQueueElement: Send + 'static {
    fn run_task(self);
}

impl StrictQueueElement for Box<dyn FnOnce() + Send + 'static> {
    fn run_task(self) {
        (self)();
    }
}

pub struct AbstractConsecutiveExecutor<R: StrictQueueElement> {
    queue: Arc<dyn StrictQueue<R>>,
    executor: Arc<dyn Fn(Box<dyn FnOnce() + Send + 'static>) + Send + Sync>,
    name: String,
    scheduled: Arc<Mutex<bool>>,
}

impl<R: StrictQueueElement> AbstractConsecutiveExecutor<R> {
    pub fn new(
        queue: Arc<dyn StrictQueue<R>>,
        executor: Arc<dyn Fn(Box<dyn FnOnce() + Send + 'static>) + Send + Sync>,
        name: String,
    ) -> Self {
        Self {
            queue,
            executor,
            name,
            scheduled: Arc::new(Mutex::new(false)),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schedule(&self, task: R) {
        self.queue.push(task);
        self.poll_task();
    }

    fn poll_task(&self) {
        let mut scheduled = self.scheduled.lock().expect("lock poisoned");
        if *scheduled || self.queue.is_empty() {
            return;
        }
        *scheduled = true;
        drop(scheduled);
        let drain = make_drain(
            self.queue.clone(),
            self.scheduled.clone(),
            self.executor.clone(),
        );
        (self.executor)(drain);
    }
}

fn make_drain<R: StrictQueueElement>(
    queue: Arc<dyn StrictQueue<R>>,
    scheduled: Arc<Mutex<bool>>,
    executor: Arc<dyn Fn(Box<dyn FnOnce() + Send + 'static>) + Send + Sync>,
) -> Box<dyn FnOnce() + Send + 'static> {
    Box::new(move || {
        if let Some(task) = queue.pop() {
            task.run_task();
        }
        let mut flag = scheduled.lock().expect("lock poisoned");
        if queue.is_empty() {
            *flag = false;
        } else {
            drop(flag);
            let next = make_drain(queue.clone(), scheduled.clone(), executor.clone());
            (executor)(next);
        }
    })
}

pub struct ConsecutiveExecutor {
    inner: AbstractConsecutiveExecutor<Box<dyn FnOnce() + Send + 'static>>,
}

impl ConsecutiveExecutor {
    // JAVA: net.minecraft.util.thread.ConsecutiveExecutor#<init>
    pub fn new(
        executor: Arc<dyn Fn(Box<dyn FnOnce() + Send + 'static>) + Send + Sync>,
        name: String,
    ) -> Self {
        Self {
            inner: AbstractConsecutiveExecutor::new(
                Arc::new(QueueStrictQueue::new(ConcurrentLinkedQueue::new())),
                executor,
                name,
            ),
        }
    }

    // JAVA: net.minecraft.util.thread.ConsecutiveExecutor#wrapRunnable
    pub fn wrap_runnable(
        &self,
        runnable: Box<dyn FnOnce() + Send + 'static>,
    ) -> Box<dyn FnOnce() + Send + 'static> {
        runnable
    }

    pub fn schedule(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        self.inner.schedule(task);
    }

    pub fn name(&self) -> &str {
        self.inner.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn wrap_runnable_returns_same_behavior() {
        let ex = ConsecutiveExecutor::new(Arc::new(|t| t()), "test".to_string());
        let counter = Arc::new(AtomicI32::new(0));
        let c2 = counter.clone();
        let wrapped = ex.wrap_runnable(Box::new(move || {
            c2.fetch_add(1, Ordering::SeqCst);
        }));
        wrapped();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn schedule_executes_tasks_in_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let ex = ConsecutiveExecutor::new(Arc::new(|t| t()), "seq".to_string());
        for i in 0..5i32 {
            let o = order.clone();
            ex.schedule(Box::new(move || {
                o.lock().unwrap().push(i);
            }));
        }
        assert_eq!(*order.lock().unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn name_is_preserved() {
        let ex = ConsecutiveExecutor::new(Arc::new(|t| t()), "my-executor".to_string());
        assert_eq!(ex.name(), "my-executor");
    }

    #[test]
    fn handles_negative_and_overflow_values() {
        let ex = ConsecutiveExecutor::new(Arc::new(|t| t()), "n".to_string());
        let val = Arc::new(AtomicI32::new(0));
        let v1 = val.clone();
        ex.schedule(Box::new(move || {
            v1.store(i32::MIN, Ordering::SeqCst);
        }));
        assert_eq!(val.load(Ordering::SeqCst), i32::MIN);
        let v2 = val.clone();
        ex.schedule(Box::new(move || {
            v2.store(i32::MAX, Ordering::SeqCst);
        }));
        assert_eq!(val.load(Ordering::SeqCst), i32::MAX);
    }

    #[test]
    fn float_bits_preserved_through_task() {
        let ex = ConsecutiveExecutor::new(Arc::new(|t| t()), "f".to_string());
        let bits = Arc::new(Mutex::new(0u64));
        let nan_bits = f64::NAN.to_bits();
        let b2 = bits.clone();
        ex.schedule(Box::new(move || {
            *b2.lock().unwrap() = nan_bits;
        }));
        assert_eq!(*bits.lock().unwrap(), nan_bits);
    }
}
