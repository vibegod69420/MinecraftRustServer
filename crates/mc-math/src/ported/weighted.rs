//! Port of `net.minecraft.util.random.Weighted` (Java 1.21.11).
//!
//! The Java type is a record `Weighted<T>(T value, int weight)` with validation,
//! codec/stream-codec helpers, and a `map` operation. This Rust port provides the
//! core data structure and behavior that are meaningful in a self-contained math/RNG
//! crate: construction with validation, accessors, and mapping.
//!
//! Java codec/serialization infrastructure (`Codec`, `MapCodec`, `RecordCodecBuilder`,
//! `StreamCodec`, `ByteBuf`, logging, IDE pause helpers, shared constants) is not part
//! of `mc-math`'s dependencies. Equivalent validation semantics are preserved.

use std::fmt;

/// Error returned when constructing a [`Weighted`] with an invalid weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeightedError {
    /// JAVA: `new IllegalArgumentException("Weight should be >= 0")`
    NegativeWeight { weight: i32 },
}

impl fmt::Display for WeightedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeightedError::NegativeWeight { weight } => {
                write!(f, "Weight should be >= 0 (got {weight})")
            }
        }
    }
}

impl std::error::Error for WeightedError {}

/// A value paired with a non-negative integer weight.
///
/// JAVA: `net.minecraft.util.random.Weighted<T>`
///
/// In Java this is a `record Weighted<T>(T value, int weight)`.
/// Construction rejects negative weights. A zero weight is accepted but would
/// trigger an IDE warning in Java; this port accepts it without side effects,
/// preserving the core behavioral contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Weighted<T> {
    value: T,
    weight: i32,
}

impl<T> Weighted<T> {
    /// Creates a new weighted value.
    ///
    /// Returns [`WeightedError::NegativeWeight`] if `weight < 0`, matching Java's
    /// constructor validation.
    ///
    /// JAVA: net.minecraft.util.random.Weighted#<init>
    pub fn new(value: T, weight: i32) -> Result<Self, WeightedError> {
        if weight < 0 {
            // JAVA: throw Util.pauseInIde(new IllegalArgumentException("Weight should be >= 0"));
            // Util.pauseInIde is an IDE debugging hook; the observable behavior is the exception.
            return Err(WeightedError::NegativeWeight { weight });
        }

        // JAVA: if (weight == 0 && SharedConstants.IS_RUNNING_IN_IDE) { LOGGER.warn(...); }
        // IDE-only warning has no behavioral effect on the created value; omitted.

        Ok(Self { value, weight })
    }

    /// Creates a new weighted value without checking the weight.
    ///
    /// This is a Rust convenience for callers that have already validated the
    /// weight. It does not correspond to a separate Java API; Java always
    /// validates through the canonical constructor.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `weight < 0`.
    pub fn new_unchecked(value: T, weight: i32) -> Self {
        debug_assert!(weight >= 0, "Weight should be >= 0 (got {weight})");
        Self { value, weight }
    }

    /// Returns the wrapped value.
    ///
    /// JAVA: net.minecraft.util.random.Weighted#value
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consumes this weighted value and returns the wrapped value.
    ///
    /// This is a Rust ownership convenience corresponding to record component access.
    ///
    /// JAVA: net.minecraft.util.random.Weighted#value
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns the weight.
    ///
    /// JAVA: net.minecraft.util.random.Weighted#weight
    pub fn weight(&self) -> i32 {
        self.weight
    }

    /// Maps the wrapped value, preserving the weight.
    ///
    /// JAVA: net.minecraft.util.random.Weighted#map
    pub fn map<U, F>(self, f: F) -> Weighted<U>
    where
        F: FnOnce(T) -> U,
    {
        // JAVA: return new Weighted<>($$0.apply(this.value()), this.weight);
        // The Java constructor validates weight; this weight is already known valid.
        Weighted {
            value: f(self.value),
            weight: self.weight,
        }
    }

