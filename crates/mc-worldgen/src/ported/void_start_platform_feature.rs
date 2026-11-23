//! Port of `net.minecraft.world.level.levelgen.feature.VoidStartPlatformFeature`.
//!
//! Self-contained module for `mc-worldgen`.

/// Minimal 3D integer position used by this feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(int,int,int)
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#getX
    pub const fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.Vec3i#getY
    pub const fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.Vec3i#getZ
    pub const fn get_z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.BlockPos#atY
    pub const fn at_y(&self, y: i32) -> Self {
        Self {
            x: self.x,
            y,
            z: self.z,
        }
    }
}

/// Mutable block position, mirroring `BlockPos.MutableBlockPos`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#<init>
    pub const fn new() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#set(int,int,int)
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#immutable (conceptual)
    pub const fn immutable(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

impl Default for MutableBlockPos {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq<BlockPos> for MutableBlockPos {
    fn eq(&self, other: &BlockPos) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

/// Minimal chunk position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#<init>(int,int)
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#<init>(net.minecraft.core.BlockPos)
    pub fn from_block_pos(pos: BlockPos) -> Self {
        Self {
            x: pos.x >> 4,
            z: pos.z >> 4,
        }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockX
    pub const fn get_min_block_x(&self) -> i32 {
        self.x << 4
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockZ
    pub const fn get_min_block_z(&self) -> i32 {
        self.z << 4
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMaxBlockX
    pub const fn get_max_block_x(&self) -> i32 {
        (self.x << 4).wrapping_add(15)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMaxBlockZ
    pub const fn get_max_block_z(&self) -> i32 {
        (self.z << 4).wrapping_add(15)
    }
}

/// Minimal block identifiers needed by this feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Cobblestone,
    Stone,
}

/// Minimal block state model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
}

impl Block {
    // JAVA: net.minecraft.world.level.block.state.BlockBehaviour#defaultBlockState
    pub const fn default_block_state(self) -> BlockState {
        BlockState { block: self }
    }
}

/// Placeholder for `NoneFeatureConfiguration`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoneFeatureConfiguration;

/// Feature context carrying level and origin.
pub struct FeaturePlaceContext<'a, L: ?Sized> {
    pub level: &'a mut L,
    pub origin: BlockPos,
}

impl<'a, L: ?Sized> FeaturePlaceContext<'a, L> {
    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#level
    pub fn level(&mut self) -> &mut L {
        self.level
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#origin
    pub fn origin(&self) -> BlockPos {
        self.origin
    }
}

/// Minimal level API required by this feature.
pub trait WorldGenLevel {
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

const PLATFORM_OFFSET: BlockPos = BlockPos::new(8, 3, 8);

fn platform_origin_chunk() -> ChunkPos {
    ChunkPos::from_block_pos(PLATFORM_OFFSET)
}

/// Port of `VoidStartPlatformFeature`.
#[derive(Clone, Copy, Debug, Default)]
pub struct VoidStartPlatformFeature;

impl VoidStartPlatformFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.VoidStartPlatformFeature#<init>
    pub fn new(_codec: Option<()>) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.VoidStartPlatformFeature#checkerboardDistance
    fn checkerboard_distance(x0: i32, z0: i32, x1: i32, z1: i32) -> i32 {
        let dx = x0.wrapping_sub(x1).abs();
        let dz = z0.wrapping_sub(z1).abs();
        dx.max(dz)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.VoidStartPlatformFeature#place
    pub fn place<L: WorldGenLevel + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L>,
    ) -> bool {
        let origin = ctx.origin();
        let chunk = ChunkPos::from_block_pos(origin);
        let platform_chunk = platform_origin_chunk();
        if Self::checkerboard_distance(chunk.x, chunk.z, platform_chunk.x, platform_chunk.z) > 1 {
            return true;
        }

        let platform_y = origin.get_y().wrapping_add(PLATFORM_OFFSET.get_y());
        let platform_pos = PLATFORM_OFFSET.at_y(platform_y);
        let mut mutable = MutableBlockPos::new();

        let mut z = chunk.get_min_block_z();
        let max_z = chunk.get_max_block_z();
        while z <= max_z {
            let mut x = chunk.get_min_block_x();
            let max_x = chunk.get_max_block_x();
            while x <= max_x {
                if Self::checkerboard_distance(
                    platform_pos.get_x(),
                    platform_pos.get_z(),
                    x,
                    z,
                ) <= 16
                {
                    mutable.set(x, platform_pos.get_y(), z);
                    if mutable == platform_pos {
                        let state = Block::Cobblestone.default_block_state();
                        ctx.level().set_block(mutable.immutable(), state, 2);
                    } else {
                        let state = Block::Stone.default_block_state();
                        ctx.level().set_block(mutable.immutable(), state, 2);
                    }
                }
                x = x.wrapping_add(1);
            }
            z = z.wrapping_add(1);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct MockLevel {
        blocks: BTreeMap<(i32, i32, i32), BlockState>,
    }

    impl WorldGenLevel for MockLevel {
        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            true
        }
    }

    #[test]
    fn checkerboard_distance_matches_java_max_abs() {
        assert_eq!(VoidStartPlatformFeature::checkerboard_distance(0, 0, 0, 0), 0);
        assert_eq!(VoidStartPlatformFeature::checkerboard_distance(1, -2, 4, 1), 3);
        assert_eq!(VoidStartPlatformFeature::checkerboard_distance(-5, 5, -1, 2), 4);
    }

    #[test]
    fn places_platform_in_origin_chunk() {
        let mut level = MockLevel::default();
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(8, 10, 8),
        };
        let feature = VoidStartPlatformFeature::new(None);
        assert!(feature.place(&mut ctx));

        let center = BlockPos::new(8, 13, 8);
        assert_eq!(
            ctx.level.blocks.get(&(center.x, center.y, center.z)).copied(),
            Some(Block::Cobblestone.default_block_state())
        );
        assert_eq!(
            ctx.level
                .blocks
                .get(&(9, 13, 8))
                .copied(),
            Some(Block::Stone.default_block_state())
        );
    }

    #[test]
    fn returns_true_and_skips_far_chunks() {
        let mut level = MockLevel::default();
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(1000, 10, 1000),
        };
        let feature = VoidStartPlatformFeature::new(None);
        assert!(feature.place(&mut ctx));
        assert!(ctx.level.blocks.is_empty());
    }

    #[test]
    fn chunk_pos_from_block_pos_uses_shift() {
        let cp = ChunkPos::from_block_pos(BlockPos::new(15, 0, -1));
        assert_eq!(cp, ChunkPos::new(0, -1));
        assert_eq!(cp.get_min_block_x(), 0);
        assert_eq!(cp.get_max_block_x(), 15);
        assert_eq!(cp.get_min_block_z(), -16);
        assert_eq!(cp.get_max_block_z(), -1);
    }

    #[test]
    fn platform_y_uses_wrapping_add() {
        let mut level = MockLevel::default();
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(8, i32::MAX, 8),
        };
        let feature = VoidStartPlatformFeature::new(None);
        assert!(feature.place(&mut ctx));
        let expected_y = i32::MAX.wrapping_add(PLATFORM_OFFSET.get_y());
        assert!(ctx
            .level
            .blocks
            .contains_key(&(8, expected_y, 8)));
    }
}
