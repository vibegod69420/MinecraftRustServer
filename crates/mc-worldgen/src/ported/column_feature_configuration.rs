/// Marker trait for feature configurations.
/// JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration
pub trait FeatureConfiguration {}

/// Placeholder for net.minecraft.util.valueproviders.IntProvider.
/// Kept minimal and self-contained for this configuration port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntProvider {
    pub value: i32,
}

impl IntProvider {
    /// Convenience constructor for tests and local use.
    pub fn constant(value: i32) -> Self {
        Self { value }
    }
}

/// Configuration for column features.
/// JAVA: net.minecraft.world.level.levelgen.feature.configurations.ColumnFeatureConfiguration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnFeatureConfiguration {
    reach: IntProvider,
    height: IntProvider,
}

impl FeatureConfiguration for ColumnFeatureConfiguration {}

impl ColumnFeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.ColumnFeatureConfiguration#ColumnFeatureConfiguration
    pub fn new(reach: IntProvider, height: IntProvider) -> Self {
        Self { reach, height }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.ColumnFeatureConfiguration#reach
    pub fn reach(&self) -> IntProvider {
        self.reach
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.ColumnFeatureConfiguration#height
    pub fn height(&self) -> IntProvider {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors() {
        let reach = IntProvider::constant(2);
        let height = IntProvider::constant(5);
        let cfg = ColumnFeatureConfiguration::new(reach, height);
        assert_eq!(cfg.reach(), reach);
        assert_eq!(cfg.height(), height);
    }

    #[test]
    fn negative_values() {
        let cfg = ColumnFeatureConfiguration::new(
            IntProvider::constant(-1),
            IntProvider::constant(-10),
        );
        assert_eq!(cfg.reach().value, -1);
        assert_eq!(cfg.height().value, -10);
    }

    #[test]
    fn integer_extremes() {
        let cfg = ColumnFeatureConfiguration::new(
            IntProvider::constant(i32::MAX),
            IntProvider::constant(i32::MIN),
        );
        assert_eq!(cfg.reach().value, i32::MAX);
        assert_eq!(cfg.height().value, i32::MIN);
    }

    #[test]
    fn copy_and_eq() {
        let a = ColumnFeatureConfiguration::new(IntProvider::constant(1), IntProvider::constant(2));
        let b = a;
        assert_eq!(a, b);
    }
}
