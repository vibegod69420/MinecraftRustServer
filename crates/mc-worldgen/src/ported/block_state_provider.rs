use std::fmt::Debug;

/// Minimal block state representation for self-contained porting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: i32,
}

/// Minimal block representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub default_state: BlockState,
}

impl Block {
    // JAVA: net.minecraft.world.level.block.Block#defaultBlockState
    pub fn default_block_state(&self) -> BlockState {
        self.default_state.clone()
    }
}

/// Minimal block position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Minimal random source interface.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Block state provider type discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStateProviderType {
    SimpleStateProvider,
    RotatedBlockProvider,
    WeightedStateProvider,
}

/// A simple block state provider that always returns the same state.
/// Included here because `BlockStateProvider::simple(...)` constructs it.
// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleStateProvider {
    pub state: BlockState,
}

impl SimpleStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider#<init>
    pub fn new(state: BlockState) -> Self {
        Self { state }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider#type
    pub fn provider_type(&self) -> BlockStateProviderType {
        BlockStateProviderType::SimpleStateProvider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider#getState
    pub fn get_state(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
        self.state.clone()
    }
}

/// Abstract base for block state providers.
// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider
pub trait BlockStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider#type
    fn provider_type(&self) -> BlockStateProviderType;

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider#getState
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState;
}

impl BlockStateProvider for SimpleStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider#type
    fn provider_type(&self) -> BlockStateProviderType {
        self.provider_type()
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider#getState
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState {
        self.get_state(random, pos)
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider#simple
pub fn simple_from_state(state: BlockState) -> SimpleStateProvider {
    SimpleStateProvider::new(state)
}

// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider#simple
pub fn simple_from_block(block: &Block) -> SimpleStateProvider {
    SimpleStateProvider::new(block.default_block_state())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(i32);
    impl RandomSource for FixedRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            self.0
        }
    }

    #[test]
    fn simple_from_state_returns_provider_with_state() {
        let state = BlockState { id: 5 };
        let provider = simple_from_state(state.clone());
        let mut rng = FixedRandom(0);
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        assert_eq!(provider.get_state(&mut rng, pos), state);
        assert_eq!(
            BlockStateProvider::provider_type(&provider),
            BlockStateProviderType::SimpleStateProvider
        );
    }

    #[test]
    fn simple_from_block_uses_default_state() {
        let block = Block {
            default_state: BlockState { id: 99 },
        };
        let provider = simple_from_block(&block);
        let mut rng = FixedRandom(1);
        let pos = BlockPos { x: -7, y: 64, z: 3 };
        assert_eq!(provider.get_state(&mut rng, pos), BlockState { id: 99 });
    }

    #[test]
    fn simple_from_block_with_negative_state_id() {
        let block = Block {
            default_state: BlockState { id: i32::MIN },
        };
        let provider = simple_from_block(&block);
        let mut rng = FixedRandom(0);
        let pos = BlockPos { x: i32::MAX, y: i32::MIN, z: -1 };
        assert_eq!(provider.get_state(&mut rng, pos).id, i32::MIN);
    }

    #[test]
    fn trait_object_get_state_works() {
        let provider = simple_from_state(BlockState { id: 11 });
        let p: &dyn BlockStateProvider = &provider;
        let mut rng = FixedRandom(0);
        let pos = BlockPos { x: 1, y: 2, z: 3 };
        assert_eq!(p.get_state(&mut rng, pos), BlockState { id: 11 });
    }
}
