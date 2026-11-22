//! Port of `net.minecraft.world.level.levelgen.heightproviders.WeightedListHeight`.

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

/// Minimal weighted entry for self-contained compilation.
#[derive(Debug, Clone)]
pub struct WeightedEntry<T> {
    pub data: T,
    pub weight: i32,
}

/// Minimal weighted list for self-contained compilation.
#[derive(Debug, Clone)]
pub struct WeightedList<T> {
    entries: Vec<WeightedEntry<T>>,
}

impl<T> WeightedList<T> {
    pub fn of(entries: Vec<WeightedEntry<T>>) -> Self {
        Self { entries }
    }

    // JAVA: net.minecraft.util.random.WeightedList#getRandomOrThrow
    pub fn get_random_or_throw<R: RandomSource + ?Sized>(&self, random: &mut R) -> &T {
        let mut total: i32 = 0;
        for entry in &self.entries {
            total = total.wrapping_add(entry.weight);
        }
        if total <= 0 {
            panic!("WeightedList total weight must be positive");
        }

        let mut r = random.next_int(total);
        for entry in &self.entries {
            r = r.wrapping_sub(entry.weight);
            if r < 0 {
                return &entry.data;
            }
        }
        panic!("WeightedList selection failed");
    }
}

/// Port of `net.minecraft.world.level.levelgen.heightproviders.WeightedListHeight`.
#[derive(Debug, Clone)]
pub struct WeightedListHeight<H> {
    distribution: WeightedList<H>,
}

impl<H> WeightedListHeight<H> {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.WeightedListHeight#<init>
    pub fn new(distribution: WeightedList<H>) -> Self {
        Self { distribution }
    }

    pub fn distribution(&self) -> &WeightedList<H> {
        &self.distribution
    }
}

impl<H: HeightProvider> HeightProvider for WeightedListHeight<H> {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.WeightedListHeight#sample
    fn sample(&self, random: &mut dyn RandomSource, context: &WorldGenerationContext) -> i32 {
        self.distribution
            .get_random_or_throw(random)
            .sample(random, context)
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.WeightedListHeight#getType
    fn get_type(&self) -> HeightProviderType {
        HeightProviderType::WeightedList
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct ConstProvider(i32);

    impl HeightProvider for ConstProvider {
        fn sample(
            &self,
            _random: &mut dyn RandomSource,
            _context: &WorldGenerationContext,
        ) -> i32 {
            self.0
        }

        fn get_type(&self) -> HeightProviderType {
            HeightProviderType::Constant
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct RandomProvider;

    impl HeightProvider for RandomProvider {
        fn sample(
            &self,
            random: &mut dyn RandomSource,
            _context: &WorldGenerationContext,
        ) -> i32 {
            random.next_int(100)
        }

        fn get_type(&self) -> HeightProviderType {
            HeightProviderType::Constant
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

    fn ctx() -> WorldGenerationContext {
        WorldGenerationContext {
            min_gen_y: -64,
            gen_depth: 384,
        }
    }

    #[test]
    fn single_entry_always_selected() {
        let wl = WeightedListHeight::new(WeightedList::of(vec![WeightedEntry {
            data: ConstProvider(42),
            weight: 1,
        }]));
        let mut rng = SeqRandom::new(vec![0]);
        assert_eq!(wl.sample(&mut rng, &ctx()), 42);
        assert_eq!(wl.get_type(), HeightProviderType::WeightedList);
    }

    #[test]
    fn weighted_selection_respects_weights() {
        let wl = WeightedListHeight::new(WeightedList::of(vec![
            WeightedEntry {
                data: ConstProvider(10),
                weight: 1,
            },
            WeightedEntry {
                data: ConstProvider(20),
                weight: 3,
            },
        ]));
        let c = ctx();

        let mut rng0 = SeqRandom::new(vec![0]);
        assert_eq!(wl.sample(&mut rng0, &c), 10);

        let mut rng1 = SeqRandom::new(vec![1]);
        assert_eq!(wl.sample(&mut rng1, &c), 20);

        let mut rng3 = SeqRandom::new(vec![3]);
        assert_eq!(wl.sample(&mut rng3, &c), 20);
    }

    #[test]
    fn sample_uses_same_random_for_selection_and_provider() {
        let wl = WeightedListHeight::new(WeightedList::of(vec![WeightedEntry {
            data: RandomProvider,
            weight: 1,
        }]));
        // first next_int(1)=0 for selection, second next_int(100)=77 for provider
        let mut rng = SeqRandom::new(vec![0, 77]);
        assert_eq!(wl.sample(&mut rng, &ctx()), 77);
    }

    #[test]
    fn negative_weight_sum_panics() {
        let wl = WeightedListHeight::new(WeightedList::of(vec![
            WeightedEntry {
                data: ConstProvider(1),
                weight: i32::MAX,
            },
            WeightedEntry {
                data: ConstProvider(2),
                weight: 1,
            },
        ]));
        let mut rng = SeqRandom::new(vec![0]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = wl.sample(&mut rng, &ctx());
        }));
        assert!(result.is_err());
    }

    #[test]
    fn selection_with_negative_input_value_still_works_via_rem_euclid() {
        let wl = WeightedListHeight::new(WeightedList::of(vec![
            WeightedEntry {
                data: ConstProvider(-5),
                weight: 2,
            },
            WeightedEntry {
                data: ConstProvider(5),
                weight: 2,
            },
        ]));
        let mut rng = SeqRandom::new(vec![-1]);
        // -1 rem_euclid 4 = 3 -> selects second
        assert_eq!(wl.sample(&mut rng, &ctx()), 5);
    }
}
