use std::cmp::{max, min};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    pub fn above(&self) -> Self {
        Self::new(self.x, self.y.wrapping_add(1), self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub const fn new() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    pub fn above(&self) -> BlockPos {
        BlockPos::new(self.x, self.y.wrapping_add(1), self.z)
    }
}

impl Default for MutableBlockPos {
    fn default() -> Self {
        Self::new()
    }
}

impl From<MutableBlockPos> for BlockPos {
    fn from(value: MutableBlockPos) -> Self {
        value.to_block_pos()
    }
}

impl From<&MutableBlockPos> for BlockPos {
    fn from(value: &MutableBlockPos) -> Self {
        value.to_block_pos()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    pub fn min_x(&self) -> i32 {
        self.min_x
    }

    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    pub fn min_z(&self) -> i32 {
        self.min_z
    }

    pub fn max_x(&self) -> i32 {
        self.max_x
    }

    pub fn max_y(&self) -> i32 {
        self.max_y
    }

    pub fn max_z(&self) -> i32 {
        self.max_z
    }

    pub fn inflated_by(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(
            self.min_x.wrapping_sub(dx),
            self.min_y.wrapping_sub(dy),
            self.min_z.wrapping_sub(dz),
            self.max_x.wrapping_add(dx),
            self.max_y.wrapping_add(dy),
            self.max_z.wrapping_add(dz),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    MotionBlockingNoLeaves,
}

pub trait RandomSource {
    fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    pub const fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn is_air(&self) -> bool {
        self.id == 0
    }

    pub fn is(&self, other: Self) -> bool {
        self.id == other.id
    }

    pub fn is_solid_render(&self) -> bool {
        !self.is_air()
    }
}

pub struct Blocks;

impl Blocks {
    pub const VINE: BlockState = BlockState::new(1);
}

pub trait BlockStateProvider {
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState;
}

pub trait LevelSimulatedReader {
    fn is_state_at_position<F>(&self, pos: BlockPos, predicate: F) -> bool
    where
        F: FnOnce(&BlockState) -> bool;

    fn get_heightmap_pos(&self, heightmap_type: HeightmapType, pos: BlockPos) -> BlockPos;
}

pub trait TreeDecoratorContext {
    type Level: LevelSimulatedReader;
    type Random: RandomSource;

    fn level(&self) -> &Self::Level;
    fn level_mut(&mut self) -> &mut Self::Level;
    fn random(&mut self) -> &mut Self::Random;
    fn check_block<F>(&self, pos: BlockPos, predicate: F) -> bool
    where
        F: FnOnce(&BlockState) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: BlockState);
}

pub struct TreeFeature;

impl TreeFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.TreeFeature#getLowestTrunkOrRootOfTree
    pub fn get_lowest_trunk_or_root_of_tree<C: TreeDecoratorContext>(context: &C) -> Vec<BlockPos> {
        context_lowest_trunk_or_root_of_tree(context)
    }
}

fn context_lowest_trunk_or_root_of_tree<C: TreeDecoratorContext>(_context: &C) -> Vec<BlockPos> {
    // This is a self-contained placeholder hook. In a full port, the context would
    // expose trunk/root/log positions collected during tree placement.
    Vec::new()
}

pub enum TreeDecoratorType {
    PlaceOnGround,
}

pub struct PlaceOnGroundDecorator<P: BlockStateProvider> {
    tries: i32,
    radius: i32,
    height: i32,
    block_state_provider: P,
}

impl<P: BlockStateProvider> PlaceOnGroundDecorator<P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.PlaceOnGroundDecorator#<init>
    pub fn new(tries: i32, radius: i32, height: i32, block_state_provider: P) -> Self {
        Self {
            tries,
            radius,
            height,
            block_state_provider,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.PlaceOnGroundDecorator#type
    pub fn type_(&self) -> TreeDecoratorType {
        TreeDecoratorType::PlaceOnGround
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.PlaceOnGroundDecorator#place
    pub fn place<C: TreeDecoratorContext>(&self, context: &mut C, trunk_or_roots: &[BlockPos]) {
        let list: Vec<BlockPos> = trunk_or_roots.to_vec();
        if !list.is_empty() {
            let first = list[0];
            let y = first.get_y();
            let mut min_x = first.get_x();
            let mut max_x = first.get_x();
            let mut min_z = first.get_z();
            let mut max_z = first.get_z();

            for pos in &list {
                if pos.get_y() == y {
                    min_x = min(min_x, pos.get_x());
                    max_x = max(max_x, pos.get_x());
                    min_z = min(min_z, pos.get_z());
                    max_z = max(max_z, pos.get_z());
                }
            }

            let bounding_box =
                BoundingBox::new(min_x, y, min_z, max_x, y, max_z).inflated_by(self.radius, self.height, self.radius);
            let mut mutable_pos = MutableBlockPos::new();

            let mut i = 0;
            while i < self.tries {
                let rx = context
                    .random()
                    .next_int_between_inclusive(bounding_box.min_x(), bounding_box.max_x());
                let ry = context
                    .random()
                    .next_int_between_inclusive(bounding_box.min_y(), bounding_box.max_y());
                let rz = context
                    .random()
                    .next_int_between_inclusive(bounding_box.min_z(), bounding_box.max_z());
                mutable_pos.set(rx, ry, rz);
                self.attempt_to_place_block_above(context, mutable_pos.to_block_pos());
                i = i.wrapping_add(1);
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.PlaceOnGroundDecorator#attemptToPlaceBlockAbove
    fn attempt_to_place_block_above<C: TreeDecoratorContext>(
        &self,
        context: &mut C,
        pos: BlockPos,
    ) {
        let above = pos.above();
        let can_place_above = context
            .level()
            .is_state_at_position(above, |state| state.is_air() || state.is(Blocks::VINE));
        if can_place_above
            && context.check_block(pos, |state| state.is_solid_render())
            && context
                .level()
                .get_heightmap_pos(HeightmapType::MotionBlockingNoLeaves, pos)
                .get_y()
                <= above.get_y()
        {
            let state = self.block_state_provider.get_state(context.random(), above);
            context.set_block(above, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};

    #[derive(Clone, Debug)]
    struct SeqRandom {
        ints: VecDeque<i32>,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>) -> Self {
            Self {
                ints: ints.into(),
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32 {
            let v = self.ints.pop_front().unwrap_or(min);
            // Clamp into range for deterministic tests even if sequence is odd.
            if v < min {
                min
            } else if v > max {
                max
            } else {
                v
            }
        }
    }

    #[derive(Clone, Copy)]
    struct FixedProvider {
        state: BlockState,
    }

    impl BlockStateProvider for FixedProvider {
        fn get_state(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
            self.state
        }
    }

    struct TestLevel {
        states: HashMap<BlockPos, BlockState>,
        heightmap: HashMap<(i32, i32), i32>,
    }

    impl TestLevel {
        fn new() -> Self {
            Self {
                states: HashMap::new(),
                heightmap: HashMap::new(),
            }
        }

        fn set_state(&mut self, pos: BlockPos, state: BlockState) {
            self.states.insert(pos, state);
        }

        fn get_state(&self, pos: BlockPos) -> BlockState {
            *self.states.get(&pos).unwrap_or(&BlockState::new(0))
        }

        fn set_height(&mut self, x: i32, z: i32, y: i32) {
            self.heightmap.insert((x, z), y);
        }
    }

    impl LevelSimulatedReader for TestLevel {
        fn is_state_at_position<F>(&self, pos: BlockPos, predicate: F) -> bool
        where
            F: FnOnce(&BlockState) -> bool,
        {
            let state = self.get_state(pos);
            predicate(&state)
        }

        fn get_heightmap_pos(&self, _heightmap_type: HeightmapType, pos: BlockPos) -> BlockPos {
            let y = *self.heightmap.get(&(pos.x, pos.z)).unwrap_or(&i32::MIN);
            BlockPos::new(pos.x, y, pos.z)
        }
    }

    struct TestContext {
        level: TestLevel,
        random: SeqRandom,
        placed: Vec<(BlockPos, BlockState)>,
    }

    impl TestContext {
        fn new(level: TestLevel, random: SeqRandom) -> Self {
            Self {
                level,
                random,
                placed: Vec::new(),
            }
        }
    }

    impl TreeDecoratorContext for TestContext {
        type Level = TestLevel;
        type Random = SeqRandom;

        fn level(&self) -> &Self::Level {
            &self.level
        }

        fn level_mut(&mut self) -> &mut Self::Level {
            &mut self.level
        }

        fn random(&mut self) -> &mut Self::Random {
            &mut self.random
        }

        fn check_block<F>(&self, pos: BlockPos, predicate: F) -> bool
        where
            F: FnOnce(&BlockState) -> bool,
        {
            let state = self.level.get_state(pos);
            predicate(&state)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState) {
            self.level.set_state(pos, state);
            self.placed.push((pos, state));
        }
    }

    fn solid() -> BlockState {
        BlockState::new(2)
    }

    fn moss() -> BlockState {
        BlockState::new(3)
    }

    #[test]
    fn place_does_nothing_for_empty_trunk_list() {
        let decorator = PlaceOnGroundDecorator::new(4, 2, 1, FixedProvider { state: moss() });
        let mut ctx = TestContext::new(TestLevel::new(), SeqRandom::new(vec![0, 0, 0]));
        decorator.place(&mut ctx, &[]);
        assert!(ctx.placed.is_empty());
    }

    #[test]
    fn places_block_above_valid_ground() {
        let decorator = PlaceOnGroundDecorator::new(1, 0, 0, FixedProvider { state: moss() });
        let mut level = TestLevel::new();
        let ground = BlockPos::new(5, 64, 7);
        level.set_state(ground, solid());
        level.set_height(5, 7, 64);
        let mut ctx = TestContext::new(level, SeqRandom::new(vec![5, 64, 7]));

        decorator.place(&mut ctx, &[BlockPos::new(5, 64, 7)]);

        assert_eq!(ctx.placed, vec![(BlockPos::new(5, 65, 7), moss())]);
        assert_eq!(ctx.level.get_state(BlockPos::new(5, 65, 7)), moss());
    }

    #[test]
    fn does_not_place_when_above_is_not_air_or_vine() {
        let decorator = PlaceOnGroundDecorator::new(1, 0, 0, FixedProvider { state: moss() });
        let mut level = TestLevel::new();
        let ground = BlockPos::new(1, 10, 2);
        level.set_state(ground, solid());
        level.set_state(ground.above(), solid()); // blocked
        level.set_height(1, 2, 10);
        let mut ctx = TestContext::new(level, SeqRandom::new(vec![1, 10, 2]));

        decorator.place(&mut ctx, &[ground]);
        assert!(ctx.placed.is_empty());
    }

    #[test]
    fn allows_placement_when_above_is_vine() {
        let decorator = PlaceOnGroundDecorator::new(1, 0, 0, FixedProvider { state: moss() });
        let mut level = TestLevel::new();
        let ground = BlockPos::new(2, 20, 3);
        level.set_state(ground, solid());
        level.set_state(ground.above(), Blocks::VINE);
        level.set_height(2, 3, 20);
        let mut ctx = TestContext::new(level, SeqRandom::new(vec![2, 20, 3]));

        decorator.place(&mut ctx, &[ground]);
        assert_eq!(ctx.placed, vec![(BlockPos::new(2, 21, 3), moss())]);
    }

    #[test]
    fn respects_heightmap_motion_blocking_no_leaves() {
        let decorator = PlaceOnGroundDecorator::new(1, 0, 0, FixedProvider { state: moss() });
        let mut level = TestLevel::new();
        let ground = BlockPos::new(0, 63, 0);
        level.set_state(ground, solid());
        level.set_height(0, 0, 65); // heightmap above target.above() => reject
        let mut ctx = TestContext::new(level, SeqRandom::new(vec![0, 63, 0]));

        decorator.place(&mut ctx, &[ground]);
        assert!(ctx.placed.is_empty());
    }

    #[test]
    fn bounding_box_uses_only_lowest_y_and_inflates_with_wrapping() {
        let decorator = PlaceOnGroundDecorator::new(1, 2, 1, FixedProvider { state: moss() });
        let mut level = TestLevel::new();
        // lowest y is 10; higher y point should not affect bounds
        let p1 = BlockPos::new(0, 10, 0);
        let p2 = BlockPos::new(2, 10, 4);
        let p3 = BlockPos::new(100, 20, 100);
        let ground = BlockPos::new(-2, 9, -2); // inside inflated box: minX=-2,minY=9,minZ=-2
        level.set_state(ground, solid());
        level.set_height(-2, -2, 9);
        let mut ctx = TestContext::new(level, SeqRandom::new(vec![-2, 9, -2]));

        decorator.place(&mut ctx, &[p1, p2, p3]);
        assert_eq!(ctx.placed, vec![(BlockPos::new(-2, 10, -2), moss())]);

        // Verify inflated_by uses wrapping semantics explicitly.
        let bb = BoundingBox::new(i32::MIN, 0, 0, i32::MAX, 0, 0).inflated_by(1, 0, 0);
        assert_eq!(bb.min_x(), i32::MAX);
        assert_eq!(bb.max_x(), i32::MIN);
    }
}
