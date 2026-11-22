use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Vec3i = Vec3i { x: 0, y: 0, z: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: BlockPos = BlockPos { x: 0, y: 0, z: 0 };

    // JAVA: net.minecraft.core.BlockPos#offset(int, int, int)
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> BlockPos {
        BlockPos {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

pub trait WorldGenLevel {
    // JAVA: net.minecraft.world.level.WorldGenLevel#isOutsideBuildHeight(net.minecraft.core.BlockPos)
    fn is_outside_build_height(&self, pos: &BlockPos) -> bool;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPredicateType {
    MATCHING_BLOCK_TAG,
    UNOBSTRUCTED,
    INSIDE_WORLD_BOUNDS,
    OTHER,
}

pub struct MapCodec<T>(PhantomData<T>);

impl<T> MapCodec<T> {
    pub const INSTANCE: Self = Self(PhantomData);
}

// JAVA: net.minecraft.world.level.levelgen.blockpredicates.InsideWorldBoundsPredicate
pub struct InsideWorldBoundsPredicate {
    offset: Vec3i,
}

impl InsideWorldBoundsPredicate {
    pub const CODEC: MapCodec<Self> = MapCodec::<Self>::INSTANCE;

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.InsideWorldBoundsPredicate#<init>
    pub fn new(offset: Vec3i) -> Self {
        Self { offset }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.InsideWorldBoundsPredicate#test(net.minecraft.world.level.WorldGenLevel, net.minecraft.core.BlockPos)
    pub fn test<L: WorldGenLevel>(&self, level: &L, pos: &BlockPos) -> bool {
        let offset_pos = pos.offset(self.offset.x, self.offset.y, self.offset.z);
        !level.is_outside_build_height(&offset_pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.InsideWorldBoundsPredicate#type
    pub fn predicate_type(&self) -> BlockPredicateType {
        BlockPredicateType::INSIDE_WORLD_BOUNDS
    }

    pub fn offset(&self) -> Vec3i {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        min_y: i32,
        height: i32,
    }

    impl WorldGenLevel for MockLevel {
        fn is_outside_build_height(&self, pos: &BlockPos) -> bool {
            pos.y < self.min_y || pos.y >= self.min_y.wrapping_add(self.height)
        }
    }

    #[test]
    fn test_inside_bounds() {
        let pred = InsideWorldBoundsPredicate::new(Vec3i::ZERO);
        let level = MockLevel { min_y: -64, height: 384 };
        assert!(pred.test(&level, &BlockPos { x: 0, y: 64, z: 0 }));
        assert!(pred.test(&level, &BlockPos { x: 0, y: -64, z: 0 }));
        assert!(pred.test(&level, &BlockPos { x: 0, y: 319, z: 0 }));
        assert_eq!(pred.predicate_type(), BlockPredicateType::INSIDE_WORLD_BOUNDS);
    }

    #[test]
    fn test_outside_bounds() {
        let pred = InsideWorldBoundsPredicate::new(Vec3i::ZERO);
        let level = MockLevel { min_y: -64, height: 384 };
        assert!(!pred.test(&level, &BlockPos { x: 0, y: -65, z: 0 }));
        assert!(!pred.test(&level, &BlockPos { x: 0, y: 320, z: 0 }));
    }

    #[test]
    fn test_offset_applied() {
        let pred = InsideWorldBoundsPredicate::new(Vec3i { x: 0, y: 10, z: 0 });
        let level = MockLevel { min_y: 0, height: 256 };
        // pos.y=250, offset +10 => 260, outside [0,256)
        assert!(!pred.test(&level, &BlockPos { x: 0, y: 250, z: 0 }));
        // pos.y=245, offset +10 => 255, inside
        assert!(pred.test(&level, &BlockPos { x: 0, y: 245, z: 0 }));
        assert_eq!(pred.offset(), Vec3i { x: 0, y: 10, z: 0 });
    }

    #[test]
    fn test_negative_offset() {
        let pred = InsideWorldBoundsPredicate::new(Vec3i { x: -1, y: -5, z: 2 });
        let level = MockLevel { min_y: 0, height: 128 };
        // pos.y=4, offset -5 => -1, outside
        assert!(!pred.test(&level, &BlockPos { x: 10, y: 4, z: 0 }));
        // pos.y=5, offset -5 => 0, inside
        assert!(pred.test(&level, &BlockPos { x: 10, y: 5, z: 0 }));
    }

    #[test]
    fn test_wrapping_offset() {
        struct AlwaysInside;
        impl WorldGenLevel for AlwaysInside {
            fn is_outside_build_height(&self, _pos: &BlockPos) -> bool {
                false
            }
        }
        let pred = InsideWorldBoundsPredicate::new(Vec3i { x: 1, y: 0, z: 0 });
        let pos = BlockPos { x: i32::MAX, y: 0, z: 0 };
        // wrapping_add: i32::MAX + 1 = i32::MIN, should not panic
        assert!(pred.test(&AlwaysInside, &pos));
    }
}
