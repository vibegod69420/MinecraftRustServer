use std::marker::PhantomData;

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

    pub fn offset(&self, offset: Vec3i) -> Self {
        Self {
            x: self.x.wrapping_add(offset.x),
            y: self.y.wrapping_add(offset.y),
            z: self.z.wrapping_add(offset.z),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockState {
    solid: bool,
}

impl BlockState {
    pub fn new(solid: bool) -> Self {
        Self { solid }
    }

    pub fn is_solid(&self) -> bool {
        self.solid
    }
}

pub trait WorldGenLevel {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
}

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

pub struct SolidPredicate {
    pub offset: Vec3i,
}

impl SolidPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.SolidPredicate#<init>
    pub fn new(offset: Vec3i) -> Self {
        Self { offset }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.StateTestingPredicate#test
    pub fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        let state = level.get_block_state(pos.offset(self.offset));
        self.test_state(&state)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.SolidPredicate#test
    fn test_state(&self, state: &BlockState) -> bool {
        state.is_solid()
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.SolidPredicate#type
    pub fn predicate_type(&self) -> BlockPredicateType {
        BlockPredicateType::Solid
    }
}

impl BlockPredicate for SolidPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: BlockPos) -> bool {
        SolidPredicate::test(self, level, pos)
    }

    fn predicate_type(&self) -> BlockPredicateType {
        SolidPredicate::predicate_type(self)
    }
}

// JAVA: net.minecraft.world.level.levelgen.blockpredicates.SolidPredicate#CODEC
pub static CODEC: MapCodec<SolidPredicate> = MapCodec::placeholder();

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        states: HashMap<(i32, i32, i32), BlockState>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                states: HashMap::new(),
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, solid: bool) {
            self.states.insert((x, y, z), BlockState::new(solid));
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or_default()
        }
    }

    #[test]
    fn solid_block_returns_true() {
        let mut level = MockLevel::new();
        level.set(1, 2, 3, true);
        let p = SolidPredicate::new(Vec3i::ZERO);
        assert!(p.test(&level, BlockPos::new(1, 2, 3)));
    }

    #[test]
    fn non_solid_block_returns_false() {
        let mut level = MockLevel::new();
        level.set(1, 2, 3, false);
        let p = SolidPredicate::new(Vec3i::ZERO);
        assert!(!p.test(&level, BlockPos::new(1, 2, 3)));
    }

    #[test]
    fn offset_is_applied() {
        let mut level = MockLevel::new();
        level.set(1, 3, 3, true);
        let p = SolidPredicate::new(Vec3i::new(0, 1, 0));
        assert!(p.test(&level, BlockPos::new(1, 2, 3)));
    }

    #[test]
    fn negative_offset_and_coords() {
        let mut level = MockLevel::new();
        level.set(-11, -21, -31, true);
        let p = SolidPredicate::new(Vec3i::new(-1, -1, -1));
        assert!(p.test(&level, BlockPos::new(-10, -20, -30)));
    }

    #[test]
    fn offset_wrapping_overflow() {
        let mut level = MockLevel::new();
        level.set(i32::MIN, 0, 0, true);
        let p = SolidPredicate::new(Vec3i::new(1, 0, 0));
        assert!(p.test(&level, BlockPos::new(i32::MAX, 0, 0)));
    }

    #[test]
    fn predicate_type_is_solid() {
        let p = SolidPredicate::new(Vec3i::ZERO);
        assert_eq!(p.predicate_type(), BlockPredicateType::Solid);
    }
}
