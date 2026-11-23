use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
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

    pub fn mutable(&self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
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

    pub fn move_direction(&mut self, dir: Direction) -> &mut Self {
        let (dx, dy, dz) = dir.step();
        self.x = self.x.wrapping_add(dx);
        self.y = self.y.wrapping_add(dy);
        self.z = self.z.wrapping_add(dz);
        self
    }

    pub fn as_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn step(&self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }
}

pub trait WorldGenLevel {
    fn is_outside_build_height(&self, y: i32) -> bool;
}

pub trait PlacementContext {
    fn get_level(&self) -> &dyn WorldGenLevel;
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

impl<R: RandomSource + ?Sized> RandomSource for &mut R {
    fn next_int(&mut self, bound: i32) -> i32 {
        (*self).next_int(bound)
    }
}

pub struct BlockPredicate {
    predicate: Box<dyn Fn(&dyn WorldGenLevel, BlockPos) -> bool>,
}

impl BlockPredicate {
    pub fn always_true() -> Self {
        Self {
            predicate: Box::new(|_, _| true),
        }
    }

    pub fn custom<F: Fn(&dyn WorldGenLevel, BlockPos) -> bool + 'static>(f: F) -> Self {
        Self {
            predicate: Box::new(f),
        }
    }

    pub fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        (self.predicate)(level, pos)
    }
}

impl fmt::Debug for BlockPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BlockPredicate(..)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacementModifierType {
    pub name: &'static str,
}

impl PlacementModifierType {
    pub const ENVIRONMENT_SCAN: Self = Self {
        name: "environment_scan",
    };
}

/// Port of `net.minecraft.world.level.levelgen.placement.EnvironmentScanPlacement`.
pub struct EnvironmentScanPlacement {
    direction_of_search: Direction,
    target_condition: BlockPredicate,
    allowed_search_condition: BlockPredicate,
    max_steps: i32,
}

impl fmt::Debug for EnvironmentScanPlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnvironmentScanPlacement")
            .field("direction_of_search", &self.direction_of_search)
            .field("target_condition", &self.target_condition)
            .field("allowed_search_condition", &self.allowed_search_condition)
            .field("max_steps", &self.max_steps)
            .finish()
    }
}

