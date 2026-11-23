use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc};

/// Rust equivalent of a Java `Executor`: runs a boxed task.
pub type Executor = Arc<dyn Fn(Box<dyn FnOnce() + Send + 'static>) + Send + Sync>;

/// Minimal std-only stand-in for `java.util.concurrent.CompletableFuture<T>`.
///
/// A completed future delivers its value immediately through a channel.
/// Asynchronous producers can create one with [`CompletableFuture::from_receiver`].
pub struct CompletableFuture<T> {
    rx: mpsc::Receiver<T>,
}

impl<T: Send + 'static> CompletableFuture<T> {
    /// Creates a completed future, analogous to `CompletableFuture.completedFuture(value)`.
    pub fn completed(value: T) -> Self {
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(value);
        Self { rx }
    }

    /// Creates a future from a receiver that will receive exactly one value.
    pub fn from_receiver(rx: mpsc::Receiver<T>) -> Self {
        Self { rx }
    }
}

/// Port of `net.minecraft.util.TaskChainer`.
pub trait TaskChainer: Send + Sync {
    // JAVA: net.minecraft.util.TaskChainer#append
    fn append<T: Send + 'static>(
        &self,
        future: CompletableFuture<T>,
        consumer: Box<dyn FnOnce(T) + Send + 'static>,
    );

    // JAVA: net.minecraft.util.TaskChainer#append
    fn append_runnable(&self, runnable: Box<dyn FnOnce() + Send + 'static>) {
        self.append(
            CompletableFuture::completed(()),
            Box::new(move |_| runnable()),
        );
    }
}

/// Concrete immediate chainer returned by [`immediate`].
pub struct ImmediateTaskChainer {
    executor: Executor,
}

impl TaskChainer for ImmediateTaskChainer {
    // JAVA: net.minecraft.util.TaskChainer#append
    fn append<T: Send + 'static>(
        &self,
        future: CompletableFuture<T>,
        consumer: Box<dyn FnOnce(T) + Send + 'static>,
    ) {
        let exec_for_consumer = self.executor.clone();
        let wait_task = Box::new(move || match future.rx.recv() {
            Ok(value) => {
                let consumer_task = Box::new(move || {
                    let result = catch_unwind(AssertUnwindSafe(|| consumer(value)));
                    if let Err(e) = result {
                        eprintln!("Task failed: {:?}", e);
                    }
                });
                (exec_for_consumer)(consumer_task);
            }
            Err(e) => {
                eprintln!("Task failed: {:?}", e);
            }
        });
        (self.executor)(wait_task);
    }
}

// JAVA: net.minecraft.util.TaskChainer#immediate
pub fn immediate(executor: Executor) -> ImmediateTaskChainer {
    ImmediateTaskChainer { executor }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    fn sync_executor() -> Executor {
        Arc::new(|task: Box<dyn FnOnce() + Send + 'static>| task())
    }

    #[test]
    fn immediate_append_completed_runs_consumer() {
        let chainer = immediate(sync_executor());
        let (tx, rx) = mpsc::channel();
        chainer.append(
            CompletableFuture::completed(42_i32),
            Box::new(move |v| {
                tx.send(v).unwrap();
            }),
        );
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn append_runnable_runs_task() {
        let chainer = immediate(sync_executor());
        let (tx, rx) = mpsc::channel();
        chainer.append_runnable(Box::new(move || {
            tx.send(7_i32).unwrap();
        }));
        assert_eq!(rx.recv().unwrap(), 7);
    }

    #[test]
    fn consumer_panic_is_caught_and_does_not_propagate() {
        let chainer = immediate(sync_executor());
        let ran = Arc::new(AtomicUsize::new(0));
        let ran2 = ran.clone();
        chainer.append(
            CompletableFuture::completed(1_i32),
            Box::new(move |_| {
                ran2.fetch_add(1, Ordering::SeqCst);
                panic!("boom");
            }),
        );
        assert_eq!(ran.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn multiple_appends_execute_in_order_on_sync_executor() {
        let chainer = immediate(sync_executor());
        let out = Arc::new(Mutex::new(Vec::new()));
        for i in 0..4_i32 {
            let out2 = out.clone();
            chainer.append(
                CompletableFuture::completed(i),
                Box::new(move |v| {
                    out2.lock().unwrap().push(v);
                }),
            );
        }
        assert_eq!(*out.lock().unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn append_from_receiver_receives_value() {
        let chainer = immediate(sync_executor());
        let (value_tx, value_rx) = mpsc::channel();
        let (out_tx, out_rx) = mpsc::channel();
        value_tx.send(99_i32).unwrap();
        chainer.append(
            CompletableFuture::from_receiver(value_rx),
            Box::new(move |v| {
                out_tx.send(v).unwrap();
            }),
        );
        assert_eq!(out_rx.recv().unwrap(), 99);
    }
}
