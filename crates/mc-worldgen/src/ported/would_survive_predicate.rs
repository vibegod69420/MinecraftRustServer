use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
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

    pub fn offset(&self, offset: &Vec3i) -> Self {
        Self {
            x: self.x.wrapping_add(offset.x),
            y: self.y.wrapping_add(offset.y),
            z: self.z.wrapping_add(offset.z),
        }
    }
}

pub trait WorldGenLevel {}

/// Self-contained stub for `BlockState`.
/// `survivable_positions` models the result of `canSurvive` for testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
    pub survivable_positions: HashSet<(i32, i32, i32)>,
}

impl BlockState {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            survivable_positions: HashSet::new(),
        }
    }

    pub fn with_survivable_positions(mut self, positions: &[(i32, i32, i32)]) -> Self {
        self.survivable_positions = positions.iter().copied().collect();
        self
    }

    // Stub for Java BlockState#canSurvive(WorldGenLevel, BlockPos).
    pub fn can_survive(&self, _level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        self.survivable_positions.contains(&(pos.x, pos.y, pos.z))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPredicateType {
    WouldSurvive,
}

pub trait BlockPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool;
    fn predicate_type(&self) -> BlockPredicateType;
}

/// Port of `net.minecraft.world.level.levelgen.blockpredicates.WouldSurvivePredicate`.
#[derive(Debug, Clone)]
pub struct WouldSurvivePredicate {
    offset: Vec3i,
    state: BlockState,
}

impl WouldSurvivePredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.WouldSurvivePredicate#<init>(Vec3i,BlockState)
    pub fn new(offset: Vec3i, state: BlockState) -> Self {
        Self { offset, state }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.WouldSurvivePredicate#test(WorldGenLevel,BlockPos)
    pub fn test_at(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        self.state.can_survive(level, &pos.offset(&self.offset))
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.WouldSurvivePredicate#type()
    pub fn get_type(&self) -> BlockPredicateType {
        BlockPredicateType::WouldSurvive
    }
}

impl BlockPredicate for WouldSurvivePredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.WouldSurvivePredicate#test(WorldGenLevel,BlockPos)
    fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        self.test_at(level, pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.WouldSurvivePredicate#type()
    fn predicate_type(&self) -> BlockPredicateType {
        self.get_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel;
    impl WorldGenLevel for MockLevel {}

    #[test]
    fn survives_at_target_position() {
        let state = BlockState::new(1).with_survivable_positions(&[(5, 6, 7)]);
        let pred = WouldSurvivePredicate::new(Vec3i::ZERO, state);
        let level = MockLevel;
        assert!(pred.test(&level, &BlockPos::new(5, 6, 7)));
        assert!(!pred.test(&level, &BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn applies_offset() {
        let state = BlockState::new(2).with_survivable_positions(&[(11, 8, 13)]);
        let pred = WouldSurvivePredicate::new(Vec3i::new(1, -2, 3), state);
        let level = MockLevel;
        assert!(pred.test(&level, &BlockPos::new(10, 10, 10)));
        assert!(!pred.test(&level, &BlockPos::new(11, 8, 13)));
    }

    #[test]
    fn negative_coordinates() {
        let state = BlockState::new(3).with_survivable_positions(&[(-11, -21, -31)]);
        let pred = WouldSurvivePredicate::new(Vec3i::new(-1, -1, -1), state);
        let level = MockLevel;
        assert!(pred.test(&level, &BlockPos::new(-10, -20, -30)));
    }

    #[test]
    fn offset_wrapping_overflow() {
        let state = BlockState::new(4).with_survivable_positions(&[(i32::MIN, 0, 0)]);
        let pred = WouldSurvivePredicate::new(Vec3i::new(1, 0, 0), state);
        let level = MockLevel;
        assert!(pred.test(&level, &BlockPos::new(i32::MAX, 0, 0)));
    }

    #[test]
    fn predicate_type_is_would_survive() {
        let pred = WouldSurvivePredicate::new(Vec3i::ZERO, BlockState::new(0));
        assert_eq!(pred.predicate_type(), BlockPredicateType::WouldSurvive);
    }
}
