pub trait FeatureConfiguration {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntProvider(pub i32);

#[derive(Debug, Clone, PartialEq)]
pub struct FloatProvider(pub f32);

#[derive(Debug, Clone, PartialEq)]
pub struct LargeDripstoneConfiguration {
    pub floor_to_ceiling_search_range: i32,
    pub column_radius: IntProvider,
    pub height_scale: FloatProvider,
    pub max_column_radius_to_cave_height_ratio: f32,
    pub stalactite_bluntness: FloatProvider,
    pub stalagmite_bluntness: FloatProvider,
    pub wind_speed: FloatProvider,
    pub min_radius_for_wind: i32,
    pub min_bluntness_for_wind: f32,
}

impl FeatureConfiguration for LargeDripstoneConfiguration {}

impl LargeDripstoneConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.LargeDripstoneConfiguration#<init>
    pub fn new(
        floor_to_ceiling_search_range: i32,
        column_radius: IntProvider,
        height_scale: FloatProvider,
        max_column_radius_to_cave_height_ratio: f32,
        stalactite_bluntness: FloatProvider,
        stalagmite_bluntness: FloatProvider,
        wind_speed: FloatProvider,
        min_radius_for_wind: i32,
        min_bluntness_for_wind: f32,
    ) -> Self {
        Self {
            floor_to_ceiling_search_range,
            column_radius,
            height_scale,
            max_column_radius_to_cave_height_ratio,
            stalactite_bluntness,
            stalagmite_bluntness,
            wind_speed,
            min_radius_for_wind,
            min_bluntness_for_wind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LargeDripstoneConfiguration {
        LargeDripstoneConfiguration::new(
            30,
            IntProvider(5),
            FloatProvider(1.0),
            0.5,
            FloatProvider(0.3),
            FloatProvider(0.4),
            FloatProvider(0.0),
            4,
            0.2,
        )
    }

    #[test]
    fn constructor_stores_all_fields() {
        let cfg = sample();
        assert_eq!(cfg.floor_to_ceiling_search_range, 30);
        assert_eq!(cfg.column_radius, IntProvider(5));
        assert_eq!(cfg.height_scale, FloatProvider(1.0));
        assert_eq!(
            cfg.max_column_radius_to_cave_height_ratio.to_bits(),
            0.5f32.to_bits()
        );
        assert_eq!(cfg.stalactite_bluntness, FloatProvider(0.3));
        assert_eq!(cfg.stalagmite_bluntness, FloatProvider(0.4));
        assert_eq!(cfg.wind_speed, FloatProvider(0.0));
        assert_eq!(cfg.min_radius_for_wind, 4);
        assert_eq!(cfg.min_bluntness_for_wind.to_bits(), 0.2f32.to_bits());
    }

    #[test]
    fn float_bits_preserved_for_special_values() {
        let cfg = LargeDripstoneConfiguration::new(
            1,
            IntProvider(1),
            FloatProvider(f32::NAN),
            f32::INFINITY,
            FloatProvider(f32::NEG_INFINITY),
            FloatProvider(-0.0),
            FloatProvider(f32::NAN),
            0,
            f32::NAN,
        );
        assert!(cfg.height_scale.0.is_nan());
        assert_eq!(cfg.max_column_radius_to_cave_height_ratio, f32::INFINITY);
        assert_eq!(cfg.stalactite_bluntness.0, f32::NEG_INFINITY);
        assert_eq!(
            cfg.stalagmite_bluntness.0.to_bits(),
            (-0.0f32).to_bits()
        );
        assert!(cfg.wind_speed.0.is_nan());
        assert!(cfg.min_bluntness_for_wind.is_nan());
    }

    #[test]
    fn negative_and_extreme_integer_values_preserved() {
        let cfg = LargeDripstoneConfiguration::new(
            i32::MIN,
            IntProvider(i32::MAX),
            FloatProvider(0.0),
            0.1,
            FloatProvider(0.1),
            FloatProvider(0.1),
            FloatProvider(0.0),
            -100,
            0.0,
        );
        assert_eq!(cfg.floor_to_ceiling_search_range, i32::MIN);
        assert_eq!(cfg.column_radius.0, i32::MAX);
        assert_eq!(cfg.min_radius_for_wind, -100);
    }

    #[test]
    fn boundary_values_match_codec_ranges() {
        let cfg = LargeDripstoneConfiguration::new(
            512,
            IntProvider(60),
            FloatProvider(20.0),
            1.0,
            FloatProvider(10.0),
            FloatProvider(10.0),
            FloatProvider(2.0),
            100,
            5.0,
        );
        assert_eq!(cfg.floor_to_ceiling_search_range, 512);
        assert_eq!(cfg.column_radius.0, 60);
        assert_eq!(cfg.height_scale.0.to_bits(), 20.0f32.to_bits());
        assert_eq!(cfg.max_column_radius_to_cave_height_ratio.to_bits(), 1.0f32.to_bits());
        assert_eq!(cfg.min_radius_for_wind, 100);
        assert_eq!(cfg.min_bluntness_for_wind.to_bits(), 5.0f32.to_bits());
    }
}
