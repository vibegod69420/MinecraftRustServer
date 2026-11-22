use std::fmt::Debug;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fluid(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T: Debug + Clone + PartialEq> {
    pub entries: Vec<T>,
}

impl<T: Debug + Clone + PartialEq> HolderSet<T> {
    pub fn new(entries: Vec<T>) -> Self {
        Self { entries }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.entries.iter().any(|e| e == value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidState {
    pub fluid: Fluid,
}

impl FluidState {
    pub fn new(fluid: Fluid) -> Self {
        Self { fluid }
    }

    pub fn is_fluid(&self, fluids: &HolderSet<Fluid>) -> bool {
        fluids.contains(&self.fluid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub fluid_state: FluidState,
}

impl BlockState {
    pub fn new(fluid: Fluid) -> Self {
        Self {
            fluid_state: FluidState::new(fluid),
        }
    }

    pub fn get_fluid_state(&self) -> &FluidState {
        &self.fluid_state
    }
}

pub trait WorldGenLevel {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPredicateType {
    MatchingFluids,
}

pub trait BlockPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool;
    fn predicate_type(&self) -> BlockPredicateType;
}

/// Port of `net.minecraft.world.level.levelgen.blockpredicates.MatchingFluidsPredicate`.
/// Extends `StateTestingPredicate` in Java; the offset+lookup delegation is inlined here
/// for self-containment.
#[derive(Debug, Clone)]
pub struct MatchingFluidsPredicate {
    offset: Vec3i,
    fluids: HolderSet<Fluid>,
}

impl MatchingFluidsPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingFluidsPredicate#<init>(Vec3i,HolderSet)
    pub fn new(offset: Vec3i, fluids: HolderSet<Fluid>) -> Self {
        Self { offset, fluids }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.StateTestingPredicate#test(WorldGenLevel,BlockPos)
    pub fn test_at(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        let state = level.get_block_state(&pos.offset(&self.offset));
        self.test_state(&state)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingFluidsPredicate#test(BlockState)
    fn test_state(&self, state: &BlockState) -> bool {
        state.get_fluid_state().is_fluid(&self.fluids)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingFluidsPredicate#type()
    pub fn get_type(&self) -> BlockPredicateType {
        BlockPredicateType::MatchingFluids
    }
}

impl BlockPredicate for MatchingFluidsPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.StateTestingPredicate#test(WorldGenLevel,BlockPos)
    fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        self.test_at(level, pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingFluidsPredicate#type()
    fn predicate_type(&self) -> BlockPredicateType {
        self.get_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        states: HashMap<(i32, i32, i32), BlockState>,
        default: BlockState,
    }

    impl MockLevel {
        fn new(default_fluid: u32) -> Self {
            Self {
                states: HashMap::new(),
                default: BlockState::new(Fluid(default_fluid)),
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, fluid_id: u32) {
            self.states
                .insert((x, y, z), BlockState::new(Fluid(fluid_id)));
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(|| self.default.clone())
        }
    }

    fn holder(ids: &[u32]) -> HolderSet<Fluid> {
        HolderSet::new(ids.iter().copied().map(Fluid).collect())
    }

    #[test]
    fn matches_fluid_in_set() {
        let pred = MatchingFluidsPredicate::new(Vec3i::ZERO, holder(&[10, 20]));
        let mut level = MockLevel::new(0);
        level.set(1, 2, 3, 20);
        assert!(pred.test(&level, &BlockPos::new(1, 2, 3)));
        assert!(!pred.test(&level, &BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn applies_offset() {
        let pred = MatchingFluidsPredicate::new(Vec3i::new(0, -1, 0), holder(&[7]));
        let mut level = MockLevel::new(0);
        level.set(5, 9, 5, 7);
        assert!(pred.test(&level, &BlockPos::new(5, 10, 5)));
    }

    #[test]
    fn negative_coordinates() {
        let pred = MatchingFluidsPredicate::new(Vec3i::new(-2, 0, -3), holder(&[3]));
        let mut level = MockLevel::new(0);
        level.set(-12, -5, -23, 3);
        assert!(pred.test(&level, &BlockPos::new(-10, -5, -20)));
    }

    #[test]
    fn offset_wrapping_overflow() {
        let pred = MatchingFluidsPredicate::new(Vec3i::new(0, 1, 0), holder(&[42]));
        let mut level = MockLevel::new(0);
        level.set(0, i32::MIN, 0, 42);
        assert!(pred.test(&level, &BlockPos::new(0, i32::MAX, 0)));
    }

    #[test]
    fn predicate_type_is_matching_fluids() {
        let pred = MatchingFluidsPredicate::new(Vec3i::ZERO, holder(&[]));
        assert_eq!(
            pred.predicate_type(),
            BlockPredicateType::MatchingFluids
        );
    }
}
