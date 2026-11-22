use crate::random::RandomSource;

/// JAVA: net.minecraft.util.valueproviders.ConstantFloat
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConstantFloat {
    value: f32,
}

impl ConstantFloat {
    /// JAVA: net.minecraft.util.valueproviders.ConstantFloat#ZERO
    pub const ZERO: ConstantFloat = ConstantFloat { value: 0.0 };

    // JAVA: net.minecraft.util.valueproviders.ConstantFloat#of
    pub fn of(value: f32) -> Self {
        if value == 0.0 {
            Self::ZERO
        } else {
            Self { value }
        }
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantFloat#getValue
    pub fn get_value(&self) -> f32 {
        self.value
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantFloat#sample
    pub fn sample(&self, _random: &mut dyn RandomSource) -> f32 {
        self.value
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantFloat#getMinValue
    pub fn get_min_value(&self) -> f32 {
        self.value
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantFloat#getMaxValue
    pub fn get_max_value(&self) -> f32 {
        self.value
    }
}

impl std::fmt::Display for ConstantFloat {
    // JAVA: net.minecraft.util.valueproviders.ConstantFloat#toString
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.value;
        if v.is_nan() {
            return write!(f, "NaN");
        }
        if v.is_infinite() {
            return if v.is_sign_positive() {
                write!(f, "Infinity")
            } else {
                write!(f, "-Infinity")
            };
        }
        let s = format!("{}", v);
        if s.contains('.') || s.contains('e') || s.contains('E') {
            write!(f, "{}", s)
        } else {
            write!(f, "{}.0", s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::LegacyRandomSource;

    #[test]
    fn of_zero_returns_zero_constant() {
        let z = ConstantFloat::of(0.0);
        assert_eq!(z.get_value().to_bits(), 0.0f32.to_bits());
        assert_eq!(z, ConstantFloat::ZERO);
    }

    #[test]
    fn of_negative_zero_normalizes_to_zero() {
        // Java: value == 0.0F is true for -0.0f, so of returns ZERO.
        let z = ConstantFloat::of(-0.0);
        assert_eq!(z, ConstantFloat::ZERO);
        assert_eq!(z.get_value().to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn sample_ignores_random_source() {
        let mut rng1 = LegacyRandomSource::new(123);
        let mut rng2 = LegacyRandomSource::new(999);
        let c = ConstantFloat::of(1.5);
        assert_eq!(c.sample(&mut rng1).to_bits(), 1.5f32.to_bits());
        assert_eq!(c.sample(&mut rng2).to_bits(), 1.5f32.to_bits());
    }

    #[test]
    fn min_max_and_value_match() {
        let c = ConstantFloat::of(-3.25);
        assert_eq!(c.get_min_value().to_bits(), (-3.25f32).to_bits());
        assert_eq!(c.get_max_value().to_bits(), (-3.25f32).to_bits());
        assert_eq!(c.get_value().to_bits(), (-3.25f32).to_bits());
    }

    #[test]
    fn to_string_handles_special_values() {
        assert_eq!(ConstantFloat::of(0.0).to_string(), "0.0");
        assert_eq!(ConstantFloat::of(-1.5).to_string(), "-1.5");
        assert_eq!(ConstantFloat::of(f32::NAN).to_string(), "NaN");
        assert_eq!(ConstantFloat::of(f32::INFINITY).to_string(), "Infinity");
        assert_eq!(
            ConstantFloat::of(f32::NEG_INFINITY).to_string(),
            "-Infinity"
        );
    }
}
