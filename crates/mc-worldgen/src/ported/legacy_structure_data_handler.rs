// === FILE: legacy_structure_data_handler.rs ===
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::LazyLock;

pub const LAST_MONOLYTH_STRUCTURE_DATA_VERSION: i32 = 1493;

static CURRENT_TO_LEGACY_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("Village".to_string(), "Village".to_string());
    m.insert("Mineshaft".to_string(), "Mineshaft".to_string());
    m.insert("Mansion".to_string(), "Mansion".to_string());
    m.insert("Igloo".to_string(), "Temple".to_string());
    m.insert("Desert_Pyramid".to_string(), "Temple".to_string());
    m.insert("Jungle_Pyramid".to_string(), "Temple".to_string());
    m.insert("Swamp_Hut".to_string(), "Temple".to_string());
    m.insert("Stronghold".to_string(), "Stronghold".to_string());
    m.insert("Monument".to_string(), "Monument".to_string());
    m.insert("Fortress".to_string(), "Fortress".to_string());
    m.insert("EndCity".to_string(), "EndCity".to_string());
    m
});

static LEGACY_TO_CURRENT_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("Iglu".to_string(), "Igloo".to_string());
    m.insert("TeDP".to_string(), "Desert_Pyramid".to_string());
    m.insert("TeJP".to_string(), "Jungle_Pyramid".to_string());
    m.insert("TeSH".to_string(), "Swamp_Hut".to_string());
    m
});

