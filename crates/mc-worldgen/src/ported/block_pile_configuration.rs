/// Placeholder for unported `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockStateProvider {
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

// JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockPileConfiguration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPileConfiguration {
    pub state_provider: BlockStateProvider,
}

impl BlockPileConfiguration {
    /// Java: `public static final Codec<BlockPileConfiguration> CODEC = ...`
    pub const CODEC: CodecPlaceholder = CodecPlaceholder;

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockPileConfiguration#<init>
    pub fn new(state_provider: BlockStateProvider) -> Self {
        Self { state_provider }
    }
}

impl FeatureConfiguration for BlockPileConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_state_provider() {
        let provider = BlockStateProvider { id: 7 };
        let cfg = BlockPileConfiguration::new(provider.clone());
        assert_eq!(cfg.state_provider, provider);
    }

    #[test]
    fn negative_provider_id_preserved() {
        let cfg = BlockPileConfiguration::new(BlockStateProvider { id: -123 });
        assert_eq!(cfg.state_provider.id, -123);
    }

    #[test]
    fn extreme_provider_ids_preserved() {
        let min_cfg = BlockPileConfiguration::new(BlockStateProvider { id: i32::MIN });
        let max_cfg = BlockPileConfiguration::new(BlockStateProvider { id: i32::MAX });
        assert_eq!(min_cfg.state_provider.id, i32::MIN);
        assert_eq!(max_cfg.state_provider.id, i32::MAX);
    }

    #[test]
    fn get_features_default_empty() {
        let cfg = BlockPileConfiguration::new(BlockStateProvider { id: 0 });
        assert!(cfg.get_features().is_empty());
    }
}
