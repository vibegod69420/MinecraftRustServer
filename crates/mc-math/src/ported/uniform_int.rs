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
pub struct UniformInt {
    min_inclusive: i32,
    max_inclusive: i32,
}

impl UniformInt {
    // JAVA: net.minecraft.util.valueproviders.UniformInt#<init>
    fn new(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    // JAVA: net.minecraft.util.valueproviders.UniformInt#of
    pub fn of(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self::new(min_inclusive, max_inclusive)
    }

    // JAVA: net.minecraft.util.valueproviders.UniformInt#sample
    pub fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        // Mth.randomBetweenInclusive(random, min, max)
        let bound = self.max_inclusive.wrapping_sub(self.min_inclusive).wrapping_add(1);
        self.min_inclusive.wrapping_add(random.next_int(bound))
    }

    // JAVA: net.minecraft.util.valueproviders.UniformInt#getMinValue
    pub fn get_min_value(&self) -> i32 {
        self.min_inclusive
    }

    // JAVA: net.minecraft.util.valueproviders.UniformInt#getMaxValue
    pub fn get_max_value(&self) -> i32 {
        self.max_inclusive
    }

    // JAVA: net.minecraft.util.valueproviders.UniformInt#getType
    pub fn get_type(&self) -> IntProviderType {
        IntProviderType::Uniform
    }
}

impl IntProvider for UniformInt {
    // JAVA: net.minecraft.util.valueproviders.UniformInt#sample
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        Self::sample(self, random)
    }

    // JAVA: net.minecraft.util.valueproviders.UniformInt#getMinValue
    fn get_min_value(&self) -> i32 {
        Self::get_min_value(self)
    }

    // JAVA: net.minecraft.util.valueproviders.UniformInt#getMaxValue
    fn get_max_value(&self) -> i32 {
        Self::get_max_value(self)
    }

    // JAVA: net.minecraft.util.valueproviders.UniformInt#getType
    fn get_type(&self) -> IntProviderType {
        Self::get_type(self)
    }
}

impl fmt::Display for UniformInt {
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
    fn uniform_int_basic() {
        let u = UniformInt::of(2, 5);
        let mut r = SeqRandom::new(vec![0, 3]);
        assert_eq!(u.sample(&mut r), 2);
        assert_eq!(u.sample(&mut r), 5);
        assert_eq!(u.get_min_value(), 2);
        assert_eq!(u.get_max_value(), 5);
        assert_eq!(u.get_type(), IntProviderType::Uniform);
    }

    #[test]
    fn uniform_int_single_value() {
        let u = UniformInt::of(7, 7);
        let mut r = SeqRandom::new(vec![0]);
        assert_eq!(u.sample(&mut r), 7);
        assert_eq!(u.to_string(), "[7-7]");
    }

    #[test]
    fn uniform_int_negative_range() {
        let u = UniformInt::of(-10, -5);
        let mut r = SeqRandom::new(vec![0, 5]);
        assert_eq!(u.sample(&mut r), -10);
        assert_eq!(u.sample(&mut r), -5);
        assert_eq!(u.to_string(), "[-10--5]");
    }

    #[test]
    fn uniform_int_overflow_boundary() {
        let u = UniformInt::of(i32::MAX - 1, i32::MAX);
        let mut r = SeqRandom::new(vec![0, 1]);
        assert_eq!(u.sample(&mut r), i32::MAX - 1);
        assert_eq!(u.sample(&mut r), i32::MAX);
    }
}
