//! Port of `net.minecraft.world.level.levelgen.feature.configurations.GeodeConfiguration`.
//!
//! This module is self-contained and provides minimal local stand-ins for
//! external Minecraft types that have not been ported yet.

/// Marker trait for feature configurations.
///
/// // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration
pub trait FeatureConfiguration {}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.GeodeBlockSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GeodeBlockSettings;

/// Minimal placeholder for `net.minecraft.world.level.levelgen.GeodeLayerSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GeodeLayerSettings;

/// Minimal placeholder for `net.minecraft.world.level.levelgen.GeodeCrackSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GeodeCrackSettings;

/// Minimal uniform integer provider.
///
/// // JAVA: net.minecraft.util.valueproviders.UniformInt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformInt {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl UniformInt {
    // JAVA: net.minecraft.util.valueproviders.UniformInt#of
    pub fn of(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }
}

/// Minimal integer provider.
///
/// // JAVA: net.minecraft.util.valueproviders.IntProvider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProvider {
    Uniform(UniformInt),
}

impl From<UniformInt> for IntProvider {
    fn from(value: UniformInt) -> Self {
        Self::Uniform(value)
    }
}

pub const CHANCE_RANGE_MIN: f64 = 0.0;
pub const CHANCE_RANGE_MAX: f64 = 1.0;
pub const DEFAULT_USE_POTENTIAL_PLACEMENTS_CHANCE: f64 = 0.35;
pub const DEFAULT_USE_ALTERNATE_LAYER0_CHANCE: f64 = 0.0;
pub const DEFAULT_PLACEMENTS_REQUIRE_LAYER0_ALTERNATE: bool = true;
pub const DEFAULT_MIN_GEN_OFFSET: i32 = -16;
pub const DEFAULT_MAX_GEN_OFFSET: i32 = 16;
pub const DEFAULT_NOISE_MULTIPLIER: f64 = 0.05;

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.GeodeConfiguration`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeodeConfiguration {
    pub geode_block_settings: GeodeBlockSettings,
    pub geode_layer_settings: GeodeLayerSettings,
    pub geode_crack_settings: GeodeCrackSettings,
    pub use_potential_placements_chance: f64,
    pub use_alternate_layer0_chance: f64,
    pub placements_require_layer0_alternate: bool,
    pub outer_wall_distance: IntProvider,
    pub distribution_points: IntProvider,
    pub point_offset: IntProvider,
    pub min_gen_offset: i32,
    pub max_gen_offset: i32,
    pub noise_multiplier: f64,
    pub invalid_blocks_threshold: i32,
}

impl FeatureConfiguration for GeodeConfiguration {}

impl GeodeConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.GeodeConfiguration#<init>
    pub fn new(
        geode_block_settings: GeodeBlockSettings,
        geode_layer_settings: GeodeLayerSettings,
        geode_crack_settings: GeodeCrackSettings,
        use_potential_placements_chance: f64,
        use_alternate_layer0_chance: f64,
        placements_require_layer0_alternate: bool,
        outer_wall_distance: IntProvider,
        distribution_points: IntProvider,
        point_offset: IntProvider,
        min_gen_offset: i32,
        max_gen_offset: i32,
        noise_multiplier: f64,
        invalid_blocks_threshold: i32,
    ) -> Self {
        Self {
            geode_block_settings,
            geode_layer_settings,
            geode_crack_settings,
            use_potential_placements_chance,
            use_alternate_layer0_chance,
            placements_require_layer0_alternate,
            outer_wall_distance,
            distribution_points,
            point_offset,
            min_gen_offset,
            max_gen_offset,
            noise_multiplier,
            invalid_blocks_threshold,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> GeodeConfiguration {
        GeodeConfiguration::new(
            GeodeBlockSettings,
            GeodeLayerSettings,
            GeodeCrackSettings,
            DEFAULT_USE_POTENTIAL_PLACEMENTS_CHANCE,
            DEFAULT_USE_ALTERNATE_LAYER0_CHANCE,
            DEFAULT_PLACEMENTS_REQUIRE_LAYER0_ALTERNATE,
            UniformInt::of(4, 5).into(),
            UniformInt::of(3, 4).into(),
            UniformInt::of(1, 2).into(),
            DEFAULT_MIN_GEN_OFFSET,
            DEFAULT_MAX_GEN_OFFSET,
            DEFAULT_NOISE_MULTIPLIER,
            1,
        )
    }

    #[test]
    fn test_new_stores_all_fields() {
        let cfg = sample_config();
        assert_eq!(cfg.geode_block_settings, GeodeBlockSettings);
        assert_eq!(cfg.geode_layer_settings, GeodeLayerSettings);
        assert_eq!(cfg.geode_crack_settings, GeodeCrackSettings);
        assert_eq!(
            cfg.use_potential_placements_chance.to_bits(),
            0.35_f64.to_bits()
        );
        assert_eq!(
            cfg.use_alternate_layer0_chance.to_bits(),
            0.0_f64.to_bits()
        );
        assert!(cfg.placements_require_layer0_alternate);
        assert_eq!(
            cfg.outer_wall_distance,
            IntProvider::Uniform(UniformInt::of(4, 5))
        );
        assert_eq!(
            cfg.distribution_points,
            IntProvider::Uniform(UniformInt::of(3, 4))
        );
        assert_eq!(
            cfg.point_offset,
            IntProvider::Uniform(UniformInt::of(1, 2))
        );
        assert_eq!(cfg.min_gen_offset, -16);
        assert_eq!(cfg.max_gen_offset, 16);
        assert_eq!(cfg.noise_multiplier.to_bits(), 0.05_f64.to_bits());
        assert_eq!(cfg.invalid_blocks_threshold, 1);
    }

    #[test]
    fn test_negative_and_extreme_int_fields() {
        let cfg = GeodeConfiguration::new(
            GeodeBlockSettings,
            GeodeLayerSettings,
            GeodeCrackSettings,
            1.0,
            1.0,
            false,
            UniformInt::of(i32::MIN, i32::MAX).into(),
            UniformInt::of(-20, -1).into(),
            UniformInt::of(0, 10).into(),
            i32::MIN,
            i32::MAX,
            0.0,
            i32::MIN,
        );

        assert_eq!(cfg.min_gen_offset, i32::MIN);
        assert_eq!(cfg.max_gen_offset, i32::MAX);
        assert_eq!(cfg.invalid_blocks_threshold, i32::MIN);
        assert_eq!(
            cfg.outer_wall_distance,
            IntProvider::Uniform(UniformInt {
                min_inclusive: i32::MIN,
                max_inclusive: i32::MAX,
            })
        );
        assert_eq!(
            cfg.distribution_points,
            IntProvider::Uniform(UniformInt::of(-20, -1))
        );
        assert!(!cfg.placements_require_layer0_alternate);
    }

    #[test]
    fn test_float_bits_preserved() {
        // Verify exact f64 bit patterns are stored without alteration.
        let chance = f64::from_bits(0x3FD6_6666_6666_6666_u64); // ~0.35
        let noise = 0.05_f64;
        let neg_zero = -0.0_f64;
        let cfg = GeodeConfiguration::new(
            GeodeBlockSettings,
            GeodeLayerSettings,
            GeodeCrackSettings,
            chance,
            neg_zero,
            true,
            UniformInt::of(1, 1).into(),
            UniformInt::of(1, 1).into(),
            UniformInt::of(1, 1).into(),
            0,
            0,
            noise,
            0,
        );

        assert_eq!(
            cfg.use_potential_placements_chance.to_bits(),
            chance.to_bits()
        );
        assert_eq!(
            cfg.use_alternate_layer0_chance.to_bits(),
            neg_zero.to_bits()
        );
        assert_ne!(
            cfg.use_alternate_layer0_chance.to_bits(),
            0.0_f64.to_bits()
        );
        assert_eq!(cfg.noise_multiplier.to_bits(), noise.to_bits());
    }

    #[test]
    fn test_uniform_int_of_and_provider_conversion() {
        let ui = UniformInt::of(4, 5);
        assert_eq!(ui.min_inclusive, 4);
        assert_eq!(ui.max_inclusive, 5);

        let provider: IntProvider = ui.into();
        assert_eq!(
            provider,
            IntProvider::Uniform(UniformInt {
                min_inclusive: 4,
                max_inclusive: 5,
            })
        );
    }

    #[test]
    fn test_default_constants_match_java_codec() {
        assert_eq!(
            DEFAULT_USE_POTENTIAL_PLACEMENTS_CHANCE.to_bits(),
            0.35_f64.to_bits()
        );
        assert_eq!(
            DEFAULT_USE_ALTERNATE_LAYER0_CHANCE.to_bits(),
            0.0_f64.to_bits()
        );
        assert!(DEFAULT_PLACEMENTS_REQUIRE_LAYER0_ALTERNATE);
        assert_eq!(DEFAULT_MIN_GEN_OFFSET, -16);
        assert_eq!(DEFAULT_MAX_GEN_OFFSET, 16);
        assert_eq!(DEFAULT_NOISE_MULTIPLIER.to_bits(), 0.05_f64.to_bits());
        assert_eq!(CHANCE_RANGE_MIN.to_bits(), 0.0_f64.to_bits());
        assert_eq!(CHANCE_RANGE_MAX.to_bits(), 1.0_f64.to_bits());
    }
}
