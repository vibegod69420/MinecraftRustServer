pub const PODIUM_RADIUS: i32 = 4;
pub const PODIUM_PILLAR_HEIGHT: i32 = 4;
pub const RIM_RADIUS: i32 = 1;
pub const CORNER_ROUNDING: f32 = 0.5_f32;

const END_PODIUM_LOCATION: BlockPos = BlockPos::ZERO;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    // JAVA: net.minecraft.core.BlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.Vec3i#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.Vec3i#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.Vec3i#offset
    pub fn offset(&self, other: BlockPos) -> Self {
        Self {
            x: self.x.wrapping_add(other.x),
            y: self.y.wrapping_add(other.y),
            z: self.z.wrapping_add(other.z),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#above
    pub fn above(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#relative
    pub fn relative(&self, dir: Direction) -> Self {
        let n = dir.normal();
        Self {
            x: self.x.wrapping_add(n.x),
            y: self.y.wrapping_add(n.y),
            z: self.z.wrapping_add(n.z),
        }
    }

    // JAVA: net.minecraft.core.Vec3i#distSqr
    pub fn dist_sqr(&self, other: BlockPos) -> f64 {
        let dx = (self.x as f64) - (other.x as f64);
        let dy = (self.y as f64) - (other.y as f64);
        let dz = (self.z as f64) - (other.z as f64);
        dx * dx + dy * dy + dz * dz
    }

    // JAVA: net.minecraft.core.BlockPos#closerThan
    pub fn closer_than(&self, other: BlockPos, distance: f64) -> bool {
        self.dist_sqr(other) < distance * distance
    }

    // JAVA: net.minecraft.core.BlockPos#betweenClosed
    pub fn between_closed(from: BlockPos, to: BlockPos) -> Vec<BlockPos> {
        let min_x = from.x.min(to.x);
        let min_y = from.y.min(to.y);
        let min_z = from.z.min(to.z);
        let max_x = from.x.max(to.x);
        let max_y = from.y.max(to.y);
        let max_z = from.z.max(to.z);
        let mut out = Vec::new();
        let mut z = min_z;
        loop {
            let mut y = min_y;
            loop {
                let mut x = min_x;
                loop {
                    out.push(BlockPos::new(x, y, z));
                    if x == max_x {
                        break;
                    }
                    x = x.wrapping_add(1);
                }
                if y == max_y {
                    break;
                }
                y = y.wrapping_add(1);
            }
            if z == max_z {
                break;
            }
            z = z.wrapping_add(1);
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const HORIZONTAL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    fn normal(self) -> BlockPos {
        match self {
            Direction::Down => BlockPos::new(0, -1, 0),
            Direction::Up => BlockPos::new(0, 1, 0),
            Direction::North => BlockPos::new(0, 0, -1),
            Direction::South => BlockPos::new(0, 0, 1),
            Direction::West => BlockPos::new(-1, 0, 0),
            Direction::East => BlockPos::new(1, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Bedrock,
    EndStone,
    EndPortal,
    WallTorch,
}

impl Block {
    // JAVA: net.minecraft.world.level.block.Block#defaultBlockState
    pub fn default_block_state(self) -> BlockState {
        BlockState {
            block: self,
            wall_torch_facing: if self == Block::WallTorch {
                Some(Direction::North)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
    pub wall_torch_facing: Option<Direction>,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockState#is
    pub fn is(&self, block: Block) -> bool {
        self.block == block
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_value(self, _prop: WallTorchFacingProperty, facing: Direction) -> Self {
        Self {
            wall_torch_facing: Some(facing),
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallTorchFacingProperty;

pub struct WallTorchBlock;

impl WallTorchBlock {
    pub const FACING: WallTorchFacingProperty = WallTorchFacingProperty;
}

pub mod blocks {
    use super::Block;

    pub const AIR: Block = Block::Air;
    pub const BEDROCK: Block = Block::Bedrock;
    pub const END_STONE: Block = Block::EndStone;
    pub const END_PORTAL: Block = Block::EndPortal;
    pub const WALL_TORCH: Block = Block::WallTorch;
}

pub trait WorldGenLevel {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
    fn destroy_block(&mut self, pos: &BlockPos, drop: bool) -> bool;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoneFeatureConfiguration;

pub struct FeaturePlaceContext<'a, L: WorldGenLevel + ?Sized> {
    origin: BlockPos,
    level: &'a mut L,
}

impl<'a, L: WorldGenLevel + ?Sized> FeaturePlaceContext<'a, L> {
    pub fn new(origin: BlockPos, level: &'a mut L, _config: NoneFeatureConfiguration) -> Self {
        Self { origin, level }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#origin
    pub fn origin(&self) -> BlockPos {
        self.origin
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#level
    pub fn level(&mut self) -> &mut L {
        self.level
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EndPodiumFeature {
    active: bool,
}

impl EndPodiumFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.EndPodiumFeature#getLocation
    pub fn get_location(pos: BlockPos) -> BlockPos {
        END_PODIUM_LOCATION.offset(pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.EndPodiumFeature#<init>
    pub fn new(active: bool) -> Self {
        Self { active }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.Feature#setBlock
    fn set_block<L: WorldGenLevel + ?Sized>(&self, level: &mut L, pos: BlockPos, state: BlockState) {
        level.set_block(pos, state, 2);
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.EndPodiumFeature#dropPreviousAndSetBlock
    fn drop_previous_and_set_block<L: WorldGenLevel + ?Sized>(
        &self,
        level: &mut L,
        pos: BlockPos,
        block: Block,
    ) {
        if !level.get_block_state(&pos).is(block) {
            level.destroy_block(&pos, true);
            self.set_block(level, pos, block.default_block_state());
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.EndPodiumFeature#place
    pub fn place<L: WorldGenLevel + ?Sized>(&self, ctx: &mut FeaturePlaceContext<'_, L>) -> bool {
        let origin = ctx.origin();
        let level = ctx.level();

        for pos in BlockPos::between_closed(
            BlockPos::new(
                origin.get_x().wrapping_sub(4),
                origin.get_y().wrapping_sub(1),
                origin.get_z().wrapping_sub(4),
            ),
            BlockPos::new(
                origin.get_x().wrapping_add(4),
                origin.get_y().wrapping_add(32),
                origin.get_z().wrapping_add(4),
            ),
        ) {
            let close_2_5 = pos.closer_than(origin, 2.5_f64);
            if close_2_5 || pos.closer_than(origin, 3.5_f64) {
                if pos.get_y() < origin.get_y() {
                    if close_2_5 {
                        self.set_block(level, pos, blocks::BEDROCK.default_block_state());
                    } else if pos.get_y() < origin.get_y() {
                        if self.active {
                            self.drop_previous_and_set_block(level, pos, blocks::END_STONE);
                        } else {
                            self.set_block(level, pos, blocks::END_STONE.default_block_state());
                        }
                    }
                } else if pos.get_y() > origin.get_y() {
                    if self.active {
                        self.drop_previous_and_set_block(level, pos, blocks::AIR);
                    } else {
                        self.set_block(level, pos, blocks::AIR.default_block_state());
                    }
                } else if !close_2_5 {
                    self.set_block(level, pos, blocks::BEDROCK.default_block_state());
                } else if self.active {
                    self.drop_previous_and_set_block(level, pos, blocks::END_PORTAL);
                } else {
                    self.set_block(level, pos, blocks::AIR.default_block_state());
                }
            }
        }

        let mut i: i32 = 0;
        while i < 4 {
            self.set_block(level, origin.above(i), blocks::BEDROCK.default_block_state());
            i = i.wrapping_add(1);
        }

        let top_pos = origin.above(2);
        for dir in Direction::HORIZONTAL {
            self.set_block(
                level,
                top_pos.relative(dir),
                blocks::WALL_TORCH
                    .default_block_state()
                    .set_value(WallTorchBlock::FACING, dir),
            );
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        blocks: HashMap<BlockPos, BlockState>,
        destroyed: Vec<(BlockPos, bool)>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
                destroyed: Vec::new(),
            }
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.blocks
                .get(pos)
                .copied()
                .unwrap_or_else(|| blocks::AIR.default_block_state())
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert(pos, state);
            true
        }

        fn destroy_block(&mut self, pos: &BlockPos, drop: bool) -> bool {
            self.destroyed.push((*pos, drop));
            self.blocks.insert(*pos, blocks::AIR.default_block_state());
            true
        }
    }

    #[test]
    fn get_location_offsets_from_origin_and_constants() {
        let pos = BlockPos::new(10, -5, 20);
        assert_eq!(EndPodiumFeature::get_location(pos), pos);
        assert_eq!(CORNER_ROUNDING.to_bits(), 0.5_f32.to_bits());
    }

    #[test]
    fn inactive_podium_places_expected_blocks() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(0, 64, 0);
        let mut ctx = FeaturePlaceContext::new(origin, &mut level, NoneFeatureConfiguration);
        let feature = EndPodiumFeature::new(false);
        assert!(feature.place(&mut ctx));

        for i in 0..4 {
            assert_eq!(
                level.get_block_state(&origin.above(i)).block,
                Block::Bedrock
            );
        }
        assert_eq!(
            level.get_block_state(&BlockPos::new(0, 63, 0)).block,
            Block::Bedrock
        );
        assert_eq!(
            level.get_block_state(&BlockPos::new(3, 63, 0)).block,
            Block::EndStone
        );
        assert_eq!(
            level.get_block_state(&BlockPos::new(2, 64, 0)).block,
            Block::Air
        );
        assert_eq!(
            level.get_block_state(&BlockPos::new(3, 64, 0)).block,
            Block::Bedrock
        );
    }

    #[test]
    fn active_podium_places_portal_and_destroys() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(0, 64, 0);
        level.set_block(
            BlockPos::new(1, 64, 0),
            blocks::END_STONE.default_block_state(),
            0,
        );
        let mut ctx = FeaturePlaceContext::new(origin, &mut level, NoneFeatureConfiguration);
        let feature = EndPodiumFeature::new(true);
        assert!(feature.place(&mut ctx));

        assert_eq!(
            level.get_block_state(&BlockPos::new(1, 64, 0)).block,
            Block::EndPortal
        );
        assert_eq!(
            level.get_block_state(&BlockPos::new(1, 65, 0)).block,
            Block::Air
        );
        assert!(level
            .destroyed
            .iter()
            .any(|(p, drop)| *p == BlockPos::new(1, 64, 0) && *drop));
    }

    #[test]
    fn closer_than_uses_strict_less_than_and_negative_coords() {
        let a = BlockPos::new(0, 0, 0);
        let b = BlockPos::new(3, 4, 0);
        assert!(!a.closer_than(b, 5.0_f64));
        assert!(a.closer_than(b, 5.0001_f64));

        let c = BlockPos::new(-100, -200, -300);
        let d = BlockPos::new(100, 200, 300);
        let ds = c.dist_sqr(d);
        assert_eq!(
            ds,
            (200.0_f64 * 200.0_f64) + (400.0_f64 * 400.0_f64) + (600.0_f64 * 600.0_f64)
        );
    }

    #[test]
    fn between_closed_handles_negative_and_reversed() {
        let positions = BlockPos::between_closed(
            BlockPos::new(-1, 0, 1),
            BlockPos::new(0, -1, 0),
        );
        assert_eq!(positions.len(), 8);
        assert_eq!(positions[0], BlockPos::new(-1, -1, 0));
        assert_eq!(*positions.last().unwrap(), BlockPos::new(0, 0, 1));
    }

    #[test]
    fn wall_torches_face_outward() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(5, 64, -3);
        let mut ctx = FeaturePlaceContext::new(origin, &mut level, NoneFeatureConfiguration);
        EndPodiumFeature::new(false).place(&mut ctx);

        let top = origin.above(2);
        for dir in Direction::HORIZONTAL {
            let state = level.get_block_state(&top.relative(dir));
            assert_eq!(state.block, Block::WallTorch);
            assert_eq!(state.wall_torch_facing, Some(dir));
        }
    }
}
