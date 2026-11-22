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
pub struct AnyOfPredicate {
    pub predicates: Vec<Box<dyn BlockPredicate>>,
}

impl AnyOfPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.AnyOfPredicate#<init>
    pub fn new(predicates: Vec<Box<dyn BlockPredicate>>) -> Self {
        Self { predicates }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.AnyOfPredicate#test
    pub fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        for predicate in &self.predicates {
            if predicate.test(level, pos) {
                return true;
            }
        }
        false
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.AnyOfPredicate#type
    pub fn predicate_type(&self) -> BlockPredicateType {
        BlockPredicateType::AnyOf
    }
}

impl BlockPredicate for AnyOfPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        AnyOfPredicate::test(self, level, pos)
    }

    fn predicate_type(&self) -> BlockPredicateType {
        AnyOfPredicate::predicate_type(self)
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
            BlockPredicateType::AnyOf
        }
    }

    #[derive(Debug)]
    struct PanicPredicate;

    impl BlockPredicate for PanicPredicate {
        fn test(&self, _level: &dyn WorldGenLevel, _pos: BlockPos) -> bool {
            panic!("should not be called due to short-circuit");
        }

        fn predicate_type(&self) -> BlockPredicateType {
            BlockPredicateType::AnyOf
        }
    }

    #[test]
    fn any_of_empty_returns_false() {
        let pred = AnyOfPredicate::new(vec![]);
        let level = MockLevel;
        assert!(!pred.test(&level, BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn any_of_returns_true_if_any_matches() {
        let pred = AnyOfPredicate::new(vec![
            Box::new(ConstPredicate(false)),
            Box::new(ConstPredicate(true)),
            Box::new(ConstPredicate(false)),
        ]);
        let level = MockLevel;
        assert!(pred.test(&level, BlockPos::new(1, -64, 3)));
    }

    #[test]
    fn any_of_returns_false_if_none_match() {
        let pred = AnyOfPredicate::new(vec![
            Box::new(ConstPredicate(false)),
            Box::new(ConstPredicate(false)),
        ]);
        let level = MockLevel;
        assert!(!pred.test(&level, BlockPos::new(i32::MIN, i32::MAX, -1)));
    }

    #[test]
    fn any_of_short_circuits_on_first_true() {
        let pred = AnyOfPredicate::new(vec![
            Box::new(ConstPredicate(false)),
            Box::new(ConstPredicate(true)),
            Box::new(PanicPredicate),
        ]);
        let level = MockLevel;
        assert!(pred.test(&level, BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn any_of_type_is_any_of() {
        let pred = AnyOfPredicate::new(vec![]);
        assert_eq!(pred.predicate_type(), BlockPredicateType::AnyOf);
    }
}
