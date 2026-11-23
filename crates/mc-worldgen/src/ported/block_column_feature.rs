#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn mutable(&self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn mutable(&self) -> MutableBlockPos {
        *self
    }

    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        match dir {
            Direction::Down => self.y = self.y.wrapping_sub(1),
            Direction::Up => self.y = self.y.wrapping_add(1),
            Direction::North => self.z = self.z.wrapping_sub(1),
            Direction::South => self.z = self.z.wrapping_add(1),
            Direction::West => self.x = self.x.wrapping_sub(1),
            Direction::East => self.x = self.x.wrapping_add(1),
        }
        self
    }

    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
}

pub trait IntProvider {
    fn sample(&self, random: &mut impl RandomSource) -> i32;
}

#[derive(Clone, Copy, Debug)]
pub struct ConstantInt(pub i32);

impl IntProvider for ConstantInt {
    fn sample(&self, _random: &mut impl RandomSource) -> i32 {
        self.0
    }
}

pub trait BlockStateProvider {
    fn get_state(&self, random: &mut impl RandomSource, pos: BlockPos) -> BlockState;
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleStateProvider(pub BlockState);

impl BlockStateProvider for SimpleStateProvider {
    fn get_state(&self, _random: &mut impl RandomSource, _pos: BlockPos) -> BlockState {
        self.0
    }
}

pub trait BlockPredicate {
    fn test(&self, level: &impl WorldGenLevel, pos: BlockPos) -> bool;
}

pub struct AlwaysTruePredicate;

impl BlockPredicate for AlwaysTruePredicate {
    fn test(&self, _level: &impl WorldGenLevel, _pos: BlockPos) -> bool {
        true
    }
}

pub struct BlockColumnLayer<H: IntProvider, S: BlockStateProvider> {
    height: H,
    state: S,
}

impl<H: IntProvider, S: BlockStateProvider> BlockColumnLayer<H, S> {
    pub fn new(height: H, state: S) -> Self {
        Self { height, state }
    }

    pub fn height(&self) -> &H {
        &self.height
    }

    pub fn state(&self) -> &S {
        &self.state
    }
}

pub struct BlockColumnConfiguration<H: IntProvider, S: BlockStateProvider, P: BlockPredicate> {
    layers: Vec<BlockColumnLayer<H, S>>,
    direction: Direction,
    allowed_placement: P,
    prioritize_tip: bool,
}

impl<H: IntProvider, S: BlockStateProvider, P: BlockPredicate> BlockColumnConfiguration<H, S, P> {
    pub fn new(
        layers: Vec<BlockColumnLayer<H, S>>,
        direction: Direction,
        allowed_placement: P,
        prioritize_tip: bool,
    ) -> Self {
        Self {
            layers,
            direction,
            allowed_placement,
            prioritize_tip,
        }
    }

    pub fn layers(&self) -> &[BlockColumnLayer<H, S>] {
        &self.layers
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn allowed_placement(&self) -> &P {
        &self.allowed_placement
    }

    pub fn prioritize_tip(&self) -> bool {
        self.prioritize_tip
    }
}

pub trait WorldGenLevel {
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub struct FeaturePlaceContext<'a, L: WorldGenLevel, R: RandomSource> {
    level: &'a mut L,
    origin: BlockPos,
    random: &'a mut R,
}

impl<'a, L: WorldGenLevel, R: RandomSource> FeaturePlaceContext<'a, L, R> {
    pub fn new(level: &'a mut L, origin: BlockPos, random: &'a mut R) -> Self {
        Self { level, origin, random }
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

pub struct BlockColumnFeature;

impl BlockColumnFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.BlockColumnFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BlockColumnFeature#place
    pub fn place<L, R, H, S, P>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
        config: &BlockColumnConfiguration<H, S, P>,
    ) -> bool
    where
        L: WorldGenLevel,
        R: RandomSource,
        H: IntProvider,
        S: BlockStateProvider,
        P: BlockPredicate,
    {
        let level = ctx.level as *mut L;
        // We need access to level and random separately through ctx; use split borrows via helper.
        // Instead, restructure to avoid borrow issues.
        let _ = level;
        self.place_impl(ctx.level, ctx.origin, ctx.random, config)
    }

