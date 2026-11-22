/// Placeholder for unported `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: i32,
}

/// Placeholder for unported Mojang Codec infrastructure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CodecPlaceholder;

// JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration
pub trait FeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration#getFeatures
    fn get_features(&self) -> Vec<()> {
        Vec::new()
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockStateConfiguration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateConfiguration {
    pub state: BlockState,
}

impl BlockStateConfiguration {
    /// Java: `public static final Codec<BlockStateConfiguration> CODEC = ...`
    pub const CODEC: CodecPlaceholder = CodecPlaceholder;

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockStateConfiguration#<init>
    pub fn new(state: BlockState) -> Self {
        Self { state }
    }
}

impl FeatureConfiguration for BlockStateConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_state() {
        let state = BlockState { id: 1 };
        let cfg = BlockStateConfiguration::new(state.clone());
        assert_eq!(cfg.state, state);
    }

    #[test]
    fn negative_state_id_preserved() {
        let cfg = BlockStateConfiguration::new(BlockState { id: -1 });
        assert_eq!(cfg.state.id, -1);
    }

    #[test]
    fn extreme_state_ids_preserved() {
        let min_cfg = BlockStateConfiguration::new(BlockState { id: i32::MIN });
        let max_cfg = BlockStateConfiguration::new(BlockState { id: i32::MAX });
        assert_eq!(min_cfg.state.id, i32::MIN);
        assert_eq!(max_cfg.state.id, i32::MAX);
    }

    #[test]
    fn get_features_default_empty() {
        let cfg = BlockStateConfiguration::new(BlockState { id: 42 });
        assert!(cfg.get_features().is_empty());
    }
}