    /// Maps the wrapped value by reference, preserving the weight.
    ///
    /// This is a Rust convenience; Java's `map` consumes/operates on the record value.
    pub fn map_ref<U, F>(&self, f: F) -> Weighted<U>
    where
        F: FnOnce(&T) -> U,
    {
        Weighted {
            value: f(&self.value),
            weight: self.weight,
        }
    }

    /// Replaces the weight, validating the new weight.
    ///
    /// Not present in Java; provided as a safe Rust helper.
    pub fn with_weight(self, weight: i32) -> Result<Self, WeightedError> {
        Self::new(self.value, weight)
    }
}

impl<T: fmt::Display> fmt::Display for Weighted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Weighted(value={}, weight={})", self.value, self.weight)
    }
}

impl<T: Default> Default for Weighted<T> {
    /// Creates a `Weighted<T>` with `T::default()` and weight `0`.
    ///
    /// Weight 0 is valid in Java (though it logs an IDE warning there).
    fn default() -> Self {
        Self {
            value: T::default(),
            weight: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_positive_weight() {
        // JAVA: net.minecraft.util.random.Weighted#<init>
        let w = Weighted::new("apple", 5).expect("positive weight should be valid");
        assert_eq!(w.value(), &"apple");
        assert_eq!(w.weight(), 5);
    }

    #[test]
    fn new_accepts_zero_weight() {
        // JAVA: net.minecraft.util.random.Weighted#<init>
        // Java accepts weight == 0 but warns in IDE; value is still constructed.
        let w = Weighted::new(42_i32, 0).expect("zero weight should be valid");
        assert_eq!(w.value(), &42_i32);
        assert_eq!(w.weight(), 0);
    }

    #[test]
    fn new_rejects_negative_weight() {
        // JAVA: net.minecraft.util.random.Weighted#<init>
        let err = Weighted::new("bad", -1).unwrap_err();
        assert_eq!(err, WeightedError::NegativeWeight { weight: -1 });
        assert_eq!(err.to_string(), "Weight should be >= 0 (got -1)");
    }

    #[test]
    fn new_rejects_i32_min_weight() {
        // JAVA: net.minecraft.util.random.Weighted#<init>
        let err = Weighted::new((), i32::MIN).unwrap_err();
        assert_eq!(
            err,
            WeightedError::NegativeWeight { weight: i32::MIN }
        );
    }

    #[test]
    fn new_rejects_minus_one_hundred() {
        // JAVA: net.minecraft.util.random.Weighted#<init>
        let err = Weighted::new(0_u8, -100).unwrap_err();
        assert_eq!(err, WeightedError::NegativeWeight { weight: -100 });
    }

    #[test]
    fn new_accepts_i32_max_weight() {
        // JAVA: net.minecraft.util.random.Weighted#<init>
        let w = Weighted::new("max", i32::MAX).expect("i32::MAX should be valid");
        assert_eq!(w.weight(), i32::MAX);
        assert_eq!(w.value(), &"max");
    }

    #[test]
    fn value_accessor() {
        // JAVA: net.minecraft.util.random.Weighted#value
        let w = Weighted::new(String::from("hello"), 3).unwrap();
        assert_eq!(w.value(), "hello");
    }

    #[test]
    fn into_value_consumes() {
        // JAVA: net.minecraft.util.random.Weighted#value
        let w = Weighted::new(String::from("owned"), 7).unwrap();
        let v = w.into_value();
        assert_eq!(v, "owned");
    }

    #[test]
    fn weight_accessor() {
        // JAVA: net.minecraft.util.random.Weighted#weight
        let w = Weighted::new((), 123).unwrap();
        assert_eq!(w.weight(), 123);
    }

    #[test]
    fn map_preserves_weight_and_transforms_value() {
        // JAVA: net.minecraft.util.random.Weighted#map
        let w = Weighted::new(10_i32, 4).unwrap();
        let mapped = w.map(|v| v.wrapping_mul(2));
        assert_eq!(mapped.value(), &20_i32);
        assert_eq!(mapped.weight(), 4);
    }

    #[test]
    fn map_changes_type() {
        // JAVA: net.minecraft.util.random.Weighted#map
        let w = Weighted::new(5_u32, 9).unwrap();
        let mapped = w.map(|v| v.to_string());
        assert_eq!(mapped.value(), "5");
        assert_eq!(mapped.weight(), 9);
    }

    #[test]
    fn map_on_zero_weight() {
        // JAVA: net.minecraft.util.random.Weighted#map
        let w = Weighted::new("x", 0).unwrap();
        let mapped = w.map(|s| s.len());
        assert_eq!(mapped.value(), &1_usize);
        assert_eq!(mapped.weight(), 0);
    }

    #[test]
    fn map_ref_preserves_original() {
        let w = Weighted::new(vec![1, 2, 3], 2).unwrap();
        let mapped = w.map_ref(|v| v.len());
        assert_eq!(mapped.value(), &3_usize);
        assert_eq!(mapped.weight(), 2);
        assert_eq!(w.value(), &vec![1, 2, 3]);
    }

    #[test]
    fn with_weight_valid() {
        let w = Weighted::new("a", 1).unwrap();
        let w2 = w.with_weight(99).unwrap();
        assert_eq!(w2.value(), &"a");
        assert_eq!(w2.weight(), 99);
    }

    #[test]
    fn with_weight_rejects_negative() {
        let w = Weighted::new("a", 1).unwrap();
        let err = w.with_weight(-5).unwrap_err();
        assert_eq!(err, WeightedError::NegativeWeight { weight: -5 });
    }

    #[test]
    fn display_format() {
        let w = Weighted::new(7_i32, 3).unwrap();
        assert_eq!(w.to_string(), "Weighted(value=7, weight=3)");
    }

    #[test]
    fn error_display() {
        let err = WeightedError::NegativeWeight { weight: -42 };
        assert_eq!(err.to_string(), "Weight should be >= 0 (got -42)");
    }

    #[test]
    fn default_is_zero_weight() {
        let w: Weighted<i32> = Weighted::default();
        assert_eq!(w.value(), &0_i32);
        assert_eq!(w.weight(), 0);
    }

    #[test]
    fn clone_and_eq() {
        let w1 = Weighted::new(String::from("same"), 10).unwrap();
        let w2 = w1.clone();
        assert_eq!(w1, w2);
    }

    #[test]
    fn copy_for_copy_types() {
        let w1 = Weighted::new(1_i32, 2).unwrap();
        let w2 = w1;
        assert_eq!(w1.value(), &1_i32);
        assert_eq!(w2.weight(), 2);
    }

    #[test]
    fn negative_weight_boundary_values() {
        // JAVA: net.minecraft.util.random.Weighted#<init>
        for weight in [i32::MIN, -1000, -2, -1] {
            let err = Weighted::new((), weight).unwrap_err();
            assert_eq!(err, WeightedError::NegativeWeight { weight });
        }
    }

    #[test]
    fn non_negative_weight_boundary_values() {
        // JAVA: net.minecraft.util.random.Weighted#<init>
        for weight in [0, 1, 2, 1000, i32::MAX] {
            let w = Weighted::new((), weight).unwrap();
            assert_eq!(w.weight(), weight);
        }
    }

    #[test]
    fn map_chain_preserves_weight() {
        // JAVA: net.minecraft.util.random.Weighted#map
        let w = Weighted::new(2_i32, 11).unwrap();
        let mapped = w.map(|v| v.wrapping_add(3)).map(|v| v.wrapping_mul(4));
        assert_eq!(mapped.value(), &20_i32);
        assert_eq!(mapped.weight(), 11);
    }

    #[test]
    fn new_unchecked_valid_weight() {
        let w = Weighted::new_unchecked(5_i32, 10);
        assert_eq!(w.value(), &5_i32);
        assert_eq!(w.weight(), 10);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "Weight should be >= 0")]
    fn new_unchecked_panics_on_negative_in_debug() {
        let _ = Weighted::new_unchecked((), -1);
    }
}
