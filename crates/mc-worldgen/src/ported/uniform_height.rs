use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationContext {
    pub min_y: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAnchor {
    Absolute(i32),
    AboveBottom(i32),
    BelowTop(i32),
}

impl VerticalAnchor {
    // JAVA: net.minecraft.world.level.levelgen.VerticalAnchor#resolveY
    pub fn resolve_y(&self, context: &WorldGenerationContext) -> i32 {
        match *self {
            Self::Absolute(y) => y,
            Self::AboveBottom(offset) => context.min_y.wrapping_add(offset),
            Self::BelowTop(offset) => context
                .min_y
                .wrapping_add(context.height)
                .wrapping_sub(1)
                .wrapping_sub(offset),
        }
    }
}

impl fmt::Display for VerticalAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Absolute(y) => write!(f, "Absolute({y})"),
            Self::AboveBottom(o) => write!(f, "AboveBottom({o})"),
            Self::BelowTop(o) => write!(f, "BelowTop({o})"),
        }
    }
}

pub trait RandomSource {
    // JAVA: net.minecraft.util.RandomSource#nextInt
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightProviderType {
    Uniform,
    BiasedToBottom,
    VeryBiasedToBottom,
}

// JAVA: net.minecraft.util.Mth#randomBetweenInclusive
fn random_between_inclusive(random: &mut dyn RandomSource, min: i32, max: i32) -> i32 {
    random
        .next_int(max.wrapping_sub(min).wrapping_add(1))
        .wrapping_add(min)
}

pub struct UniformHeight {
    min_inclusive: VerticalAnchor,
    max_inclusive: VerticalAnchor,
    warned_for: RefCell<HashSet<i64>>,
}

impl UniformHeight {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.UniformHeight#<init>
    fn new(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
            warned_for: RefCell::new(HashSet::new()),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.UniformHeight#of
    pub fn of(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> Self {
        Self::new(min_inclusive, max_inclusive)
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.UniformHeight#sample
    pub fn sample(&self, random: &mut dyn RandomSource, context: &WorldGenerationContext) -> i32 {
        let min = self.min_inclusive.resolve_y(context);
        let max = self.max_inclusive.resolve_y(context);
        if min > max {
            let key = ((i64::from(min)) << (32_u32 & 63)) | i64::from(max);
            if self.warned_for.borrow_mut().insert(key) {
                eprintln!("Empty height range: {}", self);
            }
            min
        } else {
            random_between_inclusive(random, min, max)
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.UniformHeight#getType
    pub fn get_type(&self) -> HeightProviderType {
        HeightProviderType::Uniform
    }
}

impl fmt::Display for UniformHeight {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.UniformHeight#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}]", self.min_inclusive, self.max_inclusive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(i32);

    impl RandomSource for FixedRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            self.0
        }
    }

    fn ctx() -> WorldGenerationContext {
        WorldGenerationContext {
            min_y: -64,
            height: 384,
        }
    }

    #[test]
    fn sample_within_range() {
        let h = UniformHeight::of(VerticalAnchor::Absolute(10), VerticalAnchor::Absolute(20));
        let mut rng = FixedRandom(3);
        assert_eq!(h.sample(&mut rng, &ctx()), 13);
    }

    #[test]
    fn sample_single_value() {
        let h = UniformHeight::of(VerticalAnchor::Absolute(5), VerticalAnchor::Absolute(5));
        let mut rng = FixedRandom(0);
        assert_eq!(h.sample(&mut rng, &ctx()), 5);
    }

    #[test]
    fn sample_empty_range_returns_min_and_dedups() {
        let h = UniformHeight::of(VerticalAnchor::Absolute(20), VerticalAnchor::Absolute(10));
        let mut rng = FixedRandom(0);
        assert_eq!(h.sample(&mut rng, &ctx()), 20);
        assert_eq!(h.sample(&mut rng, &ctx()), 20);
        assert_eq!(h.warned_for.borrow().len(), 1);
    }

    #[test]
    fn sample_negative_and_overflow_arithmetic() {
        let h = UniformHeight::of(VerticalAnchor::Absolute(-10), VerticalAnchor::Absolute(-5));
        let mut rng = FixedRandom(2);
        assert_eq!(h.sample(&mut rng, &ctx()), -8);

        let h2 = UniformHeight::of(VerticalAnchor::Absolute(-1), VerticalAnchor::Absolute(-5));
        let mut rng2 = FixedRandom(0);
        assert_eq!(h2.sample(&mut rng2, &ctx()), -1);

        let h3 = UniformHeight::of(
            VerticalAnchor::Absolute(i32::MIN),
            VerticalAnchor::Absolute(i32::MAX),
        );
        let mut rng3 = FixedRandom(0);
        // bound = i32::MAX.wrapping_sub(i32::MIN).wrapping_add(1) == 0 in i32 wrapping
        assert_eq!(h3.sample(&mut rng3, &ctx()), i32::MIN);
    }

    #[test]
    fn display_and_type() {
        let h = UniformHeight::of(VerticalAnchor::Absolute(1), VerticalAnchor::Absolute(2));
        assert_eq!(h.to_string(), "[Absolute(1)-Absolute(2)]");
        assert_eq!(h.get_type(), HeightProviderType::Uniform);
    }
}
