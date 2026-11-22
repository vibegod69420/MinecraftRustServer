/// Placeholder for unported `net.minecraft.world.level.levelgen.feature.ConfiguredFeature`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredFeaturePlaceholder;

/// Placeholder for unported Mojang Codec infrastructure.
/// The real Java field uses Mojang Codec; this preserves the field presence
/// in a self-contained, compilation-safe form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CodecPlaceholder;

// JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration
pub trait FeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration#getFeatures
    fn get_features(&self) -> Vec<ConfiguredFeaturePlaceholder> {
        Vec::new()
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.configurations.NoneFeatureConfiguration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoneFeatureConfiguration;

impl NoneFeatureConfiguration {
    /// Java: `public static final NoneFeatureConfiguration INSTANCE = new NoneFeatureConfiguration();`
    pub const INSTANCE: Self = Self;

    /// Java: `public static final Codec<NoneFeatureConfiguration> CODEC = MapCodec.unitCodec(INSTANCE);`
    pub const CODEC: CodecPlaceholder = CodecPlaceholder;

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.NoneFeatureConfiguration#<init>
    pub fn new() -> Self {
        Self
    }
}

impl FeatureConfiguration for NoneFeatureConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_new_and_default_are_equal() {
        assert_eq!(NoneFeatureConfiguration::INSTANCE, NoneFeatureConfiguration::new());
        assert_eq!(NoneFeatureConfiguration::INSTANCE, NoneFeatureConfiguration::default());
    }

    #[test]
    fn get_features_is_empty() {
        let cfg = NoneFeatureConfiguration::INSTANCE;
        assert!(cfg.get_features().is_empty());
    }

    #[test]
    fn copy_semantics_preserve_value() {
        let a = NoneFeatureConfiguration::INSTANCE;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn codec_placeholder_is_present() {
        let codec = NoneFeatureConfiguration::CODEC;
        assert_eq!(codec, CodecPlaceholder);
    }
}
