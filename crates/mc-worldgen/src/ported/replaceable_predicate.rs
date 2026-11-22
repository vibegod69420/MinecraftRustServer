#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn offset(&self, offset: Vec3i) -> Self {
        Self {
            x: self.x.wrapping_add(offset.x),
            y: self.y.wrapping_add(offset.y),
            z: self.z.wrapping_add(offset.z),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub replaceable: bool,
}

impl BlockState {
    pub fn can_be_replaced(&self) -> bool {
        self.replaceable
    }
}

pub trait WorldGenLevel {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockPredicateType {
    AnyOf,
    AllOf,
    Replaceable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateTestingPredicate {
    pub offset: Vec3i,
}

impl StateTestingPredicate {
    pub fn new(offset: Vec3i) -> Self {
        Self { offset }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplaceablePredicate {
    pub base: StateTestingPredicate,
}

impl ReplaceablePredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.ReplaceablePredicate#<init>
    pub fn new(offset: Vec3i) -> Self {
        Self {
            base: StateTestingPredicate::new(offset),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.ReplaceablePredicate#test
    pub fn test_state(&self, state: BlockState) -> bool {
        state.can_be_replaced()
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.ReplaceablePredicate#type
    pub fn predicate_type(&self) -> BlockPredicateType {
        BlockPredicateType::Replaceable
    }

    pub fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        let target = pos.offset(self.base.offset);
        self.test_state(level.get_block_state(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct FixedStateLevel {
        state: BlockState,
        captured: Cell<BlockPos>,
    }

    impl FixedStateLevel {
        fn new(replaceable: bool) -> Self {
            Self {
                state: BlockState { replaceable },
                captured: Cell::new(BlockPos::new(0, 0, 0)),
            }
        }
    }

    impl WorldGenLevel for FixedStateLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.captured.set(pos);
            self.state
        }
    }

    #[test]
    fn replaceable_returns_true_when_block_replaceable() {
        let level = FixedStateLevel::new(true);
        let pred = ReplaceablePredicate::new(Vec3i::new(0, 0, 0));
        assert!(pred.test(&level, BlockPos::new(1, 2, 3)));
    }

    #[test]
    fn replaceable_returns_false_when_block_not_replaceable() {
        let level = FixedStateLevel::new(false);
        let pred = ReplaceablePredicate::new(Vec3i::new(0, 0, 0));
        assert!(!pred.test(&level, BlockPos::new(-5, 64, 7)));
    }

    #[test]
    fn replaceable_applies_offset_with_wrapping() {
        let level = FixedStateLevel::new(true);
        let pred = ReplaceablePredicate::new(Vec3i::new(1, -1, i32::MIN));
        let pos = BlockPos::new(i32::MAX, i32::MIN, i32::MAX);
        pred.test(&level, pos);

        let got = level.captured.get();
        assert_eq!(got.x, i32::MIN); // i32::MAX + 1 wraps
        assert_eq!(got.y, i32::MAX); // i32::MIN + (-1) wraps
        assert_eq!(got.z, -1); // i32::MAX + i32::MIN = -1
    }

    #[test]
    fn replaceable_negative_offset() {
        let level = FixedStateLevel::new(true);
        let pred = ReplaceablePredicate::new(Vec3i::new(-10, -20, -30));
        pred.test(&level, BlockPos::new(5, 15, 25));
        let got = level.captured.get();
        assert_eq!(got, BlockPos::new(-5, -5, -5));
    }

    #[test]
    fn replaceable_type_is_replaceable() {
        let pred = ReplaceablePredicate::new(Vec3i::new(0, 0, 0));
        assert_eq!(pred.predicate_type(), BlockPredicateType::Replaceable);
    }

    #[test]
    fn replaceable_test_state_matches_can_be_replaced() {
        let pred = ReplaceablePredicate::new(Vec3i::new(0, 0, 0));
        assert!(pred.test_state(BlockState { replaceable: true }));
        assert!(!pred.test_state(BlockState { replaceable: false }));
    }
}
