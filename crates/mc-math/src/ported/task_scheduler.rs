use std::fmt;
use std::sync::{Arc, Mutex};

/// Minimal completable future used by task scheduling ports.
// JAVA: java.util.concurrent.CompletableFuture
pub struct CompletableFuture<T> {
    inner: Arc<Mutex<Option<T>>>,
}

impl<T> Clone for CompletableFuture<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Default for CompletableFuture<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CompletableFuture<T> {
    // JAVA: java.util.concurrent.CompletableFuture#new
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    // JAVA: java.util.concurrent.CompletableFuture#complete
    pub fn complete(&self, value: T) -> bool {
        let mut guard = self
            .inner
            .lock()
            .expect("CompletableFuture lock poisoned");
        if guard.is_none() {
            *guard = Some(value);
            true
        } else {
            false
        }
    }

    // JAVA: java.util.concurrent.CompletableFuture#get
    pub fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner
            .lock()
            .expect("CompletableFuture lock poisoned")
            .clone()
    }

    // JAVA: java.util.concurrent.CompletableFuture#isDone
    pub fn is_done(&self) -> bool {
        self.inner
            .lock()
            .expect("CompletableFuture lock poisoned")
            .is_some()
    }
}

// JAVA: net.minecraft.util.thread.TaskScheduler
pub trait TaskScheduler<R>: Send {
    // JAVA: net.minecraft.util.thread.TaskScheduler#name
    fn name(&self) -> &str;

    // JAVA: net.minecraft.util.thread.TaskScheduler#schedule
    fn schedule(&mut self, task: R);

    // JAVA: net.minecraft.util.thread.TaskScheduler#close
    fn close(&mut self) {}

    // JAVA: net.minecraft.util.thread.TaskScheduler#wrapRunnable
    fn wrap_runnable(&self, runnable: Box<dyn FnOnce() + Send>) -> R;

    // JAVA: net.minecraft.util.thread.TaskScheduler#scheduleWithResult
    fn schedule_with_result<T: Send + 'static>(
        &mut self,
        consumer: Box<dyn FnOnce(CompletableFuture<T>) + Send>,
    ) -> CompletableFuture<T>
    where
        Self: Sized,
        R: Send + 'static,
    {
        let future = CompletableFuture::new();
        let future_for_task = future.clone();
        let task = self.wrap_runnable(Box::new(move || consumer(future_for_task)));
        self.schedule(task);
        future
    }
}

pub struct ExecutorTaskScheduler<E> {
    name: String,
    executor: E,
}

// JAVA: net.minecraft.util.thread.TaskScheduler#wrapExecutor
pub fn wrap_executor<E>(name: String, executor: E) -> ExecutorTaskScheduler<E>
where
    E: Fn(Box<dyn FnOnce() + Send>) + Send + Sync + 'static,
{
    ExecutorTaskScheduler { name, executor }
}

impl<E> TaskScheduler<Box<dyn FnOnce() + Send>> for ExecutorTaskScheduler<E>
where
    E: Fn(Box<dyn FnOnce() + Send>) + Send + Sync + 'static,
{
    // JAVA: net.minecraft.util.thread.TaskScheduler#name
    fn name(&self) -> &str {
        &self.name
    }

    // JAVA: net.minecraft.util.thread.TaskScheduler#schedule
    fn schedule(&mut self, task: Box<dyn FnOnce() + Send>) {
        (self.executor)(task);
    }

    // JAVA: net.minecraft.util.thread.TaskScheduler#wrapRunnable
    fn wrap_runnable(&self, runnable: Box<dyn FnOnce() + Send>) -> Box<dyn FnOnce() + Send> {
        runnable
    }
}

impl<E> fmt::Display for ExecutorTaskScheduler<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn wrap_executor_runs_task_and_reports_name() {
        let ran = Arc::new(Mutex::new(false));
        let ran2 = Arc::clone(&ran);
        let exec = move |task: Box<dyn FnOnce() + Send>| task();
        let mut sched = wrap_executor("test".to_string(), exec);
        sched.schedule(Box::new(move || {
            *ran2.lock().unwrap() = true;
        }));
        assert!(*ran.lock().unwrap());
        assert_eq!(sched.name(), "test");
        assert_eq!(sched.to_string(), "test");
    }

    #[test]
    fn schedule_with_result_completes_future() {
        let exec = |task: Box<dyn FnOnce() + Send>| task();
        let mut sched = wrap_executor("r".to_string(), exec);
        let future = sched.schedule_with_result::<i32>(Box::new(|fut| {
            assert!(fut.complete(42));
        }));
        assert_eq!(future.get(), Some(42));
        assert!(future.is_done());
    }

    #[test]
    fn completable_future_complete_is_single_use() {
        let f: CompletableFuture<i32> = CompletableFuture::new();
        assert!(!f.is_done());
        assert!(f.complete(10));
        assert!(!f.complete(20));
        assert_eq!(f.get(), Some(10));
        let cloned = f.clone();
        assert_eq!(cloned.get(), Some(10));
    }

    #[test]
    fn wrap_runnable_returns_callable_runnable() {
        let exec = |task: Box<dyn FnOnce() + Send>| task();
        let sched = wrap_executor("n".to_string(), exec);
        let called = Arc::new(Mutex::new(0i32));
        let c2 = Arc::clone(&called);
        let r = sched.wrap_runnable(Box::new(move || {
            let mut g = c2.lock().unwrap();
            *g = g.wrapping_add(1);
        }));
        r();
        assert_eq!(*called.lock().unwrap(), 1);
    }

    #[test]
    fn close_default_is_noop() {
        let exec = |_: Box<dyn FnOnce() + Send>| {};
        let mut sched = wrap_executor("c".to_string(), exec);
        sched.close();
        assert_eq!(sched.name(), "c");
    }
}
