use std::future::Future;
use std::pin::Pin;

/// Placeholder for `net.minecraft.world.level.chunk.status.WorldGenContext`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldGenContext;

/// Placeholder for `net.minecraft.world.level.chunk.status.ChunkStep`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkStep;

/// Placeholder for `net.minecraft.server.level.GenerationChunkHolder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationChunkHolder;

/// Placeholder for `net.minecraft.util.StaticCache2D<T>`.
#[derive(Debug)]
pub struct StaticCache2D<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> StaticCache2D<T> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Default for StaticCache2D<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder for `net.minecraft.world.level.chunk.ChunkAccess`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkAccess(pub i32);

/// Port of `net.minecraft.world.level.chunk.status.ChunkStatusTask`.
pub trait ChunkStatusTask: Send + Sync {
    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatusTask#doWork
    fn do_work(
        &self,
        ctx: &WorldGenContext,
        step: &ChunkStep,
        cache: &StaticCache2D<GenerationChunkHolder>,
        chunk: ChunkAccess,
    ) -> Pin<Box<dyn Future<Output = ChunkAccess> + Send>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    struct NoopWaker;
    impl Wake for NoopWaker {
        fn wake(self: Arc<Self>) {}
    }

    fn block_on(mut fut: Pin<Box<dyn Future<Output = ChunkAccess> + Send>>) -> ChunkAccess {
        let waker: Waker = Arc::new(NoopWaker).into();
        let mut cx = Context::from_waker(&waker);
        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(v) => return v,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    struct IdentityTask;
    impl ChunkStatusTask for IdentityTask {
        // JAVA: net.minecraft.world.level.chunk.status.ChunkStatusTask#doWork
        fn do_work(
            &self,
            _ctx: &WorldGenContext,
            _step: &ChunkStep,
            _cache: &StaticCache2D<GenerationChunkHolder>,
            chunk: ChunkAccess,
        ) -> Pin<Box<dyn Future<Output = ChunkAccess> + Send>> {
            Box::pin(async move { chunk })
        }
    }

    struct OffsetTask(i32);
    impl ChunkStatusTask for OffsetTask {
        // JAVA: net.minecraft.world.level.chunk.status.ChunkStatusTask#doWork
        fn do_work(
            &self,
            _ctx: &WorldGenContext,
            _step: &ChunkStep,
            _cache: &StaticCache2D<GenerationChunkHolder>,
            chunk: ChunkAccess,
        ) -> Pin<Box<dyn Future<Output = ChunkAccess> + Send>> {
            let offset = self.0;
            Box::pin(async move { ChunkAccess(chunk.0.wrapping_add(offset)) })
        }
    }

    #[test]
    fn identity_task_returns_chunk() {
        let task = IdentityTask;
        let out = block_on(task.do_work(
            &WorldGenContext,
            &ChunkStep,
            &StaticCache2D::<GenerationChunkHolder>::new(),
            ChunkAccess(42),
        ));
        assert_eq!(out, ChunkAccess(42));
    }

    #[test]
    fn trait_is_object_safe() {
        let task: Box<dyn ChunkStatusTask> = Box::new(IdentityTask);
        let out = block_on(task.do_work(
            &WorldGenContext,
            &ChunkStep,
            &StaticCache2D::default(),
            ChunkAccess(7),
        ));
        assert_eq!(out.0, 7);
    }

    #[test]
    fn passes_through_negative_and_extreme_values() {
        let task = IdentityTask;
        let min_out = block_on(task.do_work(
            &WorldGenContext,
            &ChunkStep,
            &StaticCache2D::new(),
            ChunkAccess(i32::MIN),
        ));
        let max_out = block_on(task.do_work(
            &WorldGenContext,
            &ChunkStep,
            &StaticCache2D::new(),
            ChunkAccess(i32::MAX),
        ));
        assert_eq!(min_out.0, i32::MIN);
        assert_eq!(max_out.0, i32::MAX);
    }

    #[test]
    fn wrapping_offset_matches_java_int_semantics() {
        let task = OffsetTask(1);
        let out = block_on(task.do_work(
            &WorldGenContext,
            &ChunkStep,
            &StaticCache2D::new(),
            ChunkAccess(i32::MAX),
        ));
        assert_eq!(out.0, i32::MIN);
    }
}
