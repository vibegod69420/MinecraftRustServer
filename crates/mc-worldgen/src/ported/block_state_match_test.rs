use std::marker::PhantomData;

/// Minimal `RandomSource` stub for self-contained compilation.
#[derive(Debug, Clone, Copy, Default)]
pub struct RandomSource {
    pub seed: i64,
}

/// Minimal `BlockState` stub for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub id: u32,
}

/// Minimal `RuleTestType` stub for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleTestType {
    BlockStateTest,
    BlockTest,
    TagTest,
    AlwaysTrue,
}

/// Minimal `MapCodec` stub preserving the Java field shape.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> MapCodec<T> {
    pub const fn placeholder() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Base trait mirroring `RuleTest`.
pub trait RuleTest {
    fn test(&self, state: &BlockState, random: &mut RandomSource) -> bool;
    fn get_type(&self) -> RuleTestType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockStateMatchTest {
    block_state: BlockState,
}

impl BlockStateMatchTest {
    pub const CODEC: MapCodec<BlockStateMatchTest> = MapCodec::placeholder();

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockStateMatchTest#<init>
    pub fn new(block_state: BlockState) -> Self {
        Self { block_state }
    }

    pub fn block_state(&self) -> BlockState {
        self.block_state
    }
}

impl RuleTest for BlockStateMatchTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockStateMatchTest#test
    fn test(&self, state: &BlockState, _random: &mut RandomSource) -> bool {
        *state == self.block_state
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockStateMatchTest#getType
    fn get_type(&self) -> RuleTestType {
        RuleTestType::BlockStateTest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_same_block_state() {
        let state = BlockState { id: 5 };
        let test = BlockStateMatchTest::new(state);
        let mut rng = RandomSource::default();
        assert!(test.test(&state, &mut rng));
    }

    #[test]
    fn rejects_different_block_state() {
        let test = BlockStateMatchTest::new(BlockState { id: 1 });
        let mut rng = RandomSource::default();
        assert!(!test.test(&BlockState { id: 2 }, &mut rng));
    }

    #[test]
    fn get_type_is_blockstate_test() {
        let test = BlockStateMatchTest::new(BlockState { id: 0 });
        assert_eq!(test.get_type(), RuleTestType::BlockStateTest);
    }

    #[test]
    fn edge_case_ids_and_negative_seed() {
        let test = BlockStateMatchTest::new(BlockState { id: u32::MAX });
        let mut rng = RandomSource { seed: -1 };
        assert!(test.test(&BlockState { id: u32::MAX }, &mut rng));
        assert!(!test.test(&BlockState { id: u32::MIN }, &mut rng));
    }
}
