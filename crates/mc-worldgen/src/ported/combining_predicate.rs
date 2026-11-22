use std::marker::PhantomData;

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
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

pub struct CombiningPredicate {
    pub predicates: Vec<Box<dyn BlockPredicate>>,
}

impl CombiningPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.CombiningPredicate#<init>
    pub fn new(predicates: Vec<Box<dyn BlockPredicate>>) -> Self {
        Self { predicates }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.CombiningPredicate#codec
    pub fn codec<T>(_factory: fn(Vec<Box<dyn BlockPredicate>>) -> T) -> MapCodec<T> {
        MapCodec::placeholder()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyLevel;
    impl WorldGenLevel for DummyLevel {}

    struct AlwaysTrue;
    impl BlockPredicate for AlwaysTrue {
        fn test(&self, _: &dyn WorldGenLevel, _: BlockPos) -> bool {
            true
        }

        fn predicate_type(&self) -> BlockPredicateType {
            BlockPredicateType::True
        }
    }

    #[test]
    fn stores_predicates() {
        let cp = CombiningPredicate::new(vec![Box::new(AlwaysTrue)]);
        assert_eq!(cp.predicates.len(), 1);
    }

    #[test]
    fn empty_predicates() {
        let cp = CombiningPredicate::new(vec![]);
        assert!(cp.predicates.is_empty());
    }

    #[test]
    fn multiple_predicates_stored() {
        let cp = CombiningPredicate::new(vec![
            Box::new(AlwaysTrue),
            Box::new(AlwaysTrue),
            Box::new(AlwaysTrue),
        ]);
        assert_eq!(cp.predicates.len(), 3);
        let level = DummyLevel;
        assert!(cp.predicates[0].test(&level, BlockPos::new(-1, i32::MIN, i32::MAX)));
    }

    #[test]
    fn codec_placeholder_compiles() {
        let _codec: MapCodec<CombiningPredicate> =
            CombiningPredicate::codec(CombiningPredicate::new);
    }
}