impl EnvironmentScanPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.EnvironmentScanPlacement#<init>
    fn new(
        direction_of_search: Direction,
        target_condition: BlockPredicate,
        allowed_search_condition: BlockPredicate,
        max_steps: i32,
    ) -> Self {
        Self {
            direction_of_search,
            target_condition,
            allowed_search_condition,
            max_steps,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.EnvironmentScanPlacement#scanningFor
    pub fn scanning_for(
        direction_of_search: Direction,
        target_condition: BlockPredicate,
        allowed_search_condition: BlockPredicate,
        max_steps: i32,
    ) -> Self {
        Self::new(
            direction_of_search,
            target_condition,
            allowed_search_condition,
            max_steps,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.EnvironmentScanPlacement#scanningFor
    pub fn scanning_for_default(
        direction_of_search: Direction,
        target_condition: BlockPredicate,
        max_steps: i32,
    ) -> Self {
        Self::scanning_for(
            direction_of_search,
            target_condition,
            BlockPredicate::always_true(),
            max_steps,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.EnvironmentScanPlacement#getPositions
    pub fn get_positions(
        &self,
        ctx: &impl PlacementContext,
        _random: &mut impl RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        let mut mutable = pos.mutable();
        let level = ctx.get_level();
        if !self
            .allowed_search_condition
            .test(level, mutable.as_block_pos())
        {
            return Vec::new();
        }

        let mut i: i32 = 0;
        while i < self.max_steps {
            if self.target_condition.test(level, mutable.as_block_pos()) {
                return vec![mutable.as_block_pos()];
            }

            mutable.move_direction(self.direction_of_search);
            if level.is_outside_build_height(mutable.get_y()) {
                return Vec::new();
            }

            if !self
                .allowed_search_condition
                .test(level, mutable.as_block_pos())
            {
                break;
            }

            i = i.wrapping_add(1);
        }

        if self.target_condition.test(level, mutable.as_block_pos()) {
            vec![mutable.as_block_pos()]
        } else {
            Vec::new()
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.EnvironmentScanPlacement#type
    pub fn placement_type(&self) -> PlacementModifierType {
        PlacementModifierType::ENVIRONMENT_SCAN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestLevel {
        min_y: i32,
        max_y: i32,
    }

    impl WorldGenLevel for TestLevel {
        fn is_outside_build_height(&self, y: i32) -> bool {
            y < self.min_y || y >= self.max_y
        }
    }

    struct TestContext {
        level: TestLevel,
    }

    impl PlacementContext for TestContext {
        fn get_level(&self) -> &dyn WorldGenLevel {
            &self.level
        }
    }

    struct NoopRandom;

    impl RandomSource for NoopRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
    }

    fn ctx(min_y: i32, max_y: i32) -> TestContext {
        TestContext {
            level: TestLevel { min_y, max_y },
        }
    }

    #[test]
    fn finds_target_at_start() {
        let ctx = ctx(0, 256);
        let mut rng = NoopRandom;
        let target = BlockPredicate::custom(|_, pos| pos.get_y() == 10);
        let placement = EnvironmentScanPlacement::scanning_for_default(Direction::Up, target, 5);
        let result = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 10, 0));
        assert_eq!(result, vec![BlockPos::new(0, 10, 0)]);
        assert_eq!(
            placement.placement_type(),
            PlacementModifierType::ENVIRONMENT_SCAN
        );
    }

    #[test]
    fn scans_upward_to_find_target() {
        let ctx = ctx(0, 256);
        let mut rng = NoopRandom;
        let target = BlockPredicate::custom(|_, pos| pos.get_y() == 13);
        let placement = EnvironmentScanPlacement::scanning_for_default(Direction::Up, target, 5);
        let result = placement.get_positions(&ctx, &mut rng, BlockPos::new(1, 10, 2));
        assert_eq!(result, vec![BlockPos::new(1, 13, 2)]);
    }

    #[test]
    fn returns_empty_when_outside_build_height() {
        let ctx = ctx(0, 12);
        let mut rng = NoopRandom;
        let target = BlockPredicate::custom(|_, pos| pos.get_y() == 100);
        let placement = EnvironmentScanPlacement::scanning_for_default(Direction::Up, target, 10);
        let result = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 10, 0));
        assert!(result.is_empty());
    }

    #[test]
    fn break_on_disallowed_still_runs_final_target_check() {
        let ctx = ctx(0, 256);
        let mut rng = NoopRandom;
        let target = BlockPredicate::custom(|_, pos| pos.get_y() == 11);
        let allowed = BlockPredicate::custom(|_, pos| pos.get_y() <= 10);
        let placement = EnvironmentScanPlacement::scanning_for(Direction::Up, target, allowed, 5);
        let result = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 10, 0));
        assert_eq!(result, vec![BlockPos::new(0, 11, 0)]);
    }

    #[test]
    fn final_target_check_after_max_steps() {
        let ctx = ctx(0, 256);
        let mut rng = NoopRandom;
        let target = BlockPredicate::custom(|_, pos| pos.get_y() == 12);
        let placement = EnvironmentScanPlacement::scanning_for_default(Direction::Up, target, 2);
        let result = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 10, 0));
        assert_eq!(result, vec![BlockPos::new(0, 12, 0)]);
    }

    #[test]
    fn initial_not_allowed_returns_empty() {
        let ctx = ctx(0, 256);
        let mut rng = NoopRandom;
        let target = BlockPredicate::always_true();
        let allowed = BlockPredicate::custom(|_, _| false);
        let placement = EnvironmentScanPlacement::scanning_for(Direction::Up, target, allowed, 5);
        let result = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 10, 0));
        assert!(result.is_empty());
    }
}