    fn place_impl<L, R, H, S, P>(
        &self,
        level: &mut L,
        origin: BlockPos,
        random: &mut R,
        config: &BlockColumnConfiguration<H, S, P>,
    ) -> bool
    where
        L: WorldGenLevel,
        R: RandomSource,
        H: IntProvider,
        S: BlockStateProvider,
        P: BlockPredicate,
    {
        let layer_count = config.layers().len();
        let mut heights: Vec<i32> = vec![0; layer_count];
        let mut total: i32 = 0;

        let mut i: usize = 0;
        while i < layer_count {
            let h = config.layers()[i].height().sample(random);
            heights[i] = h;
            total = total.wrapping_add(h);
            i = i.wrapping_add(1);
        }

        if total == 0 {
            return false;
        }

        let mut cursor = origin.mutable();
        let mut check_pos = cursor.mutable();
        check_pos.move_dir(config.direction());

        let mut j: i32 = 0;
        while j < total {
            if !config.allowed_placement().test(level, check_pos.to_block_pos()) {
                Self::truncate(&mut heights, total, j, config.prioritize_tip());
                break;
            }
            check_pos.move_dir(config.direction());
            j = j.wrapping_add(1);
        }

        let mut layer_idx: usize = 0;
        while layer_idx < layer_count {
            let layer_height = heights[layer_idx];
            if layer_height != 0 {
                let layer = &config.layers()[layer_idx];
                let mut k: i32 = 0;
                while k < layer_height {
                    let state = layer.state().get_state(random, cursor.to_block_pos());
                    level.set_block(cursor.to_block_pos(), state, 2);
                    cursor.move_dir(config.direction());
                    k = k.wrapping_add(1);
                }
            }
            layer_idx = layer_idx.wrapping_add(1);
        }

        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BlockColumnFeature#truncate
    fn truncate(heights: &mut [i32], total: i32, placed: i32, prioritize_tip: bool) {
        let mut remaining: i32 = total.wrapping_sub(placed);
        let step: i32 = if prioritize_tip { 1 } else { -1 };
        let len_i32 = heights.len() as i32;
        let mut idx: i32 = if prioritize_tip { 0 } else { len_i32.wrapping_sub(1) };
        let end: i32 = if prioritize_tip { len_i32 } else { -1 };

        while idx != end && remaining > 0 {
            let ui = idx as usize;
            let layer_h = heights[ui];
            let take = cmp_min_i32(layer_h, remaining);
            remaining = remaining.wrapping_sub(take);
            heights[ui] = layer_h.wrapping_sub(take);
            idx = idx.wrapping_add(step);
        }
    }
}

fn cmp_min_i32(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

impl Default for BlockColumnFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestRandom {
        ints: Vec<i32>,
        idx: usize,
    }

    impl TestRandom {
        fn new(ints: Vec<i32>) -> Self {
            Self { ints, idx: 0 }
        }
    }

    impl RandomSource for TestRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.idx < self.ints.len() {
                let x = self.ints[self.idx];
                self.idx += 1;
                x
            } else {
                0
            };
            ((v % bound) + bound) % bound
        }
    }

    struct TestLevel {
        placed: Vec<(BlockPos, BlockState, i32)>,
        blocked: HashMap<(i32, i32, i32), bool>,
    }

    impl TestLevel {
        fn new() -> Self {
            Self {
                placed: Vec::new(),
                blocked: HashMap::new(),
            }
        }
    }

    impl WorldGenLevel for TestLevel {
        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.placed.push((pos, state, flags));
            true
        }
    }

    struct DenyAt {
        blocked: Vec<(i32, i32, i32)>,
    }

