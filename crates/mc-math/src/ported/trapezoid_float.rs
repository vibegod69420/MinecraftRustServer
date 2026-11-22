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
pub struct TrapezoidFloat {
    min: f32,
    max: f32,
    plateau: f32,
}

impl TrapezoidFloat {
    // JAVA: net.minecraft.util.valueproviders.TrapezoidFloat#of
    pub fn of(min: f32, max: f32, plateau: f32) -> Self {
        Self::new(min, max, plateau)
    }

    // JAVA: net.minecraft.util.valueproviders.TrapezoidFloat#<init>
    fn new(min: f32, max: f32, plateau: f32) -> Self {
        Self { min, max, plateau }
    }

    // JAVA: net.minecraft.util.valueproviders.TrapezoidFloat#sample
    pub fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        let span = self.max - self.min;
        let slope = (span - self.plateau) / 2.0_f32;
        let base = span - slope;
        self.min + random.next_float() * base + random.next_float() * slope
    }

    // JAVA: net.minecraft.util.valueproviders.TrapezoidFloat#getMinValue
    pub fn get_min_value(&self) -> f32 {
        self.min
    }

    // JAVA: net.minecraft.util.valueproviders.TrapezoidFloat#getMaxValue
    pub fn get_max_value(&self) -> f32 {
        self.max
    }

    // JAVA: net.minecraft.util.valueproviders.TrapezoidFloat#getType
    pub fn get_type(&self) -> FloatProviderType {
        FloatProviderType::Trapezoid
    }

    // JAVA: net.minecraft.util.valueproviders.TrapezoidFloat#toString
    pub fn java_to_string(&self) -> String {
        format!("trapezoid({}) in [{}-{}]", self.plateau, self.min, self.max)
    }
}

impl std::fmt::Display for TrapezoidFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.java_to_string())
    }
}

// JAVA: net.minecraft.util.valueproviders.TrapezoidFloat#CODEC
pub fn validate(min: f32, max: f32, plateau: f32) -> Result<(), String> {
    if max < min {
        return Err(format!("Max must be larger than min: [{}, {}]", min, max));
    }
    if plateau > max - min {
        return Err(format!(
            "Plateau can at most be the full span: [{}, {}]",
            min, max
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::LegacyRandomSource;

    #[test]
    fn validate_rejects_invalid_ranges() {
        let err = validate(5.0, 1.0, 1.0).unwrap_err();
        assert_eq!(err, "Max must be larger than min: [5, 1]");
        let err = validate(0.0, 4.0, 5.0).unwrap_err();
        assert_eq!(err, "Plateau can at most be the full span: [0, 4]");
        assert!(validate(0.0, 4.0, 4.0).is_ok());
    }

    #[test]
    fn sample_is_within_range() {
        let t = TrapezoidFloat::of(-2.0, 6.0, 2.0);
        let mut rng = LegacyRandomSource::new(123);
        for _ in 0..200 {
            let v = t.sample(&mut rng);
            assert!(v >= -2.0 && v <= 6.0, "out of range: {v}");
        }
    }

    #[test]
    fn getters_type_and_to_string() {
        let t = TrapezoidFloat::of(1.0, 5.0, 2.0);
        assert_eq!(t.get_min_value(), 1.0);
        assert_eq!(t.get_max_value(), 5.0);
        assert_eq!(t.get_type(), FloatProviderType::Trapezoid);
        assert_eq!(t.java_to_string(), "trapezoid(2) in [1-5]");
    }

    #[test]
    fn zero_plateau_matches_two_uniforms_formula() {
        let mut rng1 = LegacyRandomSource::new(77);
        let mut rng2 = LegacyRandomSource::new(77);
        let t = TrapezoidFloat::of(0.0, 4.0, 0.0);
        for _ in 0..20 {
            let v = t.sample(&mut rng1);
            let span = 4.0_f32;
            let slope = (span - 0.0_f32) / 2.0_f32;
            let base = span - slope;
            let expected = 0.0_f32 + rng2.next_float() * base + rng2.next_float() * slope;
            assert_eq!(v.to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn full_plateau_behaves_like_uniform_span() {
        let mut rng1 = LegacyRandomSource::new(5);
        let mut rng2 = LegacyRandomSource::new(5);
        let t = TrapezoidFloat::of(10.0, 20.0, 10.0);
        for _ in 0..20 {
            let v = t.sample(&mut rng1);
            let span = 10.0_f32;
            let slope = (span - 10.0_f32) / 2.0_f32; // 0
            let base = span - slope; // 10
            let expected = 10.0_f32 + rng2.next_float() * base + rng2.next_float() * slope;
            assert_eq!(v.to_bits(), expected.to_bits());
        }
    }
}
