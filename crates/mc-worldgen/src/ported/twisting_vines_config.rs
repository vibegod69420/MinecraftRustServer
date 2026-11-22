/// Marker trait for feature configurations.
/// JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration
pub trait FeatureConfiguration {}

/// Configuration for the twisting vines feature.
/// JAVA: net.minecraft.world.level.levelgen.feature.configurations.TwistingVinesConfig
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TwistingVinesConfig {
    spread_width: i32,
    spread_height: i32,
    max_height: i32,
}

impl FeatureConfiguration for TwistingVinesConfig {}

impl TwistingVinesConfig {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TwistingVinesConfig#TwistingVinesConfig
    pub fn new(spread_width: i32, spread_height: i32, max_height: i32) -> Self {
        Self {
            spread_width,
            spread_height,
            max_height,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TwistingVinesConfig#spreadWidth
    pub fn spread_width(&self) -> i32 {
        self.spread_width
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TwistingVinesConfig#spreadHeight
    pub fn spread_height(&self) -> i32 {
        self.spread_height
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.TwistingVinesConfig#maxHeight
    pub fn max_height(&self) -> i32 {
        self.max_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors() {
        let cfg = TwistingVinesConfig::new(8, 4, 15);
        assert_eq!(cfg.spread_width(), 8);
        assert_eq!(cfg.spread_height(), 4);
        assert_eq!(cfg.max_height(), 15);
    }

    #[test]
    fn negative_values() {
        let cfg = TwistingVinesConfig::new(-1, -2, -3);
        assert_eq!(cfg.spread_width(), -1);
        assert_eq!(cfg.spread_height(), -2);
        assert_eq!(cfg.max_height(), -3);
    }

    #[test]
    fn integer_extremes() {
        let cfg = TwistingVinesConfig::new(i32::MAX, i32::MIN, 0);
        assert_eq!(cfg.spread_width(), i32::MAX);
        assert_eq!(cfg.spread_height(), i32::MIN);
        assert_eq!(cfg.max_height(), 0);
    }

    #[test]
    fn copy_and_eq() {
        let a = TwistingVinesConfig::new(1, 2, 3);
        let b = a;
        assert_eq!(a, b);
    }
}
