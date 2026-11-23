// === FILE: entity_storage.rs ===
//! Port of `net.minecraft.world.level.chunk.storage.EntityStorage`.

use std::collections::HashSet;
use std::fmt;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;

#[allow(dead_code)]
const ENTITIES_TAG: &str = "Entities";
#[allow(dead_code)]
const POSITION_TAG: &str = "Position";

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// Minimal chunk position, matching `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#toLong
    pub fn to_long(self) -> i64 {
        ((self.x as i64) & 0xFFFF_FFFF_i64) | (((self.z as i64) & 0xFFFF_FFFF_i64) << 32)
    }
}

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.z)
    }
}

/// Minimal chunk entities container, matching `net.minecraft.world.level.entity.ChunkEntities`.
#[derive(Debug, Clone)]
pub struct ChunkEntities<E> {
    pos: ChunkPos,
    entities: Vec<E>,
}

impl<E> ChunkEntities<E> {
    pub fn new(pos: ChunkPos, entities: Vec<E>) -> Self {
        Self { pos, entities }
    }

    pub fn get_pos(&self) -> ChunkPos {
        self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn get_entities(&self) -> &[E] {
        &self.entities
    }
}

struct NoopWake;

impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
    fn wake_by_ref(self: &Arc<Self>) {}
}

fn noop_waker() -> Waker {
    Waker::from(Arc::new(NoopWake))
}

struct ThreadWaker(thread::Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<T>(mut fut: BoxFuture<T>) -> T {
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => thread::park(),
        }
    }
}

struct OneshotInner<T> {
    value: Mutex<Option<T>>,
    waker: Mutex<Option<Waker>>,
}

struct OneshotSender<T>(Arc<OneshotInner<T>>);
struct OneshotReceiver<T>(Arc<OneshotInner<T>>);

fn oneshot<T>() -> (OneshotSender<T>, OneshotReceiver<T>) {
    let inner = Arc::new(OneshotInner {
        value: Mutex::new(None),
        waker: Mutex::new(None),
    });
    (OneshotSender(inner.clone()), OneshotReceiver(inner))
}

impl<T> OneshotSender<T> {
    fn send(self, val: T) {
        if let Ok(mut slot) = self.0.value.lock() {
            *slot = Some(val);
        }
        if let Ok(mut w) = self.0.waker.lock() {
            if let Some(waker) = w.take() {
                waker.wake();
            }
        }
    }
}

impl<T: Send + 'static> Future for OneshotReceiver<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        if let Ok(mut slot) = self.0.value.lock() {
            if let Some(val) = slot.take() {
                return Poll::Ready(val);
            }
        }
        if let Ok(mut w) = self.0.waker.lock() {
            *w = Some(cx.waker().clone());
        }
        if let Ok(mut slot) = self.0.value.lock() {
            if let Some(val) = slot.take() {
                return Poll::Ready(val);
            }
        }
        Poll::Pending
    }
}

/// Abstraction over `net.minecraft.world.level.chunk.storage.SimpleRegionStorage`.
pub trait SimpleRegionStorage: Send + Sync + 'static {
    type Tag: Send + 'static;
    type Error: fmt::Display + fmt::Debug + Send + Sync + 'static;

    fn read(&self, pos: ChunkPos) -> BoxFuture<Result<Option<Self::Tag>, Self::Error>>;
    fn write(&self, pos: ChunkPos, tag: Option<Self::Tag>) -> BoxFuture<Result<(), Self::Error>>;
    fn upgrade_chunk_tag(&self, tag: Self::Tag, version: i32) -> Self::Tag;
    fn storage_info(&self) -> String;
    fn synchronize(&self, flush: bool) -> BoxFuture<()>;
    fn close(&self) -> io::Result<()>;
}

/// Abstraction over server-level entity serialization and error reporting.
pub trait EntityLevel: Send + Sync + 'static {
    type Tag: Send + 'static;
    type Entity: Send + 'static;

    fn read_chunk_position(&self, tag: &Self::Tag) -> Result<ChunkPos, String>;
    fn load_entities(&self, pos: ChunkPos, tag: &Self::Tag) -> Vec<Self::Entity>;
    fn save_entities(&self, pos: ChunkPos, entities: &[Self::Entity]) -> Self::Tag;
    fn report_misplaced_chunk(&self, found: ChunkPos, expected: ChunkPos, storage_info: &str);
    fn report_chunk_load_failure(&self, error: &str, storage_info: &str, pos: ChunkPos);
    fn report_chunk_save_failure(&self, error: &str, storage_info: &str, pos: ChunkPos);
}

