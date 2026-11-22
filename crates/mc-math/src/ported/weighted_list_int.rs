use std::fmt;

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProviderType {
    Constant,
    Uniform,
    BiasedToBottom,
    Clamped,
    WeightedList,
}

pub trait IntProvider: fmt::Debug {
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
pub struct Weighted<T> {
    pub value: T,
    pub weight: i32,
}

impl<T> Weighted<T> {
    // JAVA: net.minecraft.util.random.Weighted#<init>/value access
    pub fn new(value: T, weight: i32) -> Self {
        Self { value, weight }
    }

    // JAVA: net.minecraft.util.random.Weighted#value
    pub fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct WeightedList<T> {
    entries: Vec<Weighted<T>>,
    total_weight: i32,
}

impl<T> WeightedList<T> {
    // JAVA: net.minecraft.util.random.WeightedList#of/constructor-equivalent
    pub fn new(entries: Vec<Weighted<T>>) -> Self {
        let mut total_weight: i32 = 0;
        for e in &entries {
            total_weight = total_weight.wrapping_add(e.weight);
        }
        Self {
            entries,
            total_weight,
        }
    }

    // JAVA: net.minecraft.util.random.WeightedList#unwrap
    pub fn unwrap(&self) -> &[Weighted<T>] {
        &self.entries
    }

    // JAVA: net.minecraft.util.random.WeightedList#getRandomOrThrow
    pub fn get_random_or_throw(&self, random: &mut dyn RandomSource) -> &T {
        if self.total_weight <= 0 {
            panic!("Empty or invalid weighted list");
        }
        let mut i = random.next_int(self.total_weight);
        for entry in &self.entries {
            i = i.wrapping_sub(entry.weight);
            if i < 0 {
                return &entry.value;
            }
        }
        panic!("Weighted list selection failed")
    }
}

#[derive(Debug, Clone)]
pub struct WeightedListInt<P: IntProvider + Clone> {
    distribution: WeightedList<P>,
    min_value: i32,
    max_value: i32,
}

impl<P: IntProvider + Clone> WeightedListInt<P> {
    // JAVA: net.minecraft.util.valueproviders.WeightedListInt#<init>
    pub fn new(distribution: WeightedList<P>) -> Self {
        let mut min_value = i32::MAX;
        let mut max_value = i32::MIN;

        for weighted in distribution.unwrap() {
            let v = weighted.value();
            let min_v = v.get_min_value();
            let max_v = v.get_max_value();
            if min_v < min_value {
                min_value = min_v;
            }
            if max_v > max_value {
                max_value = max_v;
            }
        }

        Self {
            distribution,
            min_value,
            max_value,
        }
    }
}

impl<P: IntProvider + Clone> IntProvider for WeightedListInt<P> {
    // JAVA: net.minecraft.util.valueproviders.WeightedListInt#sample
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        self.distribution.get_random_or_throw(random).sample(random)
    }

    // JAVA: net.minecraft.util.valueproviders.WeightedListInt#getMinValue
    fn get_min_value(&self) -> i32 {
        self.min_value
    }

    // JAVA: net.minecraft.util.valueproviders.WeightedListInt#getMaxValue
    fn get_max_value(&self) -> i32 {
        self.max_value
    }

    // JAVA: net.minecraft.util.valueproviders.WeightedListInt#getType
    fn get_type(&self) -> IntProviderType {
        IntProviderType::WeightedList
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct ConstInt(i32);
    impl IntProvider for ConstInt {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }
        fn get_min_value(&self) -> i32 {
            self.0
        }
        fn get_max_value(&self) -> i32 {
            self.0
        }
        fn get_type(&self) -> IntProviderType {
            IntProviderType::Constant
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
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.values[self.idx];
            self.idx += 1;
            v
        }
    }

    #[test]
    fn weighted_list_int_bounds() {
        let list = WeightedList::new(vec![
            Weighted::new(ConstInt(-5), 1),
            Weighted::new(ConstInt(10), 2),
            Weighted::new(ConstInt(3), 1),
        ]);
        let w = WeightedListInt::new(list);
        assert_eq!(w.get_min_value(), -5);
        assert_eq!(w.get_max_value(), 10);
        assert_eq!(w.get_type(), IntProviderType::WeightedList);
    }

    #[test]
    fn weighted_list_int_samples_by_weight() {
        let list = WeightedList::new(vec![
            Weighted::new(ConstInt(1), 1),
            Weighted::new(ConstInt(2), 3),
        ]);
        let w = WeightedListInt::new(list);
        let mut r0 = SeqRandom::new(vec![0]); // selects first
        let mut r1 = SeqRandom::new(vec![1]); // selects second
        let mut r3 = SeqRandom::new(vec![3]); // selects second (last)
        assert_eq!(w.sample(&mut r0), 1);
        assert_eq!(w.sample(&mut r1), 2);
        assert_eq!(w.sample(&mut r3), 2);
    }

    #[test]
    fn weighted_list_single_entry() {
        let w = WeightedListInt::new(WeightedList::new(vec![Weighted::new(ConstInt(42), 5)]));
        let mut r = SeqRandom::new(vec![4]);
        assert_eq!(w.sample(&mut r), 42);
        assert_eq!(w.get_min_value(), 42);
        assert_eq!(w.get_max_value(), 42);
    }

    #[test]
    fn weighted_list_negative_values_bounds() {
        let w = WeightedListInt::new(WeightedList::new(vec![
            Weighted::new(ConstInt(i32::MIN), 1),
            Weighted::new(ConstInt(-1), 1),
        ]));
        assert_eq!(w.get_min_value(), i32::MIN);
        assert_eq!(w.get_max_value(), -1);
    }
}
