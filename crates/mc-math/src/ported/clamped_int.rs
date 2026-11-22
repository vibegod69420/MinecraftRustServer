use crate::mth;
use crate::random::RandomSource;

/// Mirror of Java `IntProviderType` constants relevant to int providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProviderType {
    Constant,
    Uniform,
    BiasedToBottom,
    Clamped,
    WeightedList,
}

/// Minimal mirror of Java `IntProvider` required by `ClampedInt`.
pub trait IntProvider {
    // JAVA: net.minecraft.util.valueproviders.IntProvider#sample
    fn sample(&self, random: &mut dyn RandomSource) -> i32;

    // JAVA: net.minecraft.util.valueproviders.IntProvider#getMinValue
    fn get_min_value(&self) -> i32;

    // JAVA: net.minecraft.util.valueproviders.IntProvider#getMaxValue
    fn get_max_value(&self) -> i32;

    // JAVA: net.minecraft.util.valueproviders.IntProvider#getType
    fn get_type(&self) -> IntProviderType;
}

#[derive(Debug, Clone)]
pub struct ClampedInt<S> {
    source: S,
    min_inclusive: i32,
    max_inclusive: i32,
}

impl<S: IntProvider> ClampedInt<S> {
    // JAVA: net.minecraft.util.valueproviders.ClampedInt#of
    pub fn of(source: S, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self::new(source, min_inclusive, max_inclusive)
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedInt#<init>
    pub fn new(source: S, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            source,
            min_inclusive,
            max_inclusive,
        }
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedInt#sample
    pub fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        mth::clamp_i32(
            self.source.sample(random),
            self.min_inclusive,
            self.max_inclusive,
        )
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedInt#getMinValue
    pub fn get_min_value(&self) -> i32 {
        self.min_inclusive.max(self.source.get_min_value())
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedInt#getMaxValue
    pub fn get_max_value(&self) -> i32 {
        self.max_inclusive.min(self.source.get_max_value())
    }

    // JAVA: net.minecraft.util.valueproviders.ClampedInt#getType
    pub fn get_type(&self) -> IntProviderType {
        IntProviderType::Clamped
    }
}

// JAVA: net.minecraft.util.valueproviders.ClampedInt#CODEC
pub fn validate(min_inclusive: i32, max_inclusive: i32) -> Result<(), String> {
    if max_inclusive < min_inclusive {
        Err(format!(
            "Max must be at least min, min_inclusive: {}, max_inclusive: {}",
            min_inclusive, max_inclusive
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{LegacyRandomSource, RandomSource};

    #[derive(Debug, Clone, Copy)]
    struct FixedInt(i32, i32, i32);

    impl IntProvider for FixedInt {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }

        fn get_min_value(&self) -> i32 {
            self.1
        }

        fn get_max_value(&self) -> i32 {
            self.2
        }

        fn get_type(&self) -> IntProviderType {
            IntProviderType::Constant
        }
    }

    #[test]
    fn sample_clamps_below_and_above() {
        let mut rng = LegacyRandomSource::new(0);
        let low = ClampedInt::of(FixedInt(-10, -10, -10), -2, 5);
        let high = ClampedInt::of(FixedInt(10, 10, 10), -2, 5);
        assert_eq!(low.sample(&mut rng), -2);
        assert_eq!(high.sample(&mut rng), 5);
    }

    #[test]
    fn min_max_intersect_with_source() {
        let c = ClampedInt::new(FixedInt(0, -5, 20), 0, 10);
        assert_eq!(c.get_min_value(), 0);
        assert_eq!(c.get_max_value(), 10);
        assert_eq!(c.get_type(), IntProviderType::Clamped);
    }

    #[test]
    fn min_max_with_narrow_source() {
        let c = ClampedInt::new(FixedInt(0, 3, 7), 0, 10);
        assert_eq!(c.get_min_value(), 3);
        assert_eq!(c.get_max_value(), 7);
    }

    #[test]
    fn validate_rejects_invalid_range() {
        let err = validate(5, 4).unwrap_err();
        assert_eq!(
            err,
            "Max must be at least min, min_inclusive: 5, max_inclusive: 4"
        );
        assert!(validate(4, 4).is_ok());
    }

    #[test]
    fn handles_extreme_int_bounds() {
        let mut rng = LegacyRandomSource::new(1);
        let c = ClampedInt::new(FixedInt(i32::MAX, i32::MIN, i32::MAX), -1, 1);
        assert_eq!(c.sample(&mut rng), 1);
        assert_eq!(c.get_min_value(), -1);
        assert_eq!(c.get_max_value(), 1);
    }
}