/// Abstraction over `net.minecraft.util.thread.ConsecutiveExecutor`.
pub trait ConsecutiveExecutor: Send + Sync + 'static {
    fn schedule(&self, task: Box<dyn FnOnce() + Send + 'static>);
    fn run_all(&self);
}

fn then_apply_async_ok<T, U, E, Q>(
    mut input: BoxFuture<Result<T, E>>,
    executor: Arc<Q>,
    f: impl FnOnce(T) -> U + Send + 'static,
) -> BoxFuture<Result<U, E>>
where
    T: Send + 'static,
    U: Send + 'static,
    E: Send + 'static,
    Q: ConsecutiveExecutor + ?Sized,
{
    let (tx, rx) = oneshot::<Result<U, E>>();
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    match input.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(val)) => {
            executor.schedule(Box::new(move || {
                tx.send(Ok(f(val)));
            }));
        }
        Poll::Ready(Err(e)) => {
            tx.send(Err(e));
        }
        Poll::Pending => {
            thread::spawn(move || match block_on(input) {
                Ok(val) => {
                    executor.schedule(Box::new(move || {
                        tx.send(Ok(f(val)));
                    }));
                }
                Err(e) => {
                    tx.send(Err(e));
                }
            });
        }
    }
    Box::pin(rx)
}

/// Port of `net.minecraft.world.level.chunk.storage.EntityStorage`.
pub struct EntityStorage<S, L, Q>
where
    S: SimpleRegionStorage,
    L: EntityLevel<Tag = S::Tag>,
    Q: ConsecutiveExecutor,
{
    simple_region_storage: Arc<S>,
    level: Arc<L>,
    empty_chunks: Arc<Mutex<HashSet<i64>>>,
    entity_deserializer_queue: Arc<Q>,
}

