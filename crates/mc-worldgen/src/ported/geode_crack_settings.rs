pub const DEFAULT_GENERATE_CRACK_CHANCE: f64 = 1.0;
pub const DEFAULT_BASE_CRACK_SIZE: f64 = 2.0;
pub const DEFAULT_CRACK_POINT_OFFSET: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeCrackSettings {
    pub generate_crack_chance: f64,
    pub base_crack_size: f64,
    pub crack_point_offset: i32,
}

impl GeodeCrackSettings {
    // JAVA: net.minecraft.world.level.levelgen.GeodeCrackSettings#<init>
    pub fn new(generate_crack_chance: f64, base_crack_size: f64, crack_point_offset: i32) -> Self {
        Self {
            generate_crack_chance,
            base_crack_size,
            crack_point_offset,
        }
    }
}

impl Default for GeodeCrackSettings {
    fn default() -> Self {
        Self::new(
            DEFAULT_GENERATE_CRACK_CHANCE,
            DEFAULT_BASE_CRACK_SIZE,
            DEFAULT_CRACK_POINT_OFFSET,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_fields() {
        let s = GeodeCrackSettings::new(0.5, 3.25, 4);
        assert_eq!(s.generate_crack_chance.to_bits(), 0.5_f64.to_bits());
        assert_eq!(s.base_crack_size.to_bits(), 3.25_f64.to_bits());
        assert_eq!(s.crack_point_offset, 4);
    }

    #[test]
    fn default_matches_codec_defaults() {
        let s = GeodeCrackSettings::default();
        assert_eq!(
            s.generate_crack_chance.to_bits(),
            DEFAULT_GENERATE_CRACK_CHANCE.to_bits()
        );
        assert_eq!(s.base_crack_size.to_bits(), DEFAULT_BASE_CRACK_SIZE.to_bits());
        assert_eq!(s.crack_point_offset, DEFAULT_CRACK_POINT_OFFSET);
    }

    #[test]
    fn preserves_float_bits_for_special_values() {
        let neg_zero = -0.0_f64;
        let s = GeodeCrackSettings::new(neg_zero, f64::INFINITY, -1);
        assert_eq!(s.generate_crack_chance.to_bits(), neg_zero.to_bits());
        assert_eq!(s.base_crack_size.to_bits(), f64::INFINITY.to_bits());
        assert_eq!(s.crack_point_offset, -1);
    }

    #[test]
    fn preserves_nan_bits() {
        let nan = f64::from_bits(0x7ff8_0000_0000_0001_u64);
        let s = GeodeCrackSettings::new(nan, 1.0, 0);
        assert_eq!(s.generate_crack_chance.to_bits(), nan.to_bits());
    }

    #[test]
    fn handles_extreme_integer_offset() {
        let s = GeodeCrackSettings::new(1.0, 2.0, i32::MIN);
        assert_eq!(s.crack_point_offset, i32::MIN);
        let s2 = GeodeCrackSettings::new(1.0, 2.0, i32::MAX);
        assert_eq!(s2.crack_point_offset, i32::MAX);
    }
}
