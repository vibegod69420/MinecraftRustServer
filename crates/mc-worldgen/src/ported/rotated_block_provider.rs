use std::fmt::Debug;

/// Minimal block state representation for self-contained porting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: i32,
    pub axis: Option<DirectionAxis>,
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

/// Direction axis, mirroring `net.minecraft.core.Direction.Axis`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionAxis {
    X,
    Y,
    Z,
}

impl DirectionAxis {
    const VALUES: [DirectionAxis; 3] = [DirectionAxis::X, DirectionAxis::Y, DirectionAxis::Z];

    // JAVA: net.minecraft.core.Direction.Axis#getRandom
    pub fn get_random(random: &mut dyn RandomSource) -> DirectionAxis {
        let idx = random.next_int(3);
        let idx = if idx < 0 { 0 } else { idx as usize % 3 };
        Self::VALUES[idx]
    }
}

/// Property key marker for RotatedPillarBlock.AXIS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotatedPillarBlockAxis;

impl BlockState {
    /// Minimal `trySetValue` equivalent for the AXIS property.
    pub fn try_set_value_axis(mut self, _property: RotatedPillarBlockAxis, axis: DirectionAxis) -> BlockState {
        self.axis = Some(axis);
        self
    }
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

/// A provider that sets a rotated pillar block's axis randomly.
// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RotatedBlockProvider
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotatedBlockProvider {
    pub block: Block,
}

impl RotatedBlockProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RotatedBlockProvider#<init>
    pub fn new(block: Block) -> Self {
        Self { block }
    }
}

impl BlockStateProvider for RotatedBlockProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RotatedBlockProvider#type
    fn provider_type(&self) -> BlockStateProviderType {
        BlockStateProviderType::RotatedBlockProvider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RotatedBlockProvider#getState
    fn get_state(&self, random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
        let axis = DirectionAxis::get_random(random);
        self.block
            .default_block_state()
            .try_set_value_axis(RotatedPillarBlockAxis, axis)
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

    fn make_block(id: i32) -> Block {
        Block {
            default_state: BlockState { id, axis: None },
        }
    }

    #[test]
    fn provider_type_is_rotated() {
        let provider = RotatedBlockProvider::new(make_block(1));
        assert_eq!(
            provider.provider_type(),
            BlockStateProviderType::RotatedBlockProvider
        );
    }

    #[test]
    fn get_state_sets_axis_from_random() {
        let provider = RotatedBlockProvider::new(make_block(10));
        let mut rng = SeqRandom::new(vec![0, 1, 2]);
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        assert_eq!(provider.get_state(&mut rng, pos).axis, Some(DirectionAxis::X));
        assert_eq!(provider.get_state(&mut rng, pos).axis, Some(DirectionAxis::Y));
        assert_eq!(provider.get_state(&mut rng, pos).axis, Some(DirectionAxis::Z));
    }

    #[test]
    fn get_state_ignores_position() {
        let provider = RotatedBlockProvider::new(make_block(7));
        let positions = [
            BlockPos { x: -100, y: i32::MIN, z: i32::MAX },
            BlockPos { x: 0, y: 64, z: 0 },
        ];
        for pos in positions {
            let mut rng = FixedRandom(1);
            let state = provider.get_state(&mut rng, pos);
            assert_eq!(state.id, 7);
            assert_eq!(state.axis, Some(DirectionAxis::Y));
        }
    }

    #[test]
    fn negative_random_index_clamps_to_x() {
        let provider = RotatedBlockProvider::new(make_block(3));
        let mut rng = FixedRandom(-5);
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        assert_eq!(provider.get_state(&mut rng, pos).axis, Some(DirectionAxis::X));
    }

    #[test]
    fn preserves_block_id_with_overflow_edge() {
        let provider = RotatedBlockProvider::new(make_block(i32::MIN));
        let mut rng = FixedRandom(2);
        let pos = BlockPos { x: 1, y: 2, z: 3 };
        let state = provider.get_state(&mut rng, pos);
        assert_eq!(state.id, i32::MIN);
        assert_eq!(state.axis, Some(DirectionAxis::Z));
    }
}
