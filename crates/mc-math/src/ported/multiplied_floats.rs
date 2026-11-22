use crate::random::RandomSource;
use std::fmt;

/// Local self-contained copy of the SampledFloat contract.
/// JAVA: net.minecraft.util.valueproviders.SampledFloat
pub trait SampledFloat: fmt::Debug {
    // JAVA: net.minecraft.util.valueproviders.SampledFloat#sample
    fn sample(&self, random: &mut dyn RandomSource) -> f32;
}

/// JAVA: net.minecraft.util.valueproviders.MultipliedFloats
#[derive(Debug)]
pub struct MultipliedFloats {
    values: Vec<Box<dyn SampledFloat>>,
}

impl MultipliedFloats {
    // JAVA: net.minecraft.util.valueproviders.MultipliedFloats#<init>
    pub fn new(values: Vec<Box<dyn SampledFloat>>) -> Self {
        Self { values }
    }

    // JAVA: net.minecraft.util.valueproviders.MultipliedFloats#sample
    pub fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        let mut result: f32 = 1.0;
        for value in &self.values {
            result *= value.sample(random);
        }
        result
    }
}

impl SampledFloat for MultipliedFloats {
    // JAVA: net.minecraft.util.valueproviders.MultipliedFloats#sample
    fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        MultipliedFloats::sample(self, random)
    }
}

impl fmt::Display for MultipliedFloats {
    // JAVA: net.minecraft.util.valueproviders.MultipliedFloats#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MultipliedFloats{:?}", self.values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::LegacyRandomSource;

    #[derive(Debug)]
    struct Const(f32);

    impl SampledFloat for Const {
        fn sample(&self, _random: &mut dyn RandomSource) -> f32 {
            self.0
        }
    }

    #[derive(Debug)]
    struct RngFloat;

    impl SampledFloat for RngFloat {
        fn sample(&self, random: &mut dyn RandomSource) -> f32 {
            random.next_float()
        }
    }

    #[test]
    fn empty_returns_one() {
        let mut rng = LegacyRandomSource::new(0);
        let mf = MultipliedFloats::new(vec![]);
        assert_eq!(mf.sample(&mut rng).to_bits(), 1.0f32.to_bits());
    }

    #[test]
    fn multiplies_constants_in_order() {
        let mut rng = LegacyRandomSource::new(1);
        let mf = MultipliedFloats::new(vec![
            Box::new(Const(2.0)),
            Box::new(Const(3.0)),
            Box::new(Const(4.0)),
        ]);
        assert_eq!(mf.sample(&mut rng).to_bits(), 24.0f32.to_bits());
    }

    #[test]
    fn deterministic_with_same_seed() {
        let mut rng1 = LegacyRandomSource::new(5);
        let mut rng2 = LegacyRandomSource::new(5);
        let mf = MultipliedFloats::new(vec![Box::new(RngFloat), Box::new(Const(2.0))]);
        let a = mf.sample(&mut rng1);
        let b = mf.sample(&mut rng2);
        assert_eq!(a.to_bits(), b.to_bits());
    }

    #[test]
    fn handles_negative_zero_nan_and_infinity() {
        let mut rng = LegacyRandomSource::new(0);
        let neg = MultipliedFloats::new(vec![Box::new(Const(-2.0)), Box::new(Const(3.5))]);
        assert_eq!(neg.sample(&mut rng).to_bits(), (-7.0f32).to_bits());

        let mut rng2 = LegacyRandomSource::new(0);
        let nan = MultipliedFloats::new(vec![Box::new(Const(f32::NAN)), Box::new(Const(2.0))]);
        assert!(nan.sample(&mut rng2).is_nan());

        let mut rng3 = LegacyRandomSource::new(0);
        let inf = MultipliedFloats::new(vec![Box::new(Const(f32::INFINITY)), Box::new(Const(2.0))]);
        assert_eq!(inf.sample(&mut rng3), f32::INFINITY);
    }

    #[test]
    fn to_string_contains_name() {
        let mf = MultipliedFloats::new(vec![Box::new(Const(1.0))]);
        let s = mf.to_string();
        assert!(s.starts_with("MultipliedFloats"));
    }
}
