use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

type BoxErr = Box<dyn Any + Send + 'static>;

#[derive(Debug)]
pub struct CancellationException;

pub trait Executor: Send + Sync {
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>);
}

pub trait TaskChainer {
    // JAVA: net.minecraft.util.TaskChainer#append
    fn append<T: Send + 'static>(
        &mut self,
        future: CompletableFuture<T>,
        consumer: Box<dyn FnOnce(T) + Send + 'static>,
    );
}

struct FutureState<T> {
    result: Option<Result<T, BoxErr>>,
}

pub struct CompletableFuture<T: Send + 'static> {
    inner: Arc<(Mutex<FutureState<T>>, Condvar)>,
}

impl<T: Send + 'static> Clone for CompletableFuture<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Send + 'static> CompletableFuture<T> {
    pub fn new_pending() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(FutureState { result: None }),
                Condvar::new(),
            )),
        }
    }

    // JAVA: java.util.concurrent.CompletableFuture#completedFuture
    pub fn completed_future(value: T) -> Self {
        let f = Self::new_pending();
        f.complete(Ok(value));
        f
    }

    pub fn completed_exception(err: BoxErr) -> Self {
        let f = Self::new_pending();
        f.complete(Err(err));
        f
    }

    fn complete(&self, result: Result<T, BoxErr>) {
        let (lock, cv) = &*self.inner;
        let mut state = lock.lock().expect("future mutex poisoned");
        if state.result.is_none() {
            state.result = Some(result);
            cv.notify_all();
        }
    }

    fn wait_result(&self) -> Result<T, BoxErr>
    where
        T: Clone,
    {
        let (lock, cv) = &*self.inner;
        let mut state = lock.lock().expect("future mutex poisoned");
        while state.result.is_none() {
            state = cv.wait(state).expect("future mutex poisoned");
        }
        match state.result.as_ref().expect("result present") {
            Ok(v) => Ok(v.clone()),
            Err(e) => Err(clone_boxed_error(e)),
        }
    }
}

fn clone_boxed_error(e: &BoxErr) -> BoxErr {
    if let Some(msg) = e.downcast_ref::<&'static str>() {
        return Box::new(*msg);
    }
    if let Some(msg) = e.downcast_ref::<String>() {
        return Box::new(msg.clone());
    }
    if e.downcast_ref::<CancellationException>().is_some() {
        return Box::new(CancellationException);
    }
    Box::new("unknown error".to_string())
}

