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

// JAVA: net.minecraft.util.Mth#nextInt
fn mth_next_int(random: &mut dyn RandomSource, min_inclusive: i32, max_inclusive: i32) -> i32 {
    if min_inclusive >= max_inclusive {
        min_inclusive
    } else {
        random
            .next_int(
                max_inclusive
                    .wrapping_sub(min_inclusive)
                    .wrapping_add(1),
            )
            .wrapping_add(min_inclusive)
    }
}

pub struct VeryBiasedToBottomHeight {
    min_inclusive: VerticalAnchor,
    max_inclusive: VerticalAnchor,
    inner: i32,
}

impl VeryBiasedToBottomHeight {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.VeryBiasedToBottomHeight#<init>
    fn new(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor, inner: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
            inner,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.VeryBiasedToBottomHeight#of
    pub fn of(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor, inner: i32) -> Self {
        Self::new(min_inclusive, max_inclusive, inner)
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.VeryBiasedToBottomHeight#sample
    pub fn sample(&self, random: &mut dyn RandomSource, context: &WorldGenerationContext) -> i32 {
        let min = self.min_inclusive.resolve_y(context);
        let max = self.max_inclusive.resolve_y(context);
        let range = max
            .wrapping_sub(min)
            .wrapping_sub(self.inner)
            .wrapping_add(1);
        if range <= 0 {
            eprintln!("Empty height range: {}", self);
            min
        } else {
            let r1 = mth_next_int(random, min.wrapping_add(self.inner), max);
            let r2 = mth_next_int(random, min, r1.wrapping_sub(1));
            mth_next_int(random, min, r2.wrapping_sub(1).wrapping_add(self.inner))
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.VeryBiasedToBottomHeight#getType
    pub fn get_type(&self) -> HeightProviderType {
        HeightProviderType::VeryBiasedToBottom
    }
}

impl fmt::Display for VeryBiasedToBottomHeight {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.VeryBiasedToBottomHeight#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "biased[{}-{} inner: {}]",
            self.min_inclusive, self.max_inclusive, self.inner
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.values[self.idx];
            self.idx += 1;
            v
        }
    }

    fn ctx() -> WorldGenerationContext {
        WorldGenerationContext {
            min_y: 0,
            height: 256,
        }
    }

    #[test]
    fn sample_normal_range() {
        let h = VeryBiasedToBottomHeight::of(VerticalAnchor::Absolute(0), VerticalAnchor::Absolute(10), 1);
        let mut rng = SeqRandom::new(vec![4, 1, 0]);
        // r1=nextInt(1,10)=5, r2=nextInt(0,4)=1, r3=nextInt(0,1)=0
        assert_eq!(h.sample(&mut rng, &ctx()), 0);
    }

    #[test]
    fn sample_empty_range_returns_min() {
        let h = VeryBiasedToBottomHeight::of(VerticalAnchor::Absolute(5), VerticalAnchor::Absolute(5), 1);
        let mut rng = SeqRandom::new(vec![]);
        assert_eq!(h.sample(&mut rng, &ctx()), 5);
    }

    #[test]
    fn sample_negative_values_with_guard() {
        let h = VeryBiasedToBottomHeight::of(VerticalAnchor::Absolute(-10), VerticalAnchor::Absolute(-2), 1);
        let mut rng = SeqRandom::new(vec![0]);
        // r1=nextInt(-9,-2)=-9, r2=nextInt(-10,-10)=-10, r3=nextInt(-10,-10)=-10
        assert_eq!(h.sample(&mut rng, &ctx()), -10);
    }

    #[test]
    fn sample_with_inner_and_overflow() {
        let h = VeryBiasedToBottomHeight::of(VerticalAnchor::Absolute(0), VerticalAnchor::Absolute(20), 5);
        let mut rng = SeqRandom::new(vec![3, 2, 1]);
        // r1=nextInt(5,20)=8, r2=nextInt(0,7)=2, r3=nextInt(0,6)=1
        assert_eq!(h.sample(&mut rng, &ctx()), 1);

        let h2 = VeryBiasedToBottomHeight::of(
            VerticalAnchor::Absolute(i32::MIN),
            VerticalAnchor::Absolute(i32::MAX),
            1,
        );
        let mut rng2 = SeqRandom::new(vec![]);
        // range wraps to -1 => empty
        assert_eq!(h2.sample(&mut rng2, &ctx()), i32::MIN);
    }

    #[test]
    fn display_and_type() {
        let h = VeryBiasedToBottomHeight::of(VerticalAnchor::Absolute(1), VerticalAnchor::Absolute(2), 3);
        assert_eq!(h.to_string(), "biased[Absolute(1)-Absolute(2) inner: 3]");
        assert_eq!(h.get_type(), HeightProviderType::VeryBiasedToBottom);
    }
}
