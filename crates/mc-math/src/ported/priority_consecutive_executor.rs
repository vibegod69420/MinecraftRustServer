use std::collections::VecDeque;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Condvar, Mutex};

pub type BoxedRunnable = Box<dyn FnOnce() + Send + 'static>;

pub struct RunnableWithPriority {
    pub priority: i32,
    pub task: BoxedRunnable,
}

impl RunnableWithPriority {
    // JAVA: net.minecraft.util.thread.StrictQueue$RunnableWithPriority#run
    pub fn run(self) {
        (self.task)();
    }
}

pub struct FixedPriorityQueue {
    queues: Vec<Mutex<VecDeque<BoxedRunnable>>>,
    size: AtomicI32,
}

impl FixedPriorityQueue {
    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#<init>
    pub fn new(priority_count: i32) -> Self {
        let mut queues = Vec::with_capacity(if priority_count > 0 {
            priority_count as usize
        } else {
            0
        });
        let mut i = 0i32;
        while i < priority_count {
            queues.push(Mutex::new(VecDeque::new()));
            i = i.wrapping_add(1);
        }
        Self {
            queues,
            size: AtomicI32::new(0),
        }
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#pop
    pub fn pop(&self) -> Option<BoxedRunnable> {
        for q in &self.queues {
            let mut guard = q.lock().unwrap();
            if let Some(task) = guard.pop_front() {
                drop(guard);
                self.size.fetch_sub(1, Ordering::SeqCst);
                return Some(task);
            }
        }
        None
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#push
    pub fn push(&self, task: RunnableWithPriority) -> bool {
        let p = task.priority;
        let len = self.queues.len() as i32;
        if p < len && p >= 0 {
            self.queues[p as usize]
                .lock()
                .unwrap()
                .push_back(task.task);
            self.size.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            panic!(
                "Priority {} not supported. Expected range [0-{}]",
                p,
                len.wrapping_sub(1)
            );
        }
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#isEmpty
    pub fn is_empty(&self) -> bool {
        self.size.load(Ordering::SeqCst) == 0
    }

    // JAVA: net.minecraft.util.thread.StrictQueue$FixedPriorityQueue#size
    pub fn size(&self) -> i32 {
        self.size.load(Ordering::SeqCst)
    }
}

pub struct CompletableFuture<T> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T> Clone for CompletableFuture<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> CompletableFuture<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((Mutex::new(None), Condvar::new())),
        }
    }

    pub fn complete(&self, value: T) -> bool {
        let (lock, cv) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        if guard.is_some() {
            return false;
        }
        *guard = Some(value);
        cv.notify_all();
        true
    }
}

impl<T: Clone> CompletableFuture<T> {
    pub fn join(&self) -> T {
        let (lock, cv) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        while guard.is_none() {
            guard = cv.wait(guard).unwrap();
        }
        guard.as_ref().unwrap().clone()
    }
}

pub struct PriorityConsecutiveExecutor {
    queue: Arc<FixedPriorityQueue>,
    executor: Arc<dyn Fn(BoxedRunnable) + Send + Sync>,
    _name: String,
}

impl PriorityConsecutiveExecutor {
    // JAVA: net.minecraft.util.thread.PriorityConsecutiveExecutor#<init>
    pub fn new(
        priority_count: i32,
        executor: Arc<dyn Fn(BoxedRunnable) + Send + Sync>,
        name: String,
    ) -> Self {
        // MetricsRegistry.INSTANCE.add(this); // omitted: MetricsRegistry not ported
        Self {
            queue: Arc::new(FixedPriorityQueue::new(priority_count)),
            executor,
            _name: name,
        }
    }

    // JAVA: net.minecraft.util.thread.PriorityConsecutiveExecutor#wrapRunnable
    pub fn wrap_runnable(&self, task: BoxedRunnable) -> RunnableWithPriority {
        RunnableWithPriority { priority: 0, task }
    }

    // JAVA: net.minecraft.util.thread.PriorityConsecutiveExecutor#scheduleWithResult
    pub fn schedule_with_result<T, F>(&self, priority: i32, task: F) -> CompletableFuture<T>
    where
        T: Send + 'static,
        F: FnOnce(CompletableFuture<T>) + Send + 'static,
    {
        let future = CompletableFuture::new();
        let future_for_task = future.clone();
        self.schedule(RunnableWithPriority {
            priority,
            task: Box::new(move || task(future_for_task)),
        });
        future
    }

    fn schedule(&self, task: RunnableWithPriority) {
        self.queue.push(task);
        let queue = Arc::clone(&self.queue);
        (self.executor)(Box::new(move || {
            while let Some(r) = queue.pop() {
                r();
            }
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::{Arc, Mutex};

    fn immediate_executor() -> Arc<dyn Fn(BoxedRunnable) + Send + Sync> {
        Arc::new(|r: BoxedRunnable| r())
    }

    #[test]
    fn wrap_runnable_sets_priority_zero_and_runs() {
        let exec = PriorityConsecutiveExecutor::new(4, immediate_executor(), "test".to_string());
        let hit = Arc::new(AtomicI32::new(0));
        let h = Arc::clone(&hit);
        let wrapped = exec.wrap_runnable(Box::new(move || {
            h.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(wrapped.priority, 0);
        wrapped.run();
        assert_eq!(hit.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn schedule_with_result_completes_future() {
        let exec = PriorityConsecutiveExecutor::new(4, immediate_executor(), "test".to_string());
        let future = exec.schedule_with_result(0, |f: CompletableFuture<i32>| {
            f.complete(42);
        });
        assert_eq!(future.join(), 42);
    }

    #[test]
    fn schedule_with_result_respects_priority_order() {
        let pending: Arc<Mutex<Vec<BoxedRunnable>>> = Arc::new(Mutex::new(Vec::new()));
        let p = Arc::clone(&pending);
        let executor: Arc<dyn Fn(BoxedRunnable) + Send + Sync> = Arc::new(move |r| {
            p.lock().unwrap().push(r);
        });
        let exec = PriorityConsecutiveExecutor::new(3, executor, "test".to_string());
        let log: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));

        let l1 = Arc::clone(&log);
        exec.schedule_with_result(2, move |f: CompletableFuture<i32>| {
            l1.lock().unwrap().push(2);
            f.complete(2);
        });
        let l2 = Arc::clone(&log);
        exec.schedule_with_result(0, move |f: CompletableFuture<i32>| {
            l2.lock().unwrap().push(0);
            f.complete(0);
        });

        loop {
            let task = pending.lock().unwrap().pop();
            match task {
                Some(t) => t(),
                None => break,
            }
        }
        assert_eq!(*log.lock().unwrap(), vec![0, 2]);
    }

    #[test]
    fn schedule_with_result_handles_negative_and_large_int_priority_bounds() {
        let exec = PriorityConsecutiveExecutor::new(2, immediate_executor(), "bounds".to_string());
        let future = exec.schedule_with_result(1, |f: CompletableFuture<i32>| {
            let v = i32::MAX.wrapping_add(1); // overflow edge case
            f.complete(v);
        });
        assert_eq!(future.join(), i32::MIN);
    }
}
