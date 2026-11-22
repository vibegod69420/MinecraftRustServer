//! Port of `net.minecraft.util.parsing.packrat.SuggestionSupplier`.
//!
//! This module provides a trait for supplying parse suggestions and a faithful
//! `empty()` supplier that returns no suggestions, matching Java's
//! `Stream.empty()`.

use std::marker::PhantomData;

/// Minimal self-contained stub for `net.minecraft.util.parsing.packrat.ParseState<S>`.
///
/// The full parser state is not ported in this module; this type preserves the
/// generic shape required by [`SuggestionSupplier`].
#[derive(Debug)]
pub struct ParseState<S> {
    _marker: PhantomData<fn() -> S>,
}

impl<S> ParseState<S> {
    /// Creates a new parse state stub.
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<S> Default for ParseState<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Clone for ParseState<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for ParseState<S> {}

impl<S> PartialEq for ParseState<S> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<S> Eq for ParseState<S> {}

/// Port of `net.minecraft.util.parsing.packrat.SuggestionSupplier<S>`.
///
/// A supplier that produces possible string suggestions for a given parse state.
pub trait SuggestionSupplier<S> {
    // JAVA: net.minecraft.util.parsing.packrat.SuggestionSupplier#possibleValues
    fn possible_values(&self, state: &ParseState<S>) -> Box<dyn Iterator<Item = String> + '_>;
}

/// Concrete empty [`SuggestionSupplier`] returned by [`empty()`].
///
/// This corresponds to the Java lambda `$$0 -> Stream.empty()`.
#[derive(Debug)]
pub struct EmptySuggestionSupplier<S> {
    _marker: PhantomData<fn() -> S>,
}

impl<S> EmptySuggestionSupplier<S> {
    /// Creates a new empty suggestion supplier.
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<S> Default for EmptySuggestionSupplier<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Clone for EmptySuggestionSupplier<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for EmptySuggestionSupplier<S> {}

impl<S> PartialEq for EmptySuggestionSupplier<S> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<S> Eq for EmptySuggestionSupplier<S> {}

// JAVA: net.minecraft.util.parsing.packrat.SuggestionSupplier#empty
pub fn empty<S>() -> EmptySuggestionSupplier<S> {
    EmptySuggestionSupplier::new()
}

impl<S> SuggestionSupplier<S> for EmptySuggestionSupplier<S> {
    // JAVA: net.minecraft.util.parsing.packrat.SuggestionSupplier#possibleValues
    fn possible_values(&self, _state: &ParseState<S>) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(std::iter::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn empty_returns_no_suggestions() {
        let state = ParseState::<()>::new();
        let supplier = empty::<()>();
        let mut values = supplier.possible_values(&state);
        assert_eq!(values.next(), None);
    }

    #[test]
    fn empty_collects_to_empty_vec() {
        let state = ParseState::<i32>::new();
        let supplier = empty::<i32>();
        let values: Vec<String> = supplier.possible_values(&state).collect();
        assert!(values.is_empty());
    }

    #[test]
    fn empty_iterator_count_is_zero() {
        let state = ParseState::<u64>::new();
        let supplier = empty::<u64>();
        assert_eq!(supplier.possible_values(&state).count(), 0);
    }

    #[test]
    fn empty_iterator_size_hint_is_zero() {
        let state = ParseState::<u8>::new();
        let supplier = empty::<u8>();
        let iter = supplier.possible_values(&state);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn empty_iterator_exhausted_stays_none() {
        let state = ParseState::<i64>::new();
        let supplier = empty::<i64>();
        let mut iter = supplier.possible_values(&state);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn empty_can_be_called_multiple_times() {
        let state = ParseState::<i32>::new();
        let supplier = empty::<i32>();
        assert_eq!(supplier.possible_values(&state).count(), 0);
        assert_eq!(supplier.possible_values(&state).count(), 0);
        assert!(supplier.possible_values(&state).next().is_none());
    }

    #[test]
    fn empty_works_for_float_types() {
        let f32_state = ParseState::<f32>::new();
        let f64_state = ParseState::<f64>::new();

        let f32_supplier = empty::<f32>();
        let f64_supplier = empty::<f64>();
        let mut f32_values = f32_supplier.possible_values(&f32_state);
        let mut f64_values = f64_supplier.possible_values(&f64_state);

        assert_eq!(f32_values.next(), None);
        assert_eq!(f64_values.next(), None);
    }

    #[test]
    fn empty_works_for_various_types() {
        let i32_state = ParseState::<i32>::new();
        let i64_state = ParseState::<i64>::new();
        let u32_state = ParseState::<u32>::new();
        let bool_state = ParseState::<bool>::new();
        let string_state = ParseState::<String>::new();

        let i32_supplier = empty::<i32>();
        let i64_supplier = empty::<i64>();
        let u32_supplier = empty::<u32>();
        let bool_supplier = empty::<bool>();
        let string_supplier = empty::<String>();

        assert_eq!(i32_supplier.possible_values(&i32_state).next(), None);
        assert_eq!(i64_supplier.possible_values(&i64_state).next(), None);
        assert_eq!(u32_supplier.possible_values(&u32_state).next(), None);
        assert_eq!(bool_supplier.possible_values(&bool_state).next(), None);
        assert_eq!(
            string_supplier.possible_values(&string_state).next(),
            None
        );
    }

    #[test]
    fn empty_works_with_negative_and_extreme_integer_type_parameters() {
        // The type parameter S does not affect empty() behavior, but we exercise
        // representative edge-case numeric types to ensure the generic interface
        // is robust.
        let i8_state = ParseState::<i8>::new();
        let i16_state = ParseState::<i16>::new();
        let i128_state = ParseState::<i128>::new();
        let isize_state = ParseState::<isize>::new();

        let i8_supplier = empty::<i8>();
        let i16_supplier = empty::<i16>();
        let i128_supplier = empty::<i128>();
        let isize_supplier = empty::<isize>();

        assert_eq!(i8_supplier.possible_values(&i8_state).count(), 0);
        assert_eq!(i16_supplier.possible_values(&i16_state).count(), 0);
        assert_eq!(i128_supplier.possible_values(&i128_state).count(), 0);
        assert_eq!(isize_supplier.possible_values(&isize_state).count(), 0);
    }

    #[test]
    fn empty_supplier_is_zst() {
        assert_eq!(size_of::<EmptySuggestionSupplier<u8>>(), 0);
        assert_eq!(size_of::<EmptySuggestionSupplier<f32>>(), 0);
        assert_eq!(size_of::<EmptySuggestionSupplier<f64>>(), 0);
        assert_eq!(size_of::<EmptySuggestionSupplier<i32>>(), 0);
    }

    #[test]
    fn parse_state_is_zst() {
        assert_eq!(size_of::<ParseState<u8>>(), 0);
        assert_eq!(size_of::<ParseState<f64>>(), 0);
        assert_eq!(size_of::<ParseState<String>>(), 0);
    }

    #[test]
    fn default_parse_state_works() {
        let state = ParseState::<u32>::default();
        let supplier = empty::<u32>();
        assert_eq!(supplier.possible_values(&state).count(), 0);
    }

    #[test]
    fn empty_supplier_default_and_new_are_equivalent() {
        let state = ParseState::<i32>::new();
        let from_new = EmptySuggestionSupplier::<i32>::new();
        let from_default = EmptySuggestionSupplier::<i32>::default();
        let from_empty = empty::<i32>();

        assert_eq!(from_new, from_default);
        assert_eq!(from_default, from_empty);

        assert_eq!(from_new.possible_values(&state).count(), 0);
        assert_eq!(from_default.possible_values(&state).count(), 0);
        assert_eq!(from_empty.possible_values(&state).count(), 0);
    }

    #[test]
    fn parse_state_copy_clone_eq() {
        let a = ParseState::<i32>::new();
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn empty_supplier_copy_clone_eq() {
        let a = empty::<i32>();
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn custom_supplier_returns_suggestions() {
        struct FixedSupplier {
            values: Vec<String>,
        }

        impl SuggestionSupplier<i32> for FixedSupplier {
            // JAVA: net.minecraft.util.parsing.packrat.SuggestionSupplier#possibleValues
            fn possible_values(
                &self,
                _state: &ParseState<i32>,
            ) -> Box<dyn Iterator<Item = String> + '_> {
                Box::new(self.values.clone().into_iter())
            }
        }

        let state = ParseState::<i32>::new();
        let supplier = FixedSupplier {
            values: vec!["alpha".to_string(), "beta".to_string()],
        };
        let values: Vec<String> = supplier.possible_values(&state).collect();
        assert_eq!(values, vec!["alpha".to_string(), "beta".to_string()]);
    }

    #[test]
    fn custom_supplier_negative_overflow_and_edge_case_integers() {
        struct EdgeCaseSupplier;

        impl SuggestionSupplier<i32> for EdgeCaseSupplier {
            // JAVA: net.minecraft.util.parsing.packrat.SuggestionSupplier#possibleValues
            fn possible_values(
                &self,
                _state: &ParseState<i32>,
            ) -> Box<dyn Iterator<Item = String> + '_> {
                let min = i32::MIN;
                let max = i32::MAX;
                let neg_one: i32 = -1;
                let overflowed = max.wrapping_add(1); // Java integer overflow semantics
                Box::new(
                    vec![
                        min.to_string(),
                        neg_one.to_string(),
                        0i32.to_string(),
                        max.to_string(),
                        overflowed.to_string(),
                    ]
                    .into_iter(),
                )
            }
        }

        let state = ParseState::<i32>::new();
        let supplier = EdgeCaseSupplier;
        let values: Vec<String> = supplier.possible_values(&state).collect();
        assert_eq!(
            values,
            vec![
                "-2147483648".to_string(),
                "-1".to_string(),
                "0".to_string(),
                "2147483647".to_string(),
                "-2147483648".to_string(),
            ]
        );
    }

    #[test]
    fn custom_supplier_float_bit_edge_cases() {
        struct FloatBitsSupplier;

        impl SuggestionSupplier<f32> for FloatBitsSupplier {
            // JAVA: net.minecraft.util.parsing.packrat.SuggestionSupplier#possibleValues
            fn possible_values(
                &self,
                _state: &ParseState<f32>,
            ) -> Box<dyn Iterator<Item = String> + '_> {
                let nan_bits = f32::NAN.to_bits();
                let pos_inf_bits = f32::INFINITY.to_bits();
                let neg_inf_bits = f32::NEG_INFINITY.to_bits();
                let neg_zero_bits = (-0.0f32).to_bits();
                let pos_zero_bits = 0.0f32.to_bits();
                let neg_one_bits = (-1.0f32).to_bits();

                Box::new(
                    vec![
                        nan_bits.to_string(),
                        pos_inf_bits.to_string(),
                        neg_inf_bits.to_string(),
                        neg_zero_bits.to_string(),
                        pos_zero_bits.to_string(),
                        neg_one_bits.to_string(),
                    ]
                    .into_iter(),
                )
            }
        }

        let state = ParseState::<f32>::new();
        let supplier = FloatBitsSupplier;
        let values: Vec<String> = supplier.possible_values(&state).collect();

        assert_eq!(values.len(), 6);
        assert_eq!(values[0], f32::NAN.to_bits().to_string());
        assert_eq!(values[1], f32::INFINITY.to_bits().to_string());
        assert_eq!(values[2], f32::NEG_INFINITY.to_bits().to_string());
        assert_eq!(values[3], (-0.0f32).to_bits().to_string());
        assert_eq!(values[4], 0.0f32.to_bits().to_string());
        assert_eq!(values[5], (-1.0f32).to_bits().to_string());
        assert_ne!(values[3], values[4], "negative and positive zero bits differ");
    }

    #[test]
    fn custom_supplier_f64_float_bit_edge_cases() {
        struct Float64BitsSupplier;

        impl SuggestionSupplier<f64> for Float64BitsSupplier {
            // JAVA: net.minecraft.util.parsing.packrat.SuggestionSupplier#possibleValues
            fn possible_values(
                &self,
                _state: &ParseState<f64>,
            ) -> Box<dyn Iterator<Item = String> + '_> {
                let values = [
                    f64::NAN.to_bits(),
                    f64::INFINITY.to_bits(),
                    f64::NEG_INFINITY.to_bits(),
                    (-0.0f64).to_bits(),
                    0.0f64.to_bits(),
                    f64::MIN.to_bits(),
                    f64::MAX.to_bits(),
                ];
                Box::new(values.into_iter().map(|b| b.to_string()))
            }
        }

        let state = ParseState::<f64>::new();
        let supplier = Float64BitsSupplier;
        let values: Vec<String> = supplier.possible_values(&state).collect();
        assert_eq!(values.len(), 7);
        assert_eq!(values[0], f64::NAN.to_bits().to_string());
        assert_eq!(values[6], f64::MAX.to_bits().to_string());
    }

    #[test]
    fn supplier_can_borrow_self_data() {
        struct BorrowingSupplier {
            prefix: String,
            count: i32,
        }

        impl SuggestionSupplier<i32> for BorrowingSupplier {
            // JAVA: net.minecraft.util.parsing.packrat.SuggestionSupplier#possibleValues
            fn possible_values(
                &self,
                _state: &ParseState<i32>,
            ) -> Box<dyn Iterator<Item = String> + '_> {
                let prefix = self.prefix.clone();
                let count = self.count;
                Box::new((0..count).map(move |i| format!("{prefix}:{i}")))
            }
        }

        let state = ParseState::<i32>::new();
        let supplier = BorrowingSupplier {
            prefix: "s".to_string(),
            count: 3,
        };
        let values: Vec<String> = supplier.possible_values(&state).collect();
        assert_eq!(
            values,
            vec!["s:0".to_string(), "s:1".to_string(), "s:2".to_string()]
        );
    }

    #[test]
    fn empty_supplier_object_safe_trait_usage() {
        let state = ParseState::<i32>::new();
        let supplier = empty::<i32>();
        let dyn_ref: &dyn SuggestionSupplier<i32> = &supplier;
        let mut iter = dyn_ref.possible_values(&state);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn boxed_supplier_object_safe_trait_usage() {
        let state = ParseState::<String>::new();
        let supplier: Box<dyn SuggestionSupplier<String>> = Box::new(empty::<String>());
        assert_eq!(supplier.possible_values(&state).count(), 0);
    }
}