impl<S, L, Q> EntityStorage<S, L, Q>
where
    S: SimpleRegionStorage,
    L: EntityLevel<Tag = S::Tag>,
    Q: ConsecutiveExecutor,
{
    // JAVA: net.minecraft.world.level.chunk.storage.EntityStorage#<init>
    pub fn new(simple_region_storage: S, level: L, entity_deserializer_queue: Q) -> Self {
        Self {
            simple_region_storage: Arc::new(simple_region_storage),
            level: Arc::new(level),
            empty_chunks: Arc::new(Mutex::new(HashSet::new())),
            entity_deserializer_queue: Arc::new(entity_deserializer_queue),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.storage.EntityStorage#emptyChunk
    fn empty_chunk(pos: ChunkPos) -> ChunkEntities<L::Entity> {
        ChunkEntities::new(pos, Vec::new())
    }

    // JAVA: net.minecraft.world.level.chunk.storage.EntityStorage#loadEntities
    pub fn load_entities(
        &self,
        pos: ChunkPos,
    ) -> BoxFuture<Result<ChunkEntities<L::Entity>, S::Error>>
    where
        L::Entity: Send + 'static,
    {
        if self
            .empty_chunks
            .lock()
            .expect("empty_chunks poisoned")
            .contains(&pos.to_long())
        {
            return Box::pin(std::future::ready(Ok(Self::empty_chunk(pos))));
        }

        let read_fut = self.simple_region_storage.read(pos);
        let read_fut = self.report_load_failure_if_present(read_fut, pos);

        let storage = Arc::clone(&self.simple_region_storage);
        let level = Arc::clone(&self.level);
        let empty_chunks = Arc::clone(&self.empty_chunks);
        let executor = Arc::clone(&self.entity_deserializer_queue);

        then_apply_async_ok(read_fut, executor, move |opt_tag| {
            let Some(tag) = opt_tag else {
                empty_chunks
                    .lock()
                    .expect("empty_chunks poisoned")
                    .insert(pos.to_long());
                return Self::empty_chunk(pos);
            };

            match level.read_chunk_position(&tag) {
                Ok(found) => {
                    if found != pos {
                        eprintln!(
                            "ERROR: Chunk file at {} is in the wrong location. (Expected {}, got {})",
                            pos, pos, found
                        );
                        let storage_info = storage.storage_info();
                        level.report_misplaced_chunk(found, pos, &storage_info);
                    }
                }
                Err(err) => {
                    eprintln!(
                        "WARN: Failed to parse chunk {} position info: {}",
                        pos, err
                    );
                    let storage_info = storage.storage_info();
                    level.report_chunk_load_failure(&err, &storage_info, pos);
                }
            }

            let upgraded = storage.upgrade_chunk_tag(tag, -1);
            let entities = level.load_entities(pos, &upgraded);
            ChunkEntities::new(pos, entities)
        })
    }

    // JAVA: net.minecraft.world.level.chunk.storage.EntityStorage#storeEntities
    pub fn store_entities(&self, entities: &ChunkEntities<L::Entity>) {
        let pos = entities.get_pos();
        if entities.is_empty() {
            let mut empty = self.empty_chunks.lock().expect("empty_chunks poisoned");
            if empty.insert(pos.to_long()) {
                let write_fut = self.simple_region_storage.write(pos, None);
                drop(empty);
                self.report_save_failure_if_present(write_fut, pos);
            }
        } else {
            let tag = self.level.save_entities(pos, entities.get_entities());
            let write_fut = self.simple_region_storage.write(pos, Some(tag));
            self.report_save_failure_if_present(write_fut, pos);
            self.empty_chunks
                .lock()
                .expect("empty_chunks poisoned")
                .remove(&pos.to_long());
        }
    }

    // JAVA: net.minecraft.world.level.chunk.storage.EntityStorage#reportSaveFailureIfPresent
    fn report_save_failure_if_present<E>(&self, mut fut: BoxFuture<Result<(), E>>, pos: ChunkPos)
    where
        E: fmt::Display + Send + 'static,
    {
        let storage_info = self.simple_region_storage.storage_info();
        let level = Arc::clone(&self.level);
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(Err(err)) => {
                let msg = err.to_string();
                eprintln!("ERROR: Failed to store entity chunk {}: {}", pos, msg);
                level.report_chunk_save_failure(&msg, &storage_info, pos);
            }
            Poll::Ready(Ok(())) => {}
            Poll::Pending => {
                thread::spawn(move || {
                    if let Err(err) = block_on(fut) {
                        let msg = err.to_string();
                        eprintln!("ERROR: Failed to store entity chunk {}: {}", pos, msg);
                        level.report_chunk_save_failure(&msg, &storage_info, pos);
                    }
                });
            }
        }
    }

    // JAVA: net.minecraft.world.level.chunk.storage.EntityStorage#reportLoadFailureIfPresent
    fn report_load_failure_if_present<T, E>(
        &self,
        mut fut: BoxFuture<Result<T, E>>,
        pos: ChunkPos,
    ) -> BoxFuture<Result<T, E>>
    where
        T: Send + 'static,
        E: fmt::Display + Send + 'static,
    {
        let storage_info = self.simple_region_storage.storage_info();
        let level = Arc::clone(&self.level);
        let (tx, rx) = oneshot::<Result<T, E>>();
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(result) => {
                if let Err(ref err) = result {
                    let msg = err.to_string();
                    eprintln!("ERROR: Failed to load entity chunk {}: {}", pos, msg);
                    level.report_chunk_load_failure(&msg, &storage_info, pos);
                }
                tx.send(result);
            }
            Poll::Pending => {
                thread::spawn(move || {
                    let result = block_on(fut);
                    if let Err(ref err) = result {
                        let msg = err.to_string();
                        eprintln!("ERROR: Failed to load entity chunk {}: {}", pos, msg);
                        level.report_chunk_load_failure(&msg, &storage_info, pos);
                    }
                    tx.send(result);
                });
            }
        }
        Box::pin(rx)
    }

    // JAVA: net.minecraft.world.level.chunk.storage.EntityStorage#flush
    pub fn flush(&self, sync: bool) {
        block_on(self.simple_region_storage.synchronize(sync));
        self.entity_deserializer_queue.run_all();
    }

    // JAVA: net.minecraft.world.level.chunk.storage.EntityStorage#close
    pub fn close(&self) -> io::Result<()> {
        self.simple_region_storage.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct MockTag {
        data: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct MockEntity(i32);

    #[derive(Clone)]
    struct MockStorage {
        read_result: Arc<Mutex<Option<Result<Option<MockTag>, TestError>>>>,
        writes: Arc<Mutex<Vec<(ChunkPos, Option<MockTag>)>>>,
        sync_count: Arc<AtomicUsize>,
        closed: Arc<Mutex<bool>>,
    }

    impl MockStorage {
        fn new_ok(tag: Option<MockTag>) -> Self {
            Self {
                read_result: Arc::new(Mutex::new(Some(Ok(tag)))),
                writes: Arc::new(Mutex::new(Vec::new())),
                sync_count: Arc::new(AtomicUsize::new(0)),
                closed: Arc::new(Mutex::new(false)),
            }
        }

        fn new_err(msg: &str) -> Self {
            Self {
                read_result: Arc::new(Mutex::new(Some(Err(TestError(msg.to_string()))))),
                writes: Arc::new(Mutex::new(Vec::new())),
                sync_count: Arc::new(AtomicUsize::new(0)),
                closed: Arc::new(Mutex::new(false)),
            }
        }
    }

    impl SimpleRegionStorage for MockStorage {
        type Tag = MockTag;
        type Error = TestError;

        fn read(&self, _pos: ChunkPos) -> BoxFuture<Result<Option<Self::Tag>, Self::Error>> {
            let result = self.read_result.lock().unwrap().take().unwrap_or(Ok(None));
            Box::pin(std::future::ready(result))
        }

        fn write(
            &self,
            pos: ChunkPos,
            tag: Option<Self::Tag>,
        ) -> BoxFuture<Result<(), Self::Error>> {
            self.writes.lock().unwrap().push((pos, tag));
            Box::pin(std::future::ready(Ok(())))
        }

        fn upgrade_chunk_tag(&self, tag: Self::Tag, _version: i32) -> Self::Tag {
            tag
        }

        fn storage_info(&self) -> String {
            "mock-storage".to_string()
        }

        fn synchronize(&self, _flush: bool) -> BoxFuture<()> {
            self.sync_count.fetch_add(1, Ordering::SeqCst);
            Box::pin(std::future::ready(()))
        }

        fn close(&self) -> io::Result<()> {
            *self.closed.lock().unwrap() = true;
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockLevel {
        load_result: Arc<Mutex<Vec<MockEntity>>>,
        save_tag: Arc<Mutex<MockTag>>,
        position_result: Arc<Mutex<Option<Result<ChunkPos, String>>>>,
        misplaced: Arc<Mutex<Vec<(ChunkPos, ChunkPos)>>>,
        load_failures: Arc<Mutex<Vec<String>>>,
        save_failures: Arc<Mutex<Vec<String>>>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                load_result: Arc::new(Mutex::new(Vec::new())),
                save_tag: Arc::new(Mutex::new(MockTag {
                    data: "saved".to_string(),
                })),
                position_result: Arc::new(Mutex::new(None)),
                misplaced: Arc::new(Mutex::new(Vec::new())),
                load_failures: Arc::new(Mutex::new(Vec::new())),
                save_failures: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl EntityLevel for MockLevel {
        type Tag = MockTag;
        type Entity = MockEntity;

        fn read_chunk_position(&self, _tag: &Self::Tag) -> Result<ChunkPos, String> {
            self.position_result
                .lock()
                .unwrap()
                .clone()
                .unwrap_or(Ok(ChunkPos::new(0, 0)))
        }

        fn load_entities(&self, _pos: ChunkPos, _tag: &Self::Tag) -> Vec<Self::Entity> {
            self.load_result.lock().unwrap().clone()
        }

        fn save_entities(&self, _pos: ChunkPos, _entities: &[Self::Entity]) -> Self::Tag {
            self.save_tag.lock().unwrap().clone()
        }

        fn report_misplaced_chunk(
            &self,
            found: ChunkPos,
            expected: ChunkPos,
            _storage_info: &str,
        ) {
            self.misplaced.lock().unwrap().push((found, expected));
        }

        fn report_chunk_load_failure(&self, error: &str, _storage_info: &str, _pos: ChunkPos) {
            self.load_failures.lock().unwrap().push(error.to_string());
        }

        fn report_chunk_save_failure(&self, error: &str, _storage_info: &str, _pos: ChunkPos) {
            self.save_failures.lock().unwrap().push(error.to_string());
        }
    }

    struct MockExecutor;

    impl ConsecutiveExecutor for MockExecutor {
        fn schedule(&self, task: Box<dyn FnOnce() + Send + 'static>) {
            task();
        }

        fn run_all(&self) {}
    }

    #[test]
    fn chunk_pos_to_long_negative_and_extreme_bits() {
        let pos = ChunkPos::new(-1, -1);
        assert_eq!(pos.to_long(), -1_i64);

        let pos2 = ChunkPos::new(i32::MIN, i32::MAX);
        let expected = ((i32::MIN as i64) & 0xFFFF_FFFF_i64)
            | (((i32::MAX as i64) & 0xFFFF_FFFF_i64) << 32);
        assert_eq!(pos2.to_long(), expected);

        let packed = pos2.to_long();
        let x_back = (packed & 0xFFFF_FFFF_i64) as i32;
        let z_back = ((packed as u64) >> 32) as i32;
        assert_eq!(x_back, i32::MIN);
        assert_eq!(z_back, i32::MAX);
    }

    #[test]
    fn load_entities_returns_cached_empty_chunk() {
        let storage = MockStorage::new_ok(Some(MockTag {
            data: "payload".to_string(),
        }));
        let read_result = storage.read_result.clone();
        let es = EntityStorage::new(storage, MockLevel::new(), MockExecutor);

        let pos = ChunkPos::new(1, 2);
        es.empty_chunks.lock().unwrap().insert(pos.to_long());

        let result = block_on(es.load_entities(pos)).unwrap();
        assert!(result.is_empty());
        assert_eq!(result.get_pos(), pos);
        assert!(
            read_result.lock().unwrap().is_some(),
            "storage read should be skipped"
        );
    }

    #[test]
    fn load_entities_empty_read_marks_empty_chunk() {
        let pos = ChunkPos::new(4, -9);
        let storage = MockStorage::new_ok(None);
        let es = EntityStorage::new(storage, MockLevel::new(), MockExecutor);

        let result = block_on(es.load_entities(pos)).unwrap();
        assert!(result.is_empty());
        assert_eq!(result.get_pos(), pos);
        assert!(
            es.empty_chunks.lock().unwrap().contains(&pos.to_long()),
            "empty chunk should be cached"
        );
    }

    #[test]
    fn store_empty_chunk_deduplicates_write() {
        let storage = MockStorage::new_ok(None);
        let writes = storage.writes.clone();
        let es = EntityStorage::new(storage, MockLevel::new(), MockExecutor);

        let pos = ChunkPos::new(7, 3);
        let empty = ChunkEntities::new(pos, Vec::<MockEntity>::new());
        es.store_entities(&empty);
        es.store_entities(&empty);

        let writes = writes.lock().unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, pos);
        assert!(writes[0].1.is_none());
    }

    #[test]
    fn store_entities_clears_empty_cache_and_writes_tag() {
        let storage = MockStorage::new_ok(None);
        let writes = storage.writes.clone();
        let es = EntityStorage::new(storage, MockLevel::new(), MockExecutor);

        let pos = ChunkPos::new(-2, 5);
        es.empty_chunks.lock().unwrap().insert(pos.to_long());

        let entities = ChunkEntities::new(pos, vec![MockEntity(1), MockEntity(2)]);
        es.store_entities(&entities);

        assert!(!es.empty_chunks.lock().unwrap().contains(&pos.to_long()));
        let writes = writes.lock().unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, pos);
        assert_eq!(writes[0].1.as_ref().unwrap().data, "saved");
    }

    #[test]
    fn load_reports_read_failure_and_position_mismatch() {
        // Read failure should be reported and propagated.
        let storage = MockStorage::new_err("disk boom");
        let level = MockLevel::new();
        let failures = level.load_failures.clone();
        let es = EntityStorage::new(storage, level, MockExecutor);
        let pos = ChunkPos::new(1, 2);
        let err = block_on(es.load_entities(pos)).unwrap_err();
        assert!(err.to_string().contains("disk boom"));
        assert_eq!(failures.lock().unwrap().len(), 1);

        // Position mismatch should be reported but entities still load.
        let storage2 = MockStorage::new_ok(Some(MockTag {
            data: "ok".to_string(),
        }));
        let level2 = MockLevel::new();
        *level2.position_result.lock().unwrap() = Some(Ok(ChunkPos::new(9, 9)));
        *level2.load_result.lock().unwrap() = vec![MockEntity(42)];
        let misplaced = level2.misplaced.clone();
        let load_failures2 = level2.load_failures.clone();
        let es2 = EntityStorage::new(storage2, level2, MockExecutor);
        let pos2 = ChunkPos::new(1, 2);
        let loaded = block_on(es2.load_entities(pos2)).unwrap();
        assert_eq!(loaded.get_pos(), pos2);
        assert_eq!(loaded.get_entities(), &[MockEntity(42)]);
        let m = misplaced.lock().unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0], (ChunkPos::new(9, 9), pos2));
        assert!(load_failures2.lock().unwrap().is_empty());
    }
}