    impl BlockPredicate for DenyAt {
        fn test(&self, _level: &impl WorldGenLevel, pos: BlockPos) -> bool {
            !self.blocked.iter().any(|b| *b == (pos.x, pos.y, pos.z))
        }
    }

    #[test]
    fn returns_false_when_total_height_zero() {
        let mut level = TestLevel::new();
        let mut rng = TestRandom::new(vec![]);
        let mut ctx = FeaturePlaceContext::new(&mut level, BlockPos::new(0, 0, 0), &mut rng);
        let config = BlockColumnConfiguration::new(
            vec![BlockColumnLayer::new(ConstantInt(0), SimpleStateProvider(BlockState { id: 1 }))],
            Direction::Up,
            AlwaysTruePredicate,
            false,
        );
        let feature = BlockColumnFeature::new();
        assert!(!feature.place(&mut ctx, &config));
        assert!(ctx.level.placed.is_empty());
    }

    #[test]
    fn places_column_upward() {
        let mut level = TestLevel::new();
        let mut rng = TestRandom::new(vec![]);
        let mut ctx = FeaturePlaceContext::new(&mut level, BlockPos::new(1, 5, 2), &mut rng);
        let config = BlockColumnConfiguration::new(
            vec![
                BlockColumnLayer::new(ConstantInt(2), SimpleStateProvider(BlockState { id: 10 })),
                BlockColumnLayer::new(ConstantInt(1), SimpleStateProvider(BlockState { id: 11 })),
            ],
            Direction::Up,
            AlwaysTruePredicate,
            false,
        );
        let feature = BlockColumnFeature::new();
        assert!(feature.place(&mut ctx, &config));
        assert_eq!(ctx.level.placed.len(), 3);
        assert_eq!(ctx.level.placed[0], (BlockPos::new(1, 5, 2), BlockState { id: 10 }, 2));
        assert_eq!(ctx.level.placed[1], (BlockPos::new(1, 6, 2), BlockState { id: 10 }, 2));
        assert_eq!(ctx.level.placed[2], (BlockPos::new(1, 7, 2), BlockState { id: 11 }, 2));
    }

    #[test]
    fn truncates_from_tip_when_prioritize_tip_true() {
        let mut heights = vec![2, 3];
        BlockColumnFeature::truncate(&mut heights, 5, 3, true);
        // remaining = 2, prioritize tip => remove from start
        assert_eq!(heights, vec![0, 3]);
    }

    #[test]
    fn truncates_from_base_when_prioritize_tip_false() {
        let mut heights = vec![2, 3];
        BlockColumnFeature::truncate(&mut heights, 5, 3, false);
        // remaining = 2, remove from end
        assert_eq!(heights, vec![2, 1]);
    }

    #[test]
    fn placement_stops_when_predicate_fails() {
        let mut level = TestLevel::new();
        let mut rng = TestRandom::new(vec![]);
        let mut ctx = FeaturePlaceContext::new(&mut level, BlockPos::new(0, 0, 0), &mut rng);
        let config = BlockColumnConfiguration::new(
            vec![BlockColumnLayer::new(ConstantInt(5), SimpleStateProvider(BlockState { id: 1 }))],
            Direction::Up,
            DenyAt { blocked: vec![(0, 3, 0)] }, // blocks at 3rd step (j==3 means check_pos at origin+4*up?)
            false,
        );
        let feature = BlockColumnFeature::new();
        assert!(feature.place(&mut ctx, &config));
        // check_pos starts at origin+dir = (0,1,0), j=0 checks (0,1,0), ..., j=3 checks (0,4,0)
        // blocked at (0,3,0) means j==2 fails -> truncate with placed=2, remaining=3, prioritize_tip false removes from end
        assert_eq!(ctx.level.placed.len(), 2);
    }

    #[test]
    fn negative_direction_wrapping() {
        let mut pos = MutableBlockPos::new(0, i32::MIN, 0);
        pos.move_dir(Direction::Down);
        assert_eq!(pos.y, i32::MAX);
    }
}
