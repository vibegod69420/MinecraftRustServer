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
pub struct UniformFloat {
    min_inclusive: f32,
    max_exclusive: f32,
}

impl UniformFloat {
    // JAVA: net.minecraft.util.valueproviders.UniformFloat#<init>
    fn new(min_inclusive: f32, max_exclusive: f32) -> Self {
        Self {
            min_inclusive,
            max_exclusive,
        }
    }

    // JAVA: net.minecraft.util.valueproviders.UniformFloat#of
    pub fn of(min_inclusive: f32, max_exclusive: f32) -> Result<Self, String> {
        if max_exclusive <= min_inclusive {
            return Err("Max must exceed min".to_string());
        }
        Ok(Self::new(min_inclusive, max_exclusive))
    }

    // JAVA: net.minecraft.util.valueproviders.UniformFloat#sample
    pub fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        self.min_inclusive + random.next_float() * (self.max_exclusive - self.min_inclusive)
    }

    // JAVA: net.minecraft.util.valueproviders.UniformFloat#getMinValue
    pub fn get_min_value(&self) -> f32 {
        self.min_inclusive
    }

    // JAVA: net.minecraft.util.valueproviders.UniformFloat#getMaxValue
    pub fn get_max_value(&self) -> f32 {
        self.max_exclusive
    }

    // JAVA: net.minecraft.util.valueproviders.UniformFloat#getType
    pub fn get_type(&self) -> FloatProviderType {
        FloatProviderType::Uniform
    }

    // JAVA: net.minecraft.util.valueproviders.UniformFloat#toString
    pub fn java_to_string(&self) -> String {
        format!("[{}-{}]", self.min_inclusive, self.max_exclusive)
    }
}

impl std::fmt::Display for UniformFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.java_to_string())
    }
}

// JAVA: net.minecraft.util.valueproviders.UniformFloat#CODEC
pub fn validate(min_inclusive: f32, max_exclusive: f32) -> Result<(), String> {
    if max_exclusive <= min_inclusive {
        Err(format!(
            "Max must be larger than min, min_inclusive: {}, max_exclusive: {}",
            min_inclusive, max_exclusive
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::LegacyRandomSource;

    #[test]
    fn of_rejects_invalid_range() {
        assert_eq!(
            UniformFloat::of(2.0, 1.0).unwrap_err(),
            "Max must exceed min"
        );
        assert_eq!(
            UniformFloat::of(2.0, 2.0).unwrap_err(),
            "Max must exceed min"
        );
    }

    #[test]
    fn codec_validate_has_java_message() {
        let err = validate(1.5, 1.0).unwrap_err();
        assert_eq!(
            err,
            "Max must be larger than min, min_inclusive: 1.5, max_exclusive: 1"
        );
        assert!(validate(1.0, 1.5).is_ok());
    }

    #[test]
    fn sample_is_within_range() {
        let u = UniformFloat::of(-3.0, 7.5).unwrap();
        let mut rng = LegacyRandomSource::new(12345);
        for _ in 0..100 {
            let v = u.sample(&mut rng);
            assert!(v >= -3.0 && v < 7.5, "out of range: {v}");
        }
    }

    #[test]
    fn getters_and_type() {
        let u = UniformFloat::of(0.25, 0.75).unwrap();
        assert_eq!(u.get_min_value(), 0.25);
        assert_eq!(u.get_max_value(), 0.75);
        assert_eq!(u.get_type(), FloatProviderType::Uniform);
    }

    #[test]
    fn to_string_matches_java() {
        let u = UniformFloat::of(-1.0, 2.5).unwrap();
        assert_eq!(u.java_to_string(), "[-1-2.5]");
        assert_eq!(u.to_string(), "[-1-2.5]");
    }
}
