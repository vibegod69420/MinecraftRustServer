use std::future::Future;
use std::pin::Pin;

/// Minimal placeholder for `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

/// Minimal placeholder for `net.minecraft.nbt.StreamTagVisitor`.
pub trait StreamTagVisitor {}

/// Port of `net.minecraft.world.level.chunk.storage.ChunkScanAccess`.
pub trait ChunkScanAccess {
    // JAVA: net.minecraft.world.level.chunk.storage.ChunkScanAccess#scanChunk
    fn scan_chunk(
        &self,
        pos: ChunkPos,
        visitor: &mut dyn StreamTagVisitor,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::task::{Context, Poll, Waker};

    struct NoopVisitor;
    impl StreamTagVisitor for NoopVisitor {}

    struct CountingScanner {
        calls: AtomicI32,
    }

    impl ChunkScanAccess for CountingScanner {
        fn scan_chunk(
            &self,
            _pos: ChunkPos,
            _visitor: &mut dyn StreamTagVisitor,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {})
        }
    }

    fn poll_ready(fut: Pin<&mut (dyn Future<Output = ()> + Send + '_)>) {
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut fut = fut;
        assert!(matches!(fut.as_mut().poll(&mut cx), Poll::Ready(())));
    }

    #[test]
    fn chunk_pos_equality_and_fields() {
        let a = ChunkPos::new(10, -20);
        let b = ChunkPos { x: 10, z: -20 };
        assert_eq!(a, b);
        assert_eq!(a.x, 10);
        assert_eq!(a.z, -20);
    }

    #[test]
    fn chunk_pos_negative_coordinates() {
        let pos = ChunkPos::new(i32::MIN, i32::MAX);
        assert_eq!(pos.x, i32::MIN);
        assert_eq!(pos.z, i32::MAX);
        let copy = pos;
        assert_eq!(pos, copy);
    }

    #[test]
    fn scan_chunk_future_completes() {
        let scanner = CountingScanner {
            calls: AtomicI32::new(0),
        };
        let mut visitor = NoopVisitor;
        let fut = scanner.scan_chunk(ChunkPos::new(7, -3), &mut visitor);
        let mut fut = Box::pin(fut);
        poll_ready(fut.as_mut());
        assert_eq!(scanner.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn scan_chunk_can_be_called_multiple_times() {
        let scanner = CountingScanner {
            calls: AtomicI32::new(0),
        };
        let mut visitor = NoopVisitor;
        for i in -2..=2 {
            let fut = scanner.scan_chunk(ChunkPos::new(i, i.wrapping_mul(2)), &mut visitor);
            let mut fut = Box::pin(fut);
            poll_ready(fut.as_mut());
        }
        assert_eq!(scanner.calls.load(Ordering::SeqCst), 5);
    }
}
