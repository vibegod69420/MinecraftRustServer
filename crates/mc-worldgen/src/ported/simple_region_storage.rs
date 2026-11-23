use std::collections::BTreeMap;
use std::fmt;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, OnceLock};

pub type BoxFutureOwned<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

static CURRENT_DATA_VERSION: AtomicI32 = AtomicI32::new(0);

/// Stand-in for `SharedConstants.getCurrentVersion().dataVersion().version()`.
pub fn current_data_version() -> i32 {
    CURRENT_DATA_VERSION.load(Ordering::Relaxed)
}

/// Test/configuration hook for the current data version.
pub fn set_current_data_version(version: i32) {
    CURRENT_DATA_VERSION.store(version, Ordering::Relaxed);
}

/// Minimal stand-in for `net.minecraft.nbt.NbtUtils#getDataVersion`.
pub fn nbt_get_data_version(tag: &CompoundTag, fallback: i32) -> i32 {
    match tag.get("DataVersion") {
        Some(Tag::Int(v)) => *v,
        _ => fallback,
    }
}

/// Minimal stand-in for `net.minecraft.nbt.NbtUtils#addCurrentDataVersion`.
pub fn nbt_add_current_data_version(tag: &mut CompoundTag) {
    tag.put(
        "DataVersion".to_string(),
        Tag::Int(current_data_version()),
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Compound(CompoundTag),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CompoundTag {
    pub entries: BTreeMap<String, Tag>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn put(&mut self, key: String, value: Tag) -> Option<Tag> {
        self.entries.insert(key, value)
    }

    pub fn remove(&mut self, key: &str) -> Option<Tag> {
        self.entries.remove(key)
    }

    pub fn get(&self, key: &str) -> Option<&Tag> {
        self.entries.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DynamicOps;

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic<T> {
    ops: DynamicOps,
    value: T,
}

impl<T> Dynamic<T> {
    pub fn new(ops: DynamicOps, value: T) -> Self {
        Self { ops, value }
    }

    pub fn get_ops(&self) -> DynamicOps {
        self.ops
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionStorageInfo {
    pub level: String,
    pub type_name: String,
}

impl RegionStorageInfo {
    pub fn new(level: String, type_name: String) -> Self {
        Self { level, type_name }
    }
}

#[derive(Debug)]
pub struct ReportedException {
    pub message: String,
    pub data_version: i32,
}

impl fmt::Display for ReportedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (Data version: {})", self.message, self.data_version)
    }
}

impl std::error::Error for ReportedException {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataFixTypes {
    Level,
    Player,
    Chunk,
    Hotbar,
    Options,
    Structure,
    Stats,
    SavedDataRandomSequences,
    SavedDataMapData,
    SavedDataMapIndex,
    SavedDataScoreboard,
    SavedDataStructureFeatureIndices,
    Advancements,
    PoiChunk,
    EntityChunk,
}

pub trait DataFixer: Send + Sync {
    fn update(&self, data_fix_type: DataFixTypes, tag: CompoundTag, version: i32) -> CompoundTag;
}

impl DataFixTypes {
    pub fn update_to_current_version(
        &self,
        fixer: &dyn DataFixer,
        tag: CompoundTag,
        version: i32,
    ) -> CompoundTag {
        fixer.update(*self, tag, version)
    }
}

pub trait LegacyTagFixer: Send + Sync {
    fn apply_fix(&self, tag: CompoundTag) -> CompoundTag;
    fn mark_chunk_done(&self, pos: &ChunkPos);
    fn target_data_version(&self) -> i32;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EmptyLegacyTagFixer;

impl LegacyTagFixer for EmptyLegacyTagFixer {
    fn apply_fix(&self, tag: CompoundTag) -> CompoundTag {
        tag
    }

    fn mark_chunk_done(&self, _pos: &ChunkPos) {}

    fn target_data_version(&self) -> i32 {
        0
    }
}

pub trait ChunkScanAccess: Send + Sync {}

pub trait IOWorker: ChunkScanAccess {
    fn is_old_chunk_around(&self, pos: &ChunkPos, radius: i32) -> bool;
    fn load_async(&self, pos: &ChunkPos) -> BoxFutureOwned<Option<CompoundTag>>;
    fn store(
        &self,
        pos: &ChunkPos,
        supplier: Box<dyn FnOnce() -> CompoundTag + Send + 'static>,
    ) -> BoxFutureOwned<()>;
    fn synchronize(&self, sync: bool) -> BoxFutureOwned<()>;
    fn close(&self) -> io::Result<()>;
    fn storage_info(&self) -> &RegionStorageInfo;
}

struct MemoizedLegacyFixer {
    init: OnceLock<Arc<dyn LegacyTagFixer>>,
    supplier: Box<dyn Fn() -> Arc<dyn LegacyTagFixer> + Send + Sync>,
}

impl MemoizedLegacyFixer {
    fn new(supplier: Box<dyn Fn() -> Arc<dyn LegacyTagFixer> + Send + Sync>) -> Self {
        Self {
            init: OnceLock::new(),
            supplier,
        }
    }

    fn get(&self) -> &Arc<dyn LegacyTagFixer> {
        self.init.get_or_init(|| (self.supplier)())
    }
}

pub struct SimpleRegionStorage {
    worker: Arc<dyn IOWorker>,
    fixer_upper: Arc<dyn DataFixer>,
    data_fix_type: DataFixTypes,
    legacy_fixer: MemoizedLegacyFixer,
}

impl SimpleRegionStorage {
    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#<init>
    pub fn new(
        worker: Arc<dyn IOWorker>,
        fixer_upper: Arc<dyn DataFixer>,
        data_fix_type: DataFixTypes,
    ) -> Self {
        Self::new_with_legacy_fixer(
            worker,
            fixer_upper,
            data_fix_type,
            Box::new(|| Arc::new(EmptyLegacyTagFixer)),
        )
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#<init>
    pub fn new_with_legacy_fixer(
        worker: Arc<dyn IOWorker>,
        fixer_upper: Arc<dyn DataFixer>,
        data_fix_type: DataFixTypes,
        legacy_fixer_supplier: Box<dyn Fn() -> Arc<dyn LegacyTagFixer> + Send + Sync>,
    ) -> Self {
        Self {
            fixer_upper,
            data_fix_type,
            worker,
            legacy_fixer: MemoizedLegacyFixer::new(legacy_fixer_supplier),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#isOldChunkAround
    pub fn is_old_chunk_around(&self, pos: &ChunkPos, radius: i32) -> bool {
        self.worker.is_old_chunk_around(pos, radius)
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#read
    pub fn read(&self, pos: &ChunkPos) -> BoxFutureOwned<Option<CompoundTag>> {
        self.worker.load_async(pos)
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#write
    pub fn write(&self, pos: ChunkPos, tag: CompoundTag) -> BoxFutureOwned<()> {
        self.write_with_supplier(pos, Box::new(move || tag))
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#write
    pub fn write_with_supplier(
        &self,
        pos: ChunkPos,
        supplier: Box<dyn FnOnce() -> CompoundTag + Send + 'static>,
    ) -> BoxFutureOwned<()> {
        self.mark_chunk_done(&pos);
        self.worker.store(&pos, supplier)
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#upgradeChunkTag
    pub fn upgrade_chunk_tag(
        &self,
        tag: CompoundTag,
        fallback_version: i32,
        context: Option<CompoundTag>,
    ) -> Result<CompoundTag, ReportedException> {
        let data_version = nbt_get_data_version(&tag, fallback_version);
        if data_version == current_data_version() {
            return Ok(tag);
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut tag = tag;
            tag = self.legacy_fixer.get().apply_fix(tag);
            Self::inject_datafixing_context(&mut tag, context.as_ref());
            let fix_version =
                std::cmp::max(self.legacy_fixer.get().target_data_version(), data_version);
            tag = self
                .data_fix_type
                .update_to_current_version(self.fixer_upper.as_ref(), tag, fix_version);
            Self::remove_datafixing_context(&mut tag);
            nbt_add_current_data_version(&mut tag);
            tag
        }));

        match result {
            Ok(tag) => Ok(tag),
            Err(_) => Err(ReportedException {
                message: "Updated chunk".to_string(),
                data_version,
            }),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#upgradeChunkTag
    pub fn upgrade_chunk_tag_with_fallback(
        &self,
        tag: CompoundTag,
        fallback_version: i32,
    ) -> Result<CompoundTag, ReportedException> {
        self.upgrade_chunk_tag(tag, fallback_version, None)
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#upgradeChunkTag
    pub fn upgrade_chunk_tag_dynamic(
        &self,
        tag: Dynamic<Tag>,
        fallback_version: i32,
    ) -> Result<Dynamic<Tag>, ReportedException> {
        let ops = tag.get_ops();
        let compound = match tag.into_value() {
            Tag::Compound(c) => c,
            _ => {
                return Err(ReportedException {
                    message: "Updated chunk: Dynamic value is not a CompoundTag".to_string(),
                    data_version: fallback_version,
                });
            }
        };
        let upgraded = self.upgrade_chunk_tag(compound, fallback_version, None)?;
        Ok(Dynamic::new(ops, Tag::Compound(upgraded)))
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#injectDatafixingContext
    pub fn inject_datafixing_context(tag: &mut CompoundTag, context: Option<&CompoundTag>) {
        if let Some(ctx) = context {
            tag.put("__context".to_string(), Tag::Compound(ctx.clone()));
        }
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#removeDatafixingContext
    fn remove_datafixing_context(tag: &mut CompoundTag) {
        tag.remove("__context");
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#markChunkDone
    pub fn mark_chunk_done(&self, pos: &ChunkPos) {
        self.legacy_fixer.get().mark_chunk_done(pos);
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#synchronize
    pub fn synchronize(&self, sync: bool) -> BoxFutureOwned<()> {
        self.worker.synchronize(sync)
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#close
    pub fn close(&self) -> io::Result<()> {
        self.worker.close()
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#chunkScanner
    pub fn chunk_scanner(&self) -> Arc<dyn ChunkScanAccess> {
        let worker: Arc<dyn IOWorker> = Arc::clone(&self.worker);
        worker
    }

    // JAVA: net.minecraft.world.level.chunk.storage.SimpleRegionStorage#storageInfo
    pub fn storage_info(&self) -> &RegionStorageInfo {
        self.worker.storage_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    struct MockDataFixer;

    impl DataFixer for MockDataFixer {
        fn update(
            &self,
            _data_fix_type: DataFixTypes,
            mut tag: CompoundTag,
            version: i32,
        ) -> CompoundTag {
            tag.put("fixed_from".to_string(), Tag::Int(version));
            tag
        }
    }

    struct CountingLegacyFixer {
        target: i32,
        marks: Arc<AtomicUsize>,
    }

    impl LegacyTagFixer for CountingLegacyFixer {
        fn apply_fix(&self, mut tag: CompoundTag) -> CompoundTag {
            tag.put("legacy_fixed".to_string(), Tag::Int(1));
            tag
        }

        fn mark_chunk_done(&self, _pos: &ChunkPos) {
            self.marks.fetch_add(1, AtomicOrdering::Relaxed);
        }

        fn target_data_version(&self) -> i32 {
            self.target
        }
    }

    struct MockWorker {
        info: RegionStorageInfo,
    }

    impl ChunkScanAccess for MockWorker {}

    impl IOWorker for MockWorker {
        fn is_old_chunk_around(&self, _pos: &ChunkPos, _radius: i32) -> bool {
            false
        }

        fn load_async(&self, _pos: &ChunkPos) -> BoxFutureOwned<Option<CompoundTag>> {
            Box::pin(std::future::ready(None))
        }

        fn store(
            &self,
            _pos: &ChunkPos,
            _supplier: Box<dyn FnOnce() -> CompoundTag + Send + 'static>,
        ) -> BoxFutureOwned<()> {
            Box::pin(std::future::ready(()))
        }

        fn synchronize(&self, _sync: bool) -> BoxFutureOwned<()> {
            Box::pin(std::future::ready(()))
        }

        fn close(&self) -> io::Result<()> {
            Ok(())
        }

        fn storage_info(&self) -> &RegionStorageInfo {
            &self.info
        }
    }

    fn make_storage(
        current_version: i32,
        legacy_target: i32,
        marks: Arc<AtomicUsize>,
    ) -> SimpleRegionStorage {
        set_current_data_version(current_version);
        let worker = Arc::new(MockWorker {
            info: RegionStorageInfo::new("overworld".to_string(), "chunk".to_string()),
        });
        let fixer = Arc::new(MockDataFixer);
        let legacy = Arc::new(CountingLegacyFixer {
            target: legacy_target,
            marks,
        });
        let legacy_supplier = Arc::clone(&legacy);
        SimpleRegionStorage::new_with_legacy_fixer(
            worker,
            fixer,
            DataFixTypes::Chunk,
            Box::new(move || Arc::clone(&legacy_supplier) as Arc<dyn LegacyTagFixer>),
        )
    }

    #[test]
    fn inject_and_remove_datafixing_context() {
        let mut tag = CompoundTag::new();
        let mut ctx = CompoundTag::new();
        ctx.put("source".to_string(), Tag::String("test".to_string()));

        SimpleRegionStorage::inject_datafixing_context(&mut tag, Some(&ctx));
        assert!(tag.contains_key("__context"));
        match tag.get("__context") {
            Some(Tag::Compound(c)) => {
                assert_eq!(c.get("source"), Some(&Tag::String("test".to_string())));
            }
            other => panic!("expected compound context, got {:?}", other),
        }

        let mut tag2 = CompoundTag::new();
        SimpleRegionStorage::inject_datafixing_context(&mut tag2, None);
        assert!(!tag2.contains_key("__context"));

        SimpleRegionStorage::remove_datafixing_context(&mut tag);
        assert!(!tag.contains_key("__context"));
    }

    #[test]
    fn upgrade_skips_when_version_is_current_and_marks_chunk_done() {
        let _guard = TEST_LOCK.lock().unwrap();
        let marks = Arc::new(AtomicUsize::new(0));
        let storage = make_storage(100, 0, Arc::clone(&marks));

        let mut tag = CompoundTag::new();
        tag.put("DataVersion".to_string(), Tag::Int(100));
        tag.put("keep".to_string(), Tag::Int(42));

        let out = storage.upgrade_chunk_tag(tag.clone(), 0, None).unwrap();
        assert_eq!(out, tag);

        storage.mark_chunk_done(&ChunkPos::new(-3, 7));
        assert_eq!(marks.load(AtomicOrdering::Relaxed), 1);
    }

    #[test]
    fn upgrade_applies_legacy_fixer_datafixer_and_context() {
        let _guard = TEST_LOCK.lock().unwrap();
        let marks = Arc::new(AtomicUsize::new(0));
        let storage = make_storage(200, 150, marks);

        let mut tag = CompoundTag::new();
        tag.put("DataVersion".to_string(), Tag::Int(100));
        let mut ctx = CompoundTag::new();
        ctx.put("ctx".to_string(), Tag::Int(7));

        let out = storage.upgrade_chunk_tag(tag, 0, Some(ctx)).unwrap();
        assert_eq!(out.get("legacy_fixed"), Some(&Tag::Int(1)));
        // fix version = max(legacy target 150, data version 100) = 150
        assert_eq!(out.get("fixed_from"), Some(&Tag::Int(150)));
        assert_eq!(out.get("DataVersion"), Some(&Tag::Int(200)));
        assert!(!out.contains_key("__context"));
    }

    #[test]
    fn upgrade_handles_negative_and_boundary_versions() {
        let _guard = TEST_LOCK.lock().unwrap();

        // negative fallback, no DataVersion tag
        let storage = make_storage(10, 0, Arc::new(AtomicUsize::new(0)));
        let out = storage
            .upgrade_chunk_tag_with_fallback(CompoundTag::new(), -1)
            .unwrap();
        assert_eq!(out.get("fixed_from"), Some(&Tag::Int(0))); // max(0, -1)
        assert_eq!(out.get("DataVersion"), Some(&Tag::Int(10)));

        // i32::MIN fallback
        set_current_data_version(0);
        let out2 = storage
            .upgrade_chunk_tag_with_fallback(CompoundTag::new(), i32::MIN)
            .unwrap();
        assert_eq!(out2.get("fixed_from"), Some(&Tag::Int(0))); // max(0, i32::MIN)
        assert_eq!(out2.get("DataVersion"), Some(&Tag::Int(0)));

        // near i32::MAX
        set_current_data_version(i32::MAX);
        let mut tag3 = CompoundTag::new();
        tag3.put("DataVersion".to_string(), Tag::Int(i32::MAX - 1));
        let out3 = storage.upgrade_chunk_tag_with_fallback(tag3, 0).unwrap();
        assert_eq!(out3.get("fixed_from"), Some(&Tag::Int(i32::MAX - 1)));
        assert_eq!(out3.get("DataVersion"), Some(&Tag::Int(i32::MAX)));

        // negative current/data version equality should skip
        set_current_data_version(-5);
        let mut tag4 = CompoundTag::new();
        tag4.put("DataVersion".to_string(), Tag::Int(-5));
        let out4 = storage.upgrade_chunk_tag(tag4.clone(), 0, None).unwrap();
        assert_eq!(out4, tag4);
    }

    #[test]
    fn float_bits_preserved_and_dynamic_upgrade() {
        let _guard = TEST_LOCK.lock().unwrap();
        let storage = make_storage(5, 0, Arc::new(AtomicUsize::new(0)));

        let mut tag = CompoundTag::new();
        let f_bits = f32::NAN.to_bits();
        let d_bits = f64::NEG_INFINITY.to_bits();
        tag.put("f".to_string(), Tag::Float(f32::from_bits(f_bits)));
        tag.put("d".to_string(), Tag::Double(f64::from_bits(d_bits)));
        tag.put("DataVersion".to_string(), Tag::Int(1));

        let dyn_tag = Dynamic::new(DynamicOps, Tag::Compound(tag));
        let out_dyn = storage.upgrade_chunk_tag_dynamic(dyn_tag, 0).unwrap();
        let out = match out_dyn.into_value() {
            Tag::Compound(c) => c,
            _ => panic!("expected compound"),
        };

        match out.get("f") {
            Some(Tag::Float(f)) => assert_eq!(f.to_bits(), f_bits),
            other => panic!("expected float, got {:?}", other),
        }
        match out.get("d") {
            Some(Tag::Double(d)) => assert_eq!(d.to_bits(), d_bits),
            other => panic!("expected double, got {:?}", other),
        }
    }

    #[test]
    fn dynamic_rejects_non_compound_and_delegation_methods() {
        let _guard = TEST_LOCK.lock().unwrap();
        let storage = make_storage(5, 0, Arc::new(AtomicUsize::new(0)));

        let dyn_tag = Dynamic::new(DynamicOps, Tag::Int(123));
        let err = storage.upgrade_chunk_tag_dynamic(dyn_tag, 7).unwrap_err();
        assert_eq!(err.data_version, 7);

        assert!(!storage.is_old_chunk_around(&ChunkPos::new(1, -2), 4));
        assert_eq!(storage.storage_info().level, "overworld");
        storage.close().unwrap();
        let _scanner: Arc<dyn ChunkScanAccess> = storage.chunk_scanner();
    }
}
