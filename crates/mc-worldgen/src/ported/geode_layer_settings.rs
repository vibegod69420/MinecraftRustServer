pub const DEFAULT_FILLING: f64 = 1.7;
pub const DEFAULT_INNER_LAYER: f64 = 2.2;
pub const DEFAULT_MIDDLE_LAYER: f64 = 3.2;
pub const DEFAULT_OUTER_LAYER: f64 = 4.2;
pub const LAYER_RANGE_MIN: f64 = 0.01;
pub const LAYER_RANGE_MAX: f64 = 50.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeLayerSettings {
    pub filling: f64,
    pub inner_layer: f64,
    pub middle_layer: f64,
    pub outer_layer: f64,
}

impl GeodeLayerSettings {
    // JAVA: net.minecraft.world.level.levelgen.GeodeLayerSettings#<init>
    pub fn new(filling: f64, inner_layer: f64, middle_layer: f64, outer_layer: f64) -> Self {
        Self {
            filling,
            inner_layer,
            middle_layer,
            outer_layer,
        }
    }
}

impl Default for GeodeLayerSettings {
    fn default() -> Self {
        Self::new(
            DEFAULT_FILLING,
            DEFAULT_INNER_LAYER,
            DEFAULT_MIDDLE_LAYER,
            DEFAULT_OUTER_LAYER,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_fields() {
        let s = GeodeLayerSettings::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(s.filling.to_bits(), 1.0_f64.to_bits());
        assert_eq!(s.inner_layer.to_bits(), 2.0_f64.to_bits());
        assert_eq!(s.middle_layer.to_bits(), 3.0_f64.to_bits());
        assert_eq!(s.outer_layer.to_bits(), 4.0_f64.to_bits());
    }

    #[test]
    fn default_matches_codec_defaults() {
        let s = GeodeLayerSettings::default();
        assert_eq!(s.filling.to_bits(), DEFAULT_FILLING.to_bits());
        assert_eq!(s.inner_layer.to_bits(), DEFAULT_INNER_LAYER.to_bits());
        assert_eq!(s.middle_layer.to_bits(), DEFAULT_MIDDLE_LAYER.to_bits());
        assert_eq!(s.outer_layer.to_bits(), DEFAULT_OUTER_LAYER.to_bits());
    }

    #[test]
    fn preserves_float_bits_for_edge_values() {
        let s = GeodeLayerSettings::new(LAYER_RANGE_MIN, LAYER_RANGE_MAX, -0.0, f64::INFINITY);
        assert_eq!(s.filling.to_bits(), LAYER_RANGE_MIN.to_bits());
        assert_eq!(s.inner_layer.to_bits(), LAYER_RANGE_MAX.to_bits());
        assert_eq!(s.middle_layer.to_bits(), (-0.0_f64).to_bits());
        assert_eq!(s.outer_layer.to_bits(), f64::INFINITY.to_bits());
    }

    #[test]
    fn preserves_nan_bits() {
        let nan = f64::from_bits(0x7ff8_0000_0000_0002_u64);
        let s = GeodeLayerSettings::new(nan, 2.2, 3.2, 4.2);
        assert_eq!(s.filling.to_bits(), nan.to_bits());
    }

    #[test]
    fn layer_range_constants_match_codec() {
        assert_eq!(LAYER_RANGE_MIN.to_bits(), 0.01_f64.to_bits());
        assert_eq!(LAYER_RANGE_MAX.to_bits(), 50.0_f64.to_bits());
    }
}
