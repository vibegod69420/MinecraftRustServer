use std::marker::PhantomData;
use std::sync::LazyLock;

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
}

pub trait WorldGenLevel {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockPredicateType {
    True,
    Solid,
    AnyOf,
    AllOf,
    Replaceable,
}

pub trait BlockPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool;
    fn predicate_type(&self) -> BlockPredicateType;
}

#[derive(Debug)]
pub struct MapCodec<T>(PhantomData<fn() -> T>);

impl<T> MapCodec<T> {
    pub fn unit(_supplier: fn() -> &'static T) -> Self {
        Self(PhantomData)
    }
}

pub struct TrueBlockPredicate;

impl TrueBlockPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.TrueBlockPredicate#<init>
    const fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.TrueBlockPredicate#test
    pub fn test(&self, _level: &dyn WorldGenLevel, _pos: BlockPos) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.TrueBlockPredicate#type
    pub fn predicate_type(&self) -> BlockPredicateType {
        BlockPredicateType::True
    }
}

impl BlockPredicate for TrueBlockPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        TrueBlockPredicate::test(self, level, pos)
    }

    fn predicate_type(&self) -> BlockPredicateType {
        TrueBlockPredicate::predicate_type(self)
    }
}

// JAVA: net.minecraft.world.level.levelgen.blockpredicates.TrueBlockPredicate#INSTANCE
pub static INSTANCE: TrueBlockPredicate = TrueBlockPredicate::new();

// JAVA: net.minecraft.world.level.levelgen.blockpredicates.TrueBlockPredicate#CODEC
pub static CODEC: LazyLock<MapCodec<TrueBlockPredicate>> =
    LazyLock::new(|| MapCodec::unit(|| &INSTANCE));

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyLevel;
    impl WorldGenLevel for DummyLevel {}

    #[test]
    fn always_returns_true() {
        let level = DummyLevel;
        assert!(INSTANCE.test(&level, BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn returns_true_for_negative_coords() {
        let level = DummyLevel;
        assert!(INSTANCE.test(&level, BlockPos::new(-1, -64, i32::MIN)));
    }

    #[test]
    fn returns_true_for_extreme_coords() {
        let level = DummyLevel;
        assert!(INSTANCE.test(&level, BlockPos::new(i32::MAX, i32::MAX, i32::MIN)));
    }

    #[test]
    fn predicate_type_is_true() {
        assert_eq!(INSTANCE.predicate_type(), BlockPredicateType::True);
    }

    #[test]
    fn trait_object_works() {
        let level = DummyLevel;
        let p: &dyn BlockPredicate = &INSTANCE;
        assert!(p.test(&level, BlockPos::new(7, -3, 99)));
        assert_eq!(p.predicate_type(), BlockPredicateType::True);
    }
}
