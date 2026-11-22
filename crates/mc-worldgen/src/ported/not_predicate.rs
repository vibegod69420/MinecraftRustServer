use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.BlockPos#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> BlockPos {
        BlockPos {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockPredicateType {
    NOT,
    MATCHING_BLOCK_TAG,
    UNOBSTRUCTED,
    INSIDE_WORLD_BOUNDS,
}

pub struct WorldGenLevel;

pub trait BlockPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.BlockPredicate#test
    fn test(&self, _level: &WorldGenLevel, _pos: &BlockPos) -> bool;

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.BlockPredicate#type
    fn type_(&self) -> BlockPredicateType;
}

pub struct NotPredicate {
    predicate: Box<dyn BlockPredicate>,
}

impl NotPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.NotPredicate#<init>
    pub fn new(predicate: Box<dyn BlockPredicate>) -> Self {
        NotPredicate { predicate }
    }
}

impl BlockPredicate for NotPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.NotPredicate#test
    fn test(&self, _level: &WorldGenLevel, _pos: &BlockPos) -> bool {
        !self.predicate.test(_level, _pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.NotPredicate#type
    fn type_(&self) -> BlockPredicateType {
        BlockPredicateType::NOT
    }
}

struct MapCodec<T>(PhantomData<T>);
impl<T> MapCodec<T> {
    pub const INSTANCE: Self = MapCodec(PhantomData);
}

impl NotPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.NotPredicate#CODEC (as const)
    pub const CODEC: MapCodec<Self> = MapCodec::<Self>::INSTANCE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_predicate_negates_true() {
        let inner = NeverPredicate;
        let predicate = NotPredicate::new(Box::new(inner));
        let level = WorldGenLevel;
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        // NeverPredicate.test returns false, so NotPredicate.test should return true
        assert!(predicate.test(&level, &pos));
    }

    #[test]
    fn test_not_predicate_negates_false() {
        let inner = AlwaysTruePredicate;
        let predicate = NotPredicate::new(Box::new(inner));
        let level = WorldGenLevel;
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        // AlwaysTruePredicate.test returns true, so NotPredicate.test should return false
        assert!(!predicate.test(&level, &pos));
    }

    #[test]
    fn test_not_predicate_type() {
        let predicate = NotPredicate::new(Box::new(AlwaysTruePredicate));
        assert_eq!(predicate.type_(), BlockPredicateType::NOT);
    }

    #[test]
    fn test_not_predicate_codec() {
        let _codec = NotPredicate::CODEC;
    }

    #[test]
    fn test_not_predicate_offset_in_wrapping() {
        let pos = BlockPos { x: i32::MAX, y: 0, z: 0 };
        let offset_pos = pos.offset(1, 0, 0);
        // Java int wrapping: i32::MAX + 1 = i32::MIN
        assert_eq!(offset_pos.x, i32::MIN);
        assert_eq!(offset_pos.y, 0);
        assert_eq!(offset_pos.z, 0);
    }
}

// --- Helper predicates for tests ---

struct NeverPredicate;
impl BlockPredicate for NeverPredicate {
    fn test(&self, _: &WorldGenLevel, _: &BlockPos) -> bool { false }
    fn type_(&self) -> BlockPredicateType { BlockPredicateType::NOT }
}

struct AlwaysTruePredicate;
impl BlockPredicate for AlwaysTruePredicate {
    fn test(&self, _: &WorldGenLevel, _: &BlockPos) -> bool { true }
    fn type_(&self) -> BlockPredicateType { BlockPredicateType::NOT }
}
