use crate::mth;
use crate::random::RandomSource;

/// Mirror of Java `FloatProviderType` constants relevant to float providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatProviderType {
    Constant,
    Uniform,
    ClampedNormal,
    Trapezoid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClampedNormalFloat {
    mean: f32,
    deviation: f32,
    min: f32,
    max: f32,
}

impl ClampedNormalFloat {
    // JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#of
    pub fn of(mean: f32, deviation: f32, min: f32, max: f32) -> Self {
        Self::new(mean, deviation, min, max)
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#<init>
    fn new(mean: f32, deviation: f32, min: f32, max: f32) -> Self {
        Self {
            mean,
            deviation,
            min,
            max,
        }
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#sample
    pub fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        Self::sample_params(random, self.mean, self.deviation, self.min, self.max)
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#sample(RandomSource,float,float,float,float)
    pub fn sample_params(
        random: &mut dyn RandomSource,
        mean: f32,
        deviation: f32,
        min: f32,
        max: f32,
    ) -> f32 {
        mth::clamp_f32(mean + (random.next_gaussian() as f32) * deviation, min, max)
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#getMinValue
    pub fn get_min_value(&self) -> f32 {
        self.min
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#getMaxValue
    pub fn get_max_value(&self) -> f32 {
        self.max
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#getType
    pub fn get_type(&self) -> FloatProviderType {
        FloatProviderType::ClampedNormal
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#toString
    pub fn java_to_string(&self) -> String {
        format!(
            "normal({}, {}) in [{}-{}]",
            self.mean, self.deviation, self.min, self.max
        )
    }
}

impl std::fmt::Display for ClampedNormalFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.java_to_string())
    }
}

// JAVA: net.minecraft.util.valueproviders.ClampedNormalFloat#CODEC
pub fn validate(min: f32, max: f32) -> Result<(), String> {
    if max < min {
        Err(format!("Max must be larger than min: [{}, {}]", min, max))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::LegacyRandomSource;

    #[test]
    fn validate_rejects_invalid_range() {
        let err = validate(2.0, 1.0).unwrap_err();
        assert_eq!(err, "Max must be larger than min: [2, 1]");
        assert!(validate(1.0, 1.0).is_ok());
    }

    #[test]
    fn sample_is_clamped() {
        let mut rng = LegacyRandomSource::new(42);
        let p = ClampedNormalFloat::of(0.0, 1000.0, -1.0, 1.0);
        for _ in 0..100 {
            let v = p.sample(&mut rng);
            assert!(v >= -1.0 && v <= 1.0, "out of range: {v}");
        }
    }

    #[test]
    fn static_sample_matches_instance() {
        let mut rng1 = LegacyRandomSource::new(7);
        let mut rng2 = LegacyRandomSource::new(7);
        let p = ClampedNormalFloat::of(5.0, 2.0, 0.0, 10.0);
        for _ in 0..20 {
            let a = p.sample(&mut rng1);
            let b = ClampedNormalFloat::sample_params(&mut rng2, 5.0, 2.0, 0.0, 10.0);
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }

    #[test]
    fn getters_type_and_to_string() {
        let p = ClampedNormalFloat::of(1.0, 0.5, -2.0, 3.0);
        assert_eq!(p.get_min_value(), -2.0);
        assert_eq!(p.get_max_value(), 3.0);
        assert_eq!(p.get_type(), FloatProviderType::ClampedNormal);
        assert_eq!(p.java_to_string(), "normal(1, 0.5) in [-2-3]");
    }

    #[test]
    fn handles_negative_mean_and_deviation() {
        let mut rng = LegacyRandomSource::new(99);
        let p = ClampedNormalFloat::of(-10.0, 0.0, -20.0, 0.0);
        let v = p.sample(&mut rng);
        assert_eq!(v, -10.0);
    }
}
