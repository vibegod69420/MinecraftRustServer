use std::collections::VecDeque;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

pub struct ReentrantBlockableEventLoop<R> {
    name: String,
    pub reentrant_count: Arc<AtomicI32>,
    queue: VecDeque<R>,
}

impl<R> ReentrantBlockableEventLoop<R> {
    // JAVA: net.minecraft.util.thread.ReentrantBlockableEventLoop#<init>
    pub fn new(name: String) -> Self {
        Self {
            name,
            reentrant_count: Arc::new(AtomicI32::new(0)),
            queue: VecDeque::new(),
        }
    }

    // JAVA: net.minecraft.util.thread.ReentrantBlockableEventLoop#scheduleExecutables
    pub fn schedule_executables(&mut self) -> bool {
        self.running_task() || !self.queue.is_empty()
    }

    // JAVA: net.minecraft.util.thread.ReentrantBlockableEventLoop#runningTask
    pub fn running_task(&self) -> bool {
        self.reentrant_count.load(Ordering::Relaxed) != 0
    }

    // JAVA: net.minecraft.util.thread.ReentrantBlockableEventLoop#doRunTask
    pub fn do_run_task(&mut self, task: R)
    where
        R: FnOnce(),
    {
        self.reentrant_count.fetch_add(1, Ordering::Relaxed);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            task();
        }));
        self.reentrant_count.fetch_sub(1, Ordering::Relaxed);
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn push_task(&mut self, task: R) {
        self.queue.push_back(task);
    }

    pub fn poll_task(&mut self) -> Option<R> {
        self.queue.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initial_state() {
        let ev: ReentrantBlockableEventLoop<Box<dyn FnOnce()>> =
            ReentrantBlockableEventLoop::new("test".to_string());
        assert_eq!(ev.name(), "test");
        assert!(!ev.running_task());
        assert_eq!(ev.reentrant_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn schedule_executables_when_running_or_queued() {
        let mut ev: ReentrantBlockableEventLoop<Box<dyn FnOnce()>> =
            ReentrantBlockableEventLoop::new("t".to_string());
        assert!(!ev.schedule_executables());
        ev.push_task(Box::new(|| {}));
        assert!(ev.schedule_executables());
        let _ = ev.poll_task();
        assert!(!ev.schedule_executables());
        ev.reentrant_count.store(1, Ordering::Relaxed);
        assert!(ev.schedule_executables());
    }

    #[test]
    fn do_run_task_increments_during_execution() {
        let mut ev = ReentrantBlockableEventLoop::<Box<dyn FnOnce()>>::new("t".to_string());
        let count = ev.reentrant_count.clone();
        let count2 = count.clone();
        ev.do_run_task(Box::new(move || {
            assert_eq!(count2.load(Ordering::Relaxed), 1);
        }));
        assert_eq!(count.load(Ordering::Relaxed), 0);
        assert!(!ev.running_task());
    }

    #[test]
    fn do_run_task_wrapping_and_negative_edge_values() {
        let mut ev = ReentrantBlockableEventLoop::<Box<dyn FnOnce()>>::new("t".to_string());
        let count = ev.reentrant_count.clone();
        count.store(i32::MAX, Ordering::Relaxed);
        let c2 = count.clone();
        ev.do_run_task(Box::new(move || {
            assert_eq!(c2.load(Ordering::Relaxed), i32::MIN);
        }));
        assert_eq!(count.load(Ordering::Relaxed), i32::MAX);

        count.store(i32::MIN, Ordering::Relaxed);
        assert!(ev.running_task());
        count.store(-1, Ordering::Relaxed);
        assert!(ev.running_task());
        count.store(0, Ordering::Relaxed);
        assert!(!ev.running_task());
    }

    #[test]
    fn do_run_task_decrements_on_panic() {
        let mut ev = ReentrantBlockableEventLoop::<Box<dyn FnOnce()>>::new("t".to_string());
        let count = ev.reentrant_count.clone();
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ev.do_run_task(Box::new(|| panic!("boom")));
        }));
        assert!(res.is_err());
        assert_eq!(count.load(Ordering::Relaxed), 0);
    }
}
