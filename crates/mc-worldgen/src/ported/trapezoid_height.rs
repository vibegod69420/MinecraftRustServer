use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorldGenerationContext {
    pub min_y: i32,
    pub height: i32,
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait VerticalAnchor: fmt::Display + fmt::Debug {
    fn resolve_y(&self, context: &WorldGenerationContext) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightProviderType {
    Trapezoid,
}

// JAVA: net.minecraft.util.Mth#randomBetweenInclusive
pub fn mth_random_between_inclusive<R: RandomSource>(
    random: &mut R,
    min_inclusive: i32,
    max_inclusive: i32,
) -> i32 {
    let bound = max_inclusive
        .wrapping_sub(min_inclusive)
        .wrapping_add(1);
    random.next_int(bound).wrapping_add(min_inclusive)
}

#[derive(Debug, Clone)]
pub struct TrapezoidHeight<Min, Max> {
    min_inclusive: Min,
    max_inclusive: Max,
    plateau: i32,
}

impl<Min: VerticalAnchor, Max: VerticalAnchor> TrapezoidHeight<Min, Max> {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeight#<init>
    fn new(min_inclusive: Min, max_inclusive: Max, plateau: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
            plateau,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeight#of
    pub fn of(min_inclusive: Min, max_inclusive: Max, plateau: i32) -> Self {
        Self::new(min_inclusive, max_inclusive, plateau)
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeight#of
    pub fn of_triangle(min_inclusive: Min, max_inclusive: Max) -> Self {
        Self::of(min_inclusive, max_inclusive, 0)
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeight#sample
    pub fn sample<R: RandomSource>(
        &self,
        random: &mut R,
        context: &WorldGenerationContext,
    ) -> i32 {
        let min = self.min_inclusive.resolve_y(context);
        let max = self.max_inclusive.resolve_y(context);
        if min > max {
            eprintln!("Empty height range: {}", self);
            return min;
        }

        let range = max.wrapping_sub(min);
        if self.plateau >= range {
            return mth_random_between_inclusive(random, min, max);
        }

        let mid = range.wrapping_sub(self.plateau) / 2;
        let top = range.wrapping_sub(mid);
        min.wrapping_add(mth_random_between_inclusive(random, 0, top))
            .wrapping_add(mth_random_between_inclusive(random, 0, mid))
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeight#getType
    pub fn get_type(&self) -> HeightProviderType {
        HeightProviderType::Trapezoid
    }
}

impl<Min: VerticalAnchor, Max: VerticalAnchor> fmt::Display for TrapezoidHeight<Min, Max> {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeight#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.plateau == 0 {
            write!(f, "triangle ({}-{})", self.min_inclusive, self.max_inclusive)
        } else {
            write!(
                f,
                "trapezoid({}) in [{}-{}]",
                self.plateau, self.min_inclusive, self.max_inclusive
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ConstAnchor(i32);

    impl fmt::Display for ConstAnchor {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl VerticalAnchor for ConstAnchor {
        fn resolve_y(&self, _context: &WorldGenerationContext) -> i32 {
            self.0
        }
    }

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
        fn next_int(&mut self, bound: i32) -> i32 {
            assert!(bound > 0, "bound must be positive");
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    #[test]
    fn empty_range_returns_min_and_formats() {
        let h = TrapezoidHeight::of(ConstAnchor(10), ConstAnchor(5), 0);
        let mut rng = SeqRandom::new(vec![0]);
        let ctx = WorldGenerationContext::default();
        assert_eq!(h.sample(&mut rng, &ctx), 10);
        assert_eq!(h.to_string(), "triangle (10-5)");
    }

    #[test]
    fn plateau_covers_range_uses_uniform() {
        let h = TrapezoidHeight::of(ConstAnchor(5), ConstAnchor(10), 5);
        let mut rng = SeqRandom::new(vec![0, 5, 3]);
        let ctx = WorldGenerationContext::default();
        assert_eq!(h.sample(&mut rng, &ctx), 5);
        assert_eq!(h.sample(&mut rng, &ctx), 10);
        assert_eq!(h.sample(&mut rng, &ctx), 8);
        assert_eq!(h.get_type(), HeightProviderType::Trapezoid);
    }

    #[test]
    fn trapezoid_sample_and_display() {
        let h = TrapezoidHeight::of(ConstAnchor(0), ConstAnchor(10), 2);
        let mut rng = SeqRandom::new(vec![3, 2]);
        let ctx = WorldGenerationContext::default();
        // range=10, mid=(10-2)/2=4, top=10-4=6
        // result = 0 + next_int(7)->3 + next_int(5)->2 = 5
        assert_eq!(h.sample(&mut rng, &ctx), 5);
        assert_eq!(h.to_string(), "trapezoid(2) in [0-10]");
    }

    #[test]
    fn negative_inputs_and_triangle_factory() {
        let h = TrapezoidHeight::of_triangle(ConstAnchor(-10), ConstAnchor(-2));
        let mut rng = SeqRandom::new(vec![1, 0]);
        let ctx = WorldGenerationContext::default();
        // range=8, plateau=0, mid=4, top=4
        // result = -10 + next_int(5)->1 + next_int(5)->0 = -9
        assert_eq!(h.sample(&mut rng, &ctx), -9);
        assert_eq!(h.to_string(), "triangle (-10--2)");
    }

    #[test]
    fn wrapping_addition_near_i32_max() {
        let h = TrapezoidHeight::of(ConstAnchor(i32::MAX - 2), ConstAnchor(i32::MAX), 0);
        let mut rng = SeqRandom::new(vec![1, 1]);
        let ctx = WorldGenerationContext::default();
        // range=2, mid=1, top=1
        // result = (MAX-2).wrapping_add(1).wrapping_add(1) = MAX
        assert_eq!(h.sample(&mut rng, &ctx), i32::MAX);
    }

    #[test]
    fn negative_plateau_and_wrapping_subtraction() {
        let h = TrapezoidHeight::of(ConstAnchor(i32::MIN), ConstAnchor(i32::MIN + 4), -2);
        let mut rng = SeqRandom::new(vec![0, 0]);
        let ctx = WorldGenerationContext::default();
        // range=4, plateau -2 < 4
        // mid = (4 - (-2)) / 2 = 3, top = 1
        // result = MIN + next_int(2)->0 + next_int(4)->0 = MIN
        assert_eq!(h.sample(&mut rng, &ctx), i32::MIN);
        assert_eq!(
            h.to_string(),
            format!("trapezoid(-2) in [{}-{}]", i32::MIN, i32::MIN + 4)
        );
    }
}
