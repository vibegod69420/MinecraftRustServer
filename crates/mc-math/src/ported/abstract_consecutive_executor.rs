use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Weak};

// JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor$Status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Status {
    Sleeping = 0,
    Running = 1,
    Closed = 2,
}

/// Minimal stand-in for `java.lang.Runnable` used by queued tasks.
pub trait Runnable: Send + 'static {
    fn run(self);
}

impl<F: FnOnce() + Send + 'static> Runnable for F {
    fn run(self) {
        self();
    }
}

/// Minimal stand-in for `net.minecraft.util.thread.StrictQueue`.
pub trait StrictQueue<T: Send>: Send + Sync {
    fn push(&self, value: T);
    fn pop(&self) -> Option<T>;
    fn is_empty(&self) -> bool;
    fn size(&self) -> i32;
}

/// Minimal stand-in for `java.util.concurrent.RejectedExecutionException`.
#[derive(Debug)]
pub struct RejectedExecutionException;

impl fmt::Display for RejectedExecutionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RejectedExecutionException")
    }
}

impl std::error::Error for RejectedExecutionException {}

/// Minimal stand-in for `java.util.concurrent.Executor`.
pub trait Executor: Send + Sync {
    fn execute(
        &self,
        task: Box<dyn FnOnce() + Send + 'static>,
    ) -> Result<(), RejectedExecutionException>;
}

/// Minimal stand-in for profiling metric metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricSampler {
    pub name: String,
}

/// Minimal stand-in for `net.minecraft.Util#runNamed`.
fn run_named<T: Runnable>(task: T, _name: &str) {
    task.run();
}

pub struct AbstractConsecutiveExecutor<
    T: Runnable + Send + 'static,
    Q: StrictQueue<T> + 'static,
    E: Executor + 'static,
