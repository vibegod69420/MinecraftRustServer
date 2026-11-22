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

// JAVA: net.minecraft.world.level.levelgen.feature.configurations.ProbabilityFeatureConfiguration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbabilityFeatureConfiguration {
    pub probability: f32,
}

impl ProbabilityFeatureConfiguration {
    /// Java: `public static final Codec<ProbabilityFeatureConfiguration> CODEC = ...`
    pub const CODEC: CodecPlaceholder = CodecPlaceholder;

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.ProbabilityFeatureConfiguration#<init>
    pub fn new(probability: f32) -> Self {
        Self { probability }
    }
}

impl FeatureConfiguration for ProbabilityFeatureConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_probability_bits() {
        let cfg = ProbabilityFeatureConfiguration::new(0.5_f32);
        assert_eq!(cfg.probability.to_bits(), 0.5_f32.to_bits());
    }

    #[test]
    fn boundary_values_preserved() {
        let zero = ProbabilityFeatureConfiguration::new(0.0_f32);
        let one = ProbabilityFeatureConfiguration::new(1.0_f32);
        assert_eq!(zero.probability.to_bits(), 0.0_f32.to_bits());
        assert_eq!(one.probability.to_bits(), 1.0_f32.to_bits());
    }

    #[test]
    fn nan_bits_preserved() {
        let nan = f32::from_bits(0x7FC0_0001);
        let cfg = ProbabilityFeatureConfiguration::new(nan);
        assert_eq!(cfg.probability.to_bits(), nan.to_bits());
        assert!(cfg.probability.is_nan());
    }

    #[test]
    fn out_of_range_values_preserved_as_stored() {
        // Java constructor stores the float as-is; codec range validation is separate.
        let neg = ProbabilityFeatureConfiguration::new(-1.0_f32);
        let big = ProbabilityFeatureConfiguration::new(2.0_f32);
        assert_eq!(neg.probability.to_bits(), (-1.0_f32).to_bits());
        assert_eq!(big.probability.to_bits(), 2.0_f32.to_bits());
    }

    #[test]
    fn copy_preserves_exact_bits() {
        let cfg = ProbabilityFeatureConfiguration::new(f32::from_bits(0x3F00_0001));
        let copied = cfg;
        assert_eq!(cfg.probability.to_bits(), copied.probability.to_bits());
    }
}