static OLD_STRUCTURE_REGISTRY_KEYS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    [
        "pillager_outpost",
        "mineshaft",
        "mansion",
        "jungle_pyramid",
        "desert_pyramid",
        "igloo",
        "ruined_portal",
        "shipwreck",
        "swamp_hut",
        "stronghold",
        "monument",
        "ocean_ruin",
        "fortress",
        "endcity",
        "buried_treasure",
        "village",
        "nether_fossil",
        "bastion_remnant",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DimensionKey {
    Overworld,
    Nether,
    End,
    Other,
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

    pub fn to_long(self) -> i64 {
        Self::as_long(self.x, self.z)
    }

    pub fn as_long(x: i32, z: i32) -> i64 {
        ((x as i64) & 0xFFFF_FFFF_i64) | (((z as i64) & 0xFFFF_FFFF_i64) << 32)
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
    List(Vec<Tag>),
    Compound(CompoundTag),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CompoundTag {
    entries: HashMap<String, Tag>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_compound(&self, key: &str) -> Option<CompoundTag> {
        match self.entries.get(key) {
            Some(Tag::Compound(c)) => Some(c.clone()),
            _ => None,
        }
    }

    pub fn get_compound_or_empty(&self, key: &str) -> CompoundTag {
        self.get_compound(key).unwrap_or_default()
    }

    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.entries.get(key) {
            Some(Tag::Byte(b)) => Some(*b != 0),
            _ => None,
        }
    }

    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        match self.entries.get(key) {
            Some(Tag::Int(v)) => *v,
            _ => default,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.entries.get(key) {
            Some(Tag::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn get_long_array(&self, key: &str) -> Option<Vec<i64>> {
        match self.entries.get(key) {
            Some(Tag::LongArray(v)) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn get_list_or_empty(&self, key: &str) -> Vec<Tag> {
        match self.entries.get(key) {
            Some(Tag::List(v)) => v.clone(),
            _ => Vec::new(),
        }
    }

    pub fn put_compound(&mut self, key: impl Into<String>, value: CompoundTag) {
        self.entries.insert(key.into(), Tag::Compound(value));
    }

    pub fn put_long_array(&mut self, key: impl Into<String>, value: Vec<i64>) {
        self.entries.insert(key.into(), Tag::LongArray(value));
    }

    pub fn put_string(&mut self, key: impl Into<String>, value: String) {
        self.entries.insert(key.into(), Tag::String(value));
    }

    pub fn put_int(&mut self, key: impl Into<String>, value: i32) {
        self.entries.insert(key.into(), Tag::Int(value));
    }

    pub fn put_byte(&mut self, key: impl Into<String>, value: i8) {
        self.entries.insert(key.into(), Tag::Byte(value));
    }

    pub fn put(&mut self, key: impl Into<String>, tag: Tag) {
        self.entries.insert(key.into(), tag);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Tag)> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Tag)> {
        self.entries.iter_mut()
    }
}

pub struct NbtUtils;

impl NbtUtils {
    pub fn get_data_version(tag: &CompoundTag) -> i32 {
        tag.get_int_or("DataVersion", 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataFixTypes {
    Chunk,
    SavedDataStructureFeatureIndices,
}

pub trait DataFixer {
    fn update(
        &self,
        data_type: DataFixTypes,
        tag: CompoundTag,
        from_version: i32,
        to_version: i32,
    ) -> CompoundTag;
}

impl DataFixTypes {
    pub fn update<F: DataFixer + ?Sized>(
        self,
        fixer: &F,
        tag: CompoundTag,
        from_version: i32,
        to_version: i32,
    ) -> CompoundTag {
        fixer.update(self, tag, from_version, to_version)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StructureFeatureIndexSavedData {
    all: HashSet<i64>,
    unhandled: HashSet<i64>,
}

impl StructureFeatureIndexSavedData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_unhandled_index(&self, pos: i64) -> bool {
        self.unhandled.contains(&pos)
    }

    pub fn remove_index(&mut self, pos: i64) {
        self.unhandled.remove(&pos);
    }

    pub fn has_start_index(&self, pos: i64) -> bool {
        self.all.contains(&pos)
    }

    pub fn add_index(&mut self, pos: i64) {
        self.all.insert(pos);
        self.unhandled.insert(pos);
    }

    pub fn get_all(&self) -> &HashSet<i64> {
        &self.all
    }
}

pub trait DimensionDataStorage {
    fn read_tag_from_disk(
        &self,
        name: &str,
        data_type: DataFixTypes,
        version: i32,
    ) -> io::Result<CompoundTag>;

    fn compute_if_absent_structure_feature_index(
        &mut self,
        key: &str,
    ) -> StructureFeatureIndexSavedData;
}

pub trait LegacyTagFixer {
    fn mark_chunk_done(&mut self, pos: ChunkPos);
    fn target_data_version(&self) -> i32;
    fn apply_fix(&mut self, tag: CompoundTag) -> CompoundTag;
}

pub struct EmptyLegacyTagFixer;

impl LegacyTagFixer for EmptyLegacyTagFixer {
    fn mark_chunk_done(&mut self, _pos: ChunkPos) {}

    fn target_data_version(&self) -> i32 {
        0
    }

    fn apply_fix(&mut self, tag: CompoundTag) -> CompoundTag {
        tag
    }
}

pub struct LegacyStructureDataHandler<S, F> {
    has_legacy_data: bool,
    data_map: HashMap<String, HashMap<i64, CompoundTag>>,
    index_map: HashMap<String, StructureFeatureIndexSavedData>,
    dimension_data_storage: Option<S>,
    legacy_keys: Vec<String>,
    current_keys: Vec<String>,
    data_fixer: F,
    caches_initialized: bool,
}

impl<S: DimensionDataStorage, F: DataFixer> LegacyStructureDataHandler<S, F> {
    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#<init>
    pub fn new(
        dimension_data_storage: Option<S>,
        legacy_keys: Vec<String>,
        current_keys: Vec<String>,
        data_fixer: F,
    ) -> Self {
        let data_map: HashMap<String, HashMap<i64, CompoundTag>> = HashMap::new();
        let mut has_legacy_data = false;
        for key in &current_keys {
            has_legacy_data |= data_map.get(key.as_str()).is_some();
        }
        Self {
            has_legacy_data,
            data_map,
            index_map: HashMap::new(),
            dimension_data_storage,
            legacy_keys,
            current_keys,
            data_fixer,
            caches_initialized: false,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#markChunkDone
    pub fn mark_chunk_done(&mut self, pos: ChunkPos) {
        let pos_long = pos.to_long();
        let keys = self.legacy_keys.clone();
        for key in &keys {
            if let Some(index) = self.index_map.get_mut(key.as_str()) {
                if index.has_unhandled_index(pos_long) {
                    index.remove_index(pos_long);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#targetDataVersion
    pub fn target_data_version(&self) -> i32 {
        1493
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#applyFix
    pub fn apply_fix(&mut self, mut tag: CompoundTag) -> CompoundTag {
        if !self.caches_initialized && self.dimension_data_storage.is_some() {
            let mut storage = self.dimension_data_storage.take().unwrap();
            self.populate_caches(&mut storage);
            self.dimension_data_storage = Some(storage);
        }

        let version = NbtUtils::get_data_version(&tag);
        if version < LAST_MONOLYTH_STRUCTURE_DATA_VERSION {
            tag = DataFixTypes::Chunk.update(
                &self.data_fixer,
                tag,
                version,
                LAST_MONOLYTH_STRUCTURE_DATA_VERSION,
            );
            let has_legacy = tag
                .get_compound("Level")
                .and_then(|lvl| lvl.get_boolean("hasLegacyStructureData"))
                .unwrap_or(false);
            if has_legacy {
                tag = self.update_from_legacy(tag);
            }
        }

        tag
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#updateFromLegacy
    fn update_from_legacy(&mut self, mut tag: CompoundTag) -> CompoundTag {
        let mut level = tag.get_compound_or_empty("Level");
        let chunk_pos = ChunkPos::new(level.get_int_or("xPos", 0), level.get_int_or("zPos", 0));
        if self.is_unhandled_structure_start(chunk_pos.x, chunk_pos.z) {
            tag = self.update_structure_start(tag, chunk_pos);
            level = tag.get_compound_or_empty("Level");
        }

        let mut structures = level.get_compound_or_empty("Structures");
        let mut references = structures.get_compound_or_empty("References");

        let current_keys = self.current_keys.clone();
        for key in &current_keys {
            let lower = key.to_ascii_lowercase();
            let is_old = OLD_STRUCTURE_REGISTRY_KEYS.contains(lower.as_str());
            if references.get_long_array(key).is_none() && is_old {
                let mut long_list: Vec<i64> = Vec::new();
                let x_start = chunk_pos.x.wrapping_sub(8);
                let x_end = chunk_pos.x.wrapping_add(8);
                let z_start = chunk_pos.z.wrapping_sub(8);
                let z_end = chunk_pos.z.wrapping_add(8);
                let mut dx = x_start;
                loop {
                    let mut dz = z_start;
                    loop {
                        if self.has_legacy_start(dx, dz, key) {
                            long_list.push(ChunkPos::as_long(dx, dz));
                        }
                        if dz == z_end {
                            break;
                        }
                        dz = dz.wrapping_add(1);
                    }
                    if dx == x_end {
                        break;
                    }
                    dx = dx.wrapping_add(1);
                }
                references.put_long_array(key.clone(), long_list);
            }
        }

        structures.put_compound("References", references);
        level.put_compound("Structures", structures);
        tag.put_compound("Level", level);
        tag
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#hasLegacyStart
    fn has_legacy_start(&self, x: i32, z: i32, structure: &str) -> bool {
        if !self.has_legacy_data {
            return false;
        }
        if self.data_map.get(structure).is_none() {
            return false;
        }
        let legacy_key = match CURRENT_TO_LEGACY_MAP.get(structure) {
            Some(k) => k.as_str(),
            None => return false,
        };
        match self.index_map.get(legacy_key) {
            Some(index) => index.has_start_index(ChunkPos::as_long(x, z)),
            None => false,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#isUnhandledStructureStart
    fn is_unhandled_structure_start(&self, x: i32, z: i32) -> bool {
        if !self.has_legacy_data {
            return false;
        }
        let pos_long = ChunkPos::as_long(x, z);
        for key in &self.current_keys {
            if self.data_map.get(key.as_str()).is_some() {
                if let Some(legacy_key) = CURRENT_TO_LEGACY_MAP.get(key.as_str()) {
                    if let Some(index) = self.index_map.get(legacy_key.as_str()) {
                        if index.has_unhandled_index(pos_long) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#updateStructureStart
    fn update_structure_start(&self, mut tag: CompoundTag, chunk_pos: ChunkPos) -> CompoundTag {
        let mut level = tag.get_compound_or_empty("Level");
        let mut structures = level.get_compound_or_empty("Structures");
        let mut starts = structures.get_compound_or_empty("Starts");

        let pos_long = chunk_pos.to_long();
        for key in &self.current_keys {
            if let Some(structure_map) = self.data_map.get(key.as_str()) {
                let has_unhandled = CURRENT_TO_LEGACY_MAP
                    .get(key.as_str())
                    .and_then(|lk| self.index_map.get(lk.as_str()))
                    .map(|idx| idx.has_unhandled_index(pos_long))
                    .unwrap_or(false);
                if has_unhandled {
                    if let Some(start_tag) = structure_map.get(&pos_long) {
                        starts.put_compound(key.clone(), start_tag.clone());
                    }
                }
            }
        }

        structures.put_compound("Starts", starts);
        level.put_compound("Structures", structures);
        tag.put_compound("Level", level);
        tag
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#populateCaches
    fn populate_caches(&mut self, storage: &mut S) {
        if self.caches_initialized {
            return;
        }
        let legacy_keys = self.legacy_keys.clone();
        for key in &legacy_keys {
            let mut features = CompoundTag::new();
            match storage.read_tag_from_disk(
                key,
                DataFixTypes::SavedDataStructureFeatureIndices,
                1493,
            ) {
                Ok(tag) => {
                    features = tag
                        .get_compound_or_empty("data")
                        .get_compound_or_empty("Features");
                    if features.is_empty() {
                        continue;
                    }
                }
                Err(_) => {}
            }

            for (_fk, v) in features.iter_mut() {
                let Tag::Compound(child) = v else {
                    continue;
                };
                let chunk_x = child.get_int_or("ChunkX", 0);
                let chunk_z = child.get_int_or("ChunkZ", 0);
                let pos_long = ChunkPos::as_long(chunk_x, chunk_z);
                let children_list = child.get_list_or_empty("Children");
                if !children_list.is_empty() {
                    let first_id = children_list.first().and_then(|t| match t {
                        Tag::Compound(c) => c.get_string("id"),
                        _ => None,
                    });
                    if let Some(id) = first_id {
                        if let Some(mapped) = LEGACY_TO_CURRENT_MAP.get(id.as_str()) {
                            child.put_string("id", mapped.clone());
                        }
                    }
                }
                if let Some(id) = child.get_string("id") {
                    self.data_map
                        .entry(id)
                        .or_insert_with(HashMap::new)
                        .insert(pos_long, child.clone());
                }
            }

            let index_key = format!("{}_index", key);
            let index_data = storage.compute_if_absent_structure_feature_index(&index_key);
            if index_data.get_all().is_empty() {
                let mut new_index = StructureFeatureIndexSavedData::new();
                for (_fk, v) in features.iter() {
                    if let Tag::Compound(c) = v {
                        let cx = c.get_int_or("ChunkX", 0);
                        let cz = c.get_int_or("ChunkZ", 0);
                        new_index.add_index(ChunkPos::as_long(cx, cz));
                    }
                }
                self.index_map.insert(key.clone(), new_index);
            } else {
                self.index_map.insert(key.clone(), index_data);
            }
        }
        self.caches_initialized = true;
    }
}

impl<S: DimensionDataStorage, F: DataFixer> LegacyTagFixer for LegacyStructureDataHandler<S, F> {
    fn mark_chunk_done(&mut self, pos: ChunkPos) {
        LegacyStructureDataHandler::mark_chunk_done(self, pos);
    }

    fn target_data_version(&self) -> i32 {
        LegacyStructureDataHandler::target_data_version(self)
    }

    fn apply_fix(&mut self, tag: CompoundTag) -> CompoundTag {
        LegacyStructureDataHandler::apply_fix(self, tag)
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler#getLegacyTagFixer
pub fn get_legacy_tag_fixer<S, F>(
    dimension: DimensionKey,
    storage_supplier: impl FnOnce() -> Option<S> + 'static,
    data_fixer: F,
) -> Box<dyn FnOnce() -> Box<dyn LegacyTagFixer>>
where
    S: DimensionDataStorage + 'static,
    F: DataFixer + 'static,
{
    if dimension == DimensionKey::Overworld {
        return Box::new(move || {
            Box::new(LegacyStructureDataHandler::new(
                storage_supplier(),
                vec![
                    "Monument".to_string(),
                    "Stronghold".to_string(),
                    "Village".to_string(),
                    "Mineshaft".to_string(),
                    "Temple".to_string(),
                    "Mansion".to_string(),
                ],
                vec![
                    "Village".to_string(),
                    "Mineshaft".to_string(),
                    "Mansion".to_string(),
                    "Igloo".to_string(),
                    "Desert_Pyramid".to_string(),
                    "Jungle_Pyramid".to_string(),
                    "Swamp_Hut".to_string(),
                    "Stronghold".to_string(),
                    "Monument".to_string(),
                ],
                data_fixer,
            ))
        });
    }
    if dimension == DimensionKey::Nether {
        let keys = vec!["Fortress".to_string()];
        return Box::new(move || {
            Box::new(LegacyStructureDataHandler::new(
                storage_supplier(),
                keys.clone(),
                keys.clone(),
                data_fixer,
            ))
        });
    }
    if dimension == DimensionKey::End {
        let keys = vec!["EndCity".to_string()];
        return Box::new(move || {
            Box::new(LegacyStructureDataHandler::new(
                storage_supplier(),
                keys.clone(),
                keys.clone(),
                data_fixer,
            ))
        });
    }
    let _ = (storage_supplier, data_fixer);
    Box::new(|| Box::new(EmptyLegacyTagFixer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct NoopFixer;
    impl DataFixer for NoopFixer {
        fn update(
            &self,
            _data_type: DataFixTypes,
            tag: CompoundTag,
            _from_version: i32,
            _to_version: i32,
        ) -> CompoundTag {
            tag
        }
    }

    struct EmptyStorage;
    impl DimensionDataStorage for EmptyStorage {
        fn read_tag_from_disk(
            &self,
            _name: &str,
            _data_type: DataFixTypes,
            _version: i32,
        ) -> io::Result<CompoundTag> {
            Err(io::Error::new(io::ErrorKind::NotFound, "no"))
        }

        fn compute_if_absent_structure_feature_index(
            &mut self,
            _key: &str,
        ) -> StructureFeatureIndexSavedData {
            StructureFeatureIndexSavedData::new()
        }
    }

    #[test]
    fn chunk_pos_as_long_negative_and_bits() {
        let pos = ChunkPos::new(-1, -1);
        let l = pos.to_long();
        assert_eq!(l, -1_i64);
        assert_eq!(ChunkPos::as_long(0, 0), 0_i64);
        let l2 = ChunkPos::as_long(i32::MIN, i32::MAX);
        let x = (l2 & 0xFFFF_FFFF_i64) as i32;
        let z = ((l2 as u64) >> 32) as i32;
        assert_eq!(x, i32::MIN);
        assert_eq!(z, i32::MAX);
    }

    #[test]
    fn target_version_and_initial_state() {
        let handler = LegacyStructureDataHandler::new(
            None::<EmptyStorage>,
            vec!["Temple".to_string()],
            vec!["Desert_Pyramid".to_string()],
            NoopFixer,
        );
        assert_eq!(handler.target_data_version(), 1493);
        assert!(!handler.has_legacy_data);
        assert!(!handler.caches_initialized);

        let supplier = get_legacy_tag_fixer(
            DimensionKey::Other,
            || None::<EmptyStorage>,
            NoopFixer,
        );
        let fixer = supplier();
        assert_eq!(fixer.target_data_version(), 0);
    }

    #[test]
    fn populate_caches_remaps_ids_and_builds_index() {
        struct LegacyStorage {
            data: HashMap<String, CompoundTag>,
        }
        impl DimensionDataStorage for LegacyStorage {
            fn read_tag_from_disk(
                &self,
                name: &str,
                _data_type: DataFixTypes,
                _version: i32,
            ) -> io::Result<CompoundTag> {
                self.data
                    .get(name)
                    .cloned()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing"))
            }

            fn compute_if_absent_structure_feature_index(
                &mut self,
                _key: &str,
            ) -> StructureFeatureIndexSavedData {
                StructureFeatureIndexSavedData::new()
            }
        }

        let mut feature = CompoundTag::new();
        feature.put_int("ChunkX", 1);
        feature.put_int("ChunkZ", -2);
        feature.put_string("id", "TeDP".to_string());
        let mut child = CompoundTag::new();
        child.put_string("id", "TeDP".to_string());
        feature.put("Children", Tag::List(vec![Tag::Compound(child)]));

        let mut features = CompoundTag::new();
        features.put_compound("start", feature);
        let mut data = CompoundTag::new();
        data.put_compound("Features", features);
        let mut root = CompoundTag::new();
        root.put_compound("data", data);

        let mut storage_data = HashMap::new();
        storage_data.insert("Temple".to_string(), root);

        let mut handler = LegacyStructureDataHandler::new(
            Some(LegacyStorage { data: storage_data }),
            vec!["Temple".to_string()],
            vec!["Desert_Pyramid".to_string()],
            NoopFixer,
        );

        let mut tag = CompoundTag::new();
        tag.put_int("DataVersion", 100);
        let mut level = CompoundTag::new();
        level.put_int("xPos", 0);
        level.put_int("zPos", 0);
        level.put_byte("hasLegacyStructureData", 0);
        tag.put_compound("Level", level);
        let _ = handler.apply_fix(tag);

        assert!(handler.caches_initialized);
        assert!(handler.data_map.contains_key("Desert_Pyramid"));
        let pos_long = ChunkPos::as_long(1, -2);
        assert!(handler.data_map["Desert_Pyramid"].contains_key(&pos_long));
        assert!(handler.index_map.contains_key("Temple"));
        assert!(handler.index_map["Temple"].has_start_index(pos_long));
    }

    #[test]
    fn mark_chunk_done_removes_unhandled() {
        let mut handler = LegacyStructureDataHandler::new(
            None::<EmptyStorage>,
            vec!["Temple".to_string()],
            vec!["Desert_Pyramid".to_string()],
            NoopFixer,
        );
        let pos = ChunkPos::new(3, -7);
        let pos_long = pos.to_long();
        let mut index = StructureFeatureIndexSavedData::new();
        index.add_index(pos_long);
        handler.index_map.insert("Temple".to_string(), index);
        assert!(handler.index_map["Temple"].has_unhandled_index(pos_long));

        handler.mark_chunk_done(pos);
        assert!(!handler.index_map["Temple"].has_unhandled_index(pos_long));
        assert!(handler.index_map["Temple"].has_start_index(pos_long));
    }

    #[test]
    fn update_from_legacy_overflow_coords_no_panic() {
        let mut handler = LegacyStructureDataHandler::new(
            None::<EmptyStorage>,
            vec!["Temple".to_string()],
            vec!["Desert_Pyramid".to_string()],
            NoopFixer,
        );
        let mut tag = CompoundTag::new();
        tag.put_int("DataVersion", 100);
        let mut level = CompoundTag::new();
        level.put_int("xPos", i32::MAX);
        level.put_int("zPos", i32::MIN);
        level.put_byte("hasLegacyStructureData", 1);
        let mut structures = CompoundTag::new();
        structures.put_compound("References", CompoundTag::new());
        level.put_compound("Structures", structures);
        tag.put_compound("Level", level);

        let out = handler.apply_fix(tag);
        let refs = out
            .get_compound_or_empty("Level")
            .get_compound_or_empty("Structures")
            .get_compound_or_empty("References");
        if let Some(arr) = refs.get_long_array("Desert_Pyramid") {
            assert!(arr.is_empty());
        }
    }

    #[test]
    fn float_bits_preserved_in_tag() {
        let mut tag = CompoundTag::new();
        let nan_bits = f32::NAN.to_bits();
        tag.put("f", Tag::Float(f32::from_bits(nan_bits)));
        let neg_inf_bits = f64::NEG_INFINITY.to_bits();
        tag.put("d", Tag::Double(f64::from_bits(neg_inf_bits)));
        match tag.entries.get("f") {
            Some(Tag::Float(v)) => assert_eq!(v.to_bits(), nan_bits),
            _ => panic!("expected float"),
        }
        match tag.entries.get("d") {
            Some(Tag::Double(v)) => assert_eq!(v.to_bits(), neg_inf_bits),
            _ => panic!("expected double"),
        }
    }
}
