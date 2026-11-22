use crate::random::RandomSource;

/// JAVA: net.minecraft.util.valueproviders.ConstantInt
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstantInt {
    value: i32,
}

impl ConstantInt {
    /// JAVA: net.minecraft.util.valueproviders.ConstantInt#ZERO
    pub const ZERO: ConstantInt = ConstantInt { value: 0 };

    // JAVA: net.minecraft.util.valueproviders.ConstantInt#of
    pub fn of(value: i32) -> Self {
        if value == 0 {
            Self::ZERO
        } else {
            Self { value }
        }
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantInt#getValue
    pub fn get_value(&self) -> i32 {
        self.value
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantInt#sample
    pub fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
        self.value
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantInt#getMinValue
    pub fn get_min_value(&self) -> i32 {
        self.value
    }

    // JAVA: net.minecraft.util.valueproviders.ConstantInt#getMaxValue
    pub fn get_max_value(&self) -> i32 {
        self.value
    }
}

impl std::fmt::Display for ConstantInt {
    // JAVA: net.minecraft.util.valueproviders.ConstantInt#toString
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::LegacyRandomSource;

    #[test]
    fn of_zero_returns_zero_constant() {
        let z = ConstantInt::of(0);
        assert_eq!(z.get_value(), 0);
        assert_eq!(z, ConstantInt::ZERO);
    }

    #[test]
    fn of_nonzero_preserves_value() {
        let c = ConstantInt::of(42);
        assert_eq!(c.get_value(), 42);
        assert_eq!(c.get_min_value(), 42);
        assert_eq!(c.get_max_value(), 42);
    }

    #[test]
    fn sample_ignores_random_source() {
        let mut rng1 = LegacyRandomSource::new(123);
        let mut rng2 = LegacyRandomSource::new(999);
        let c = ConstantInt::of(-7);
        assert_eq!(c.sample(&mut rng1), -7);
        assert_eq!(c.sample(&mut rng2), -7);
    }

    #[test]
    fn handles_int_extremes() {
        let min = ConstantInt::of(i32::MIN);
        let max = ConstantInt::of(i32::MAX);
        let mut rng = LegacyRandomSource::new(0);
        assert_eq!(min.sample(&mut rng), i32::MIN);
        assert_eq!(max.sample(&mut rng), i32::MAX);
        assert_eq!(min.to_string(), i32::MIN.to_string());
        assert_eq!(max.to_string(), i32::MAX.to_string());
    }

    #[test]
    fn to_string_matches_java_integer_to_string() {
        assert_eq!(ConstantInt::of(0).to_string(), "0");
        assert_eq!(ConstantInt::of(-123).to_string(), "-123");
        assert_eq!(ConstantInt::of(456).to_string(), "456");
    }
}
