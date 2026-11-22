use crate::random::RandomSource;

/// Mirror of Java `FloatProviderType` constants used by float providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatProviderType {
    Constant,
    Uniform,
    ClampedNormal,
    Trapezoid,
}

/// Minimal mirror of Java `SampledFloat`.
pub trait SampledFloat {
    // JAVA: net.minecraft.util.valueproviders.SampledFloat#sample
    fn sample(&self, random: &mut dyn RandomSource) -> f32;
}

/// Faithful port of the abstract base contract from Java `FloatProvider`.
///
/// Codec/registry dispatch from the Java class is not ported here because the
/// Mojang codec framework and registries are not present in `mc-math`; the
/// range-validation behavior is exposed as [`validate_range`].
pub trait FloatProvider: SampledFloat {
    // JAVA: net.minecraft.util.valueproviders.FloatProvider#getMinValue
    fn get_min_value(&self) -> f32;

    // JAVA: net.minecraft.util.valueproviders.FloatProvider#getMaxValue
    fn get_max_value(&self) -> f32;

    // JAVA: net.minecraft.util.valueproviders.FloatProvider#getType
    fn get_type(&self) -> FloatProviderType;
}

// JAVA: net.minecraft.util.valueproviders.FloatProvider#codec(float,float)
pub fn validate_range(
    provider_min: f32,
    provider_max: f32,
    min: f32,
    max: f32,
) -> Result<(), String> {
    if provider_min < min {
        return Err(format!(
            "Value provider too low: {} [{}-{}]",
            min, provider_min, provider_max
        ));
    }
    if provider_max > max {
        return Err(format!(
            "Value provider too high: {} [{}-{}]",
            max, provider_min, provider_max
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{LegacyRandomSource, RandomSource};

    #[derive(Debug)]
    struct FixedFloat(f32);

    impl SampledFloat for FixedFloat {
        fn sample(&self, _random: &mut dyn RandomSource) -> f32 {
            self.0
        }
    }

    impl FloatProvider for FixedFloat {
        fn get_min_value(&self) -> f32 {
            self.0
        }

        fn get_max_value(&self) -> f32 {
            self.0
        }

        fn get_type(&self) -> FloatProviderType {
            FloatProviderType::Constant
        }
    }

    #[test]
    fn validate_range_accepts_in_bounds() {
        assert_eq!(validate_range(1.0, 2.0, 0.0, 3.0), Ok(()));
        assert_eq!(validate_range(0.0, 3.0, 0.0, 3.0), Ok(()));
    }

    #[test]
    fn validate_range_rejects_too_low() {
        let err = validate_range(-1.0, 2.0, 0.0, 3.0).unwrap_err();
        assert_eq!(err, "Value provider too low: 0 [-1-2]");
    }

    #[test]
    fn validate_range_rejects_too_high() {
        let err = validate_range(1.0, 4.0, 0.0, 3.0).unwrap_err();
        assert_eq!(err, "Value provider too high: 3 [1-4]");
    }

    #[test]
    fn trait_impl_is_callable() {
        let mut rng = LegacyRandomSource::new(1);
        let p = FixedFloat(2.5);
        assert_eq!(p.sample(&mut rng), 2.5);
        assert_eq!(p.get_min_value(), 2.5);
        assert_eq!(p.get_max_value(), 2.5);
        assert_eq!(p.get_type(), FloatProviderType::Constant);
    }

    #[test]
    fn validate_range_handles_negative_and_float_bits() {
        assert_eq!(validate_range(-5.5, -1.25, -6.0, -1.0), Ok(()));
        let nan_min = f32::NAN;
        // NaN comparisons are false; behavior should match Java comparison semantics.
        assert_eq!(validate_range(nan_min, 1.0, 0.0, 2.0), Ok(()));
    }
}
