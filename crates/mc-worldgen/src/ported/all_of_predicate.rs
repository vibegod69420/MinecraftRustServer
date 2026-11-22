use std::fmt;

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
}

pub trait WorldGenLevel {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockPredicateType {
    AnyOf,
    AllOf,
    Replaceable,
}

pub trait BlockPredicate: fmt::Debug {
    fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool;
    fn predicate_type(&self) -> BlockPredicateType;
}

#[derive(Debug)]
pub struct AllOfPredicate {
    pub predicates: Vec<Box<dyn BlockPredicate>>,
}

impl AllOfPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.AllOfPredicate#<init>
    pub fn new(predicates: Vec<Box<dyn BlockPredicate>>) -> Self {
        Self { predicates }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.AllOfPredicate#test
    pub fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        for predicate in &self.predicates {
            if !predicate.test(level, pos) {
                return false;
            }
        }
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.AllOfPredicate#type
    pub fn predicate_type(&self) -> BlockPredicateType {
        BlockPredicateType::AllOf
    }
}

impl BlockPredicate for AllOfPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        AllOfPredicate::test(self, level, pos)
    }

    fn predicate_type(&self) -> BlockPredicateType {
        AllOfPredicate::predicate_type(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel;
    impl WorldGenLevel for MockLevel {}

    #[derive(Debug)]
    struct ConstPredicate(bool);

    impl BlockPredicate for ConstPredicate {
        fn test(&self, _level: &dyn WorldGenLevel, _pos: BlockPos) -> bool {
            self.0
        }

        fn predicate_type(&self) -> BlockPredicateType {
            BlockPredicateType::AllOf
        }
    }

    #[derive(Debug)]
    struct PanicPredicate;

    impl BlockPredicate for PanicPredicate {
        fn test(&self, _level: &dyn WorldGenLevel, _pos: BlockPos) -> bool {
            panic!("should not be called due to short-circuit");
        }

        fn predicate_type(&self) -> BlockPredicateType {
            BlockPredicateType::AllOf
        }
    }

    #[test]
    fn all_of_empty_returns_true() {
        let pred = AllOfPredicate::new(vec![]);
        let level = MockLevel;
        assert!(pred.test(&level, BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn all_of_returns_true_if_all_match() {
        let pred = AllOfPredicate::new(vec![
            Box::new(ConstPredicate(true)),
            Box::new(ConstPredicate(true)),
        ]);
        let level = MockLevel;
        assert!(pred.test(&level, BlockPos::new(-1, 64, i32::MAX)));
    }

    #[test]
    fn all_of_returns_false_if_any_fails() {
        let pred = AllOfPredicate::new(vec![
            Box::new(ConstPredicate(true)),
            Box::new(ConstPredicate(false)),
            Box::new(ConstPredicate(true)),
        ]);
        let level = MockLevel;
        assert!(!pred.test(&level, BlockPos::new(7, -8, 9)));
    }

    #[test]
    fn all_of_short_circuits_on_first_false() {
        let pred = AllOfPredicate::new(vec![
            Box::new(ConstPredicate(true)),
            Box::new(ConstPredicate(false)),
            Box::new(PanicPredicate),
        ]);
        let level = MockLevel;
        assert!(!pred.test(&level, BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn all_of_type_is_all_of() {
        let pred = AllOfPredicate::new(vec![]);
        assert_eq!(pred.predicate_type(), BlockPredicateType::AllOf);
    }
}
