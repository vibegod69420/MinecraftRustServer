use std::fmt::Debug;

/// Minimal block state representation for self-contained porting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: i32,
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

/// Abstract base for block state providers.
// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider
pub trait BlockStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider#type
    fn provider_type(&self) -> BlockStateProviderType;

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider#getState
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState;
}

/// A block state provider that always returns the same state.
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
}

impl BlockStateProvider for SimpleStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider#type
    fn provider_type(&self) -> BlockStateProviderType {
        BlockStateProviderType::SimpleStateProvider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider#getState
    fn get_state(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
        self.state.clone()
    }
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

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }
    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }
    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = if self.idx < self.values.len() {
                let v = self.values[self.idx];
                self.idx += 1;
                v
            } else {
                0
            };
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }
    }

    #[test]
    fn get_state_returns_same_state() {
        let provider = SimpleStateProvider::new(BlockState { id: 42 });
        let mut rng = FixedRandom(0);
        let pos = BlockPos { x: 1, y: 2, z: 3 };
        assert_eq!(provider.get_state(&mut rng, pos), BlockState { id: 42 });
    }

    #[test]
    fn get_state_ignores_position_and_random() {
        let provider = SimpleStateProvider::new(BlockState { id: 7 });
        let mut rng = SeqRandom::new(vec![0, 1, 2, 99]);
        let positions = [
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: -100, y: 255, z: i32::MIN },
            BlockPos { x: i32::MAX, y: -1, z: 42 },
        ];
        for pos in positions {
            assert_eq!(provider.get_state(&mut rng, pos), BlockState { id: 7 });
        }
    }

    #[test]
    fn provider_type_is_simple() {
        let provider = SimpleStateProvider::new(BlockState { id: 0 });
        assert_eq!(
            provider.provider_type(),
            BlockStateProviderType::SimpleStateProvider
        );
    }

    #[test]
    fn negative_and_overflow_state_ids() {
        let provider_neg = SimpleStateProvider::new(BlockState { id: -1 });
        let provider_min = SimpleStateProvider::new(BlockState { id: i32::MIN });
        let mut rng = FixedRandom(0);
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        assert_eq!(provider_neg.get_state(&mut rng, pos).id, -1);
        assert_eq!(provider_min.get_state(&mut rng, pos).id, i32::MIN);
    }

    #[test]
    fn clone_preserves_state() {
        let provider = SimpleStateProvider::new(BlockState { id: 123 });
        let cloned = provider.clone();
        let mut rng = FixedRandom(5);
        let pos = BlockPos { x: 10, y: 20, z: 30 };
        assert_eq!(provider.get_state(&mut rng, pos), cloned.get_state(&mut rng, pos));
    }
}
