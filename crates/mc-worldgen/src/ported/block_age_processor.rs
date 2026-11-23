// === FILE: block_age_processor.rs ===
//! Port of `net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor` (1.21.11).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub trait LevelReader {}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CompoundTag;

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

impl<R: RandomSource + ?Sized> RandomSource for &mut R {
    fn next_float(&mut self) -> f32 {
        (**self).next_float()
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        (**self).next_int(bound)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Half {
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StairShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlabType {
    Top,
    Bottom,
    Double,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WallHeight {
    None,
    Low,
    Tall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    StoneBricks,
    Stone,
    ChiseledStoneBricks,
    CrackedStoneBricks,
    MossyStoneBricks,
    StoneSlab,
    StoneBrickSlab,
    MossyStoneBrickSlab,
    StoneBrickStairs,
    MossyStoneBrickStairs,
    MossyStoneBrickWall,
    Obsidian,
    CryingObsidian,
    // Generic tagged blocks for extensibility/testing.
    GenericStairs,
    GenericSlab,
    GenericWall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockTag {
    Stairs,
    Slabs,
    Walls,
}

impl Block {
    pub fn default_block_state(self) -> BlockState {
        let mut state = BlockState::new(self);
        if self.has_facing() {
            state.facing = Some(Direction::North);
        }
        if self.has_half() {
            state.half = Some(Half::Bottom);
        }
        if self.has_stair_shape() {
            state.stair_shape = Some(StairShape::Straight);
        }
        if self.has_slab_type() {
            state.slab_type = Some(SlabType::Bottom);
        }
        if self.has_waterlogged() {
            state.waterlogged = Some(false);
        }
        if self.is_wall_block() {
            state.wall_north = Some(WallHeight::None);
            state.wall_east = Some(WallHeight::None);
            state.wall_south = Some(WallHeight::None);
            state.wall_west = Some(WallHeight::None);
            state.wall_up = Some(true);
        }
        state
    }

    fn has_facing(self) -> bool {
        matches!(
            self,
            Block::StoneBrickStairs | Block::MossyStoneBrickStairs | Block::GenericStairs
        )
    }

    fn has_half(self) -> bool {
        matches!(
            self,
            Block::StoneBrickStairs | Block::MossyStoneBrickStairs | Block::GenericStairs
        )
    }

    fn has_stair_shape(self) -> bool {
        matches!(
            self,
            Block::StoneBrickStairs | Block::MossyStoneBrickStairs | Block::GenericStairs
        )
    }

    fn has_slab_type(self) -> bool {
        matches!(
            self,
            Block::StoneSlab
                | Block::StoneBrickSlab
                | Block::MossyStoneBrickSlab
                | Block::GenericSlab
        )
    }

    fn has_waterlogged(self) -> bool {
        self.has_facing() || self.has_slab_type() || self.is_wall_block()
    }

    fn is_wall_block(self) -> bool {
        matches!(self, Block::MossyStoneBrickWall | Block::GenericWall)
    }

    pub fn is_tag(self, tag: BlockTag) -> bool {
        match tag {
            BlockTag::Stairs => self.has_facing() && self.has_half() && self.has_stair_shape(),
            BlockTag::Slabs => self.has_slab_type(),
            BlockTag::Walls => self.is_wall_block(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
    pub facing: Option<Direction>,
    pub half: Option<Half>,
    pub stair_shape: Option<StairShape>,
    pub slab_type: Option<SlabType>,
    pub waterlogged: Option<bool>,
    pub wall_north: Option<WallHeight>,
    pub wall_east: Option<WallHeight>,
    pub wall_south: Option<WallHeight>,
    pub wall_west: Option<WallHeight>,
    pub wall_up: Option<bool>,
}

impl BlockState {
    fn new(block: Block) -> Self {
        Self {
            block,
            facing: None,
            half: None,
            stair_shape: None,
            slab_type: None,
            waterlogged: None,
            wall_north: None,
            wall_east: None,
            wall_south: None,
            wall_west: None,
            wall_up: None,
        }
    }

    pub fn is<P: BlockPredicate>(&self, pred: P) -> bool {
        pred.test(self)
    }

    pub fn set_value_facing(mut self, value: Direction) -> Self {
        self.facing = Some(value);
        self
    }

    pub fn set_value_half(mut self, value: Half) -> Self {
        self.half = Some(value);
        self
    }

    pub fn with_properties_of(mut self, other: &BlockState) -> Self {
        if self.block.has_facing() {
            if let Some(v) = other.facing {
                self.facing = Some(v);
            }
        }
        if self.block.has_half() {
            if let Some(v) = other.half {
                self.half = Some(v);
            }
        }
        if self.block.has_stair_shape() {
            if let Some(v) = other.stair_shape {
                self.stair_shape = Some(v);
            }
        }
        if self.block.has_slab_type() {
            if let Some(v) = other.slab_type {
                self.slab_type = Some(v);
            }
        }
        if self.block.has_waterlogged() {
            if let Some(v) = other.waterlogged {
                self.waterlogged = Some(v);
            }
        }
        if self.block.is_wall_block() {
            if let Some(v) = other.wall_north {
                self.wall_north = Some(v);
            }
            if let Some(v) = other.wall_east {
                self.wall_east = Some(v);
            }
            if let Some(v) = other.wall_south {
                self.wall_south = Some(v);
            }
            if let Some(v) = other.wall_west {
                self.wall_west = Some(v);
            }
            if let Some(v) = other.wall_up {
                self.wall_up = Some(v);
            }
        }
        self
    }
}

pub trait BlockPredicate {
    fn test(&self, state: &BlockState) -> bool;
}

impl BlockPredicate for Block {
    fn test(&self, state: &BlockState) -> bool {
        state.block == *self
    }
}

impl BlockPredicate for BlockTag {
    fn test(&self, state: &BlockState) -> bool {
        state.block.is_tag(*self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: Option<CompoundTag>,
}

pub trait StructurePlaceSettings {
    fn get_random(&mut self, pos: &BlockPos) -> &mut dyn RandomSource;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureProcessorType {
    BlockAge,
}

pub const PROBABILITY_OF_REPLACING_FULL_BLOCK: f32 = 0.5;
pub const PROBABILITY_OF_REPLACING_STAIRS: f32 = 0.5;
pub const PROBABILITY_OF_REPLACING_OBSIDIAN: f32 = 0.15;

const HORIZONTAL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

const HALF_VALUES: [Half; 2] = [Half::Top, Half::Bottom];

fn non_mossy_replacements() -> [BlockState; 2] {
    [
        Block::StoneSlab.default_block_state(),
        Block::StoneBrickSlab.default_block_state(),
    ]
}

fn get_random_from_slice<'a, T>(slice: &'a [T], random: &mut dyn RandomSource) -> &'a T {
    let idx = random.next_int(slice.len() as i32) as usize;
    &slice[idx]
}

// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#getRandomFacingStairs
fn get_random_facing_stairs(random: &mut dyn RandomSource, block: Block) -> BlockState {
    let facing = *get_random_from_slice(&HORIZONTAL_DIRECTIONS, random);
    let half = *get_random_from_slice(&HALF_VALUES, random);
    block
        .default_block_state()
        .set_value_facing(facing)
        .set_value_half(half)
}

// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#getRandomBlock
fn get_random_block_from_slice(random: &mut dyn RandomSource, states: &[BlockState]) -> BlockState {
    let idx = random.next_int(states.len() as i32) as usize;
    states[idx].clone()
}

pub struct BlockAgeProcessor {
    pub mossiness: f32,
}

impl BlockAgeProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#<init>
    pub fn new(mossiness: f32) -> Self {
        Self { mossiness }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#processBlock
    pub fn process_block(
        &self,
        _level: &dyn LevelReader,
        _offset: &BlockPos,
        _pos: &BlockPos,
        _original: &StructureBlockInfo,
        current: &StructureBlockInfo,
        settings: &mut dyn StructurePlaceSettings,
    ) -> StructureBlockInfo {
        let random = settings.get_random(&current.pos);
        let state = &current.state;
        let pos = current.pos;
        let replacement: Option<BlockState> = if state.is(Block::StoneBricks)
            || state.is(Block::Stone)
            || state.is(Block::ChiseledStoneBricks)
        {
            self.maybe_replace_full_stone_block(random)
        } else if state.is(BlockTag::Stairs) {
            self.maybe_replace_stairs(state, random)
        } else if state.is(BlockTag::Slabs) {
            self.maybe_replace_slab(state, random)
        } else if state.is(BlockTag::Walls) {
            self.maybe_replace_wall(state, random)
        } else if state.is(Block::Obsidian) {
            self.maybe_replace_obsidian(random)
        } else {
            None
        };

        match replacement {
            Some(new_state) => StructureBlockInfo {
                pos,
                state: new_state,
                nbt: current.nbt.clone(),
            },
            None => current.clone(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#maybeReplaceFullStoneBlock
    fn maybe_replace_full_stone_block(
        &self,
        random: &mut dyn RandomSource,
    ) -> Option<BlockState> {
        if random.next_float() >= 0.5_f32 {
            return None;
        }

        let replacements_normal = [
            Block::CrackedStoneBricks.default_block_state(),
            get_random_facing_stairs(random, Block::StoneBrickStairs),
        ];
        let replacements_mossy = [
            Block::MossyStoneBricks.default_block_state(),
            get_random_facing_stairs(random, Block::MossyStoneBrickStairs),
        ];
        Some(self.get_random_block(random, &replacements_normal, &replacements_mossy))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#maybeReplaceStairs
    fn maybe_replace_stairs(
        &self,
        state: &BlockState,
        random: &mut dyn RandomSource,
    ) -> Option<BlockState> {
        if random.next_float() >= 0.5_f32 {
            return None;
        }

        let replacements_mossy = [
            Block::MossyStoneBrickStairs
                .default_block_state()
                .with_properties_of(state),
            Block::MossyStoneBrickSlab.default_block_state(),
        ];
        let replacements_normal = non_mossy_replacements();
        Some(self.get_random_block(random, &replacements_normal, &replacements_mossy))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#maybeReplaceSlab
    fn maybe_replace_slab(
        &self,
        state: &BlockState,
        random: &mut dyn RandomSource,
    ) -> Option<BlockState> {
        if random.next_float() < self.mossiness {
            Some(
                Block::MossyStoneBrickSlab
                    .default_block_state()
                    .with_properties_of(state),
            )
        } else {
            None
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#maybeReplaceWall
    fn maybe_replace_wall(
        &self,
        state: &BlockState,
        random: &mut dyn RandomSource,
    ) -> Option<BlockState> {
        if random.next_float() < self.mossiness {
            Some(
                Block::MossyStoneBrickWall
                    .default_block_state()
                    .with_properties_of(state),
            )
        } else {
            None
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#maybeReplaceObsidian
    fn maybe_replace_obsidian(&self, random: &mut dyn RandomSource) -> Option<BlockState> {
        if random.next_float() < 0.15_f32 {
            Some(Block::CryingObsidian.default_block_state())
        } else {
            None
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#getRandomBlock
    fn get_random_block(
        &self,
        random: &mut dyn RandomSource,
        normal: &[BlockState],
        mossy: &[BlockState],
    ) -> BlockState {
        if random.next_float() < self.mossiness {
            get_random_block_from_slice(random, mossy)
        } else {
            get_random_block_from_slice(random, normal)
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::BlockAge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        float_idx: usize,
        int_idx: usize,
    }

    impl MockRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                ints,
                float_idx: 0,
                int_idx: 0,
            }
        }
    }

    impl RandomSource for MockRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.float_idx];
            self.float_idx += 1;
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints[self.int_idx];
            self.int_idx += 1;
            let mut r = v % bound;
            if r < 0 {
                r += bound;
            }
            r
        }
    }

    struct FixedSettings {
        random: MockRandom,
    }

    impl FixedSettings {
        fn new(random: MockRandom) -> Self {
            Self { random }
        }
    }

    impl StructurePlaceSettings for FixedSettings {
        fn get_random(&mut self, _pos: &BlockPos) -> &mut dyn RandomSource {
            &mut self.random
        }
    }

    struct DummyLevel;
    impl LevelReader for DummyLevel {}

    fn info_at(pos: BlockPos, block: Block) -> StructureBlockInfo {
        StructureBlockInfo {
            pos,
            state: block.default_block_state(),
            nbt: Some(CompoundTag),
        }
    }

    #[test]
    fn full_block_threshold_and_mossy_selection() {
        let proc = BlockAgeProcessor::new(1.0);
        // >= 0.5 means no replacement
        let mut rng = MockRandom::new(vec![0.5], vec![]);
        assert!(proc.maybe_replace_full_stone_block(&mut rng).is_none());

        // < 0.5 replaces; mossiness 1.0 selects mossy branch, index 0 => MossyStoneBricks
        // floats: [threshold check, mossy selection], ints: [normal stair facing, half, mossy stair facing, half, pick]
        let mut rng = MockRandom::new(vec![0.0, 0.0], vec![0, 1, 2, 0, 0]);
        let result = proc.maybe_replace_full_stone_block(&mut rng).unwrap();
        assert_eq!(result.block, Block::MossyStoneBricks);

        // mossiness 0.0 selects normal branch, index 1 => StoneBrickStairs with consumed randoms
        let proc = BlockAgeProcessor::new(0.0);
        let mut rng = MockRandom::new(vec![0.0, 0.0], vec![1, 0, 3, 1, 1]);
        let result = proc.maybe_replace_full_stone_block(&mut rng).unwrap();
        assert_eq!(result.block, Block::StoneBrickStairs);
        assert_eq!(result.facing, Some(Direction::East));
        assert_eq!(result.half, Some(Half::Top));
    }

    #[test]
    fn negative_mossiness_and_float_bits() {
        let neg = BlockAgeProcessor::new(-1.0_f32);
        let slab = Block::StoneSlab.default_block_state();
        let mut rng = MockRandom::new(vec![0.0], vec![]);
        assert!(neg.maybe_replace_slab(&slab, &mut rng).is_none());

        let nan_bits = f32::from_bits(0x7FC0_0001);
        assert!(nan_bits.is_nan());
        let nan_proc = BlockAgeProcessor::new(nan_bits);
        let mut rng = MockRandom::new(vec![0.0], vec![]);
        assert!(nan_proc.maybe_replace_slab(&slab, &mut rng).is_none());

        // NaN mossiness: full block passes 0.5 check, but mossy selection (x < NaN) is false => normal
        let mut rng = MockRandom::new(vec![0.0, 0.0], vec![0, 0, 0, 0, 0]);
        let result = nan_proc.maybe_replace_full_stone_block(&mut rng).unwrap();
        assert_eq!(result.block, Block::CrackedStoneBricks);

        // Negative zero behaves like 0.0
        let nz = BlockAgeProcessor::new(f32::from_bits(0x8000_0000));
        let mut rng = MockRandom::new(vec![0.0], vec![]);
        assert!(nz.maybe_replace_wall(&Block::MossyStoneBrickWall.default_block_state(), &mut rng).is_none());
    }

    #[test]
    fn obsidian_boundary_and_negative_next_int() {
        let proc = BlockAgeProcessor::new(0.5);
        let mut rng = MockRandom::new(vec![0.15], vec![]);
        assert!(proc.maybe_replace_obsidian(&mut rng).is_none());
        let mut rng = MockRandom::new(vec![0.149_999_99], vec![]);
        assert_eq!(
            proc.maybe_replace_obsidian(&mut rng).unwrap().block,
            Block::CryingObsidian
        );

        // Negative next_int is normalized into [0, bound)
        let proc0 = BlockAgeProcessor::new(0.0);
        // ints: 4 for stairs, then -1 % 2 => 1
        let mut rng = MockRandom::new(vec![0.0, 0.0], vec![0, 0, 0, 0, -1]);
        let result = proc0.maybe_replace_full_stone_block(&mut rng).unwrap();
        assert_eq!(result.block, Block::StoneBrickStairs);
    }

    #[test]
    fn stairs_and_slab_preserve_properties() {
        let proc = BlockAgeProcessor::new(1.0);
        let stair = Block::GenericStairs
            .default_block_state()
            .set_value_facing(Direction::East)
            .set_value_half(Half::Top);
        // replace threshold pass, mossy selection -> mossy, pick index 0 => mossy stairs
        let mut rng = MockRandom::new(vec![0.0, 0.0], vec![0]);
        let result = proc.maybe_replace_stairs(&stair, &mut rng).unwrap();
        assert_eq!(result.block, Block::MossyStoneBrickStairs);
        assert_eq!(result.facing, Some(Direction::East));
        assert_eq!(result.half, Some(Half::Top));

        let mut slab = Block::StoneBrickSlab.default_block_state();
        slab.slab_type = Some(SlabType::Top);
        let mut rng = MockRandom::new(vec![0.0], vec![]);
        let result = proc.maybe_replace_slab(&slab, &mut rng).unwrap();
        assert_eq!(result.block, Block::MossyStoneBrickSlab);
        assert_eq!(result.slab_type, Some(SlabType::Top));
    }

    #[test]
    fn process_block_integrates_and_preserves_nbt() {
        let proc = BlockAgeProcessor::new(1.0);
        let pos = BlockPos { x: 5, y: 64, z: -3 };
        let input = info_at(pos, Block::StoneBricks);
        let original = input.clone();
        let mut settings =
            FixedSettings::new(MockRandom::new(vec![0.0, 0.0], vec![0, 0, 0, 0, 0]));
        let out = proc.process_block(
            &DummyLevel,
            &BlockPos { x: 0, y: 0, z: 0 },
            &pos,
            &original,
            &input,
            &mut settings,
        );
        assert_eq!(out.pos, pos);
        assert_eq!(out.state.block, Block::MossyStoneBricks);
        assert_eq!(out.nbt, Some(CompoundTag));
        assert_eq!(proc.get_type(), StructureProcessorType::BlockAge);

        let untouched = StructureBlockInfo {
            pos,
            state: Block::CryingObsidian.default_block_state(),
            nbt: None,
        };
        let mut settings2 = FixedSettings::new(MockRandom::new(vec![], vec![]));
        let out2 = proc.process_block(
            &DummyLevel,
            &BlockPos { x: 0, y: 0, z: 0 },
            &pos,
            &untouched,
            &untouched,
            &mut settings2,
        );
        assert_eq!(out2, untouched);
    }
}