fn is_cancellation(e: &(dyn Any + Send + 'static)) -> bool {
    e.downcast_ref::<CancellationException>().is_some()
}

pub struct FutureChain {
    head: Mutex<CompletableFuture<()>>,
    executor: Arc<dyn Executor>,
    closed: Arc<AtomicBool>,
}

impl FutureChain {
    // JAVA: net.minecraft.util.FutureChain#<init>
    pub fn new(executor: Arc<dyn Executor>) -> Self {
        Self {
            head: Mutex::new(CompletableFuture::completed_future(())),
            executor,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    // JAVA: net.minecraft.util.FutureChain#append
    pub fn append<T: Send + Clone + 'static>(
        &self,
        future: CompletableFuture<T>,
        consumer: impl FnOnce(T) + Send + 'static,
    ) {
        let head = self
            .head
            .lock()
            .expect("future chain head poisoned")
            .clone();
        let executor = Arc::clone(&self.executor);
        let closed = Arc::clone(&self.closed);
        let next_head = CompletableFuture::<()>::new_pending();
        let next_head_for_task = next_head.clone();

        executor.execute(Box::new(move || {
            let _head_result = head.wait_result();
            let fut_result = future.wait_result();

            let stage_result: Result<(), BoxErr> = match fut_result {
                Ok(value) => {
                    if closed.load(Ordering::SeqCst) {
                        Ok(())
                    } else {
                        match panic::catch_unwind(AssertUnwindSafe(|| consumer(value))) {
                            Ok(()) => Ok(()),
                            Err(e) => Err(e),
                        }
                    }
                }
                Err(e) => Err(e),
            };

            let final_result = match stage_result {
                Ok(()) => Ok(()),
                Err(e) => {
                    if is_cancellation(&*e) {
                        // JAVA: if ($$0x instanceof CancellationException $$2) { throw $$2; }
                        // Preserve cancellation by completing exceptionally.
                        next_head_for_task.complete(Err(Box::new(CancellationException)));
                        return;
                    }
                    // JAVA: LOGGER.error("Chain link failed, continuing to next one", $$0x);
                    eprintln!("Chain link failed, continuing to next one");
                    Ok(())
                }
            };

            next_head_for_task.complete(final_result);
        }));

        *self.head.lock().expect("future chain head poisoned") = next_head;
    }

    // JAVA: net.minecraft.util.FutureChain#close
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

impl TaskChainer for FutureChain {
    fn append<T: Send + 'static>(
        &mut self,
        future: CompletableFuture<T>,
        consumer: Box<dyn FnOnce(T) + Send + 'static>,
    ) {
        // TaskChainer in Java is generic over T; our internal append requires Clone for wait_result.
        // Provide a wrapper by moving value through Arc<Mutex<Option<T>>>.
        let slot: Arc<Mutex<Option<T>>> = Arc::new(Mutex::new(None));
        let slot_for_set = Arc::clone(&slot);
        let bridge = CompletableFuture::<()>::new_pending();
        let bridge_for_task = bridge.clone();
        let executor = Arc::clone(&self.executor);
        executor.execute(Box::new(move || {
            let res = future.wait_result_no_clone();
            match res {
                Ok(v) => {
                    *slot_for_set.lock().expect("slot poisoned") = Some(v);
                    bridge_for_task.complete(Ok(()));
                }
                Err(e) => bridge_for_task.complete(Err(e)),
            }
        }));

        FutureChain::append(self, bridge, move |_| {
            let value = slot
                .lock()
                .expect("slot poisoned")
                .take()
                .expect("value present");
            consumer(value);
        });
    }
}

impl<T: Send + 'static> CompletableFuture<T> {
    fn wait_result_no_clone(&self) -> Result<T, BoxErr> {
        let (lock, cv) = &*self.inner;
        let mut state = lock.lock().expect("future mutex poisoned");
        while state.result.is_none() {
            state = cv.wait(state).expect("future mutex poisoned");
        }
        match state.result.take().expect("result present") {
            Ok(v) => {
                // Put nothing back; this helper is for one-shot bridge.
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }
}

impl Drop for FutureChain {
    fn drop(&mut self) {
        // JAVA: AutoCloseable#close
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    struct ThreadExecutor;
    impl Executor for ThreadExecutor {
        fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
            thread::spawn(task);
        }
    }

    #[test]
    fn append_runs_consumer() {
        let chain = FutureChain::new(Arc::new(ThreadExecutor));
        let (tx, rx) = mpsc::channel();
        chain.append(CompletableFuture::completed_future(7_i32), move |v| {
            tx.send(v).unwrap();
        });
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), 7);
    }

    #[test]
    fn close_prevents_consumer_execution() {
        let chain = FutureChain::new(Arc::new(ThreadExecutor));
        let (tx, rx) = mpsc::channel::<i32>();
        chain.close();
        chain.append(CompletableFuture::completed_future(1_i32), move |v| {
            tx.send(v).unwrap();
        });
        assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
    }

    #[test]
    fn cancellation_is_propagated_as_exceptional_completion() {
        let chain = FutureChain::new(Arc::new(ThreadExecutor));
        let cancelled = CompletableFuture::<i32>::completed_exception(Box::new(CancellationException));
        chain.append(cancelled, |_v| {});
        // Allow task to run; chain should remain usable and not deadlock.
        thread::sleep(Duration::from_millis(50));
        let (tx, rx) = mpsc::channel();
        chain.append(CompletableFuture::completed_future(2_i32), move |v| {
            tx.send(v).unwrap();
        });
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), 2);
    }

    #[test]
    fn non_cancellation_error_continues_chain() {
        let chain = FutureChain::new(Arc::new(ThreadExecutor));
        let failed = CompletableFuture::<i32>::completed_exception(Box::new("boom"));
        chain.append(failed, |_v| {});
        let (tx, rx) = mpsc::channel();
        chain.append(CompletableFuture::completed_future(3_i32), move |v| {
            tx.send(v).unwrap();
        });
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), 3);
    }

    #[test]
    fn task_chainer_trait_append_works() {
        let mut chain = FutureChain::new(Arc::new(ThreadExecutor));
        let (tx, rx) = mpsc::channel();
        TaskChainer::append(
            &mut chain,
            CompletableFuture::completed_future(9_i32),
            Box::new(move |v| {
                tx.send(v).unwrap();
            }),
        );
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), 9);
    }
}
