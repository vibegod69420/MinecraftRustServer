//! Port of `net.minecraft.world.level.levelgen.heightproviders.ConstantHeight`.

use std::fmt;

/// Minimal world generation context stub for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationContext {
    pub min_gen_y: i32,
    pub gen_depth: i32,
}

/// Minimal random source stub for self-contained compilation.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

impl<R: RandomSource + ?Sized> RandomSource for &mut R {
    fn next_int(&mut self, bound: i32) -> i32 {
        (**self).next_int(bound)
    }
}

/// Minimal vertical anchor stub for self-contained compilation.
///
/// Models an absolute anchor value, which is sufficient for `ConstantHeight`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalAnchor {
    value: i32,
}

impl VerticalAnchor {
    pub const fn absolute(y: i32) -> Self {
        Self { value: y }
    }

    pub fn resolve_y(self, _context: &WorldGenerationContext) -> i32 {
        self.value
    }
}

impl fmt::Display for VerticalAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Minimal height provider type identifier for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightProviderType {
    Constant,
    Uniform,
    BiasedToBottom,
    VeryBiasedToBottom,
    Trapezoid,
    WeightedList,
}

/// Minimal height provider contract for self-contained compilation.
pub trait HeightProvider {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.HeightProvider#sample
    fn sample(&self, random: &mut dyn RandomSource, context: &WorldGenerationContext) -> i32;

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.HeightProvider#getType
    fn get_type(&self) -> HeightProviderType;
}

/// Port of `net.minecraft.world.level.levelgen.heightproviders.ConstantHeight`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantHeight {
    value: VerticalAnchor,
}

impl ConstantHeight {
    pub const ZERO: Self = Self {
        value: VerticalAnchor::absolute(0),
    };

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.ConstantHeight#of
    pub fn of(value: VerticalAnchor) -> Self {
        Self::new(value)
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.ConstantHeight#<init>
    fn new(value: VerticalAnchor) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.ConstantHeight#getValue
    pub fn get_value(self) -> VerticalAnchor {
        self.value
    }
}

impl HeightProvider for ConstantHeight {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.ConstantHeight#sample
    fn sample(&self, _random: &mut dyn RandomSource, context: &WorldGenerationContext) -> i32 {
        self.value.resolve_y(context)
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.ConstantHeight#getType
    fn get_type(&self) -> HeightProviderType {
        HeightProviderType::Constant
    }
}

impl fmt::Display for ConstantHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRandom;

    impl RandomSource for DummyRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
    }

    fn ctx(min_gen_y: i32, gen_depth: i32) -> WorldGenerationContext {
        WorldGenerationContext {
            min_gen_y,
            gen_depth,
        }
    }

    #[test]
    fn zero_constant_resolves_to_zero() {
        let mut rng = DummyRandom;
        let z = ConstantHeight::ZERO;
        assert_eq!(z.get_value(), VerticalAnchor::absolute(0));
        assert_eq!(z.sample(&mut rng, &ctx(-64, 384)), 0);
        assert_eq!(z.get_type(), HeightProviderType::Constant);
    }

    #[test]
    fn of_and_get_value_roundtrip() {
        let anchor = VerticalAnchor::absolute(64);
        let h = ConstantHeight::of(anchor);
        assert_eq!(h.get_value(), anchor);
    }

    #[test]
    fn sample_returns_absolute_anchor_value() {
        let mut rng = DummyRandom;
        let h = ConstantHeight::of(VerticalAnchor::absolute(-16));
        assert_eq!(h.sample(&mut rng, &ctx(-64, 384)), -16);
        assert_eq!(h.sample(&mut rng, &ctx(0, 256)), -16);
    }

    #[test]
    fn sample_handles_extreme_and_negative_values() {
        let mut rng = DummyRandom;
        let c = ctx(0, 16);
        assert_eq!(
            ConstantHeight::of(VerticalAnchor::absolute(i32::MIN)).sample(&mut rng, &c),
            i32::MIN
        );
        assert_eq!(
            ConstantHeight::of(VerticalAnchor::absolute(i32::MAX)).sample(&mut rng, &c),
            i32::MAX
        );
        assert_eq!(
            ConstantHeight::of(VerticalAnchor::absolute(-1)).sample(&mut rng, &c),
            -1
        );
    }

    #[test]
    fn display_delegates_to_anchor() {
        let h = ConstantHeight::of(VerticalAnchor::absolute(42));
        assert_eq!(h.to_string(), "42");
        assert_eq!(
            ConstantHeight::of(VerticalAnchor::absolute(-7)).to_string(),
            "-7"
        );
    }
}
