use crate::mth;
use crate::random::RandomSource;
use std::fmt;

/// Minimal local placeholder for the unported Java `IntProviderType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProviderType {
    ClampedNormal,
}

/// Faithful port of `net.minecraft.util.valueproviders.ClampedNormalInt`.
#[derive(Debug, Clone, PartialEq)]
pub struct ClampedNormalInt {
    mean: f32,
    deviation: f32,
    min_inclusive: i32,
    max_inclusive: i32,
}

impl ClampedNormalInt {
    // JAVA: net.minecraft.util.valueproviders.ClampedNormalInt#of
    pub fn of(mean: f32, deviation: f32, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self::new(mean, deviation, min_inclusive, max_inclusive)
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalInt#<init>
    fn new(mean: f32, deviation: f32, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            mean,
            deviation,
            min_inclusive,
            max_inclusive,
        }
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalInt#sample
    pub fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        Self::sample_params(
            random,
            self.mean,
            self.deviation,
            self.min_inclusive as f32,
            self.max_inclusive as f32,
        )
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalInt#sample
    pub fn sample_params(
        random: &mut dyn RandomSource,
        mean: f32,
        deviation: f32,
        min: f32,
        max: f32,
    ) -> i32 {
        let normal = mean + (random.next_gaussian() as f32) * deviation;
        mth::clamp_f32(normal, min, max) as i32
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalInt#getMinValue
    pub fn get_min_value(&self) -> i32 {
        self.min_inclusive
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalInt#getMaxValue
    pub fn get_max_value(&self) -> i32 {
        self.max_inclusive
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalInt#getType
    pub fn get_type(&self) -> IntProviderType {
        IntProviderType::ClampedNormal
    }

    pub fn mean(&self) -> f32 {
        self.mean
    }

    pub fn deviation(&self) -> f32 {
        self.deviation
    }
}

impl fmt::Display for ClampedNormalInt {
    // JAVA: net.minecraft.util.valueproviders.ClampedNormalInt#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "normal({}, {}) in [{}-{}]",
            self.mean, self.deviation, self.min_inclusive, self.max_inclusive
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{PositionalRandomFactory, RandomSource};

    struct FixedGaussianRandom {
        gaussian: f64,
    }

    impl FixedGaussianRandom {
        fn new(gaussian: f64) -> Self {
            Self { gaussian }
        }
    }

    impl RandomSource for FixedGaussianRandom {
        fn set_seed(&mut self, _seed: i64) {}

        fn next_int(&mut self) -> i32 {
            0
        }

        fn next_int_bound(&mut self, _bound: i32) -> i32 {
            0
        }

        fn next_long(&mut self) -> i64 {
            0
        }

        fn next_boolean(&mut self) -> bool {
            false
        }

        fn next_float(&mut self) -> f32 {
            0.0
        }

        fn next_double(&mut self) -> f64 {
            0.0
        }

        fn next_gaussian(&mut self) -> f64 {
            self.gaussian
        }

        fn fork(&mut self) -> Box<dyn RandomSource> {
            Box::new(FixedGaussianRandom::new(self.gaussian))
        }

        fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
            unimplemented!("fork_positional not needed in test mock")
        }
    }

    #[test]
    fn sample_clamps_to_range() {
        let provider = ClampedNormalInt::of(5.0, 2.0, 0, 10);
        let mut low = FixedGaussianRandom::new(-100.0);
        let mut high = FixedGaussianRandom::new(100.0);
        let mut mid = FixedGaussianRandom::new(0.0);

        assert_eq!(provider.sample(&mut low), 0);
        assert_eq!(provider.sample(&mut high), 10);
        assert_eq!(provider.sample(&mut mid), 5);
        assert_eq!(provider.get_min_value(), 0);
        assert_eq!(provider.get_max_value(), 10);
    }

    #[test]
    fn sample_negative_inputs_and_range() {
        let provider = ClampedNormalInt::of(-10.0, 1.5, -20, -5);
        let mut rng = FixedGaussianRandom::new(0.0);
        assert_eq!(provider.sample(&mut rng), -10);

        let mut rng_low = FixedGaussianRandom::new(-100.0);
        assert_eq!(provider.sample(&mut rng_low), -20);

        let mut rng_high = FixedGaussianRandom::new(100.0);
        assert_eq!(provider.sample(&mut rng_high), -5);
    }

    #[test]
    fn sample_static_truncates_toward_zero() {
        // mean + gaussian * deviation = 1.5 + 0.2 * 1.0 = 1.7 -> (int) 1.7 == 1
        let mut rng_pos = FixedGaussianRandom::new(0.2);
        let value_pos = ClampedNormalInt::sample_params(&mut rng_pos, 1.5, 1.0, -10.0, 10.0);
        assert_eq!(value_pos, 1);

        // -1.5 + (-0.2) * 1.0 = -1.7 -> (int) -1.7 == -1 (truncate toward zero)
        let mut rng_neg = FixedGaussianRandom::new(-0.2);
        let value_neg = ClampedNormalInt::sample_params(&mut rng_neg, -1.5, 1.0, -10.0, 10.0);
        assert_eq!(value_neg, -1);
    }

    #[test]
    fn float_bits_preserved_in_fields() {
        let mean = f32::from_bits(0x3F8C_CCCD); // ~1.1
        let deviation = f32::from_bits(0xBF00_0000); // -0.5
        let provider = ClampedNormalInt::of(mean, deviation, -3, 7);

        assert_eq!(provider.mean().to_bits(), mean.to_bits());
        assert_eq!(provider.deviation().to_bits(), deviation.to_bits());
        assert_eq!(provider.get_type(), IntProviderType::ClampedNormal);
    }

    #[test]
    fn to_string_matches_java_format() {
        let provider = ClampedNormalInt::of(2.5, 0.75, -1, 4);
        assert_eq!(provider.to_string(), "normal(2.5, 0.75) in [-1-4]");
    }

    #[test]
    fn sample_uses_float_gaussian_conversion() {
        // Mirrors Java: mean + (float)random.nextGaussian() * deviation
        // nextGaussian() returns f64, cast to f32 before multiply/add in effect.
        let mut rng = FixedGaussianRandom::new(1.0000001_f64);
        let provider = ClampedNormalInt::of(0.0, 1.0, -10, 10);
        let expected = (0.0f32 + (1.0000001_f64 as f32) * 1.0f32) as i32;
        assert_eq!(provider.sample(&mut rng), expected);
    }
}
