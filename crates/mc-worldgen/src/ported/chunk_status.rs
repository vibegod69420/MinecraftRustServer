use std::fmt;

/// Maximum structure distance used by chunk status generation.
pub const MAX_STRUCTURE_DISTANCE: i32 = 8;

/// Minimal local replacement for `net.minecraft.world.level.levelgen.Heightmap.Types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HeightmapType {
    OceanFloorWg = 1 << 0,
    WorldSurfaceWg = 1 << 1,
    OceanFloor = 1 << 2,
    WorldSurface = 1 << 3,
    MotionBlocking = 1 << 4,
    MotionBlockingNoLeaves = 1 << 5,
}

/// Minimal local replacement for `EnumSet<Heightmap.Types>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct HeightmapTypeSet(u8);

impl HeightmapTypeSet {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn contains(self, t: HeightmapType) -> bool {
        (self.0 & (t as u8)) != 0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

pub const WORLDGEN_HEIGHTMAPS: HeightmapTypeSet = HeightmapTypeSet::from_bits(
    HeightmapType::OceanFloorWg as u8 | HeightmapType::WorldSurfaceWg as u8,
);

pub const FINAL_HEIGHTMAPS: HeightmapTypeSet = HeightmapTypeSet::from_bits(
    HeightmapType::OceanFloor as u8
        | HeightmapType::WorldSurface as u8
        | HeightmapType::MotionBlocking as u8
        | HeightmapType::MotionBlockingNoLeaves as u8,
);

/// Minimal local replacement for `net.minecraft.world.level.chunk.status.ChunkType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkType {
    ProtoChunk,
    LevelChunk,
}

/// Port of `net.minecraft.world.level.chunk.status.ChunkStatus`.
///
/// This is a self-contained representation: registry/codec integration is omitted,
/// and status names are stored directly for name lookup/formatting.
#[derive(Debug, Clone, Copy, Eq)]
pub struct ChunkStatus {
    index: i32,
    parent_index: i32,
    chunk_type: ChunkType,
    heightmaps_after: HeightmapTypeSet,
    name: &'static str,
}

impl PartialEq for ChunkStatus {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.parent_index == other.parent_index
            && self.chunk_type == other.chunk_type
            && self.heightmaps_after == other.heightmaps_after
            && self.name == other.name
    }
}

