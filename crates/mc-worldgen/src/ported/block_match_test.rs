use std::marker::PhantomData;

/// Minimal block identifier stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: u32,
}

/// Minimal block state stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
}

impl BlockState {
    pub fn is_block(&self, block: Block) -> bool {
        self.block == block
    }
}

/// Minimal random source stub.
#[derive(Debug, Default)]
pub struct RandomSource;

/// Placeholder for Mojang's serialization `MapCodec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<T>(PhantomData<fn() -> T>);

impl<T> MapCodec<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Minimal rule test type stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTestType {
    BlockTest,
    TagTest,
}

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.BlockMatchTest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockMatchTest {
    block: Block,
}

pub const CODEC: MapCodec<BlockMatchTest> = MapCodec::new();

impl BlockMatchTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockMatchTest#<init>
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockMatchTest#test
    pub fn test(&self, state: &BlockState, _random: &mut RandomSource) -> bool {
        state.is_block(self.block)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockMatchTest#getType
    pub fn get_type(&self) -> RuleTestType {
        RuleTestType::BlockTest
    }

    pub fn block(&self) -> Block {
        self.block
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_same_block() {
        let block = Block { id: 1 };
        let rule = BlockMatchTest::new(block);
        let state = BlockState { block };
        let mut rng = RandomSource;
        assert!(rule.test(&state, &mut rng));
    }

    #[test]
    fn test_does_not_match_different_block() {
        let rule = BlockMatchTest::new(Block { id: 1 });
        let state = BlockState {
            block: Block { id: 2 },
        };
        let mut rng = RandomSource;
        assert!(!rule.test(&state, &mut rng));
    }

    #[test]
    fn get_type_is_block_test() {
        let rule = BlockMatchTest::new(Block { id: 0 });
        assert_eq!(rule.get_type(), RuleTestType::BlockTest);
    }

    #[test]
    fn test_matches_block_id_zero_and_rejects_max() {
        let rule = BlockMatchTest::new(Block { id: 0 });
        let mut rng = RandomSource;
        assert!(rule.test(&BlockState { block: Block { id: 0 } }, &mut rng));
        assert!(!rule.test(
            &BlockState {
                block: Block { id: u32::MAX },
            },
            &mut rng
        ));
    }

    #[test]
    fn test_matches_block_id_max() {
        let rule = BlockMatchTest::new(Block { id: u32::MAX });
        let mut rng = RandomSource;
        assert!(rule.test(
            &BlockState {
                block: Block { id: u32::MAX },
            },
            &mut rng
        ));
        assert!(!rule.test(
            &BlockState {
                block: Block {
                    id: u32::MAX.wrapping_sub(1),
                },
            },
            &mut rng
        ));
    }
}
