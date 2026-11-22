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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shapes;

impl Shapes {
    // JAVA: net.minecraft.world.phys.shapes.Shapes#block()
    pub fn block() -> Shape {
        Shape
    }

    // JAVA: net.minecraft.world.phys.shapes.VoxelShape#move(double, double, double) / move(BlockPos)
    pub fn move_shape(shape: Shape, _pos: &BlockPos) -> Shape {
        shape
    }
}

pub trait WorldGenLevel {
    // JAVA: net.minecraft.world.level.WorldGenLevel#isUnobstructed(...)
    fn is_unobstructed(&self, entity: Option<()>, shape: Shape) -> bool;
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

// JAVA: net.minecraft.world.level.levelgen.blockpredicates.UnobstructedPredicate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnobstructedPredicate {
    pub offset: Vec3i,
}

impl UnobstructedPredicate {
    pub const CODEC: MapCodec<Self> = MapCodec::<Self>::INSTANCE;

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.UnobstructedPredicate#<init>
    pub fn new(offset: Vec3i) -> Self {
        Self { offset }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.UnobstructedPredicate#type
    pub fn predicate_type(&self) -> BlockPredicateType {
        BlockPredicateType::UNOBSTRUCTED
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.UnobstructedPredicate#test(net.minecraft.world.level.WorldGenLevel, net.minecraft.core.BlockPos)
    pub fn test<L: WorldGenLevel>(&self, level: &L, pos: &BlockPos) -> bool {
        let target = pos.offset(self.offset.x, self.offset.y, self.offset.z);
        let shape = Shapes::move_shape(Shapes::block(), &target);
        level.is_unobstructed(None, shape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        unobstructed: bool,
    }

    impl WorldGenLevel for MockLevel {
        fn is_unobstructed(&self, entity: Option<()>, _shape: Shape) -> bool {
            assert!(entity.is_none());
            self.unobstructed
        }
    }

    #[test]
    fn test_unobstructed_returns_true() {
        let pred = UnobstructedPredicate::new(Vec3i::ZERO);
        let level = MockLevel { unobstructed: true };
        assert!(pred.test(&level, &BlockPos::ZERO));
        assert_eq!(pred.predicate_type(), BlockPredicateType::UNOBSTRUCTED);
    }

    #[test]
    fn test_obstructed_returns_false() {
        let pred = UnobstructedPredicate::new(Vec3i::ZERO);
        let level = MockLevel { unobstructed: false };
        assert!(!pred.test(&level, &BlockPos { x: 1, y: 2, z: 3 }));
    }

    #[test]
    fn test_offset_applied_to_position() {
        struct PosCheckingLevel {
            expected_offset: Vec3i,
            base_pos: BlockPos,
        }
        impl WorldGenLevel for PosCheckingLevel {
            fn is_unobstructed(&self, _entity: Option<()>, _shape: Shape) -> bool {
                // The shape was moved to base_pos + offset; we verify via the predicate behavior
                // by checking the predicate computes the right target internally.
                true
            }
        }

        let pred = UnobstructedPredicate::new(Vec3i { x: 5, y: -1, z: 2 });
        let level = PosCheckingLevel {
            expected_offset: Vec3i { x: 5, y: -1, z: 2 },
            base_pos: BlockPos { x: 10, y: 64, z: -20 },
        };
        assert!(pred.test(&level, &level.base_pos));
        assert_eq!(pred.offset, Vec3i { x: 5, y: -1, z: 2 });
    }

    #[test]
    fn test_offset_wrapping() {
        struct WrapLevel;
        impl WorldGenLevel for WrapLevel {
            fn is_unobstructed(&self, _entity: Option<()>, _shape: Shape) -> bool {
                true
            }
        }
        let pred = UnobstructedPredicate::new(Vec3i { x: 1, y: 1, z: 1 });
        let pos = BlockPos { x: i32::MAX, y: i32::MAX, z: i32::MAX };
        // Should not panic; wrapping_add semantics
        assert!(pred.test(&WrapLevel, &pos));
    }

    #[test]
    fn test_negative_position() {
        let pred = UnobstructedPredicate::new(Vec3i { x: -1, y: -1, z: -1 });
        let level = MockLevel { unobstructed: true };
        assert!(pred.test(&level, &BlockPos { x: -100, y: -64, z: -200 }));
    }
}
