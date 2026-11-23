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

pub struct BiasedToBottomHeight {
    min_inclusive: VerticalAnchor,
    max_inclusive: VerticalAnchor,
    inner: i32,
}

impl BiasedToBottomHeight {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.BiasedToBottomHeight#<init>
    fn new(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor, inner: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
            inner,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.BiasedToBottomHeight#of
    pub fn of(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor, inner: i32) -> Self {
        Self::new(min_inclusive, max_inclusive, inner)
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.BiasedToBottomHeight#sample
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
            let r = random.next_int(range);
            random.next_int(r.wrapping_add(self.inner)).wrapping_add(min)
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.BiasedToBottomHeight#getType
    pub fn get_type(&self) -> HeightProviderType {
        HeightProviderType::BiasedToBottom
    }
}

impl fmt::Display for BiasedToBottomHeight {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.BiasedToBottomHeight#toString
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
        let h = BiasedToBottomHeight::of(VerticalAnchor::Absolute(0), VerticalAnchor::Absolute(10), 1);
        let mut rng = SeqRandom::new(vec![3, 2]);
        // range = 10, r=3, result = nextInt(4)+0 = 2
        assert_eq!(h.sample(&mut rng, &ctx()), 2);
    }

    #[test]
    fn sample_empty_range_returns_min() {
        let h = BiasedToBottomHeight::of(VerticalAnchor::Absolute(5), VerticalAnchor::Absolute(5), 1);
        let mut rng = SeqRandom::new(vec![]);
        assert_eq!(h.sample(&mut rng, &ctx()), 5);

        let h2 = BiasedToBottomHeight::of(VerticalAnchor::Absolute(0), VerticalAnchor::Absolute(10), 20);
        let mut rng2 = SeqRandom::new(vec![]);
        assert_eq!(h2.sample(&mut rng2, &ctx()), 0);
    }

    #[test]
    fn sample_negative_values() {
        let h = BiasedToBottomHeight::of(VerticalAnchor::Absolute(-10), VerticalAnchor::Absolute(-2), 1);
        let mut rng = SeqRandom::new(vec![7, 7]);
        // range = 8, r=7, result = nextInt(8)+(-10) = -3
        assert_eq!(h.sample(&mut rng, &ctx()), -3);
    }

    #[test]
    fn sample_with_inner() {
        let h = BiasedToBottomHeight::of(VerticalAnchor::Absolute(0), VerticalAnchor::Absolute(10), 3);
        let mut rng = SeqRandom::new(vec![2, 1]);
        // range = 8, r=2, result = nextInt(5)+0 = 1
        assert_eq!(h.sample(&mut rng, &ctx()), 1);
    }

    #[test]
    fn display_and_type() {
        let h = BiasedToBottomHeight::of(VerticalAnchor::Absolute(1), VerticalAnchor::Absolute(2), 3);
        assert_eq!(h.to_string(), "biased[Absolute(1)-Absolute(2) inner: 3]");
        assert_eq!(h.get_type(), HeightProviderType::BiasedToBottom);
    }
}