> {
    status: AtomicU8,
    queue: Q,
    executor: E,
    name: String,
    self_weak: Weak<Self>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Runnable + Send + 'static, Q: StrictQueue<T> + 'static, E: Executor + 'static>
    AbstractConsecutiveExecutor<T, Q, E>
{
    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#AbstractConsecutiveExecutor
    pub fn new(queue: Q, executor: E, name: String) -> Arc<Self> {
        // MetricsRegistry.INSTANCE.add(this); -- omitted (no profiling registry in this port)
        Arc::new_cyclic(|weak| Self {
            status: AtomicU8::new(Status::Sleeping as u8),
            queue,
            executor,
            name,
            self_weak: weak.clone(),
            _marker: PhantomData,
        })
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#canBeScheduled
    fn can_be_scheduled(&self) -> bool {
        !self.is_closed() && !self.queue.is_empty()
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#close
    pub fn close(&self) {
        self.status.store(Status::Closed as u8, Ordering::Release);
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#pollTask
    fn poll_task(&self) -> bool {
        if !self.is_running() {
            return false;
        }
        let task = match self.queue.pop() {
            Some(t) => t,
            None => return false,
        };
        run_named(task, &self.name);
        true
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#run
    pub fn run(&self) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.poll_task();
        }));
        self.set_sleeping();
        self.register_for_execution();
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#runAll
    pub fn run_all(&self) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            while self.poll_task() {}
        }));
        self.set_sleeping();
        self.register_for_execution();
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#schedule
    pub fn schedule(&self, task: T) {
        self.queue.push(task);
        self.register_for_execution();
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#registerForExecution
    fn register_for_execution(&self) {
        if self.can_be_scheduled() && self.set_running() {
            let Some(this) = self.self_weak.upgrade() else {
                return;
            };
            let task = Box::new(move || {
                this.run();
            });
            if self.executor.execute(task).is_err() {
                let Some(this2) = self.self_weak.upgrade() else {
                    return;
                };
                let task2 = Box::new(move || {
                    this2.run();
                });
                if self.executor.execute(task2).is_err() {
                    eprintln!("Could not schedule ConsecutiveExecutor");
                }
            }
        }
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#size
    pub fn size(&self) -> i32 {
        self.queue.size()
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#hasWork
    pub fn has_work(&self) -> bool {
        self.is_running() && !self.queue.is_empty()
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#name
    pub fn name(&self) -> &str {
        &self.name
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#profiledMetrics
    pub fn profiled_metrics(&self) -> Vec<MetricSampler> {
        vec![MetricSampler {
            name: format!("{}-queue-size", self.name),
        }]
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#setRunning
    fn set_running(&self) -> bool {
        self.status
            .compare_exchange(
                Status::Sleeping as u8,
                Status::Running as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#setSleeping
    fn set_sleeping(&self) {
        let _ = self.status.compare_exchange(
            Status::Running as u8,
            Status::Sleeping as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#isRunning
    fn is_running(&self) -> bool {
        self.status.load(Ordering::Acquire) == Status::Running as u8
    }

    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#isClosed
    fn is_closed(&self) -> bool {
        self.status.load(Ordering::Acquire) == Status::Closed as u8
    }
}

impl<T: Runnable + Send + 'static, Q: StrictQueue<T> + 'static, E: Executor + 'static> fmt::Display
    for AbstractConsecutiveExecutor<T, Q, E>
{
    // JAVA: net.minecraft.util.thread.AbstractConsecutiveExecutor#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self.status.load(Ordering::Acquire) {
            x if x == Status::Sleeping as u8 => "SLEEPING",
            x if x == Status::Running as u8 => "RUNNING",
            x if x == Status::Closed as u8 => "CLOSED",
            _ => "UNKNOWN",
        };
        write!(f, "{} {} {}", self.name, status, self.queue.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering as AOrdering};
    use std::sync::{Arc, Mutex};

    struct TestQueue<T>(Mutex<VecDeque<T>>);

    impl<T> TestQueue<T> {
        fn new() -> Self {
            Self(Mutex::new(VecDeque::new()))
        }
    }

    impl<T: Send> StrictQueue<T> for TestQueue<T> {
        fn push(&self, value: T) {
            self.0.lock().unwrap().push_back(value);
        }

        fn pop(&self) -> Option<T> {
            self.0.lock().unwrap().pop_front()
        }

        fn is_empty(&self) -> bool {
            self.0.lock().unwrap().is_empty()
        }

        fn size(&self) -> i32 {
            self.0.lock().unwrap().len() as i32
        }
    }

    struct SyncExecutor;

    impl Executor for SyncExecutor {
        fn execute(
            &self,
            task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Result<(), RejectedExecutionException> {
            task();
            Ok(())
        }
    }

    struct DeferredExecutor(Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>);

    impl DeferredExecutor {
        fn new() -> Self {
            Self(Mutex::new(Vec::new()))
        }
    }

    impl Executor for DeferredExecutor {
        fn execute(
            &self,
            task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Result<(), RejectedExecutionException> {
            self.0.lock().unwrap().push(task);
            Ok(())
        }
    }

    type Task = Box<dyn FnOnce() + Send + 'static>;

    #[test]
    fn test_schedule_executes_in_order() {
        let q: TestQueue<Task> = TestQueue::new();
        let ex = AbstractConsecutiveExecutor::new(q, SyncExecutor, "test".to_string());
        let log = Arc::new(Mutex::new(Vec::new()));
        for i in 0..4i32 {
            let log = Arc::clone(&log);
            ex.schedule(Box::new(move || {
                log.lock().unwrap().push(i);
            }));
        }
        assert_eq!(*log.lock().unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(ex.size(), 0);
        assert!(!ex.has_work());
    }

    #[test]
    fn test_close_prevents_execution_and_reports_state() {
        let q: TestQueue<Task> = TestQueue::new();
        let ex = AbstractConsecutiveExecutor::new(q, SyncExecutor, "closed".to_string());
        ex.close();
        let ran = Arc::new(AtomicI32::new(0));
        let r = Arc::clone(&ran);
        ex.schedule(Box::new(move || {
            r.fetch_add(1, AOrdering::SeqCst);
        }));
        assert_eq!(ran.load(AOrdering::SeqCst), 0);
        assert_eq!(ex.size(), 1);
        assert!(!ex.has_work());
        let s = ex.to_string();
        assert_eq!(s, "closed CLOSED false");
    }

    #[test]
    fn test_run_all_drains_queue_while_running() {
        let q: TestQueue<Task> = TestQueue::new();
        let deferred = DeferredExecutor::new();
        let ex = AbstractConsecutiveExecutor::new(q, deferred, "drain".to_string());
        let log = Arc::new(Mutex::new(Vec::new()));
        for i in 0..3i32 {
            let log = Arc::clone(&log);
            ex.schedule(Box::new(move || {
                log.lock().unwrap().push(i);
            }));
        }
        // First schedule set RUNNING and deferred run(); subsequent schedules only enqueued.
        assert_eq!(ex.size(), 3);
        assert!(ex.has_work());
        ex.run_all();
        assert_eq!(*log.lock().unwrap(), vec![0, 1, 2]);
        assert_eq!(ex.size(), 0);
        assert!(!ex.has_work());
    }

    #[test]
    fn test_rejected_execution_retry_and_stuck_on_double_failure() {
        struct RejectOnce(AtomicUsize);
        impl Executor for RejectOnce {
            fn execute(
                &self,
                task: Box<dyn FnOnce() + Send + 'static>,
            ) -> Result<(), RejectedExecutionException> {
                let n = self.0.fetch_add(1, AOrdering::SeqCst);
                if n == 0 {
                    Err(RejectedExecutionException)
                } else {
                    task();
                    Ok(())
                }
            }
        }

        let q: TestQueue<Task> = TestQueue::new();
        let ex = AbstractConsecutiveExecutor::new(q, RejectOnce(AtomicUsize::new(0)), "retry".to_string());
        let ran = Arc::new(AtomicI32::new(0));
        let r = Arc::clone(&ran);
        ex.schedule(Box::new(move || {
            r.fetch_add(1, AOrdering::SeqCst);
        }));
        assert_eq!(ran.load(AOrdering::SeqCst), 1);
        assert_eq!(ex.size(), 0);

        struct RejectAlways;
        impl Executor for RejectAlways {
            fn execute(
                &self,
                _task: Box<dyn FnOnce() + Send + 'static>,
            ) -> Result<(), RejectedExecutionException> {
                Err(RejectedExecutionException)
            }
        }

        let q2: TestQueue<Task> = TestQueue::new();
        let ex2 = AbstractConsecutiveExecutor::new(q2, RejectAlways, "stuck".to_string());
        ex2.schedule(Box::new(|| {}));
        assert_eq!(ex2.size(), 1);
        assert!(ex2.has_work());
        assert_eq!(ex2.to_string(), "stuck RUNNING false");
    }

    #[test]
    fn test_name_metrics_and_wrapping_float_bits_in_tasks() {
        let q: TestQueue<Task> = TestQueue::new();
        let ex = AbstractConsecutiveExecutor::new(q, SyncExecutor, "calc".to_string());
        assert_eq!(ex.name(), "calc");
        let metrics = ex.profiled_metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "calc-queue-size");

        let out = Arc::new(Mutex::new((0i32, 0i32, 0u32, 0u32)));
        let o = Arc::clone(&out);
        ex.schedule(Box::new(move || {
            let wrapped = i32::MAX.wrapping_add(1);
            let neg = (-5i32).wrapping_sub(2);
            let neg_zero_bits = (-0.0f32).to_bits();
            let nan_bits = f32::NAN.to_bits();
            *o.lock().unwrap() = (wrapped, neg, neg_zero_bits, nan_bits);
        }));

        let r = *out.lock().unwrap();
        assert_eq!(r.0, i32::MIN);
        assert_eq!(r.1, -7i32);
        assert_eq!(r.2, 0x8000_0000u32);
        assert!(f32::from_bits(r.3).is_nan());
        assert_eq!(ex.to_string(), "calc SLEEPING true");
    }
}
