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

/// Stub for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    value: T,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.core.Holder#value
    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Stub for `net.minecraft.world.level.levelgen.placement.PlacedFeature`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedFeature {
    pub id: u32,
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.RandomPatchConfiguration`.
///
/// Codec defaults (Java): tries=128, xz_spread=7, y_spread=3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomPatchConfiguration {
    pub tries: i32,
    pub xz_spread: i32,
    pub y_spread: i32,
    pub feature: Holder<PlacedFeature>,
}

impl RandomPatchConfiguration {
    pub const DEFAULT_TRIES: i32 = 128;
    pub const DEFAULT_XZ_SPREAD: i32 = 7;
    pub const DEFAULT_Y_SPREAD: i32 = 3;

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomPatchConfiguration#<init>
    pub fn new(tries: i32, xz_spread: i32, y_spread: i32, feature: Holder<PlacedFeature>) -> Self {
        Self {
            tries,
            xz_spread,
            y_spread,
            feature,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomPatchConfiguration#tries
    pub fn tries(&self) -> i32 {
        self.tries
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomPatchConfiguration#xzSpread
    pub fn xz_spread(&self) -> i32 {
        self.xz_spread
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomPatchConfiguration#ySpread
    pub fn y_spread(&self) -> i32 {
        self.y_spread
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RandomPatchConfiguration#feature
    pub fn feature(&self) -> &Holder<PlacedFeature> {
        &self.feature
    }
}

impl FeatureConfiguration for RandomPatchConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RandomPatchConfiguration {
        RandomPatchConfiguration::new(64, 5, 2, Holder::new(PlacedFeature { id: 1 }))
    }

    #[test]
    fn constructor_and_accessors() {
        let cfg = sample();
        assert_eq!(cfg.tries(), 64);
        assert_eq!(cfg.xz_spread(), 5);
        assert_eq!(cfg.y_spread(), 2);
        assert_eq!(cfg.feature().value().id, 1);
    }

    #[test]
    fn codec_defaults_match_java() {
        assert_eq!(RandomPatchConfiguration::DEFAULT_TRIES, 128);
        assert_eq!(RandomPatchConfiguration::DEFAULT_XZ_SPREAD, 7);
        assert_eq!(RandomPatchConfiguration::DEFAULT_Y_SPREAD, 3);
    }

    #[test]
    fn default_feature_configuration_returns_empty_features() {
        let cfg = sample();
        assert!(cfg.get_features().is_empty());
    }

    #[test]
    fn stores_negative_and_extreme_integer_values() {
        let cfg = RandomPatchConfiguration::new(
            -1,
            i32::MIN,
            i32::MAX,
            Holder::new(PlacedFeature { id: u32::MAX }),
        );
        assert_eq!(cfg.tries(), -1);
        assert_eq!(cfg.xz_spread(), i32::MIN);
        assert_eq!(cfg.y_spread(), i32::MAX);
        assert_eq!(cfg.feature().value().id, u32::MAX);
    }

    #[test]
    fn default_values_can_construct_configuration() {
        let cfg = RandomPatchConfiguration::new(
            RandomPatchConfiguration::DEFAULT_TRIES,
            RandomPatchConfiguration::DEFAULT_XZ_SPREAD,
            RandomPatchConfiguration::DEFAULT_Y_SPREAD,
            Holder::new(PlacedFeature { id: 0 }),
        );
        assert_eq!(cfg.tries(), 128);
        assert_eq!(cfg.xz_spread(), 7);
        assert_eq!(cfg.y_spread(), 3);
    }
}
