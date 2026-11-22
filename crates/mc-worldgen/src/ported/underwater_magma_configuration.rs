/// Marker trait for `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration`.
pub trait FeatureConfiguration {}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.UnderwaterMagmaConfiguration`.
///
/// JAVA: `public static final Codec<UnderwaterMagmaConfiguration> CODEC = RecordCodecBuilder.create(...)`
/// - `floor_search_range`: `Codec.intRange(0, 512)`
/// - `placement_radius_around_floor`: `Codec.intRange(0, 64)`
/// - `placement_probability_per_valid_position`: `Codec.floatRange(0.0F, 1.0F)`
#[derive(Debug, Clone, PartialEq)]
pub struct UnderwaterMagmaConfiguration {
    pub floor_search_range: i32,
    pub placement_radius_around_floor: i32,
    pub placement_probability_per_valid_position: f32,
}

impl FeatureConfiguration for UnderwaterMagmaConfiguration {}

impl UnderwaterMagmaConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.UnderwaterMagmaConfiguration#<init>
    pub fn new(
        floor_search_range: i32,
        placement_radius_around_floor: i32,
        placement_probability_per_valid_position: f32,
    ) -> Self {
        Self {
            floor_search_range,
            placement_radius_around_floor,
            placement_probability_per_valid_position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_and_values() {
        let cfg = UnderwaterMagmaConfiguration::new(10, 5, 0.5_f32);
        assert_eq!(cfg.floor_search_range, 10);
        assert_eq!(cfg.placement_radius_around_floor, 5);
        assert_eq!(
            cfg.placement_probability_per_valid_position.to_bits(),
            0.5_f32.to_bits()
        );
    }

    #[test]
    fn float_bits_preserved() {
        let nan_cfg = UnderwaterMagmaConfiguration::new(0, 0, f32::NAN);
        assert!(nan_cfg.placement_probability_per_valid_position.is_nan());

        let neg_zero_cfg = UnderwaterMagmaConfiguration::new(0, 0, -0.0_f32);
        assert_eq!(
            neg_zero_cfg
                .placement_probability_per_valid_position
                .to_bits(),
            (-0.0_f32).to_bits()
        );

        let inf_cfg = UnderwaterMagmaConfiguration::new(0, 0, f32::INFINITY);
        assert_eq!(
            inf_cfg.placement_probability_per_valid_position.to_bits(),
            f32::INFINITY.to_bits()
        );
    }

    #[test]
    fn integer_edge_cases() {
        let cfg = UnderwaterMagmaConfiguration::new(i32::MAX, i32::MIN, 1.0_f32);
        assert_eq!(cfg.floor_search_range, i32::MAX);
        assert_eq!(cfg.placement_radius_around_floor, i32::MIN);

        let neg = UnderwaterMagmaConfiguration::new(-1, -64, 0.0_f32);
        assert_eq!(neg.floor_search_range, -1);
        assert_eq!(neg.placement_radius_around_floor, -64);
    }

    #[test]
    fn probability_boundary_values() {
        let zero = UnderwaterMagmaConfiguration::new(0, 0, 0.0_f32);
        assert_eq!(
            zero.placement_probability_per_valid_position.to_bits(),
            0.0_f32.to_bits()
        );

        let one = UnderwaterMagmaConfiguration::new(512, 64, 1.0_f32);
        assert_eq!(
            one.placement_probability_per_valid_position.to_bits(),
            1.0_f32.to_bits()
        );
    }
}
