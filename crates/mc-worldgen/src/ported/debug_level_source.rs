use std::sync::{LazyLock, RwLock};

pub const BLOCK_MARGIN: i32 = 2;
pub const HEIGHT: i32 = 70;
pub const BARRIER_HEIGHT: i32 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState(pub u32);

pub const AIR: BlockState = BlockState(0);
pub const BARRIER: BlockState = BlockState(1);

static ALL_BLOCKS: LazyLock<RwLock<Vec<BlockState>>> = LazyLock::new(|| RwLock::new(Vec::new()));

/// Initializes the global debug block list (mirrors `DebugLevelSource.ALL_BLOCKS`).
/// Intended for test/setup use; the Java class initializes this from the block registry.
pub fn set_all_blocks(blocks: Vec<BlockState>) {
    let mut guard = ALL_BLOCKS.write().expect("ALL_BLOCKS lock poisoned");
    *guard = blocks;
}

fn all_blocks_snapshot() -> Vec<BlockState> {
    ALL_BLOCKS
        .read()
        .expect("ALL_BLOCKS lock poisoned")
        .clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32) -> BlockPos {
        self.x = x;
        self.y = y;
        self.z = z;
        BlockPos { x, y, z }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoiseColumn {
    pub min_y: i32,
    pub states: Vec<BlockState>,
}

impl NoiseColumn {
    pub fn new(min_y: i32, states: Vec<BlockState>) -> Self {
        Self { min_y, states }
    }
}

pub trait ChunkAccess {
    fn get_pos(&self) -> ChunkPos;
}

pub trait WorldGenLevel {
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub trait LevelHeightAccessor {}
pub trait StructureManager {}
pub trait RandomState {}
pub trait Blender {}
pub trait BiomeManager {}
pub trait WorldGenRegion: WorldGenLevel {}

/// JAVA: net.minecraft.core.SectionPos#sectionToBlockCoord
fn section_to_block_coord(section: i32, offset: i32) -> i32 {
    section.wrapping_shl(4).wrapping_add(offset)
}

fn mth_sqrt(v: f32) -> f32 {
    (v as f64).sqrt() as f32
}

fn mth_ceil(v: f32) -> i32 {
    let i = v as i32;
    if v > (i as f32) {
        i.wrapping_add(1)
    } else {
        i
    }
}

fn mth_abs(v: i32) -> i32 {
    if v >= 0 {
        v
    } else {
        v.wrapping_neg()
    }
}

fn grid_width_for(block_count: usize) -> i32 {
    mth_ceil(mth_sqrt(block_count as f32))
}

fn grid_height_for(block_count: usize, grid_width: i32) -> i32 {
    if grid_width == 0 {
        0
    } else {
        mth_ceil((block_count as f32) / (grid_width as f32))
    }
}

/// Returns the current debug grid width computed from the global block list.
/// Mirrors the static `GRID_WIDTH` field in Java.
pub fn grid_width() -> i32 {
    let blocks = ALL_BLOCKS.read().expect("ALL_BLOCKS lock poisoned");
    grid_width_for(blocks.len())
}

/// Returns the current debug grid height computed from the global block list.
/// Mirrors the static `GRID_HEIGHT` field in Java.
pub fn grid_height() -> i32 {
    let blocks = ALL_BLOCKS.read().expect("ALL_BLOCKS lock poisoned");
    let gw = grid_width_for(blocks.len());
    grid_height_for(blocks.len(), gw)
}

/// Test/helper variant of `get_block_state_for` that operates on an explicit block list.
pub fn get_block_state_for_with_blocks(mut x: i32, mut z: i32, blocks: &[BlockState]) -> BlockState {
    let mut state = AIR;
    if x > 0 && z > 0 && (x % 2) != 0 && (z % 2) != 0 {
        x /= 2;
        z /= 2;
        let gw = grid_width_for(blocks.len());
        let gh = grid_height_for(blocks.len(), gw);
        if x <= gw && z <= gh {
            let idx = mth_abs(x.wrapping_mul(gw).wrapping_add(z));
            if idx >= 0 && (idx as usize) < blocks.len() {
                state = blocks[idx as usize];
            }
        }
    }
    state
}

#[derive(Debug, Clone)]
pub struct DebugLevelSource<B = ()> {
    pub biome: B,
}

impl<B> DebugLevelSource<B> {
    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#<init>
    pub fn new(biome: B) -> Self {
        Self { biome }
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#codec
    pub fn codec(&self) {
        // MapCodec is not represented in this self-contained port.
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#buildSurface
    pub fn build_surface<
        R: WorldGenRegion + ?Sized,
        S: StructureManager + ?Sized,
        RS: RandomState + ?Sized,
        C: ChunkAccess + ?Sized,
    >(
        &self,
        _region: &mut R,
        _structure_manager: &S,
        _random_state: &RS,
        _chunk: &C,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#applyBiomeDecoration
    pub fn apply_biome_decoration<
        W: WorldGenLevel + ?Sized,
        C: ChunkAccess + ?Sized,
        S: StructureManager + ?Sized,
    >(
        &self,
        level: &mut W,
        chunk: &C,
        _structure_manager: &S,
    ) {
        let mut pos = MutableBlockPos::new();
        let chunk_pos = chunk.get_pos();
        let cx = chunk_pos.x;
        let cz = chunk_pos.z;

        let mut dx = 0i32;
        while dx < 16 {
            let mut dz = 0i32;
            while dz < 16 {
                let x = section_to_block_coord(cx, dx);
                let z = section_to_block_coord(cz, dz);
                level.set_block(pos.set(x, BARRIER_HEIGHT, z), BARRIER, 2);
                let state = get_block_state_for(x, z);
                level.set_block(pos.set(x, HEIGHT, z), state, 2);
                dz = dz.wrapping_add(1);
            }
            dx = dx.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#fillFromNoise
    pub fn fill_from_noise<
        BL: Blender + ?Sized,
        RS: RandomState + ?Sized,
        S: StructureManager + ?Sized,
        C: ChunkAccess,
    >(
        &self,
        _blender: &BL,
        _random_state: &RS,
        _structure_manager: &S,
        chunk: C,
    ) -> C {
        chunk
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#getBaseHeight
    pub fn get_base_height<L: LevelHeightAccessor + ?Sized, RS: RandomState + ?Sized>(
        &self,
        _x: i32,
        _z: i32,
        _heightmap_type: HeightmapType,
        _level: &L,
        _random_state: &RS,
    ) -> i32 {
        0
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#getBaseColumn
    pub fn get_base_column<L: LevelHeightAccessor + ?Sized, RS: RandomState + ?Sized>(
        &self,
        _x: i32,
        _z: i32,
        _level: &L,
        _random_state: &RS,
    ) -> NoiseColumn {
        NoiseColumn::new(0, Vec::new())
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#addDebugScreenInfo
    pub fn add_debug_screen_info<RS: RandomState + ?Sized>(
        &self,
        _info: &mut Vec<String>,
        _random_state: &RS,
        _pos: BlockPos,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#applyCarvers
    pub fn apply_carvers<
        R: WorldGenRegion + ?Sized,
        RS: RandomState + ?Sized,
        BM: BiomeManager + ?Sized,
        S: StructureManager + ?Sized,
        C: ChunkAccess + ?Sized,
    >(
        &self,
        _region: &mut R,
        _seed: i64,
        _random_state: &RS,
        _biome_manager: &BM,
        _structure_manager: &S,
        _chunk: &C,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#spawnOriginalMobs
    pub fn spawn_original_mobs<R: WorldGenRegion + ?Sized>(&self, _region: &mut R) {}

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#getMinY
    pub fn get_min_y(&self) -> i32 {
        0
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#getGenDepth
    pub fn get_gen_depth(&self) -> i32 {
        384
    }

    // JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#getSeaLevel
    pub fn get_sea_level(&self) -> i32 {
        63
    }
}

// JAVA: net.minecraft.world.level.levelgen.DebugLevelSource#getBlockStateFor
pub fn get_block_state_for(x: i32, z: i32) -> BlockState {
    let blocks = all_blocks_snapshot();
    get_block_state_for_with_blocks(x, z, &blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_blocks(n: usize) -> Vec<BlockState> {
        (0..n).map(|i| BlockState(i as u32)).collect()
    }

    #[test]
    fn test_get_block_state_basic_grid_lookup() {
        let blocks = make_blocks(100); // gw=10, gh=10
                                       // x=3,z=3 -> x/=2=1,z/=2=1 -> idx=abs(1*10+1)=11
        assert_eq!(
            get_block_state_for_with_blocks(3, 3, &blocks),
            BlockState(11)
        );
        // x=1,z=3 -> x/=2=0,z/=2=1 -> idx=abs(0*10+1)=1
        assert_eq!(get_block_state_for_with_blocks(1, 3, &blocks), BlockState(1));
        // x=3,z=1 -> x/=2=1,z/=2=0 -> idx=abs(1*10+0)=10
        assert_eq!(
            get_block_state_for_with_blocks(3, 1, &blocks),
            BlockState(10)
        );
    }

    #[test]
    fn test_negative_zero_and_even_coords_return_air() {
        let blocks = make_blocks(100);
        assert_eq!(get_block_state_for_with_blocks(0, 1, &blocks), AIR);
        assert_eq!(get_block_state_for_with_blocks(1, 0, &blocks), AIR);
        assert_eq!(get_block_state_for_with_blocks(-1, 3, &blocks), AIR);
        assert_eq!(get_block_state_for_with_blocks(3, -7, &blocks), AIR);
        assert_eq!(get_block_state_for_with_blocks(2, 3, &blocks), AIR); // even x
        assert_eq!(get_block_state_for_with_blocks(3, 4, &blocks), AIR); // even z
    }

    #[test]
    fn test_grid_dimensions_and_float_bits() {
        // 100 blocks: sqrt(100.0)=10.0 exactly
        assert_eq!(grid_width_for(100), 10);
        assert_eq!(grid_height_for(100, 10), 10);

        // 101 blocks: verify f32 sqrt/ceil bit-faithful path
        let sqrt_bits = mth_sqrt(101.0f32).to_bits();
        let expected_sqrt_bits = ((101.0f64).sqrt() as f32).to_bits();
        assert_eq!(sqrt_bits, expected_sqrt_bits);

        let gw = grid_width_for(101);
        assert_eq!(gw, mth_ceil(mth_sqrt(101.0f32)));
        let gh = grid_height_for(101, gw);
        assert_eq!(gh, mth_ceil((101.0f32) / (gw as f32)));
    }

    #[test]
    fn test_overflow_and_abs_semantics() {
        // Java Mth.abs(Integer.MIN_VALUE) returns Integer.MIN_VALUE due to wrapping negation.
        assert_eq!(mth_abs(i32::MIN), i32::MIN);
        assert_eq!(mth_abs(-1), 1);
        assert_eq!(mth_abs(0), 0);

        // sectionToBlockCoord uses wrapping int arithmetic
        assert_eq!(section_to_block_coord(0, 5), 5);
        assert_eq!(section_to_block_coord(1, 0), 16);
        assert_eq!(section_to_block_coord(-1, 15), -1);
        let s = i32::MAX;
        let expected = s.wrapping_shl(4).wrapping_add(1);
        assert_eq!(section_to_block_coord(s, 1), expected);
    }

    #[test]
    fn test_out_of_range_and_source_constants() {
        let blocks = make_blocks(10); // gw=4, gh=3
                                      // x=7,z=7 -> 3,3 -> idx=abs(3*4+3)=15 >= 10 => AIR
        assert_eq!(get_block_state_for_with_blocks(7, 7, &blocks), AIR);

        let src = DebugLevelSource::new(());
        assert_eq!(src.get_min_y(), 0);
        assert_eq!(src.get_gen_depth(), 384);
        assert_eq!(src.get_sea_level(), 63);

        let col = src.get_base_column(0, 0, &DummyAccessor, &DummyState);
        assert_eq!(col.min_y, 0);
        assert!(col.states.is_empty());
        assert_eq!(
            src.get_base_height(
                0,
                0,
                HeightmapType::WorldSurfaceWg,
                &DummyAccessor,
                &DummyState
            ),
            0
        );
    }

    struct DummyAccessor;
    impl LevelHeightAccessor for DummyAccessor {}
    struct DummyState;
    impl RandomState for DummyState {}
}
