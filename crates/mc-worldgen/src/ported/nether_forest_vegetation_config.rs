/// Stub for `net.minecraft.world.level.levelgen.feature.ConfiguredFeature<?, ?>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredFeature;

/// Marker trait for feature configurations.
pub trait FeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration#getFeatures
    fn get_features(&self) -> Vec<ConfiguredFeature> {
        Vec::new()
    }
}

/// Stub for `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateProvider {
    pub id: u32,
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.BlockPileConfiguration`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPileConfiguration {
    pub state_provider: BlockStateProvider,
}

impl BlockPileConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockPileConfiguration#<init>
    pub fn new(state_provider: BlockStateProvider) -> Self {
        Self { state_provider }
    }
}

impl FeatureConfiguration for BlockPileConfiguration {}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.NetherForestVegetationConfig`.
///
/// Codec shape (Java): RecordCodecBuilder with
/// `state_provider`, `spread_width` (POSITIVE_INT), `spread_height` (POSITIVE_INT).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetherForestVegetationConfig {
    pub state_provider: BlockStateProvider,
    pub spread_width: i32,
    pub spread_height: i32,
}

impl NetherForestVegetationConfig {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.NetherForestVegetationConfig#<init>
    pub fn new(state_provider: BlockStateProvider, spread_width: i32, spread_height: i32) -> Self {
        // Mirrors Java: super(stateProvider); this.spreadWidth = spreadWidth; this.spreadHeight = spreadHeight;
        let _base = BlockPileConfiguration::new(state_provider.clone());
        Self {
            state_provider: _base.state_provider,
            spread_width,
            spread_height,
        }
    }

    /// Returns the base `BlockPileConfiguration` view of this config.
    pub fn as_block_pile_configuration(&self) -> BlockPileConfiguration {
        BlockPileConfiguration::new(self.state_provider.clone())
    }
}

impl FeatureConfiguration for NetherForestVegetationConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_all_fields() {
        let provider = BlockStateProvider { id: 42 };
        let cfg = NetherForestVegetationConfig::new(provider.clone(), 3, 4);
        assert_eq!(cfg.state_provider, provider);
        assert_eq!(cfg.spread_width, 3);
        assert_eq!(cfg.spread_height, 4);
    }

    #[test]
    fn base_configuration_view_matches_state_provider() {
        let cfg = NetherForestVegetationConfig::new(BlockStateProvider { id: 7 }, 5, 6);
        let base = cfg.as_block_pile_configuration();
        assert_eq!(base.state_provider.id, 7);
    }

    #[test]
    fn stores_negative_and_zero_spreads() {
        let cfg = NetherForestVegetationConfig::new(BlockStateProvider { id: 0 }, -1, 0);
        assert_eq!(cfg.spread_width, -1);
        assert_eq!(cfg.spread_height, 0);
    }

    #[test]
    fn stores_extreme_integer_values() {
        let cfg = NetherForestVegetationConfig::new(
            BlockStateProvider { id: u32::MAX },
            i32::MAX,
            i32::MIN,
        );
        assert_eq!(cfg.spread_width, i32::MAX);
        assert_eq!(cfg.spread_height, i32::MIN);
        assert_eq!(cfg.state_provider.id, u32::MAX);
    }

    #[test]
    fn default_feature_configuration_returns_empty_features() {
        let cfg = NetherForestVegetationConfig::new(BlockStateProvider { id: 1 }, 1, 1);
        assert!(cfg.get_features().is_empty());
    }
}
