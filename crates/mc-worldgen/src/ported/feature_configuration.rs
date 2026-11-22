/// Placeholder for unported `net.minecraft.world.level.levelgen.feature.ConfiguredFeature`.
/// Kept minimal so this module remains self-contained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredFeaturePlaceholder {
    pub id: u32,
}

// JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration
pub trait FeatureConfiguration {
    // Java: `NoneFeatureConfiguration NONE = NoneFeatureConfiguration.INSTANCE;`
    // This constant is omitted here because `NoneFeatureConfiguration` lives in a
    // separate self-contained module.

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration#getFeatures
    fn get_features(&self) -> Vec<ConfiguredFeaturePlaceholder> {
        Vec::new()
    }
}

/// Simple concrete implementation used for testing the default trait behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DefaultFeatureConfiguration;

impl FeatureConfiguration for DefaultFeatureConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_get_features_is_empty() {
        let cfg = DefaultFeatureConfiguration;
        assert!(cfg.get_features().is_empty());
    }

    #[test]
    fn default_get_features_is_empty_on_repeated_calls() {
        let cfg = DefaultFeatureConfiguration;
        assert_eq!(cfg.get_features().len(), 0);
        assert_eq!(cfg.get_features(), Vec::new());
    }

    #[test]
    fn placeholder_preserves_edge_id_values() {
        let min = ConfiguredFeaturePlaceholder { id: 0 };
        let max = ConfiguredFeaturePlaceholder { id: u32::MAX };
        assert_eq!(min.id, 0);
        assert_eq!(max.id, u32::MAX);
        assert_ne!(min, max);
    }

    #[test]
    fn placeholder_clone_and_equality() {
        let a = ConfiguredFeaturePlaceholder { id: 42 };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
