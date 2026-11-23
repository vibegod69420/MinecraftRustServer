use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InclusiveRange<T> {
    min_inclusive: T,
    max_inclusive: T,
}

impl<T: Ord + fmt::Debug> InclusiveRange<T> {
    // JAVA: net.minecraft.util.InclusiveRange#<init>
    pub fn new(min_inclusive: T, max_inclusive: T) -> Self {
        if min_inclusive > max_inclusive {
            panic!("min_inclusive must be less than or equal to max_inclusive");
        }
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    // JAVA: net.minecraft.util.InclusiveRange#<init>
    pub fn single(value: T) -> Self
    where
        T: Clone,
    {
        Self::new(value.clone(), value)
    }

    // JAVA: net.minecraft.util.InclusiveRange#create
    pub fn create(min_inclusive: T, max_inclusive: T) -> Result<Self, String> {
        if min_inclusive <= max_inclusive {
            Ok(Self {
                min_inclusive,
                max_inclusive,
            })
        } else {
            Err("min_inclusive must be less than or equal to max_inclusive".to_string())
        }
    }
}

impl<T: Ord> InclusiveRange<T> {
    // JAVA: net.minecraft.util.InclusiveRange#minInclusive
    pub fn min_inclusive(&self) -> &T {
        &self.min_inclusive
    }

    // JAVA: net.minecraft.util.InclusiveRange#maxInclusive
    pub fn max_inclusive(&self) -> &T {
        &self.max_inclusive
    }

    // JAVA: net.minecraft.util.InclusiveRange#map
    pub fn map<S: Ord + fmt::Debug>(&self, f: impl Fn(&T) -> S) -> InclusiveRange<S> {
        InclusiveRange::new(f(&self.min_inclusive), f(&self.max_inclusive))
    }

    // JAVA: net.minecraft.util.InclusiveRange#isValueInRange
    pub fn is_value_in_range(&self, value: &T) -> bool {
        value >= &self.min_inclusive && value <= &self.max_inclusive
    }

    // JAVA: net.minecraft.util.InclusiveRange#contains
    pub fn contains(&self, other: &InclusiveRange<T>) -> bool {
        other.min_inclusive >= self.min_inclusive && other.max_inclusive <= self.max_inclusive
    }
}

impl<T: fmt::Display> fmt::Display for InclusiveRange<T> {
    // JAVA: net.minecraft.util.InclusiveRange#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.min_inclusive, self.max_inclusive)
    }
}

#[cfg(test)]
mod tests {
    use super::InclusiveRange;

    #[test]
    fn new_and_accessors_and_display() {
        let r = InclusiveRange::new(1, 5);
        assert_eq!(*r.min_inclusive(), 1);
        assert_eq!(*r.max_inclusive(), 5);
        assert_eq!(r.to_string(), "[1, 5]");
    }

    #[test]
    #[should_panic(expected = "min_inclusive must be less than or equal to max_inclusive")]
    fn new_invalid_panics() {
        let _ = InclusiveRange::new(5, 1);
    }

    #[test]
    fn create_and_single() {
        let ok = InclusiveRange::create(2, 2).unwrap();
        assert_eq!(ok, InclusiveRange::single(2));
        let err = InclusiveRange::create(3, 2).unwrap_err();
        assert_eq!(
            err,
            "min_inclusive must be less than or equal to max_inclusive"
        );
    }

    #[test]
    fn is_value_in_range_and_contains() {
        let r = InclusiveRange::new(-2, 4);
        assert!(r.is_value_in_range(&-2));
        assert!(r.is_value_in_range(&4));
        assert!(!r.is_value_in_range(&5));
        assert!(r.contains(&InclusiveRange::new(-1, 3)));
        assert!(r.contains(&InclusiveRange::new(-2, 4)));
        assert!(!r.contains(&InclusiveRange::new(-3, 4)));
    }

    #[test]
    fn map_and_extreme_bounds() {
        let r = InclusiveRange::new(i32::MIN, i32::MAX);
        let mapped = r.map(|v| i64::from(*v) + 1);
        assert_eq!(*mapped.min_inclusive(), i64::from(i32::MIN) + 1);
        assert_eq!(*mapped.max_inclusive(), i64::from(i32::MAX) + 1);
        assert_eq!(mapped.to_string(), format!("[{}, {}]", i64::from(i32::MIN) + 1, i64::from(i32::MAX) + 1));
    }
}
