// === FILE: fallen_tree_feature.rs ===

use std::collections::HashSet;

pub const STUMP_HEIGHT: i32 = 1;
pub const STUMP_HEIGHT_PLUS_EMPTY_SPACE: i32 = 2;
pub const FALLEN_LOG_MAX_FALL_HEIGHT_TO_GROUND: i32 = 5;
pub const FALLEN_LOG_MAX_GROUND_GAP: i32 = 2;
pub const FALLEN_LOG_MAX_SPACE_FROM_STUMP: i32 = 2;

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

    pub fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    pub fn relative(self, direction: Direction, amount: i32) -> Self {
        Self {
            x: self
                .x
                .wrapping_add(direction.step_x().wrapping_mul(amount)),
            y: self
                .y
                .wrapping_add(direction.step_y().wrapping_mul(amount)),
            z: self
                .z
                .wrapping_add(direction.step_z().wrapping_mul(amount)),
        }
    }

    pub fn mutable(self) -> MutableBlockPos {
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

    pub fn immutable(self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn relative(self, direction: Direction, amount: i32) -> Self {
        self.immutable().relative(direction, amount).mutable()
    }

    pub fn move_dir(&mut self, direction: Direction, amount: i32) -> &mut Self {
        self.x = self
            .x
            .wrapping_add(direction.step_x().wrapping_mul(amount));
        self.y = self
            .y
            .wrapping_add(direction.step_y().wrapping_mul(amount));
        self.z = self
            .z
            .wrapping_add(direction.step_z().wrapping_mul(amount));
        self
    }

    pub fn move_direction(&mut self, direction: Direction) -> &mut Self {
        self.move_dir(direction, 1)
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
pub enum Axis {
    X,
    Y,
    Z,
}

impl Direction {
    pub const HORIZONTAL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    pub fn step_x(self) -> i32 {
        match self {
            Direction::West => -1,
            Direction::East => 1,
            _ => 0,
        }
    }

    pub fn step_y(self) -> i32 {
        match self {
            Direction::Down => -1,
            Direction::Up => 1,
            _ => 0,
        }
    }

    pub fn step_z(self) -> i32 {
        match self {
            Direction::North => -1,
            Direction::South => 1,
            _ => 0,
        }
    }

    pub fn get_opposite(self) -> Self {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    pub fn get_axis(self) -> Axis {
        match self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
        }
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait BlockState: Clone {
    fn try_set_value_axis(self, axis: Axis) -> Self;
}

pub trait LevelAccessor {
    type State: BlockState;

    fn get_block_state(&self, pos: BlockPos) -> Self::State;
    fn set_block(&mut self, pos: BlockPos, state: Self::State, flags: i32) -> bool;
    fn is_face_sturdy(&self, solid_pos: BlockPos, query_pos: BlockPos, face: Direction) -> bool;
    fn valid_tree_pos(&self, pos: BlockPos) -> bool;
    fn mark_above_for_post_processing(&mut self, pos: BlockPos);
}

pub trait IntProvider {
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

pub trait BlockStateProvider<S: BlockState> {
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> S;
}

pub struct TreeDecoratorContext<'a, L: LevelAccessor + ?Sized> {
    pub level: &'a mut L,
    pub random: &'a mut dyn RandomSource,
    pub log_positions: HashSet<BlockPos>,
    pub leaf_positions: HashSet<BlockPos>,
}

impl<'a, L: LevelAccessor + ?Sized> TreeDecoratorContext<'a, L> {
    pub fn place_block(&mut self, pos: BlockPos, state: L::State) {
        self.level.set_block(pos, state, 19);
    }
}

pub trait TreeDecorator<L: LevelAccessor + ?Sized> {
    fn place(&self, context: &mut TreeDecoratorContext<'_, L>);
}

pub struct FallenTreeConfiguration<L: LevelAccessor + ?Sized> {
    pub log_length: Box<dyn IntProvider>,
    pub trunk_provider: Box<dyn BlockStateProvider<L::State>>,
    pub stump_decorators: Vec<Box<dyn TreeDecorator<L>>>,
    pub log_decorators: Vec<Box<dyn TreeDecorator<L>>>,
}

pub struct FeaturePlaceContext<'a, L: LevelAccessor + ?Sized> {
    pub config: &'a FallenTreeConfiguration<L>,
    pub origin: BlockPos,
    pub level: &'a mut L,
    pub random: &'a mut dyn RandomSource,
}

fn random_horizontal_direction(random: &mut dyn RandomSource) -> Direction {
    let idx = random.next_int(4).rem_euclid(4) as usize;
    Direction::HORIZONTAL[idx]
}

fn valid_tree_pos<L: LevelAccessor + ?Sized>(level: &L, pos: BlockPos) -> bool {
    level.valid_tree_pos(pos)
}

// JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#getSidewaysStateModifier
fn get_sideways_state_modifier<S: BlockState>(direction: Direction) -> impl Fn(S) -> S {
    move |state| state.try_set_value_axis(direction.get_axis())
}

pub struct FallenTreeFeature;

impl FallenTreeFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#place
    pub fn place<L: LevelAccessor + ?Sized>(
        &self,
        context: &mut FeaturePlaceContext<'_, L>,
    ) -> bool {
        self.place_fallen_tree(context.config, context.origin, context.level, context.random);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#placeFallenTree
    fn place_fallen_tree<L: LevelAccessor + ?Sized>(
        &self,
        config: &FallenTreeConfiguration<L>,
        origin: BlockPos,
        level: &mut L,
        random: &mut dyn RandomSource,
    ) {
        self.place_stump(config, level, random, origin.mutable());
        let direction = random_horizontal_direction(random);
        let length = config.log_length.sample(random).wrapping_sub(2);
        let mut fallen_pos = origin
            .relative(direction, 2_i32.wrapping_add(random.next_int(2)))
            .mutable();
        self.set_ground_height_for_fallen_log_start_pos(level, &mut fallen_pos);
        if self.can_place_entire_fallen_log(level, length, &mut fallen_pos, direction) {
            self.place_fallen_log(config, level, random, length, &mut fallen_pos, direction);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#setGroundHeightForFallenLogStartPos
    fn set_ground_height_for_fallen_log_start_pos<L: LevelAccessor + ?Sized>(
        &self,
        level: &L,
        pos: &mut MutableBlockPos,
    ) {
        pos.move_dir(Direction::Up, 1);

        for _ in 0..6 {
            if self.may_place_on(level, pos.immutable()) {
                return;
            }

            pos.move_direction(Direction::Down);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#placeStump
    fn place_stump<L: LevelAccessor + ?Sized>(
        &self,
        config: &FallenTreeConfiguration<L>,
        level: &mut L,
        random: &mut dyn RandomSource,
        pos: MutableBlockPos,
    ) {
        let stump_pos = self.place_log_block(config, level, random, pos, |s| s);
        let mut positions = HashSet::new();
        positions.insert(stump_pos);
        self.decorate_logs(level, random, positions, &config.stump_decorators);
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#canPlaceEntireFallenLog
    fn can_place_entire_fallen_log<L: LevelAccessor + ?Sized>(
        &self,
        level: &L,
        length: i32,
        pos: &mut MutableBlockPos,
        direction: Direction,
    ) -> bool {
        let mut ground_gap_count: i32 = 0;

        let mut i: i32 = 0;
        while i < length {
            if !valid_tree_pos(level, pos.immutable()) {
                return false;
            }

            if !self.is_over_solid_ground(level, pos.immutable()) {
                ground_gap_count = ground_gap_count.wrapping_add(1);
                if ground_gap_count > FALLEN_LOG_MAX_GROUND_GAP {
                    return false;
                }
            } else {
                ground_gap_count = 0;
            }

            pos.move_direction(direction);
            i = i.wrapping_add(1);
        }

        pos.move_dir(direction.get_opposite(), length);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#placeFallenLog
    fn place_fallen_log<L: LevelAccessor + ?Sized>(
        &self,
        config: &FallenTreeConfiguration<L>,
        level: &mut L,
        random: &mut dyn RandomSource,
        length: i32,
        pos: &mut MutableBlockPos,
        direction: Direction,
    ) {
        let mut log_positions: HashSet<BlockPos> = HashSet::new();
        let modifier = get_sideways_state_modifier(direction);

        let mut i: i32 = 0;
        while i < length {
            let placed = self.place_log_block(config, level, random, *pos, &modifier);
            log_positions.insert(placed);
            pos.move_direction(direction);
            i = i.wrapping_add(1);
        }

        self.decorate_logs(level, random, log_positions, &config.log_decorators);
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#mayPlaceOn
    fn may_place_on<L: LevelAccessor + ?Sized>(&self, level: &L, pos: BlockPos) -> bool {
        valid_tree_pos(level, pos) && self.is_over_solid_ground(level, pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#isOverSolidGround
    fn is_over_solid_ground<L: LevelAccessor + ?Sized>(&self, level: &L, pos: BlockPos) -> bool {
        level.is_face_sturdy(pos.below(), pos, Direction::Up)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#placeLogBlock
    fn place_log_block<L: LevelAccessor + ?Sized, F: Fn(L::State) -> L::State>(
        &self,
        config: &FallenTreeConfiguration<L>,
        level: &mut L,
        random: &mut dyn RandomSource,
        pos: MutableBlockPos,
        state_modifier: F,
    ) -> BlockPos {
        let state = state_modifier(config.trunk_provider.get_state(random, pos.immutable()));
        level.set_block(pos.immutable(), state, 3);
        level.mark_above_for_post_processing(pos.immutable());
        pos.immutable()
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#decorateLogs
    fn decorate_logs<L: LevelAccessor + ?Sized>(
        &self,
        level: &mut L,
        random: &mut dyn RandomSource,
        log_positions: HashSet<BlockPos>,
        decorators: &[Box<dyn TreeDecorator<L>>],
    ) {
        if !decorators.is_empty() {
            let mut context = TreeDecoratorContext {
                level,
                random,
                log_positions,
                leaf_positions: HashSet::new(),
            };
            for decorator in decorators {
                decorator.place(&mut context);
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FallenTreeFeature#getDecorationSetter
    fn get_decoration_setter<'a, L: LevelAccessor + ?Sized>(
        &self,
        level: &'a mut L,
    ) -> impl FnMut(BlockPos, L::State) + 'a {
        move |pos, state| {
            level.set_block(pos, state, 19);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestState {
        id: u32,
        axis: Option<Axis>,
    }

    impl BlockState for TestState {
        fn try_set_value_axis(mut self, axis: Axis) -> Self {
            self.axis = Some(axis);
            self
        }
    }

    struct TestLevel {
        blocks: HashMap<BlockPos, TestState>,
        ground_y: i32,
    }

    impl TestLevel {
        fn new(ground_y: i32) -> Self {
            Self {
                blocks: HashMap::new(),
                ground_y,
            }
        }
    }

    impl LevelAccessor for TestLevel {
        type State = TestState;

        fn get_block_state(&self, pos: BlockPos) -> Self::State {
            self.blocks
                .get(&pos)
                .cloned()
                .unwrap_or(TestState { id: 0, axis: None })
        }

        fn set_block(&mut self, pos: BlockPos, state: Self::State, _flags: i32) -> bool {
            self.blocks.insert(pos, state);
            true
        }

        fn is_face_sturdy(
            &self,
            solid_pos: BlockPos,
            _query_pos: BlockPos,
            _face: Direction,
        ) -> bool {
            solid_pos.y < self.ground_y
        }

        fn valid_tree_pos(&self, pos: BlockPos) -> bool {
            !self.blocks.contains_key(&pos)
        }

        fn mark_above_for_post_processing(&mut self, _pos: BlockPos) {}
    }

    struct SeqRandom {
        values: Vec<i32>,
        index: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.index < self.values.len() {
                let val = self.values[self.index];
                self.index = self.index.wrapping_add(1);
                val
            } else {
                0
            };
            v.rem_euclid(bound)
        }
    }

    struct ConstIntProvider(i32);
    impl IntProvider for ConstIntProvider {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }
    }

    struct SimpleBlockStateProvider {
        state: TestState,
    }
    impl BlockStateProvider<TestState> for SimpleBlockStateProvider {
        fn get_state(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> TestState {
            self.state.clone()
        }
    }

    fn make_config(log_length: i32, state_id: u32) -> FallenTreeConfiguration<TestLevel> {
        FallenTreeConfiguration {
            log_length: Box::new(ConstIntProvider(log_length)),
            trunk_provider: Box::new(SimpleBlockStateProvider {
                state: TestState {
                    id: state_id,
                    axis: None,
                },
            }),
            stump_decorators: vec![],
            log_decorators: vec![],
        }
    }

    #[test]
    fn stump_always_placed_and_returns_true() {
        let mut level = TestLevel::new(1);
        let mut random = SeqRandom::new(vec![0, 0]);
        let config = make_config(2, 1); // length = 0, no fallen log
        let origin = BlockPos::new(0, 1, 0);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin,
            level: &mut level,
            random: &mut random,
        };

        assert!(FallenTreeFeature.place(&mut ctx));
        assert_eq!(
            level.get_block_state(origin),
            TestState { id: 1, axis: None }
        );
    }

    #[test]
    fn fallen_log_places_with_sideways_axis() {
        let mut level = TestLevel::new(1);
        // direction index 0 => North, nextInt(2)=0
        let mut random = SeqRandom::new(vec![0, 0]);
        let config = make_config(5, 7); // length = 3
        let origin = BlockPos::new(0, 1, 0);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin,
            level: &mut level,
            random: &mut random,
        };

        FallenTreeFeature.place(&mut ctx);

        assert_eq!(level.get_block_state(origin).id, 7);
        for z in [-2, -3, -4] {
            let s = level.get_block_state(BlockPos::new(0, 1, z));
            assert_eq!(s.id, 7);
            assert_eq!(s.axis, Some(Axis::Z)); // North/South axis is Z
        }
    }

    #[test]
    fn fallen_log_cancelled_when_too_many_gaps() {
        struct GapLevel {
            blocks: HashMap<BlockPos, TestState>,
        }
        impl LevelAccessor for GapLevel {
            type State = TestState;
            fn get_block_state(&self, pos: BlockPos) -> Self::State {
                self.blocks
                    .get(&pos)
                    .cloned()
                    .unwrap_or(TestState { id: 0, axis: None })
            }
            fn set_block(&mut self, pos: BlockPos, state: Self::State, _flags: i32) -> bool {
                self.blocks.insert(pos, state);
                true
            }
            fn is_face_sturdy(
                &self,
                solid_pos: BlockPos,
                _query_pos: BlockPos,
                _face: Direction,
            ) -> bool {
                solid_pos.y == 0 && solid_pos.x == 0
            }
            fn valid_tree_pos(&self, pos: BlockPos) -> bool {
                !self.blocks.contains_key(&pos)
            }
            fn mark_above_for_post_processing(&mut self, _pos: BlockPos) {}
        }

        let mut level = GapLevel {
            blocks: HashMap::new(),
        };
        // direction index 1 => East, offset 0; length sample 6 => 4
        let mut random = SeqRandom::new(vec![1, 0]);
        let config = FallenTreeConfiguration {
            log_length: Box::new(ConstIntProvider(6)),
            trunk_provider: Box::new(SimpleBlockStateProvider {
                state: TestState { id: 2, axis: None },
            }),
            stump_decorators: vec![],
            log_decorators: vec![],
        };
        let origin = BlockPos::new(0, 1, 0);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin,
            level: &mut level,
            random: &mut random,
        };

        FallenTreeFeature.place(&mut ctx);

        assert!(level.blocks.contains_key(&origin)); // stump placed
        assert!(
            !level.blocks.contains_key(&BlockPos::new(2, 1, 0)),
            "fallen log should be cancelled due to excessive ground gaps"
        );
    }

    #[test]
    fn negative_coordinates_and_overflow_wrapping() {
        // Negative coordinates
        let mut level = TestLevel::new(-5);
        let mut random = SeqRandom::new(vec![0, 0]);
        let config = make_config(3, 3); // length = 1
        let origin = BlockPos::new(-10, -5, -20);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin,
            level: &mut level,
            random: &mut random,
        };
        FallenTreeFeature.place(&mut ctx);
        assert_eq!(level.get_block_state(origin).id, 3);

        // Wrapping movement
        let mut pos = MutableBlockPos::new(i32::MAX, 0, 0);
        pos.move_dir(Direction::East, 1);
        assert_eq!(pos.x, i32::MIN);

        let mut pos2 = MutableBlockPos::new(i32::MIN, 10, 0);
        pos2.move_dir(Direction::West, 1);
        assert_eq!(pos2.x, i32::MAX);

        let below = BlockPos::new(0, i32::MIN, 0).below();
        assert_eq!(below.y, i32::MAX);
    }

    #[test]
    fn can_place_entire_fallen_log_restores_position_and_resets_gaps() {
        struct CustomLevel;
        impl LevelAccessor for CustomLevel {
            type State = TestState;
            fn get_block_state(&self, _pos: BlockPos) -> Self::State {
                TestState { id: 0, axis: None }
            }
            fn set_block(&mut self, _pos: BlockPos, _state: Self::State, _flags: i32) -> bool {
                true
            }
            fn is_face_sturdy(
                &self,
                solid_pos: BlockPos,
                _query_pos: BlockPos,
                _face: Direction,
            ) -> bool {
                // gap at x==2 only; solid elsewhere in range
                matches!(solid_pos.x, 0 | 1 | 3 | 4 | 5) && solid_pos.y == 0
            }
            fn valid_tree_pos(&self, _pos: BlockPos) -> bool {
                true
            }
            fn mark_above_for_post_processing(&mut self, _pos: BlockPos) {}
        }

        let level = CustomLevel;
        let feature = FallenTreeFeature;
        let mut pos = MutableBlockPos::new(0, 1, 0);
        let original = pos.immutable();

        let ok = feature.can_place_entire_fallen_log(&level, 5, &mut pos, Direction::East);
        assert!(ok, "single gap should be allowed and count should reset");
        assert_eq!(pos.immutable(), original, "position must be restored");
    }

    #[test]
    fn float_bits_sanity_and_direction_axis_mapping() {
        // Float bits sanity (no float math in this feature, but verify bit fidelity behavior)
        let bits = 1.5f32.to_bits();
        assert_eq!(f32::from_bits(bits).to_bits(), bits);
        let neg_zero = (-0.0f64).to_bits();
        assert_ne!(neg_zero, 0.0f64.to_bits());

        assert_eq!(Direction::North.get_axis(), Axis::Z);
        assert_eq!(Direction::East.get_axis(), Axis::X);
        assert_eq!(Direction::Up.get_axis(), Axis::Y);
        assert_eq!(Direction::West.get_opposite(), Direction::East);
    }
}
