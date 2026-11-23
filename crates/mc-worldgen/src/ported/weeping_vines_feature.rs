use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Minimal self-contained types mirroring the Java classes used by
// net.minecraft.world.level.levelgen.feature.WeepingVinesFeature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn above(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }

    pub fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn set_with_offset(&mut self, pos: impl Into<BlockPos>, dx: i32, dy: i32, dz: i32) -> &mut Self {
        let pos = pos.into();
        self.x = pos.x.wrapping_add(dx);
        self.y = pos.y.wrapping_add(dy);
        self.z = pos.z.wrapping_add(dz);
        self
    }

    pub fn set_with_offset_dir(&mut self, pos: impl Into<BlockPos>, dir: Direction) -> &mut Self {
        let (dx, dy, dz) = dir.normal();
        self.set_with_offset(pos, dx, dy, dz)
    }

    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        let (dx, dy, dz) = dir.normal();
        self.x = self.x.wrapping_add(dx);
        self.y = self.y.wrapping_add(dy);
        self.z = self.z.wrapping_add(dz);
        self
    }

    pub fn above(&self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }

    pub fn below(&self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl From<MutableBlockPos> for BlockPos {
    fn from(p: MutableBlockPos) -> Self {
        p.to_block_pos()
    }
}

impl From<&MutableBlockPos> for BlockPos {
    fn from(p: &MutableBlockPos) -> Self {
        p.to_block_pos()
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
    pub const VALUES: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    pub fn normal(self) -> (i32, i32, i32) {
        match self {
            Direction::Down => (0, -1, 0),
            Direction::Up => (0, 1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Netherrack,
    NetherWartBlock,
    WeepingVines,
    WeepingVinesPlant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
    pub age: i32,
}

pub struct AgeProperty;

pub struct GrowingPlantHeadBlock;

impl GrowingPlantHeadBlock {
    pub const AGE: AgeProperty = AgeProperty;
}

impl BlockState {
    pub fn is(&self, block: Block) -> bool {
        self.block == block
    }

    pub fn set_value(self, _prop: AgeProperty, value: i32) -> Self {
        Self { age: value, ..self }
    }
}

impl Block {
    pub fn default_block_state(self) -> BlockState {
        BlockState {
            block: self,
            age: 0,
        }
    }
}

pub struct Blocks;

impl Blocks {
    pub const NETHERRACK: Block = Block::Netherrack;
    pub const NETHER_WART_BLOCK: Block = Block::NetherWartBlock;
    pub const WEEPING_VINES: Block = Block::WeepingVines;
    pub const WEEPING_VINES_PLANT: Block = Block::WeepingVinesPlant;
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait LevelAccessor {
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub trait WorldGenLevel: LevelAccessor {}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoneFeatureConfiguration;

pub struct FeaturePlaceContext<'a, L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized> {
    pub level: &'a mut L,
    pub origin: BlockPos,
    pub random: &'a mut R,
}

impl<'a, L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized> FeaturePlaceContext<'a, L, R> {
    pub fn new(level: &'a mut L, origin: BlockPos, random: &'a mut R) -> Self {
        Self {
            level,
            origin,
            random,
        }
    }

    pub fn level(&mut self) -> &mut L {
        self.level
    }

    pub fn origin(&self) -> BlockPos {
        self.origin
    }

    pub fn random(&mut self) -> &mut R {
        self.random
    }
}

// JAVA: net.minecraft.util.Mth#nextInt
pub fn mth_next_int(random: &mut (impl RandomSource + ?Sized), min: i32, max: i32) -> i32 {
    if min >= max {
        min
    } else {
        random
            .next_int(max.wrapping_sub(min).wrapping_add(1))
            .wrapping_add(min)
    }
}

// ---------------------------------------------------------------------------
// Port of net.minecraft.world.level.levelgen.feature.WeepingVinesFeature
// ---------------------------------------------------------------------------

pub struct WeepingVinesFeature;

impl WeepingVinesFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.WeepingVinesFeature#<init>
    pub fn new(_codec: Option<NoneFeatureConfiguration>) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WeepingVinesFeature#place
    pub fn place<L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let level = &mut *ctx.level;
        let origin = ctx.origin;
        let random = &mut *ctx.random;

        if !level.is_empty_block(origin) {
            return false;
        }

        let above_state = level.get_block_state(origin.above());
        if !above_state.is(Blocks::NETHERRACK) && !above_state.is(Blocks::NETHER_WART_BLOCK) {
            return false;
        }

        self.place_roof_nether_wart(level, random, origin);
        self.place_roof_weeping_vines(level, random, origin);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WeepingVinesFeature#placeRoofNetherWart
    fn place_roof_nether_wart(
        &self,
        level: &mut (impl LevelAccessor + ?Sized),
        random: &mut (impl RandomSource + ?Sized),
        pos: BlockPos,
    ) {
        level.set_block(pos, Blocks::NETHER_WART_BLOCK.default_block_state(), 2);
        let mut mutable_pos = MutableBlockPos::new(0, 0, 0);
        let mut neighbor_pos = MutableBlockPos::new(0, 0, 0);

        let mut i: i32 = 0;
        while i < 200 {
            let dx = random.next_int(6).wrapping_sub(random.next_int(6));
            let dy = random.next_int(2).wrapping_sub(random.next_int(5));
            let dz = random.next_int(6).wrapping_sub(random.next_int(6));
            mutable_pos.set_with_offset(pos, dx, dy, dz);

            if level.is_empty_block(mutable_pos.to_block_pos()) {
                let mut adjacent_count: i32 = 0;

                for &dir in Direction::VALUES.iter() {
                    neighbor_pos.set_with_offset_dir(mutable_pos.to_block_pos(), dir);
                    let state = level.get_block_state(neighbor_pos.to_block_pos());
                    if state.is(Blocks::NETHERRACK) || state.is(Blocks::NETHER_WART_BLOCK) {
                        adjacent_count = adjacent_count.wrapping_add(1);
                    }

                    if adjacent_count > 1 {
                        break;
                    }
                }

                if adjacent_count == 1 {
                    level.set_block(
                        mutable_pos.to_block_pos(),
                        Blocks::NETHER_WART_BLOCK.default_block_state(),
                        2,
                    );
                }
            }

            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WeepingVinesFeature#placeRoofWeepingVines
    fn place_roof_weeping_vines(
        &self,
        level: &mut (impl LevelAccessor + ?Sized),
        random: &mut (impl RandomSource + ?Sized),
        pos: BlockPos,
    ) {
        let mut mutable_pos = MutableBlockPos::new(0, 0, 0);

        let mut i: i32 = 0;
        while i < 100 {
            let dx = random.next_int(8).wrapping_sub(random.next_int(8));
            let dy = random.next_int(2).wrapping_sub(random.next_int(7));
            let dz = random.next_int(8).wrapping_sub(random.next_int(8));
            mutable_pos.set_with_offset(pos, dx, dy, dz);

            if level.is_empty_block(mutable_pos.to_block_pos()) {
                let above_state = level.get_block_state(mutable_pos.above());
                if above_state.is(Blocks::NETHERRACK) || above_state.is(Blocks::NETHER_WART_BLOCK) {
                    let mut length: i32 = mth_next_int(random, 1, 8);
                    if random.next_int(6) == 0 {
                        length = length.wrapping_mul(2);
                    }

                    if random.next_int(5) == 0 {
                        length = 1;
                    }

                    let _min_age: i32 = 17;
                    let _max_age: i32 = 25;
                    Self::place_weeping_vines_column(level, random, &mut mutable_pos, length, 17, 25);
                }
            }

            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WeepingVinesFeature#placeWeepingVinesColumn
    pub fn place_weeping_vines_column(
        level: &mut (impl LevelAccessor + ?Sized),
        random: &mut (impl RandomSource + ?Sized),
        pos: &mut MutableBlockPos,
        length: i32,
        min_age: i32,
        max_age: i32,
    ) {
        let mut i: i32 = 0;
        while i <= length {
            if level.is_empty_block(pos.to_block_pos()) {
                if i == length || !level.is_empty_block(pos.below()) {
                    level.set_block(
                        pos.to_block_pos(),
                        Blocks::WEEPING_VINES
                            .default_block_state()
                            .set_value(GrowingPlantHeadBlock::AGE, mth_next_int(random, min_age, max_age)),
                        2,
                    );
                    break;
                }

                level.set_block(
                    pos.to_block_pos(),
                    Blocks::WEEPING_VINES_PLANT.default_block_state(),
                    2,
                );
            }

            pos.move_dir(Direction::Down);
            i = i.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl MockRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for MockRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.idx < self.values.len() {
                self.values[self.idx]
            } else {
                0
            };
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    #[derive(Default)]
    struct MockLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
            }
        }

        fn set(&mut self, pos: BlockPos, block: Block) {
            self.blocks
                .insert((pos.x, pos.y, pos.z), block.default_block_state());
        }
    }

    impl LevelAccessor for MockLevel {
        fn is_empty_block(&self, pos: BlockPos) -> bool {
            match self.blocks.get(&(pos.x, pos.y, pos.z)) {
                None => true,
                Some(s) => s.block == Block::Air,
            }
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(BlockState {
                    block: Block::Air,
                    age: 0,
                })
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            true
        }
    }

    impl WorldGenLevel for MockLevel {}

    #[test]
    fn place_returns_false_when_origin_not_empty() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(0, 64, 0);
        level.set(origin, Block::Netherrack);
        let mut random = MockRandom::new(vec![]);
        let mut ctx = FeaturePlaceContext::new(&mut level, origin, &mut random);
        let feature = WeepingVinesFeature::new(None);
        assert!(!feature.place(&mut ctx));
    }

    #[test]
    fn place_returns_false_when_above_invalid() {
        let mut level = MockLevel::new();
        let origin = BlockPos::new(0, 64, 0);
        level.set(origin.above(), Block::Air);
        let mut random = MockRandom::new(vec![]);
        let mut ctx = FeaturePlaceContext::new(&mut level, origin, &mut random);
        let feature = WeepingVinesFeature::new(None);
        assert!(!feature.place(&mut ctx));
    }

    #[test]
    fn place_weeping_vines_column_places_head_and_plants() {
        let mut level = MockLevel::new();
        // below y=8 is solid, so length 2: plant at 10,9, head at 8
        level.set(BlockPos::new(0, 7, 0), Block::Netherrack);
        let mut pos = MutableBlockPos::new(0, 10, 0);
        let mut random = MockRandom::new(vec![0]); // age = min_age + 0
        WeepingVinesFeature::place_weeping_vines_column(&mut level, &mut random, &mut pos, 2, 17, 25);

        assert_eq!(
            level.get_block_state(BlockPos::new(0, 10, 0)).block,
            Block::WeepingVinesPlant
        );
        assert_eq!(
            level.get_block_state(BlockPos::new(0, 9, 0)).block,
            Block::WeepingVinesPlant
        );
        let head = level.get_block_state(BlockPos::new(0, 8, 0));
        assert_eq!(head.block, Block::WeepingVines);
        assert_eq!(head.age, 17);
    }

    #[test]
    fn place_weeping_vines_column_negative_length_noop() {
        let mut level = MockLevel::new();
        let mut pos = MutableBlockPos::new(5, 10, -3);
        let mut random = MockRandom::new(vec![]);
        WeepingVinesFeature::place_weeping_vines_column(&mut level, &mut random, &mut pos, -1, 17, 25);
        assert!(level.is_empty_block(BlockPos::new(5, 10, -3)));
        assert_eq!(pos.to_block_pos(), BlockPos::new(5, 10, -3));
    }

    #[test]
    fn mth_next_int_and_wrapping_coords() {
        let mut random = MockRandom::new(vec![0]);
        assert_eq!(mth_next_int(&mut random, 5, 5), 5);
        assert_eq!(mth_next_int(&mut random, 10, 5), 10); // min >= max

        let mut pos = MutableBlockPos::new(0, 0, 0);
        pos.set_with_offset(BlockPos::new(i32::MAX, i32::MIN, 0), 1, -1, 0);
        assert_eq!(pos.x, i32::MIN);
        assert_eq!(pos.y, i32::MAX);

        let mut p2 = MutableBlockPos::new(0, 0, 0);
        p2.set_with_offset_dir(BlockPos::new(i32::MIN, 0, i32::MAX), Direction::West);
        assert_eq!(p2.x, i32::MAX);
        assert_eq!(p2.z, i32::MAX);
    }

    #[test]
    fn float_bits_preserved() {
        let f32_bits: u32 = 1.5f32.to_bits();
        assert_eq!(f32_bits, 0x3FC0_0000u32);
        let f64_bits: u64 = 1.5f64.to_bits();
        assert_eq!(f64_bits, 0x3FF8_0000_0000_0000u64);
    }
}