impl ChunkStatus {
    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#<init>
    pub const fn new(
        parent: Option<ChunkStatus>,
        heightmaps_after: HeightmapTypeSet,
        chunk_type: ChunkType,
        name: &'static str,
    ) -> Self {
        match parent {
            None => Self {
                index: 0,
                parent_index: 0,
                chunk_type,
                heightmaps_after,
                name,
            },
            Some(p) => Self {
                index: p.index.wrapping_add(1),
                parent_index: p.index,
                chunk_type,
                heightmaps_after,
                name,
            },
        }
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#getIndex
    pub const fn get_index(self) -> i32 {
        self.index
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#getParent
    pub fn get_parent(self) -> ChunkStatus {
        if self.parent_index >= 0 && (self.parent_index as usize) < ALL_STATUSES.len() {
            ALL_STATUSES[self.parent_index as usize]
        } else {
            self
        }
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#getChunkType
    pub const fn get_chunk_type(self) -> ChunkType {
        self.chunk_type
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#byName
    pub fn by_name(name: &str) -> Option<ChunkStatus> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);
        let mut i = 0usize;
        while i < ALL_STATUSES.len() {
            let status = ALL_STATUSES[i];
            if status.name == name {
                return Some(status);
            }
            i += 1;
        }
        None
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#heightmapsAfter
    pub const fn heightmaps_after(self) -> HeightmapTypeSet {
        self.heightmaps_after
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#isOrAfter
    pub fn is_or_after(self, other: ChunkStatus) -> bool {
        self.get_index() >= other.get_index()
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#isAfter
    pub fn is_after(self, other: ChunkStatus) -> bool {
        self.get_index() > other.get_index()
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#isOrBefore
    pub fn is_or_before(self, other: ChunkStatus) -> bool {
        self.get_index() <= other.get_index()
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#isBefore
    pub fn is_before(self, other: ChunkStatus) -> bool {
        self.get_index() < other.get_index()
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#max
    pub fn max(a: ChunkStatus, b: ChunkStatus) -> ChunkStatus {
        if a.is_after(b) {
            a
        } else {
            b
        }
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#getStatusList
    pub fn get_status_list() -> Vec<ChunkStatus> {
        let mut list: Vec<ChunkStatus> = Vec::new();
        let mut current = FULL;
        loop {
            list.push(current);
            let parent = current.get_parent();
            if parent == current {
                break;
            }
            current = parent;
        }
        list.reverse();
        list
    }

    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#getName
    pub fn get_name(self) -> String {
        format!("minecraft:{}", self.name)
    }

    fn raw_name(self) -> &'static str {
        self.name
    }
}

impl fmt::Display for ChunkStatus {
    // JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.get_name())
    }
}

// JAVA: net.minecraft.world.level.chunk.status.ChunkStatus#register
const fn register(
    name: &'static str,
    parent: Option<ChunkStatus>,
    heightmaps_after: HeightmapTypeSet,
    chunk_type: ChunkType,
) -> ChunkStatus {
    // Registry registration is omitted in this self-contained port.
    ChunkStatus::new(parent, heightmaps_after, chunk_type, name)
}

pub const EMPTY: ChunkStatus = register("empty", None, WORLDGEN_HEIGHTMAPS, ChunkType::ProtoChunk);
pub const STRUCTURE_STARTS: ChunkStatus = register(
    "structure_starts",
    Some(EMPTY),
    WORLDGEN_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const STRUCTURE_REFERENCES: ChunkStatus = register(
    "structure_references",
    Some(STRUCTURE_STARTS),
    WORLDGEN_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const BIOMES: ChunkStatus = register(
    "biomes",
    Some(STRUCTURE_REFERENCES),
    WORLDGEN_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const NOISE: ChunkStatus = register(
    "noise",
    Some(BIOMES),
    WORLDGEN_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const SURFACE: ChunkStatus = register(
    "surface",
    Some(NOISE),
    WORLDGEN_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const CARVERS: ChunkStatus = register(
    "carvers",
    Some(SURFACE),
    FINAL_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const FEATURES: ChunkStatus = register(
    "features",
    Some(CARVERS),
    FINAL_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const INITIALIZE_LIGHT: ChunkStatus = register(
    "initialize_light",
    Some(FEATURES),
    FINAL_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const LIGHT: ChunkStatus = register(
    "light",
    Some(INITIALIZE_LIGHT),
    FINAL_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const SPAWN: ChunkStatus = register(
    "spawn",
    Some(LIGHT),
    FINAL_HEIGHTMAPS,
    ChunkType::ProtoChunk,
);
pub const FULL: ChunkStatus = register(
    "full",
    Some(SPAWN),
    FINAL_HEIGHTMAPS,
    ChunkType::LevelChunk,
);

pub const ALL_STATUSES: [ChunkStatus; 12] = [
    EMPTY,
    STRUCTURE_STARTS,
    STRUCTURE_REFERENCES,
    BIOMES,
    NOISE,
    SURFACE,
    CARVERS,
    FEATURES,
    INITIALIZE_LIGHT,
    LIGHT,
    SPAWN,
    FULL,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_list_order_indices_and_parents() {
        let list = ChunkStatus::get_status_list();
        let names: Vec<&str> = list.iter().map(|s| s.raw_name()).collect();
        assert_eq!(
            names,
            vec![
                "empty",
                "structure_starts",
                "structure_references",
                "biomes",
                "noise",
                "surface",
                "carvers",
                "features",
                "initialize_light",
                "light",
                "spawn",
                "full",
            ]
        );
        for (i, status) in list.iter().enumerate() {
            assert_eq!(status.get_index(), i as i32);
        }
        assert_eq!(EMPTY.get_parent(), EMPTY);
        assert_eq!(FULL.get_parent(), SPAWN);
        assert_eq!(STRUCTURE_STARTS.get_parent(), EMPTY);
    }

    #[test]
    fn comparisons_and_max() {
        assert!(FULL.is_or_after(SPAWN));
        assert!(FULL.is_after(SPAWN));
        assert!(SPAWN.is_or_before(FULL));
        assert!(SPAWN.is_before(FULL));
        assert!(NOISE.is_or_after(NOISE));
        assert!(NOISE.is_or_before(NOISE));
        assert!(!NOISE.is_after(NOISE));
        assert!(!NOISE.is_before(NOISE));
        assert_eq!(ChunkStatus::max(EMPTY, FULL), FULL);
        assert_eq!(ChunkStatus::max(FULL, EMPTY), FULL);
        assert_eq!(ChunkStatus::max(BIOMES, BIOMES), BIOMES);
    }

    #[test]
    fn heightmaps_chunk_types_and_names() {
        assert_eq!(EMPTY.get_chunk_type(), ChunkType::ProtoChunk);
        assert_eq!(FULL.get_chunk_type(), ChunkType::LevelChunk);
        assert_eq!(SURFACE.heightmaps_after(), WORLDGEN_HEIGHTMAPS);
        assert_eq!(CARVERS.heightmaps_after(), FINAL_HEIGHTMAPS);
        assert!(WORLDGEN_HEIGHTMAPS.contains(HeightmapType::OceanFloorWg));
        assert!(!WORLDGEN_HEIGHTMAPS.contains(HeightmapType::MotionBlocking));
        assert!(FINAL_HEIGHTMAPS.contains(HeightmapType::MotionBlockingNoLeaves));

        assert_eq!(EMPTY.get_name(), "minecraft:empty");
        assert_eq!(FULL.to_string(), "minecraft:full");
        assert_eq!(ChunkStatus::by_name("empty"), Some(EMPTY));
        assert_eq!(ChunkStatus::by_name("minecraft:full"), Some(FULL));
        assert_eq!(ChunkStatus::by_name("does_not_exist"), None);
    }

    #[test]
    fn negative_index_and_overflow_construction() {
        let neg_parent = ChunkStatus {
            index: -5,
            parent_index: -5,
            chunk_type: ChunkType::ProtoChunk,
            heightmaps_after: WORLDGEN_HEIGHTMAPS,
            name: "neg",
        };
        let child = ChunkStatus::new(
            Some(neg_parent),
            FINAL_HEIGHTMAPS,
            ChunkType::LevelChunk,
            "child",
        );
        assert_eq!(child.get_index(), -4);
        assert_eq!(child.get_chunk_type(), ChunkType::LevelChunk);
        assert!(child.is_after(neg_parent));
        assert!(neg_parent.is_before(EMPTY));

        let max_parent = ChunkStatus {
            index: i32::MAX,
            parent_index: i32::MAX,
            chunk_type: ChunkType::ProtoChunk,
            heightmaps_after: WORLDGEN_HEIGHTMAPS,
            name: "max",
        };
        let overflow_child = ChunkStatus::new(
            Some(max_parent),
            WORLDGEN_HEIGHTMAPS,
            ChunkType::ProtoChunk,
            "overflow",
        );
        assert_eq!(overflow_child.get_index(), i32::MIN);
        assert!(overflow_child.is_before(max_parent));
    }

    #[test]
    fn float_bits_sanity() {
        // No float fields in ChunkStatus; verify f32 bit patterns remain stable.
        let pos = 1.5f32;
        let neg_zero = -0.0f32;
        assert_eq!(f32::from_bits(pos.to_bits()), pos);
        assert_eq!(pos.to_bits(), 0x3FC0_0000);
        assert_ne!(neg_zero.to_bits(), 0.0f32.to_bits());
        assert_eq!(f32::from_bits(neg_zero.to_bits()).to_bits(), neg_zero.to_bits());
    }
}
