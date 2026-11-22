/// Placeholder for unported `net.minecraft.util.valueproviders.ConstantInt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantInt {
    value: i32,
}

impl ConstantInt {
    // JAVA: net.minecraft.util.valueproviders.ConstantInt#of
    pub fn of(value: i32) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantInt#getValue
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

/// Placeholder for unported `net.minecraft.util.valueproviders.IntProvider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProvider {
    Constant(ConstantInt),
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

// JAVA: net.minecraft.world.level.levelgen.feature.configurations.CountConfiguration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountConfiguration {
    count: IntProvider,
}

impl CountConfiguration {
    /// Java: `public static final Codec<CountConfiguration> CODEC = ...`
    pub const CODEC: CodecPlaceholder = CodecPlaceholder;

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.CountConfiguration#<init>
    pub fn new(count: i32) -> Self {
        Self {
            count: IntProvider::Constant(ConstantInt::of(count)),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.CountConfiguration#<init>
    pub fn with_provider(count: IntProvider) -> Self {
        Self { count }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.CountConfiguration#count
    pub fn count(&self) -> IntProvider {
        self.count
    }
}

impl FeatureConfiguration for CountConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_from_int_stores_constant_provider() {
        let cfg = CountConfiguration::new(5);
        assert_eq!(cfg.count(), IntProvider::Constant(ConstantInt::of(5)));
    }

    #[test]
    fn with_provider_stores_provider() {
        let provider = IntProvider::Constant(ConstantInt::of(10));
        let cfg = CountConfiguration::with_provider(provider);
        assert_eq!(cfg.count(), provider);
    }

    #[test]
    fn negative_count_preserved() {
        let cfg = CountConfiguration::new(-1);
        match cfg.count() {
            IntProvider::Constant(c) => assert_eq!(c.get_value(), -1),
        }
    }

    #[test]
    fn extreme_int_values_preserved() {
        let min_cfg = CountConfiguration::new(i32::MIN);
        let max_cfg = CountConfiguration::new(i32::MAX);
        match min_cfg.count() {
            IntProvider::Constant(c) => assert_eq!(c.get_value(), i32::MIN),
        }
        match max_cfg.count() {
            IntProvider::Constant(c) => assert_eq!(c.get_value(), i32::MAX),
        }
    }

    #[test]
    fn wrapping_overflow_value_can_be_stored() {
        let wrapped = i32::MAX.wrapping_add(1);
        let cfg = CountConfiguration::new(wrapped);
        match cfg.count() {
            IntProvider::Constant(c) => assert_eq!(c.get_value(), i32::MIN),
        }
    }

    #[test]
    fn get_features_default_empty() {
        let cfg = CountConfiguration::new(0);
        assert!(cfg.get_features().is_empty());
    }
}
