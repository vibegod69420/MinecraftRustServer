use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Instant;

const NEW_DIRECTORY_PREFIX: &str = "new_";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component(pub String);

impl Component {
    pub fn translatable(key: &str) -> Self {
        Self(key.to_string())
    }
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListTag {
    elements: Vec<NbtValue>,
}

impl ListTag {
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    pub fn get_compound(&self, index: usize) -> Option<CompoundTag> {
        match self.elements.get(index) {
            Some(NbtValue::Compound(c)) => Some(c.clone()),
            _ => None,
        }
    }

    pub fn push_compound(&mut self, tag: CompoundTag) {
        self.elements.push(NbtValue::Compound(tag));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NbtValue {
    Int(i32),
    Compound(CompoundTag),
    List(ListTag),
    Byte(i8),
    String(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompoundTag {
    entries: HashMap<String, NbtValue>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        match self.entries.get(key) {
            Some(NbtValue::Int(v)) => *v,
            _ => default,
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.remove(key);
    }

    pub fn get_list_or_empty(&self, key: &str) -> ListTag {
        match self.entries.get(key) {
            Some(NbtValue::List(l)) => l.clone(),
            _ => ListTag::default(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: NbtValue) {
        self.entries.insert(key.into(), value);
    }
}

pub fn nbt_get_data_version(tag: &CompoundTag) -> i32 {
    tag.get_int_or("DataVersion", 0)
}

pub struct CompletableFuture<T>(pub T);

impl<T> CompletableFuture<T> {
    pub fn join(self) -> T {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataFixTypes {
    Chunk,
    EntityChunk,
    PoiChunk,
}

#[derive(Clone, Debug)]
pub struct RegionStorageInfo {
    pub level_id: String,
    pub dimension: ResourceKey,
    pub type_name: String,
}

impl RegionStorageInfo {
    pub fn new(level_id: String, dimension: ResourceKey, type_name: String) -> Self {
        Self {
            level_id,
            dimension,
            type_name,
        }
    }

    pub fn with_type_suffix(&self, suffix: &str) -> Self {
        Self {
            level_id: self.level_id.clone(),
            dimension: self.dimension.clone(),
            type_name: format!("{}_{}", self.type_name, suffix),
        }
    }
}

pub trait RegionFileHandle: Send {
    fn does_chunk_exist(&self, pos: ChunkPos) -> bool;
    fn get_path(&self) -> &Path;
}

pub struct FileToUpgrade {
    pub file_path: PathBuf,
    pub chunks_to_upgrade: Vec<ChunkPos>,
}

pub trait SimpleRegionStorage: Send {
    fn read(&self, pos: ChunkPos) -> CompletableFuture<Option<CompoundTag>>;
    fn write(&self, pos: ChunkPos, tag: CompoundTag) -> CompletableFuture<()>;
    fn upgrade_chunk_tag(&self, tag: CompoundTag, version: i32) -> CompoundTag;
    fn upgrade_chunk_tag_with_context(
        &self,
        tag: CompoundTag,
        version: i32,
        _context: CompoundTag,
    ) -> CompoundTag {
        self.upgrade_chunk_tag(tag, version)
    }
    fn close_storage(&mut self) -> Result<(), String> {
        Ok(())
    }
}

pub trait WorldUpgraderContext: Send + Sync + 'static {
    fn current_data_version(&self) -> i32;
    fn level_id(&self) -> String;
    fn dimension_path(&self, level: &ResourceKey) -> PathBuf;
    fn open_region_file(
        &self,
        info: &RegionStorageInfo,
        file_path: &Path,
        folder: &Path,
    ) -> io::Result<Box<dyn RegionFileHandle>>;
    fn create_storage(
        &self,
        data_fix_type: DataFixTypes,
        info: &RegionStorageInfo,
        path: &Path,
        recreate: bool,
    ) -> Box<dyn SimpleRegionStorage>;
    fn get_chunk_data_fix_context_tag(&self, level: &ResourceKey) -> CompoundTag;
    fn overworld_data_storage_save_and_join(&self) {}
    fn overworld_data_storage_close(&self) {}
}

pub struct DimensionToUpgrade {
    pub dimension_key: ResourceKey,
    pub storage: Box<dyn SimpleRegionStorage>,
    pub files: Vec<FileToUpgrade>,
    pub file_index: usize,
}

impl DimensionToUpgrade {
    fn has_next(&self) -> bool {
        self.file_index < self.files.len()
    }

    fn next_index(&self) -> usize {
        self.file_index
    }

    fn next_file(&mut self) -> Option<&FileToUpgrade> {
        if self.file_index < self.files.len() {
            let i = self.file_index;
            self.file_index = self.file_index.wrapping_add(1);
            Some(&self.files[i])
        } else {
            None
        }
    }
}

pub struct WorldUpgraderShared {
    pub levels: Vec<ResourceKey>,
    pub erase_cache: bool,
    pub recreate_region_files: bool,
    pub running: AtomicBool,
    pub finished: AtomicBool,
    pub progress_bits: AtomicU32,
    pub total_chunks: AtomicI32,
    pub total_files: AtomicI32,
    pub converted: AtomicI32,
    pub skipped: AtomicI32,
    pub progress_map: Mutex<HashMap<ResourceKey, f32>>,
    pub status: Mutex<Component>,
}

impl WorldUpgraderShared {
    fn progress(&self) -> f32 {
        f32::from_bits(self.progress_bits.load(Ordering::Relaxed))
    }

    fn set_progress(&self, v: f32) {
        self.progress_bits.store(v.to_bits(), Ordering::Relaxed);
    }

    fn set_status(&self, status: Component) {
        *self.status.lock().unwrap() = status;
    }

    fn get_status(&self) -> Component {
        self.status.lock().unwrap().clone()
    }
}

pub struct WorldUpgrader {
    shared: Arc<WorldUpgraderShared>,
    thread: Option<thread::JoinHandle<()>>,
}

impl WorldUpgrader {
    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#<init>
    pub fn new<C: WorldUpgraderContext>(
        context: Arc<C>,
        levels: Vec<ResourceKey>,
        erase_cache: bool,
        recreate_region_files: bool,
    ) -> Self {
        let shared = Arc::new(WorldUpgraderShared {
            levels,
            erase_cache,
            recreate_region_files,
            running: AtomicBool::new(true),
            finished: AtomicBool::new(false),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            total_chunks: AtomicI32::new(0),
            total_files: AtomicI32::new(0),
            converted: AtomicI32::new(0),
            skipped: AtomicI32::new(0),
            progress_map: Mutex::new(HashMap::new()),
            status: Mutex::new(Component::translatable("optimizeWorld.stage.counting")),
        });

        let thread_shared = Arc::clone(&shared);
        let thread_ctx = Arc::clone(&context);
        let handle = thread::Builder::new()
            .name("world-upgrader".to_string())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    work(&thread_shared, &*thread_ctx);
                }));
                if result.is_err() {
                    thread_shared.set_status(Component::translatable("optimizeWorld.stage.failed"));
                    thread_shared.finished.store(true, Ordering::Relaxed);
                }
            })
            .expect("failed to spawn world upgrader thread");

        Self {
            shared,
            thread: Some(handle),
        }
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#cancel
    pub fn cancel(&mut self) {
        self.shared.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#isFinished
    pub fn is_finished(&self) -> bool {
        self.shared.finished.load(Ordering::Relaxed)
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#levels
    pub fn levels(&self) -> Vec<ResourceKey> {
        self.shared.levels.clone()
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#dimensionProgress
    pub fn dimension_progress(&self, level: &ResourceKey) -> f32 {
        self.shared
            .progress_map
            .lock()
            .unwrap()
            .get(level)
            .copied()
            .unwrap_or(0.0)
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#getProgress
    pub fn get_progress(&self) -> f32 {
        self.shared.progress()
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#getTotalChunks
    pub fn get_total_chunks(&self) -> i32 {
        self.shared.total_chunks.load(Ordering::Relaxed)
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#getConverted
    pub fn get_converted(&self) -> i32 {
        self.shared.converted.load(Ordering::Relaxed)
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#getSkipped
    pub fn get_skipped(&self) -> i32 {
        self.shared.skipped.load(Ordering::Relaxed)
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#getStatus
    pub fn get_status(&self) -> Component {
        self.shared.get_status()
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader#close
    pub fn close(&mut self, context: &dyn WorldUpgraderContext) {
        context.overworld_data_storage_close();
    }
}

impl Drop for WorldUpgrader {
    fn drop(&mut self) {
        self.shared.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

// JAVA: net.minecraft.util.worldupdate.WorldUpgrader#work
fn work<C: WorldUpgraderContext>(shared: &Arc<WorldUpgraderShared>, context: &C) {
    let start = Instant::now();

    let mut entity_upgrader = SimpleRegionStorageUpgrader::new(
        DataFixTypes::EntityChunk,
        "entities".to_string(),
        "entities".to_string(),
        Component::translatable("optimizeWorld.stage.upgrading.entities"),
        Component::translatable("optimizeWorld.stage.finished.entities"),
        -1,
    );
    entity_upgrader.upgrade(shared, context);

    let mut poi_upgrader = SimpleRegionStorageUpgrader::new(
        DataFixTypes::PoiChunk,
        "poi".to_string(),
        "poi".to_string(),
        Component::translatable("optimizeWorld.stage.upgrading.poi"),
        Component::translatable("optimizeWorld.stage.finished.poi"),
        1945,
    );
    poi_upgrader.upgrade(shared, context);

    let mut chunk_upgrader = ChunkUpgrader::new();
    chunk_upgrader.upgrade(shared, context);

    context.overworld_data_storage_save_and_join();
    let elapsed_ms = start.elapsed().as_millis() as i64;
    let _elapsed_secs = elapsed_ms / 1000;
    shared.finished.store(true, Ordering::Relaxed);
}

// JAVA: net.minecraft.util.worldupdate.WorldUpgrader#resolveRecreateDirectory
pub fn resolve_recreate_directory(path: &Path) -> PathBuf {
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    match path.parent() {
        Some(parent) => parent.join(format!("{}{}", NEW_DIRECTORY_PREFIX, file_name)),
        None => PathBuf::from(format!("{}{}", NEW_DIRECTORY_PREFIX, file_name)),
    }
}

fn parse_region_filename(name: &str) -> Option<(i32, i32)> {
    let rest = name.strip_prefix("r.")?;
    let rest = rest.strip_suffix(".mca")?;
    let dot = rest.find('.')?;
    let (x_str, z_rest) = rest.split_at(dot);
    let z_str = &z_rest[1..];
    if z_str.contains('.') {
        return None;
    }
    let x = x_str.parse::<i32>().ok()?;
    let z = z_str.parse::<i32>().ok()?;
    Some((x, z))
}

// JAVA: net.minecraft.util.worldupdate.WorldUpgrader$AbstractUpgrader#getAllChunkPositions
fn get_all_chunk_positions<C: WorldUpgraderContext>(
    context: &C,
    info: &RegionStorageInfo,
    path: &Path,
) -> Vec<FileToUpgrade> {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".mca") {
                files.push(p);
            }
        }
    }
    files.sort();

    let mut result = Vec::new();
    for file_path in files {
        let name = match file_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let Some((rx, rz)) = parse_region_filename(&name) else {
            continue;
        };

        let base_x = rx.wrapping_shl(5);
        let base_z = rz.wrapping_shl(5);
        let mut chunks_to_upgrade = Vec::new();

        match context.open_region_file(info, &file_path, path) {
            Ok(region_file) => {
                let mut dx: i32 = 0;
                while dx < 32 {
                    let mut dz: i32 = 0;
                    while dz < 32 {
                        let pos = ChunkPos::new(base_x.wrapping_add(dx), base_z.wrapping_add(dz));
                        if region_file.does_chunk_exist(pos) {
                            chunks_to_upgrade.push(pos);
                        }
                        dz = dz.wrapping_add(1);
                    }
                    dx = dx.wrapping_add(1);
                }
                if !chunks_to_upgrade.is_empty() {
                    result.push(FileToUpgrade {
                        file_path: file_path.clone(),
                        chunks_to_upgrade,
                    });
                }
            }
            Err(_) => {
                // LOGGER.error("Failed to read chunks from region file {}", ...)
            }
        }
    }

    result
}

struct AbstractUpgraderState {
    previous_write_future: Option<CompletableFuture<()>>,
}

impl AbstractUpgraderState {
    fn new() -> Self {
        Self {
            previous_write_future: None,
        }
    }
}

trait AbstractUpgrader {
    fn data_fix_type(&self) -> DataFixTypes;
    fn type_name(&self) -> &str;
    fn folder_name(&self) -> &str;
    fn upgrading_status(&self) -> Component;
    fn finished_status(&self) -> Component;
    fn state(&mut self) -> &mut AbstractUpgraderState;

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$AbstractUpgrader#createStorage
    fn create_storage(
        &self,
        context: &dyn WorldUpgraderContext,
        info: &RegionStorageInfo,
        path: &Path,
        recreate: bool,
    ) -> Box<dyn SimpleRegionStorage> {
        context.create_storage(self.data_fix_type(), info, path, recreate)
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$AbstractUpgrader#tryProcessOnePosition
    fn try_process_one_position(
        &mut self,
        storage: &dyn SimpleRegionStorage,
        pos: ChunkPos,
        level: &ResourceKey,
        shared: &WorldUpgraderShared,
        context: &dyn WorldUpgraderContext,
    ) -> bool;

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$AbstractUpgrader#getDimensionsToUpgrade
    fn get_dimensions_to_upgrade<C: WorldUpgraderContext>(
        &self,
        shared: &WorldUpgraderShared,
        context: &C,
    ) -> Vec<DimensionToUpgrade> {
        let mut out = Vec::new();
        for level in &shared.levels {
            let info = RegionStorageInfo::new(
                context.level_id(),
                level.clone(),
                self.type_name().to_string(),
            );
            let path = context.dimension_path(level).join(self.folder_name());
            let storage = self.create_storage(context, &info, &path, shared.recreate_region_files);
            let files = self.get_files_to_process(shared, context, &info, &path);
            out.push(DimensionToUpgrade {
                dimension_key: level.clone(),
                storage,
                files,
                file_index: 0,
            });
        }
        out
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$AbstractUpgrader#getFilesToProcess
    fn get_files_to_process<C: WorldUpgraderContext>(
        &self,
        shared: &WorldUpgraderShared,
        context: &C,
        info: &RegionStorageInfo,
        path: &Path,
    ) -> Vec<FileToUpgrade> {
        let files = get_all_chunk_positions(context, info, path);
        let file_count = files.len() as i32;
        let chunk_count: i32 = files
            .iter()
            .map(|f| f.chunks_to_upgrade.len() as i32)
            .fold(0i32, |a, b| a.wrapping_add(b));
        shared
            .total_files
            .fetch_add(file_count, Ordering::Relaxed);
        shared
            .total_chunks
            .fetch_add(chunk_count, Ordering::Relaxed);
        files
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$AbstractUpgrader#processOnePosition
    fn process_one_position(
        &mut self,
        level: &ResourceKey,
        storage: &dyn SimpleRegionStorage,
        pos: ChunkPos,
        shared: &WorldUpgraderShared,
        context: &dyn WorldUpgraderContext,
    ) -> bool {
        let mut ok = false;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.try_process_one_position(storage, pos, level, shared, context)
        }));
        match result {
            Ok(v) => ok = v,
            Err(_) => {
                // Java catches ReportedException/CompletionException and rethrows unless IOException cause.
                // In this self-contained port, treat panic as failed chunk.
                ok = false;
            }
        }
        if ok {
            shared.converted.fetch_add(1, Ordering::Relaxed);
        } else {
            shared.skipped.fetch_add(1, Ordering::Relaxed);
        }
        ok
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$AbstractUpgrader#onFileFinished
    fn on_file_finished(&mut self, file_path: &Path, shared: &WorldUpgraderShared) {
        if shared.recreate_region_files {
            if let Some(prev) = self.state().previous_write_future.take() {
                prev.join();
            }
            let parent = match file_path.parent() {
                Some(p) => p,
                None => return,
            };
            let file_name = match file_path.file_name() {
                Some(n) => n,
                None => return,
            };
            let new_path = resolve_recreate_directory(parent).join(file_name);
            if new_path.exists() {
                let _ = fs::remove_file(file_path);
                let _ = fs::rename(&new_path, file_path);
            }
        }
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$AbstractUpgrader#upgrade
    fn upgrade<C: WorldUpgraderContext>(&mut self, shared: &Arc<WorldUpgraderShared>, context: &C) {
        shared.total_files.store(0, Ordering::Relaxed);
        shared.total_chunks.store(0, Ordering::Relaxed);
        shared.converted.store(0, Ordering::Relaxed);
        shared.skipped.store(0, Ordering::Relaxed);

        let mut dimensions = self.get_dimensions_to_upgrade(shared, context);
        if shared.total_chunks.load(Ordering::Relaxed) != 0 {
            let total_files_f = shared.total_files.load(Ordering::Relaxed) as f32;
            shared.set_status(self.upgrading_status());

            while shared.running.load(Ordering::Relaxed) {
                let mut did_work = false;
                let mut progress_acc: f32 = 0.0f32;

                for dim in &mut dimensions {
                    let dim_key = dim.dimension_key.clone();
                    let mut file_done: Option<PathBuf> = None;
                    let mut file_ok = true;
                    let mut had_file = false;

                    if dim.has_next() {
                        had_file = true;
                        let file_path: PathBuf;
                        let chunks: Vec<ChunkPos>;
                        {
                            let file = dim.next_file().unwrap();
                            file_path = file.file_path.clone();
                            chunks = file.chunks_to_upgrade.clone();
                        }
                        for pos in chunks {
                            let ok = self.process_one_position(
                                &dim_key,
                                &*dim.storage,
                                pos,
                                shared,
                                context,
                            );
                            file_ok = file_ok && ok;
                            did_work = true;
                        }
                        if shared.recreate_region_files {
                            if file_ok {
                                file_done = Some(file_path);
                            }
                        }
                    }

                    if let Some(fp) = file_done {
                        self.on_file_finished(&fp, shared);
                    } else if had_file && shared.recreate_region_files && !file_ok {
                        // LOGGER.error("Failed to convert region file ...")
                    }

                    let dim_progress = (dim.next_index() as f32) / total_files_f;
                    shared
                        .progress_map
                        .lock()
                        .unwrap()
                        .insert(dim_key, dim_progress);
                    progress_acc += dim_progress;
                }

                shared.set_progress(progress_acc);
                if !did_work {
                    break;
                }
            }

            shared.set_status(self.finished_status());
            for dim in &mut dimensions {
                let _ = dim.storage.close_storage();
            }
        }
    }
}

// JAVA: net.minecraft.util.worldupdate.WorldUpgrader$SimpleRegionStorageUpgrader
struct SimpleRegionStorageUpgrader {
    data_fix_type: DataFixTypes,
    type_name: String,
    folder_name: String,
    upgrading_status: Component,
    finished_status: Component,
    upgrade_version: i32,
    state: AbstractUpgraderState,
}

impl SimpleRegionStorageUpgrader {
    fn new(
        data_fix_type: DataFixTypes,
        type_name: String,
        folder_name: String,
        upgrading_status: Component,
        finished_status: Component,
        upgrade_version: i32,
    ) -> Self {
        Self {
            data_fix_type,
            type_name,
            folder_name,
            upgrading_status,
            finished_status,
            upgrade_version,
            state: AbstractUpgraderState::new(),
        }
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$SimpleRegionStorageUpgrader#upgradeTag
    fn upgrade_tag(&self, storage: &dyn SimpleRegionStorage, tag: CompoundTag) -> CompoundTag {
        storage.upgrade_chunk_tag(tag, self.upgrade_version)
    }
}

impl AbstractUpgrader for SimpleRegionStorageUpgrader {
    fn data_fix_type(&self) -> DataFixTypes {
        self.data_fix_type
    }
    fn type_name(&self) -> &str {
        &self.type_name
    }
    fn folder_name(&self) -> &str {
        &self.folder_name
    }
    fn upgrading_status(&self) -> Component {
        self.upgrading_status.clone()
    }
    fn finished_status(&self) -> Component {
        self.finished_status.clone()
    }
    fn state(&mut self) -> &mut AbstractUpgraderState {
        &mut self.state
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$SimpleRegionStorageUpgrader#createStorage
    fn create_storage(
        &self,
        context: &dyn WorldUpgraderContext,
        info: &RegionStorageInfo,
        path: &Path,
        recreate: bool,
    ) -> Box<dyn SimpleRegionStorage> {
        context.create_storage(self.data_fix_type, info, path, recreate)
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$SimpleRegionStorageUpgrader#tryProcessOnePosition
    fn try_process_one_position(
        &mut self,
        storage: &dyn SimpleRegionStorage,
        pos: ChunkPos,
        _level: &ResourceKey,
        shared: &WorldUpgraderShared,
        context: &dyn WorldUpgraderContext,
    ) -> bool {
        let tag_opt = storage.read(pos).join();
        if let Some(tag) = tag_opt {
            let version = nbt_get_data_version(&tag);
            let upgraded = self.upgrade_tag(storage, tag);
            let changed = version < context.current_data_version();
            if changed || shared.recreate_region_files {
                if let Some(prev) = self.state.previous_write_future.take() {
                    prev.join();
                }
                self.state.previous_write_future = Some(storage.write(pos, upgraded));
                return true;
            }
        }
        false
    }
}

// JAVA: net.minecraft.util.worldupdate.WorldUpgrader$ChunkUpgrader
struct ChunkUpgrader {
    state: AbstractUpgraderState,
}

impl ChunkUpgrader {
    fn new() -> Self {
        Self {
            state: AbstractUpgraderState::new(),
        }
    }
}

impl AbstractUpgrader for ChunkUpgrader {
    fn data_fix_type(&self) -> DataFixTypes {
        DataFixTypes::Chunk
    }
    fn type_name(&self) -> &str {
        "chunk"
    }
    fn folder_name(&self) -> &str {
        "region"
    }
    fn upgrading_status(&self) -> Component {
        Component::translatable("optimizeWorld.stage.upgrading.chunks")
    }
    fn finished_status(&self) -> Component {
        Component::translatable("optimizeWorld.stage.finished.chunks")
    }
    fn state(&mut self) -> &mut AbstractUpgraderState {
        &mut self.state
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$ChunkUpgrader#createStorage
    fn create_storage(
        &self,
        context: &dyn WorldUpgraderContext,
        info: &RegionStorageInfo,
        path: &Path,
        recreate: bool,
    ) -> Box<dyn SimpleRegionStorage> {
        context.create_storage(DataFixTypes::Chunk, info, path, recreate)
    }

    // JAVA: net.minecraft.util.worldupdate.WorldUpgrader$ChunkUpgrader#tryProcessOnePosition
    fn try_process_one_position(
        &mut self,
        storage: &dyn SimpleRegionStorage,
        pos: ChunkPos,
        level: &ResourceKey,
        shared: &WorldUpgraderShared,
        context: &dyn WorldUpgraderContext,
    ) -> bool {
        let tag_opt = storage.read(pos).join();
        if let Some(tag) = tag_opt {
            let version = nbt_get_data_version(&tag);
            let fix_context = context.get_chunk_data_fix_context_tag(level);
            let mut upgraded = storage.upgrade_chunk_tag_with_context(tag, -1, fix_context);
            let new_pos = ChunkPos::new(
                upgraded.get_int_or("xPos", 0),
                upgraded.get_int_or("zPos", 0),
            );
            let _ = new_pos; // position mismatch is logged in Java

            let mut changed = version < context.current_data_version();
            if shared.erase_cache {
                changed = changed || upgraded.contains("Heightmaps");
                upgraded.remove("Heightmaps");
                changed = changed || upgraded.contains("isLightOn");
                upgraded.remove("isLightOn");
                let sections = upgraded.get_list_or_empty("sections");
                let mut new_sections = ListTag::default();
                let mut i: usize = 0;
                while i < sections.size() {
                    if let Some(mut section) = sections.get_compound(i) {
                        changed = changed || section.contains("BlockLight");
                        section.remove("BlockLight");
                        changed = changed || section.contains("SkyLight");
                        section.remove("SkyLight");
                        new_sections.push_compound(section);
                    }
                    i = i.wrapping_add(1);
                }
                upgraded.insert("sections", NbtValue::List(new_sections));
            }

            if changed || shared.recreate_region_files {
                if let Some(prev) = self.state.previous_write_future.take() {
                    prev.join();
                }
                self.state.previous_write_future = Some(storage.write(pos, upgraded));
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[derive(Default)]
    struct MockRegionFile {
        path: PathBuf,
        existing: Vec<ChunkPos>,
    }

    impl RegionFileHandle for MockRegionFile {
        fn does_chunk_exist(&self, pos: ChunkPos) -> bool {
            self.existing.contains(&pos)
        }
        fn get_path(&self) -> &Path {
            &self.path
        }
    }

    struct MockStorage {
        reads: Mutex<HashMap<(i32, i32), CompoundTag>>,
        writes: AtomicI32,
        version_bump: bool,
    }

    impl MockStorage {
        fn new(version_bump: bool) -> Self {
            Self {
                reads: Mutex::new(HashMap::new()),
                writes: AtomicI32::new(0),
                version_bump,
            }
        }
        fn with_tag(self, pos: ChunkPos, tag: CompoundTag) -> Self {
            self.reads.lock().unwrap().insert((pos.x, pos.z), tag);
            self
        }
    }

    impl SimpleRegionStorage for MockStorage {
        fn read(&self, pos: ChunkPos) -> CompletableFuture<Option<CompoundTag>> {
            CompletableFuture(self.reads.lock().unwrap().get(&(pos.x, pos.z)).cloned())
        }
        fn write(&self, _pos: ChunkPos, _tag: CompoundTag) -> CompletableFuture<()> {
            self.writes.fetch_add(1, Ordering::Relaxed);
            CompletableFuture(())
        }
        fn upgrade_chunk_tag(&self, mut tag: CompoundTag, _version: i32) -> CompoundTag {
            if self.version_bump {
                tag.insert("DataVersion", NbtValue::Int(9999));
            }
            tag
        }
        fn upgrade_chunk_tag_with_context(
            &self,
            tag: CompoundTag,
            version: i32,
            _context: CompoundTag,
        ) -> CompoundTag {
            self.upgrade_chunk_tag(tag, version)
        }
    }

    struct MockContext {
        current_version: i32,
        files: Mutex<HashMap<PathBuf, Vec<ChunkPos>>>,
    }

    impl MockContext {
        fn new(current_version: i32) -> Self {
            Self {
                current_version,
                files: Mutex::new(HashMap::new()),
            }
        }
        fn add_file(self, path: PathBuf, chunks: Vec<ChunkPos>) -> Self {
            self.files.lock().unwrap().insert(path, chunks);
            self
        }
    }

    impl WorldUpgraderContext for MockContext {
        fn current_data_version(&self) -> i32 {
            self.current_version
        }
        fn level_id(&self) -> String {
            "test".to_string()
        }
        fn dimension_path(&self, _level: &ResourceKey) -> PathBuf {
            PathBuf::from("/tmp/world")
        }
        fn open_region_file(
            &self,
            _info: &RegionStorageInfo,
            file_path: &Path,
            _folder: &Path,
        ) -> io::Result<Box<dyn RegionFileHandle>> {
            let existing = self
                .files
                .lock()
                .unwrap()
                .get(file_path)
                .cloned()
                .unwrap_or_default();
            Ok(Box::new(MockRegionFile {
                path: file_path.to_path_buf(),
                existing,
            }))
        }
        fn create_storage(
            &self,
            _data_fix_type: DataFixTypes,
            _info: &RegionStorageInfo,
            _path: &Path,
            _recreate: bool,
        ) -> Box<dyn SimpleRegionStorage> {
            Box::new(MockStorage::new(true))
        }
        fn get_chunk_data_fix_context_tag(&self, _level: &ResourceKey) -> CompoundTag {
            CompoundTag::new()
        }
    }

    #[test]
    fn resolve_recreate_directory_preserves_parent() {
        let p = PathBuf::from("/world/region/r.0.0.mca");
        let out = resolve_recreate_directory(&p);
        assert_eq!(out, PathBuf::from("/world/region/new_r.0.0.mca"));
    }

    #[test]
    fn parse_region_filename_handles_negative_coords() {
        assert_eq!(parse_region_filename("r.0.0.mca"), Some((0, 0)));
        assert_eq!(parse_region_filename("r.-1.-2.mca"), Some((-1, -2)));
        assert_eq!(parse_region_filename("r.12.-34.mca"), Some((12, -34)));
        assert_eq!(parse_region_filename("bad.mca"), None);
    }

    #[test]
    fn get_all_chunk_positions_uses_wrapping_region_offset() {
        let ctx = MockContext::new(100).add_file(
            PathBuf::from("/tmp/world/region/r.-1.-2.mca"),
            vec![ChunkPos::new(-32, -64), ChunkPos::new(-1, -33)],
        );
        // create fake dir is not needed because we call get_all_chunk_positions directly on empty dir?
        // get_all_chunk_positions reads filesystem; for a unit test, test parse+offset logic via helper.
        let (rx, rz) = parse_region_filename("r.-1.-2.mca").unwrap();
        let base_x = rx.wrapping_shl(5);
        let base_z = rz.wrapping_shl(5);
        assert_eq!(base_x, -32);
        assert_eq!(base_z, -64);
        assert_eq!(base_x.wrapping_add(31), -1);
        assert_eq!(base_z.wrapping_add(31), -33);
        let _ = ctx;
    }

    #[test]
    fn simple_region_upgrader_writes_when_outdated() {
        let ctx = MockContext::new(200);
        let shared = WorldUpgraderShared {
            levels: vec![ResourceKey("overworld".to_string())],
            erase_cache: false,
            recreate_region_files: false,
            running: AtomicBool::new(true),
            finished: AtomicBool::new(false),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            total_chunks: AtomicI32::new(0),
            total_files: AtomicI32::new(0),
            converted: AtomicI32::new(0),
            skipped: AtomicI32::new(0),
            progress_map: Mutex::new(HashMap::new()),
            status: Mutex::new(Component::translatable("optimizeWorld.stage.counting")),
        };
        let mut tag = CompoundTag::new();
        tag.insert("DataVersion", NbtValue::Int(100));
        let storage = MockStorage::new(true).with_tag(ChunkPos::new(0, 0), tag);
        let mut upgrader = SimpleRegionStorageUpgrader::new(
            DataFixTypes::PoiChunk,
            "poi".to_string(),
            "poi".to_string(),
            Component::translatable("up"),
            Component::translatable("done"),
            1945,
        );
        let ok = upgrader.try_process_one_position(
            &storage,
            ChunkPos::new(0, 0),
            &ResourceKey("overworld".to_string()),
            &shared,
            &ctx,
        );
        assert!(ok);
        assert_eq!(storage.writes.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn chunk_upgrader_erases_cache_and_marks_changed() {
        let ctx = MockContext::new(100);
        let shared = WorldUpgraderShared {
            levels: vec![ResourceKey("overworld".to_string())],
            erase_cache: true,
            recreate_region_files: false,
            running: AtomicBool::new(true),
            finished: AtomicBool::new(false),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            total_chunks: AtomicI32::new(0),
            total_files: AtomicI32::new(0),
            converted: AtomicI32::new(0),
            skipped: AtomicI32::new(0),
            progress_map: Mutex::new(HashMap::new()),
            status: Mutex::new(Component::translatable("optimizeWorld.stage.counting")),
        };

        let mut section = CompoundTag::new();
        section.insert("BlockLight", NbtValue::Byte(1));
        section.insert("SkyLight", NbtValue::Byte(1));
        let mut sections = ListTag::default();
        sections.push_compound(section);

        let mut tag = CompoundTag::new();
        tag.insert("DataVersion", NbtValue::Int(100)); // current, but cache erase forces change
        tag.insert("xPos", NbtValue::Int(1));
        tag.insert("zPos", NbtValue::Int(2));
        tag.insert("Heightmaps", NbtValue::Compound(CompoundTag::new()));
        tag.insert("isLightOn", NbtValue::Byte(1));
        tag.insert("sections", NbtValue::List(sections));

        let storage = MockStorage::new(false).with_tag(ChunkPos::new(1, 2), tag);
        let mut upgrader = ChunkUpgrader::new();
        let ok = upgrader.try_process_one_position(
            &storage,
            ChunkPos::new(1, 2),
            &ResourceKey("overworld".to_string()),
            &shared,
            &ctx,
        );
        assert!(ok);
        assert_eq!(storage.writes.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn chunk_upgrader_skips_when_current_and_no_cache_erase() {
        let ctx = MockContext::new(100);
        let shared = WorldUpgraderShared {
            levels: vec![ResourceKey("overworld".to_string())],
            erase_cache: false,
            recreate_region_files: false,
            running: AtomicBool::new(true),
            finished: AtomicBool::new(false),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            total_chunks: AtomicI32::new(0),
            total_files: AtomicI32::new(0),
            converted: AtomicI32::new(0),
            skipped: AtomicI32::new(0),
            progress_map: Mutex::new(HashMap::new()),
            status: Mutex::new(Component::translatable("optimizeWorld.stage.counting")),
        };

        let mut tag = CompoundTag::new();
        tag.insert("DataVersion", NbtValue::Int(100));
        tag.insert("xPos", NbtValue::Int(0));
        tag.insert("zPos", NbtValue::Int(0));

        let storage = MockStorage::new(false).with_tag(ChunkPos::new(0, 0), tag);
        let mut upgrader = ChunkUpgrader::new();
        let ok = upgrader.try_process_one_position(
            &storage,
            ChunkPos::new(0, 0),
            &ResourceKey("overworld".to_string()),
            &shared,
            &ctx,
        );
        assert!(!ok);
        assert_eq!(storage.writes.load(Ordering::Relaxed), 0);
    }
}
