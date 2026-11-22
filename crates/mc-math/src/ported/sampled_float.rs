use crate::random::RandomSource;

/// JAVA: net.minecraft.util.valueproviders.SampledFloat
pub trait SampledFloat {
    // JAVA: net.minecraft.util.valueproviders.SampledFloat#sample
    fn sample(&self, random: &mut dyn RandomSource) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::LegacyRandomSource;

    struct ConstSampler(f32);

    impl SampledFloat for ConstSampler {
        fn sample(&self, _random: &mut dyn RandomSource) -> f32 {
            self.0
        }
    }

    struct RngSampler;

    impl SampledFloat for RngSampler {
        fn sample(&self, random: &mut dyn RandomSource) -> f32 {
            random.next_float()
        }
    }

    #[test]
    fn constant_sampler_returns_value() {
        let mut rng = LegacyRandomSource::new(0);
        let s = ConstSampler(1.5);
        assert_eq!(s.sample(&mut rng).to_bits(), 1.5f32.to_bits());
    }

    #[test]
    fn rng_sampler_is_deterministic_for_same_seed() {
        let mut rng1 = LegacyRandomSource::new(999);
        let mut rng2 = LegacyRandomSource::new(999);
        let s = RngSampler;
        let a = s.sample(&mut rng1);
        let b = s.sample(&mut rng2);
        assert_eq!(a.to_bits(), b.to_bits());
    }

    #[test]
    fn trait_object_works() {
        let mut rng = LegacyRandomSource::new(42);
        let s: Box<dyn SampledFloat> = Box::new(ConstSampler(-0.0));
        assert_eq!(s.sample(&mut rng).to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn nan_and_infinity_preserved() {
        let mut rng = LegacyRandomSource::new(1);
        let nan = ConstSampler(f32::NAN);
        let inf = ConstSampler(f32::INFINITY);
        let neg_inf = ConstSampler(f32::NEG_INFINITY);
        assert!(nan.sample(&mut rng).is_nan());
        assert_eq!(inf.sample(&mut rng), f32::INFINITY);
        assert_eq!(neg_inf.sample(&mut rng), f32::NEG_INFINITY);
    }
}
