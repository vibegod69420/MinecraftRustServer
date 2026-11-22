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

pub trait IntProvider {
    // JAVA: net.minecraft.util.valueproviders.IntProvider#sample
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
    // JAVA: net.minecraft.util.valueproviders.IntProvider#getMinValue
    fn get_min_value(&self) -> i32;
    // JAVA: net.minecraft.util.valueproviders.IntProvider#getMaxValue
    fn get_max_value(&self) -> i32;
    // JAVA: net.minecraft.util.valueproviders.IntProvider#getType
    fn get_type(&self) -> IntProviderType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiasedToBottomInt {
    min_inclusive: i32,
    max_inclusive: i32,
}

impl BiasedToBottomInt {
    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#<init>
    fn new(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#of
    pub fn of(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self::new(min_inclusive, max_inclusive)
    }

    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#sample
    pub fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        let range = self.max_inclusive.wrapping_sub(self.min_inclusive).wrapping_add(1);
        let inner = random.next_int(range);
        let outer_bound = inner.wrapping_add(1);
        self.min_inclusive.wrapping_add(random.next_int(outer_bound))
    }

    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#getMinValue
    pub fn get_min_value(&self) -> i32 {
        self.min_inclusive
    }

    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#getMaxValue
    pub fn get_max_value(&self) -> i32 {
        self.max_inclusive
    }

    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#getType
    pub fn get_type(&self) -> IntProviderType {
        IntProviderType::BiasedToBottom
    }
}

impl IntProvider for BiasedToBottomInt {
    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#sample
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        Self::sample(self, random)
    }

    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#getMinValue
    fn get_min_value(&self) -> i32 {
        Self::get_min_value(self)
    }

    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#getMaxValue
    fn get_max_value(&self) -> i32 {
        Self::get_max_value(self)
    }

    // JAVA: net.minecraft.util.valueproviders.BiasedToBottomInt#getType
    fn get_type(&self) -> IntProviderType {
        Self::get_type(self)
    }
}

impl fmt::Display for BiasedToBottomInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}]", self.min_inclusive, self.max_inclusive)
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
        fn next_int(&mut self, bound: i32) -> i32 {
            assert!(bound > 0);
            let v = self.values[self.idx];
            self.idx += 1;
            v
        }
    }

    #[test]
    fn biased_to_bottom_basic() {
        let b = BiasedToBottomInt::of(1, 4);
        // range = 4, inner = nextInt(4)=3, outer = nextInt(4)=0 -> 1
        let mut r = SeqRandom::new(vec![3, 0]);
        assert_eq!(b.sample(&mut r), 1);
        assert_eq!(b.get_min_value(), 1);
        assert_eq!(b.get_max_value(), 4);
        assert_eq!(b.get_type(), IntProviderType::BiasedToBottom);
    }

    #[test]
    fn biased_to_bottom_can_reach_max() {
        let b = BiasedToBottomInt::of(0, 3);
        // range=4, inner=3, outer bound=4, outer=3 -> 3
        let mut r = SeqRandom::new(vec![3, 3]);
        assert_eq!(b.sample(&mut r), 3);
    }

    #[test]
    fn biased_to_bottom_single_value() {
        let b = BiasedToBottomInt::of(5, 5);
        let mut r = SeqRandom::new(vec![0, 0]);
        assert_eq!(b.sample(&mut r), 5);
        assert_eq!(b.to_string(), "[5-5]");
    }

    #[test]
    fn biased_to_bottom_negative_range() {
        let b = BiasedToBottomInt::of(-3, -1);
        // range=3, inner=2, outer bound=3, outer=2 -> -1
        let mut r = SeqRandom::new(vec![2, 2]);
        assert_eq!(b.sample(&mut r), -1);
    }
}
