/// Marker trait mirroring `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration`.
pub trait FeatureConfiguration {}

/// Placeholder for `net.minecraft.util.valueproviders.IntProvider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IntProvider {
    pub min: i32,
    pub max: i32,
}

/// Placeholder for `net.minecraft.util.valueproviders.FloatProvider`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FloatProvider {
    pub min: f32,
    pub max: f32,
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.DripstoneClusterConfiguration`.
///
/// The Java `CODEC` field is not ported because no codec framework is available in this crate.
#[derive(Debug, Clone, PartialEq)]
pub struct DripstoneClusterConfiguration {
    pub floor_to_ceiling_search_range: i32,
    pub height: IntProvider,
    pub radius: IntProvider,
    pub max_stalagmite_stalactite_height_diff: i32,
    pub height_deviation: i32,
    pub dripstone_block_layer_thickness: IntProvider,
    pub density: FloatProvider,
    pub wetness: FloatProvider,
    pub chance_of_dripstone_column_at_max_distance_from_center: f32,
    pub max_distance_from_edge_affecting_chance_of_dripstone_column: i32,
    pub max_distance_from_center_affecting_height_bias: i32,
}

impl FeatureConfiguration for DripstoneClusterConfiguration {}

impl DripstoneClusterConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DripstoneClusterConfiguration#<init>
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        floor_to_ceiling_search_range: i32,
        height: IntProvider,
        radius: IntProvider,
        max_stalagmite_stalactite_height_diff: i32,
        height_deviation: i32,
        dripstone_block_layer_thickness: IntProvider,
        density: FloatProvider,
        wetness: FloatProvider,
        chance_of_dripstone_column_at_max_distance_from_center: f32,
        max_distance_from_edge_affecting_chance_of_dripstone_column: i32,
        max_distance_from_center_affecting_height_bias: i32,
    ) -> Self {
        Self {
            floor_to_ceiling_search_range,
            height,
            radius,
            max_stalagmite_stalactite_height_diff,
            height_deviation,
            dripstone_block_layer_thickness,
            density,
            wetness,
            chance_of_dripstone_column_at_max_distance_from_center,
            max_distance_from_edge_affecting_chance_of_dripstone_column,
            max_distance_from_center_affecting_height_bias,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DripstoneClusterConfiguration {
        DripstoneClusterConfiguration::new(
            12,
            IntProvider { min: 3, max: 6 },
            IntProvider { min: 2, max: 8 },
            1,
            3,
            IntProvider { min: 2, max: 4 },
            FloatProvider { min: 0.0, max: 0.7 },
            FloatProvider { min: 0.1, max: 0.6 },
            0.2_f32,
            3,
            8,
        )
    }

    #[test]
    fn constructor_stores_all_fields() {
        let cfg = sample();
        assert_eq!(cfg.floor_to_ceiling_search_range, 12);
        assert_eq!(cfg.height, IntProvider { min: 3, max: 6 });
        assert_eq!(cfg.radius, IntProvider { min: 2, max: 8 });
        assert_eq!(cfg.max_stalagmite_stalactite_height_diff, 1);
        assert_eq!(cfg.height_deviation, 3);
        assert_eq!(
            cfg.dripstone_block_layer_thickness,
            IntProvider { min: 2, max: 4 }
        );
        assert_eq!(cfg.density, FloatProvider { min: 0.0, max: 0.7 });
        assert_eq!(cfg.wetness, FloatProvider { min: 0.1, max: 0.6 });
        assert_eq!(
            cfg.chance_of_dripstone_column_at_max_distance_from_center
                .to_bits(),
            0.2_f32.to_bits()
        );
        assert_eq!(cfg.max_distance_from_edge_affecting_chance_of_dripstone_column, 3);
        assert_eq!(cfg.max_distance_from_center_affecting_height_bias, 8);
    }

    #[test]
    fn float_fields_preserve_exact_bits() {
        let nan_bits = 0x7FC0_0001_u32;
        let cfg = DripstoneClusterConfiguration::new(
            1,
            IntProvider::default(),
            IntProvider::default(),
            0,
            1,
            IntProvider::default(),
            FloatProvider {
                min: f32::from_bits(nan_bits),
                max: -0.0_f32,
            },
            FloatProvider {
                min: f32::INFINITY,
                max: f32::NEG_INFINITY,
            },
            f32::from_bits(nan_bits),
            1,
            1,
        );
        assert_eq!(cfg.density.min.to_bits(), nan_bits);
        assert_eq!(cfg.density.max.to_bits(), (-0.0_f32).to_bits());
        assert!(cfg.wetness.min.is_infinite() && cfg.wetness.min.is_sign_positive());
        assert!(cfg.wetness.max.is_infinite() && cfg.wetness.max.is_sign_negative());
        assert_eq!(
            cfg.chance_of_dripstone_column_at_max_distance_from_center
                .to_bits(),
            nan_bits
        );
    }

    #[test]
    fn integer_edge_and_negative_values() {
        let cfg = DripstoneClusterConfiguration::new(
            -1,
            IntProvider {
                min: i32::MIN,
                max: i32::MAX,
            },
            IntProvider { min: -100, max: -1 },
            i32::MIN,
            i32::MAX,
            IntProvider { min: -2, max: -2 },
            FloatProvider { min: -1.0, max: 2.0 },
            FloatProvider { min: -0.5, max: 0.5 },
            -0.25_f32,
            -64,
            0,
        );
        assert_eq!(cfg.floor_to_ceiling_search_range, -1);
        assert_eq!(cfg.height.min, i32::MIN);
        assert_eq!(cfg.height.max, i32::MAX);
        assert_eq!(cfg.radius.min, -100);
        assert_eq!(cfg.max_stalagmite_stalactite_height_diff, i32::MIN);
        assert_eq!(cfg.height_deviation, i32::MAX);
        assert_eq!(cfg.dripstone_block_layer_thickness.min, -2);
        assert_eq!(
            cfg.max_distance_from_edge_affecting_chance_of_dripstone_column,
            -64
        );
        assert_eq!(cfg.max_distance_from_center_affecting_height_bias, 0);
        assert_eq!(cfg.chance_of_dripstone_column_at_max_distance_from_center.to_bits(), (-0.25_f32).to_bits());
    }

    #[test]
    fn wrapping_overflow_values_are_stored_as_i32() {
        let overflowed = i32::MAX.wrapping_add(2); // i32::MIN + 1
        let cfg = DripstoneClusterConfiguration::new(
            overflowed,
            IntProvider {
                min: overflowed,
                max: overflowed,
            },
            IntProvider::default(),
            overflowed,
            overflowed,
            IntProvider {
                min: overflowed,
                max: overflowed,
            },
            FloatProvider::default(),
            FloatProvider::default(),
            1.0_f32,
            overflowed,
            overflowed,
        );
        let expected = i32::MIN.wrapping_add(1);
        assert_eq!(cfg.floor_to_ceiling_search_range, expected);
        assert_eq!(cfg.height.min, expected);
        assert_eq!(cfg.height.max, expected);
        assert_eq!(cfg.max_stalagmite_stalactite_height_diff, expected);
        assert_eq!(cfg.height_deviation, expected);
        assert_eq!(cfg.dripstone_block_layer_thickness.min, expected);
        assert_eq!(
            cfg.max_distance_from_edge_affecting_chance_of_dripstone_column,
            expected
        );
        assert_eq!(cfg.max_distance_from_center_affecting_height_bias, expected);
    }
}
