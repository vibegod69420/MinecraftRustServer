#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
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

    // JAVA: net.minecraft.core.BlockPos#mutable
    pub fn mutable(&self) -> MutableBlockPos {
        MutableBlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#<init>
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

    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#set
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#move
    pub fn move_dir(&mut self, dir: Direction, amount: i32) -> BlockPos {
        let n = dir.normal();
        self.x = self.x.wrapping_add(n.x.wrapping_mul(amount));
        self.y = self.y.wrapping_add(n.y.wrapping_mul(amount));
        self.z = self.z.wrapping_add(n.z.wrapping_mul(amount));
        self.to_block_pos()
    }

    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
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
pub enum BambooLeaves {
    None,
    Small,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Bamboo,
    Podzol,
    Dirt,
    GrassBlock,
    CoarseDirt,
    RootedDirt,
    Mycelium,
    Mud,
    MuddyMangroveRoots,
    MossBlock,
    Sand,
    RedSand,
    Gravel,
    BambooSapling,
}

impl Block {
    // JAVA: net.minecraft.world.level.block.Block#defaultBlockState
    pub fn default_block_state(self) -> BlockState {
        BlockState {
            block: self,
            bamboo_age: 0,
            bamboo_leaves: BambooLeaves::None,
            bamboo_stage: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
    pub bamboo_age: i32,
    pub bamboo_leaves: BambooLeaves,
    pub bamboo_stage: i32,
}

impl Default for BlockState {
    fn default() -> Self {
        Block::Air.default_block_state()
    }
}

pub struct BambooStalkBlock;
pub struct BambooAgeProperty;
pub struct BambooLeavesProperty;
pub struct BambooStageProperty;

impl BambooStalkBlock {
    pub const AGE: BambooAgeProperty = BambooAgeProperty;
    pub const LEAVES: BambooLeavesProperty = BambooLeavesProperty;
    pub const STAGE: BambooStageProperty = BambooStageProperty;
}

pub trait BlockProperty<V> {
    fn set_on(self, state: BlockState, value: V) -> BlockState;
}

impl BlockProperty<i32> for BambooAgeProperty {
    fn set_on(self, mut state: BlockState, value: i32) -> BlockState {
        state.bamboo_age = value;
        state
    }
}

impl BlockProperty<BambooLeaves> for BambooLeavesProperty {
    fn set_on(self, mut state: BlockState, value: BambooLeaves) -> BlockState {
        state.bamboo_leaves = value;
        state
    }
}

impl BlockProperty<i32> for BambooStageProperty {
    fn set_on(self, mut state: BlockState, value: i32) -> BlockState {
        state.bamboo_stage = value;
        state
    }
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_value<V, P: BlockProperty<V>>(self, prop: P, value: V) -> Self {
        prop.set_on(self, value)
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#canSurvive
    pub fn can_survive<L: WorldGenLevel + ?Sized>(&self, level: &L, pos: BlockPos) -> bool {
        match self.block {
            Block::Bamboo => {
                let below = level.get_block_state(BlockPos::new(
                    pos.x,
                    pos.y.wrapping_sub(1),
                    pos.z,
                ));
                below.block == Block::Bamboo
                    || below.block == Block::BambooSapling
                    || is_dirt(below)
                    || below.block == Block::Sand
                    || below.block == Block::RedSand
                    || below.block == Block::Gravel
            }
            _ => true,
        }
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.Feature#isDirt
pub fn is_dirt(state: BlockState) -> bool {
    matches!(
        state.block,
        Block::Dirt
            | Block::GrassBlock
            | Block::Podzol
            | Block::CoarseDirt
            | Block::RootedDirt
            | Block::Mycelium
            | Block::Mud
            | Block::MuddyMangroveRoots
            | Block::MossBlock
    )
}

pub mod blocks {
    use super::Block;

    pub const BAMBOO: Block = Block::Bamboo;
    pub const PODZOL: Block = Block::Podzol;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    WorldSurface,
}

pub trait WorldGenLevel {
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub trait RandomSource {
    // JAVA: net.minecraft.util.RandomSource#nextInt
    fn next_int(&mut self, bound: i32) -> i32;
    // JAVA: net.minecraft.util.RandomSource#nextFloat
    fn next_float(&mut self) -> f32;
}

#[derive(Debug, Clone, Copy)]
pub struct ProbabilityFeatureConfiguration {
    pub probability: f32,
}

pub struct FeaturePlaceContext<'a, L: ?Sized, R: ?Sized> {
    origin: BlockPos,
    level: &'a mut L,
    random: &'a mut R,
    config: ProbabilityFeatureConfiguration,
}

impl<'a, L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized> FeaturePlaceContext<'a, L, R> {
    pub fn new(
        origin: BlockPos,
        level: &'a mut L,
        random: &'a mut R,
        config: ProbabilityFeatureConfiguration,
    ) -> Self {
        Self {
            origin,
            level,
            random,
            config,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#origin
    pub fn origin(&self) -> BlockPos {
        self.origin
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#level
    pub fn level(&mut self) -> &mut L {
        self.level
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#random
    pub fn random(&mut self) -> &mut R {
        self.random
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#config
    pub fn config(&self) -> ProbabilityFeatureConfiguration {
        self.config
    }

    fn parts(&mut self) -> (BlockPos, &mut L, &mut R, ProbabilityFeatureConfiguration) {
        (self.origin, &mut *self.level, &mut *self.random, self.config)
    }
}

const BAMBOO_TRUNK: BlockState = BlockState {
    block: Block::Bamboo,
    bamboo_age: 1,
    bamboo_leaves: BambooLeaves::None,
    bamboo_stage: 0,
};
const BAMBOO_FINAL_LARGE: BlockState = BlockState {
    block: Block::Bamboo,
    bamboo_age: 1,
    bamboo_leaves: BambooLeaves::Large,
    bamboo_stage: 1,
};
const BAMBOO_TOP_LARGE: BlockState = BlockState {
    block: Block::Bamboo,
    bamboo_age: 1,
    bamboo_leaves: BambooLeaves::Large,
    bamboo_stage: 0,
};
const BAMBOO_TOP_SMALL: BlockState = BlockState {
    block: Block::Bamboo,
    bamboo_age: 1,
    bamboo_leaves: BambooLeaves::Small,
    bamboo_stage: 0,
};

pub struct BambooFeature;

impl BambooFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.BambooFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BambooFeature#place
    pub fn place<L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let mut count: i32 = 0;
        let (origin, level, random, config) = ctx.parts();
        let mut mpos = origin.mutable();
        let mut mpos2 = origin.mutable();

        if level.is_empty_block(mpos.to_block_pos()) {
            if blocks::BAMBOO
                .default_block_state()
                .can_survive(level, mpos.to_block_pos())
            {
                let height = random.next_int(12).wrapping_add(5);
                if random.next_float() < config.probability {
                    let radius = random.next_int(4).wrapping_add(1);
                    let mut x = origin.get_x().wrapping_sub(radius);
                    let x_end = origin.get_x().wrapping_add(radius);
                    loop {
                        let mut z = origin.get_z().wrapping_sub(radius);
                        let z_end = origin.get_z().wrapping_add(radius);
                        loop {
                            let dx = x.wrapping_sub(origin.get_x());
                            let dz = z.wrapping_sub(origin.get_z());
                            if dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz))
                                <= radius.wrapping_mul(radius)
                            {
                                let surface_y = level
                                    .get_height(HeightmapType::WorldSurface, x, z)
                                    .wrapping_sub(1);
                                mpos2.set(x, surface_y, z);
                                if is_dirt(level.get_block_state(mpos2.to_block_pos())) {
                                    level.set_block(
                                        mpos2.to_block_pos(),
                                        blocks::PODZOL.default_block_state(),
                                        2,
                                    );
                                }
                            }
                            if z == z_end {
                                break;
                            }
                            z = z.wrapping_add(1);
                        }
                        if x == x_end {
                            break;
                        }
                        x = x.wrapping_add(1);
                    }
                }

                let mut j: i32 = 0;
                while j < height && level.is_empty_block(mpos.to_block_pos()) {
                    level.set_block(mpos.to_block_pos(), BAMBOO_TRUNK, 2);
                    mpos.move_dir(Direction::Up, 1);
                    j = j.wrapping_add(1);
                }

                if mpos.get_y().wrapping_sub(origin.get_y()) >= 3 {
                    level.set_block(mpos.to_block_pos(), BAMBOO_FINAL_LARGE, 2);
                    level.set_block(mpos.move_dir(Direction::Down, 1), BAMBOO_TOP_LARGE, 2);
                    level.set_block(mpos.move_dir(Direction::Down, 1), BAMBOO_TOP_SMALL, 2);
                }
            }

            count = count.wrapping_add(1);
        }

        count > 0
    }
}

impl Default for BambooFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Clone, Default)]
    struct MockLevel {
        blocks: BTreeMap<(i32, i32, i32), BlockState>,
        heights: BTreeMap<(i32, i32), i32>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self::default()
        }

        fn set_height(&mut self, x: i32, z: i32, h: i32) {
            self.heights.insert((x, z), h);
        }

        fn get(&self, pos: BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or_else(|| Block::Air.default_block_state())
        }
    }

    impl WorldGenLevel for MockLevel {
        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.get(pos).block == Block::Air
        }

        fn get_height(&self, _heightmap: HeightmapType, x: i32, z: i32) -> i32 {
            self.heights.get(&(x, z)).copied().unwrap_or(64)
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.get(pos)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            true
        }
    }

    struct SeqRandom {
        ints: Vec<i32>,
        floats: Vec<f32>,
        i: usize,
        f: usize,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>, floats: Vec<f32>) -> Self {
            Self {
                ints,
                floats,
                i: 0,
                f: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.i).copied().unwrap_or(0);
            self.i += 1;
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }

        fn next_float(&mut self) -> f32 {
            let v = self.floats.get(self.f).copied().unwrap_or(1.0_f32);
            self.f += 1;
            v
        }
    }

    #[test]
    fn places_bamboo_column_with_top() {
        let mut level = MockLevel::new();
        level.set_block(BlockPos::new(0, 63, 0), Block::Dirt.default_block_state(), 0);
        let mut rng = SeqRandom::new(vec![0], vec![1.0_f32]); // height = 5
        let config = ProbabilityFeatureConfiguration { probability: 0.0_f32 };
        let mut ctx = FeaturePlaceContext::new(BlockPos::new(0, 64, 0), &mut level, &mut rng, config);
        assert!(BambooFeature::new().place(&mut ctx));

        assert_eq!(level.get_block_state(BlockPos::new(0, 64, 0)), BAMBOO_TRUNK);
        assert_eq!(level.get_block_state(BlockPos::new(0, 66, 0)), BAMBOO_TRUNK);
        assert_eq!(level.get_block_state(BlockPos::new(0, 67, 0)), BAMBOO_TOP_SMALL);
        assert_eq!(level.get_block_state(BlockPos::new(0, 68, 0)), BAMBOO_TOP_LARGE);
        assert_eq!(level.get_block_state(BlockPos::new(0, 69, 0)), BAMBOO_FINAL_LARGE);
    }

    #[test]
    fn returns_false_when_origin_not_empty() {
        let mut level = MockLevel::new();
        level.set_block(BlockPos::new(0, 64, 0), Block::Dirt.default_block_state(), 0);
        let mut rng = SeqRandom::new(vec![], vec![]);
        let config = ProbabilityFeatureConfiguration { probability: 0.0_f32 };
        let mut ctx = FeaturePlaceContext::new(BlockPos::new(0, 64, 0), &mut level, &mut rng, config);
        assert!(!BambooFeature::new().place(&mut ctx));
    }

    #[test]
    fn returns_true_when_empty_but_cannot_survive() {
        let mut level = MockLevel::new();
        let mut rng = SeqRandom::new(vec![], vec![]);
        let config = ProbabilityFeatureConfiguration { probability: 0.0_f32 };
        let mut ctx = FeaturePlaceContext::new(BlockPos::new(0, 64, 0), &mut level, &mut rng, config);
        assert!(BambooFeature::new().place(&mut ctx));
        assert_eq!(
            level.get_block_state(BlockPos::new(0, 64, 0)).block,
            Block::Air
        );
    }

    #[test]
    fn places_podzol_patch_when_probability_passes() {
        let mut level = MockLevel::new();
        level.set_block(BlockPos::new(0, 63, 0), Block::Dirt.default_block_state(), 0);
        for x in -1..=1 {
            for z in -1..=1 {
                level.set_block(BlockPos::new(x, 62, z), Block::Dirt.default_block_state(), 0);
                level.set_height(x, z, 63);
            }
        }
        let mut rng = SeqRandom::new(vec![0, 0], vec![0.0_f32]); // height=5, radius=1
        let config = ProbabilityFeatureConfiguration { probability: 1.0_f32 };
        let mut ctx = FeaturePlaceContext::new(BlockPos::new(0, 64, 0), &mut level, &mut rng, config);
        assert!(BambooFeature::new().place(&mut ctx));
        assert_eq!(
            level.get_block_state(BlockPos::new(0, 62, 0)).block,
            Block::Podzol
        );
    }

    #[test]
    fn no_top_decoration_when_column_too_short() {
        let mut level = MockLevel::new();
        level.set_block(BlockPos::new(0, 63, 0), Block::Dirt.default_block_state(), 0);
        level.set_block(BlockPos::new(0, 65, 0), Block::Dirt.default_block_state(), 0);
        let mut rng = SeqRandom::new(vec![0], vec![1.0_f32]); // height=5
        let config = ProbabilityFeatureConfiguration { probability: 0.0_f32 };
        let mut ctx = FeaturePlaceContext::new(BlockPos::new(0, 64, 0), &mut level, &mut rng, config);
        assert!(BambooFeature::new().place(&mut ctx));

        assert_eq!(level.get_block_state(BlockPos::new(0, 64, 0)), BAMBOO_TRUNK);
        assert_eq!(
            level.get_block_state(BlockPos::new(0, 65, 0)).block,
            Block::Dirt
        );
    }

    #[test]
    fn podzol_probability_strict_less_than_and_negative_coords() {
        let mut level = MockLevel::new();
        level.set_block(BlockPos::new(-5, 9, -7), Block::Dirt.default_block_state(), 0);
        level.set_block(BlockPos::new(-5, 8, -7), Block::Dirt.default_block_state(), 0);
        level.set_height(-5, -7, 9);
        let mut rng = SeqRandom::new(vec![0, 0], vec![0.5_f32]); // float == probability
        let config = ProbabilityFeatureConfiguration { probability: 0.5_f32 };
        let mut ctx = FeaturePlaceContext::new(BlockPos::new(-5, 10, -7), &mut level, &mut rng, config);
        assert!(BambooFeature::new().place(&mut ctx));
        assert_eq!(
            level.get_block_state(BlockPos::new(-5, 8, -7)).block,
            Block::Dirt
        );
        assert_eq!(
            level.get_block_state(BlockPos::new(-5, 10, -7)).block,
            Block::Bamboo
        );
    }
}
