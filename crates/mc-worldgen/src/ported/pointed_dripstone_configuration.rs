pub trait FeatureConfiguration {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointedDripstoneConfiguration {
    pub chance_of_taller_dripstone: f32,
    pub chance_of_directional_spread: f32,
    pub chance_of_spread_radius2: f32,
    pub chance_of_spread_radius3: f32,
}

impl FeatureConfiguration for PointedDripstoneConfiguration {}

impl PointedDripstoneConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.PointedDripstoneConfiguration#<init>
    pub fn new(
        chance_of_taller_dripstone: f32,
        chance_of_directional_spread: f32,
        chance_of_spread_radius2: f32,
        chance_of_spread_radius3: f32,
    ) -> Self {
        Self {
            chance_of_taller_dripstone,
            chance_of_directional_spread,
            chance_of_spread_radius2,
            chance_of_spread_radius3,
        }
    }
}

impl Default for PointedDripstoneConfiguration {
    fn default() -> Self {
        // Matches CODEC defaults: 0.2F, 0.7F, 0.5F, 0.5F
        Self::new(0.2_f32, 0.7_f32, 0.5_f32, 0.5_f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_fields() {
        let c = PointedDripstoneConfiguration::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.chance_of_taller_dripstone.to_bits(), 0.1_f32.to_bits());
        assert_eq!(c.chance_of_directional_spread.to_bits(), 0.2_f32.to_bits());
        assert_eq!(c.chance_of_spread_radius2.to_bits(), 0.3_f32.to_bits());
        assert_eq!(c.chance_of_spread_radius3.to_bits(), 0.4_f32.to_bits());
    }

    #[test]
    fn default_values_match_codec_defaults() {
        let c = PointedDripstoneConfiguration::default();
        assert_eq!(c.chance_of_taller_dripstone.to_bits(), 0.2_f32.to_bits());
        assert_eq!(c.chance_of_directional_spread.to_bits(), 0.7_f32.to_bits());
        assert_eq!(c.chance_of_spread_radius2.to_bits(), 0.5_f32.to_bits());
        assert_eq!(c.chance_of_spread_radius3.to_bits(), 0.5_f32.to_bits());
    }

    #[test]
    fn handles_edge_float_values() {
        let c = PointedDripstoneConfiguration::new(0.0, 1.0, f32::NAN, f32::INFINITY);
        assert_eq!(c.chance_of_taller_dripstone.to_bits(), 0.0_f32.to_bits());
        assert_eq!(c.chance_of_directional_spread.to_bits(), 1.0_f32.to_bits());
        assert!(c.chance_of_spread_radius2.is_nan());
        assert_eq!(c.chance_of_spread_radius3.to_bits(), f32::INFINITY.to_bits());
    }

    #[test]
    fn handles_negative_and_special_float_bits() {
        let neg_zero = -0.0_f32;
        let c = PointedDripstoneConfiguration::new(
            neg_zero,
            -1.0,
            f32::NEG_INFINITY,
            f32::MIN_POSITIVE,
        );
        assert_eq!(c.chance_of_taller_dripstone.to_bits(), neg_zero.to_bits());
        assert_ne!(
            c.chance_of_taller_dripstone.to_bits(),
            0.0_f32.to_bits()
        );
        assert_eq!(c.chance_of_directional_spread.to_bits(), (-1.0_f32).to_bits());
        assert_eq!(
            c.chance_of_spread_radius2.to_bits(),
            f32::NEG_INFINITY.to_bits()
        );
        assert_eq!(
            c.chance_of_spread_radius3.to_bits(),
            f32::MIN_POSITIVE.to_bits()
        );
    }
}
