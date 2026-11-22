//! Faithful port of `net.minecraft.util.parsing.packrat.ErrorEntry` (Minecraft Java 1.21.11).
//!
//! This module is self-contained: `SuggestionSupplier<S>` is provided as a minimal
//! local stand-in for the unported Java functional interface, and the Java `Object reason`
//! component is represented as a generic `R` so callers can choose an appropriate type.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Minimal self-contained stand-in for
/// `net.minecraft.util.parsing.packrat.SuggestionSupplier<S>`.
///
/// The real Java type is a functional interface used by the packrat parser.
/// This port models it as a shareable callback. Equality and hashing use
/// callback pointer identity, matching typical lambda identity semantics.
pub struct SuggestionSupplier<S> {
    callback: Arc<dyn Fn(&S) -> Vec<String> + Send + Sync>,
}

impl<S> SuggestionSupplier<S> {
    /// Creates a new suggestion supplier from a callback.
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(&S) -> Vec<String> + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(callback),
        }
    }

    /// Creates an empty suggestion supplier that always returns no suggestions.
    pub fn empty() -> Self {
        Self::new(|_| Vec::new())
    }

    /// Invokes the underlying suggestion callback.
    pub fn suggest(&self, state: &S) -> Vec<String> {
        (self.callback)(state)
    }
}

impl<S> Clone for SuggestionSupplier<S> {
    fn clone(&self) -> Self {
        Self {
            callback: Arc::clone(&self.callback),
        }
    }
}

impl<S> fmt::Debug for SuggestionSupplier<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SuggestionSupplier(..)")
    }
}

impl<S> PartialEq for SuggestionSupplier<S> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

impl<S> Eq for SuggestionSupplier<S> {}

impl<S> Hash for SuggestionSupplier<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.callback).hash(state);
    }
}

/// Port of Java record `net.minecraft.util.parsing.packrat.ErrorEntry<S>`.
///
/// Java components:
/// - `int cursor`
/// - `SuggestionSupplier<S> suggestions`
/// - `Object reason`
///
/// The `reason` component is represented as generic `R` for a self-contained,
/// type-safe Rust port while preserving the record's component semantics.
pub struct ErrorEntry<S, R = String> {
    cursor: i32,
    suggestions: SuggestionSupplier<S>,
    reason: R,
}

impl<S, R> ErrorEntry<S, R> {
    // JAVA: net.minecraft.util.parsing.packrat.ErrorEntry#<init>
    pub fn new(cursor: i32, suggestions: SuggestionSupplier<S>, reason: R) -> Self {
        Self {
            cursor,
            suggestions,
            reason,
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.ErrorEntry#cursor
    pub fn cursor(&self) -> i32 {
        self.cursor
    }

    // JAVA: net.minecraft.util.parsing.packrat.ErrorEntry#suggestions
    pub fn suggestions(&self) -> &SuggestionSupplier<S> {
        &self.suggestions
    }

    // JAVA: net.minecraft.util.parsing.packrat.ErrorEntry#reason
    pub fn reason(&self) -> &R {
        &self.reason
    }
}

impl<S, R: Clone> Clone for ErrorEntry<S, R> {
    fn clone(&self) -> Self {
        Self {
            cursor: self.cursor,
            suggestions: self.suggestions.clone(),
            reason: self.reason.clone(),
        }
    }
}

impl<S, R: fmt::Debug> fmt::Debug for ErrorEntry<S, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorEntry")
            .field("cursor", &self.cursor)
            .field("suggestions", &self.suggestions)
            .field("reason", &self.reason)
            .finish()
    }
}

impl<S, R: PartialEq> PartialEq for ErrorEntry<S, R> {
    // JAVA: net.minecraft.util.parsing.packrat.ErrorEntry#equals
    fn eq(&self, other: &Self) -> bool {
        self.cursor == other.cursor
            && self.suggestions == other.suggestions
            && self.reason == other.reason
    }
}

impl<S, R: Eq> Eq for ErrorEntry<S, R> {}

impl<S, R: Hash> Hash for ErrorEntry<S, R> {
    // JAVA: net.minecraft.util.parsing.packrat.ErrorEntry#hashCode
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cursor.hash(state);
        self.suggestions.hash(state);
        self.reason.hash(state);
    }
}

impl<S, R: fmt::Debug> fmt::Display for ErrorEntry<S, R> {
    // JAVA: net.minecraft.util.parsing.packrat.ErrorEntry#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ErrorEntry[cursor={}, suggestions={:?}, reason={:?}]",
            self.cursor, self.suggestions, self.reason
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn constructor_and_accessors() {
        let suggestions = SuggestionSupplier::<()>::empty();
        let entry = ErrorEntry::new(42, suggestions, "bad token".to_string());
        assert_eq!(entry.cursor(), 42);
        assert_eq!(entry.reason(), "bad token");
        assert!(entry.suggestions().suggest(&()).is_empty());
    }

    #[test]
    fn cursor_negative_and_extremes() {
        let entry_min = ErrorEntry::new(i32::MIN, SuggestionSupplier::<()>::empty(), ());
        let entry_max = ErrorEntry::new(i32::MAX, SuggestionSupplier::<()>::empty(), ());
        let entry_neg = ErrorEntry::new(-1, SuggestionSupplier::<()>::empty(), ());
        let entry_zero = ErrorEntry::new(0, SuggestionSupplier::<()>::empty(), ());

        assert_eq!(entry_min.cursor(), i32::MIN);
        assert_eq!(entry_max.cursor(), i32::MAX);
        assert_eq!(entry_neg.cursor(), -1);
        assert_eq!(entry_zero.cursor(), 0);
    }

    #[test]
    fn cursor_wrapping_add_overflow_boundary() {
        // Java int arithmetic wraps; verify cursor preserves wrapped i32 values.
        let cursor = i32::MAX.wrapping_add(1);
        let entry = ErrorEntry::new(cursor, SuggestionSupplier::<()>::empty(), "overflow");
        assert_eq!(entry.cursor(), i32::MIN);

        let cursor_neg = i32::MIN.wrapping_sub(1);
        let entry_neg = ErrorEntry::new(cursor_neg, SuggestionSupplier::<()>::empty(), "underflow");
        assert_eq!(entry_neg.cursor(), i32::MAX);
    }

    #[test]
    fn cursor_boundary_round_trip() {
        let values = [
            i32::MIN,
            i32::MIN.wrapping_add(1),
            -1024,
            -1,
            0,
            1,
            1024,
            i32::MAX.wrapping_sub(1),
            i32::MAX,
        ];
        for &value in &values {
            let entry = ErrorEntry::new(value, SuggestionSupplier::<()>::empty(), value);
            assert_eq!(entry.cursor(), value);
            assert_eq!(*entry.reason(), value);
        }
    }

    #[test]
    fn equals_same_components() {
        let sug = SuggestionSupplier::<()>::empty();
        let a = ErrorEntry::new(10, sug.clone(), "reason".to_string());
        let b = ErrorEntry::new(10, sug, "reason".to_string());
        assert_eq!(a, b);
    }

    #[test]
    fn equals_different_cursor() {
        let sug = SuggestionSupplier::<()>::empty();
        let a = ErrorEntry::new(10, sug.clone(), "reason");
        let b = ErrorEntry::new(11, sug, "reason");
        assert_ne!(a, b);
    }

    #[test]
    fn equals_different_reason() {
        let sug = SuggestionSupplier::<()>::empty();
        let a = ErrorEntry::new(10, sug.clone(), "a");
        let b = ErrorEntry::new(10, sug, "b");
        assert_ne!(a, b);
    }

    #[test]
    fn equals_different_suggestions_pointer_identity() {
        // SuggestionSupplier equality is pointer identity for the wrapped callback.
        let a = ErrorEntry::new(10, SuggestionSupplier::<()>::empty(), "r");
        let b = ErrorEntry::new(10, SuggestionSupplier::<()>::empty(), "r");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_consistent_with_equals() {
        let sug = SuggestionSupplier::<()>::empty();
        let a = ErrorEntry::new(7, sug.clone(), "x".to_string());
        let b = ErrorEntry::new(7, sug, "x".to_string());
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn hash_extreme_cursor_values() {
        let sug = SuggestionSupplier::<()>::empty();
        let min_entry = ErrorEntry::new(i32::MIN, sug.clone(), 0_i32);
        let max_entry = ErrorEntry::new(i32::MAX, sug, 0_i32);
        assert_ne!(min_entry, max_entry);
        // Hashes should be computable for extreme values.
        let _ = hash_of(&min_entry);
        let _ = hash_of(&max_entry);
    }

    #[test]
    fn to_string_format() {
        let sug = SuggestionSupplier::<()>::empty();
        let entry = ErrorEntry::new(5, sug, "oops");
        let s = entry.to_string();
        assert!(s.starts_with("ErrorEntry["));
        assert!(s.contains("cursor=5"));
        assert!(s.contains("suggestions=SuggestionSupplier(..)"));
        assert!(s.contains("reason="));
        assert!(s.ends_with(']'));
    }

    #[test]
    fn to_string_negative_cursor() {
        let entry = ErrorEntry::new(-999, SuggestionSupplier::<()>::empty(), "err");
        let s = entry.to_string();
        assert!(s.contains("cursor=-999"));
    }

    #[test]
    fn reason_float_bits_f32() {
        let nan_bits: u32 = 0xFFC0_0001;
        let nan = f32::from_bits(nan_bits);
        let entry_bits = ErrorEntry::new(0, SuggestionSupplier::<()>::empty(), nan.to_bits());
        assert_eq!(*entry_bits.reason(), nan_bits);

        let entry_f = ErrorEntry::new(-1, SuggestionSupplier::<()>::empty(), nan);
        assert_eq!(entry_f.reason().to_bits(), nan_bits);

        let neg_zero = f32::from_bits(0x8000_0000);
        let pos_zero = f32::from_bits(0x0000_0000);
        let neg_entry = ErrorEntry::new(1, SuggestionSupplier::<()>::empty(), neg_zero.to_bits());
        let pos_entry = ErrorEntry::new(1, SuggestionSupplier::<()>::empty(), pos_zero.to_bits());
        assert_ne!(neg_entry.reason(), pos_entry.reason());
        assert_eq!(*neg_entry.reason(), 0x8000_0000);
        assert_eq!(*pos_entry.reason(), 0x0000_0000);
    }

    #[test]
    fn reason_float_bits_f64() {
        let nan_bits: u64 = 0x7FF8_0000_0000_0001;
        let nan = f64::from_bits(nan_bits);
        let entry_bits = ErrorEntry::new(i32::MIN, SuggestionSupplier::<()>::empty(), nan.to_bits());
        assert_eq!(*entry_bits.reason(), nan_bits);

        let entry_f = ErrorEntry::new(i32::MAX, SuggestionSupplier::<()>::empty(), nan);
        assert_eq!(entry_f.reason().to_bits(), nan_bits);

        let neg_inf_bits: u64 = 0xFFF0_0000_0000_0000;
        let pos_inf_bits: u64 = 0x7FF0_0000_0000_0000;
        let neg_inf = ErrorEntry::new(0, SuggestionSupplier::<()>::empty(), f64::from_bits(neg_inf_bits));
        let pos_inf = ErrorEntry::new(0, SuggestionSupplier::<()>::empty(), f64::from_bits(pos_inf_bits));
        assert_eq!(neg_inf.reason().to_bits(), neg_inf_bits);
        assert_eq!(pos_inf.reason().to_bits(), pos_inf_bits);
        assert_ne!(neg_inf.reason().to_bits(), pos_inf.reason().to_bits());
    }

    #[test]
    fn suggestion_supplier_callback() {
        let sug = SuggestionSupplier::<i32>::new(|state| vec![format!("s{}", state)]);
        assert_eq!(sug.suggest(&3), vec!["s3".to_string()]);
        assert_eq!(sug.suggest(&-7), vec!["s-7".to_string()]);
    }

    #[test]
    fn suggestion_supplier_clone_shares_identity() {
        let sug = SuggestionSupplier::<()>::new(|_| vec!["a".to_string()]);
        let cloned = sug.clone();
        assert_eq!(sug, cloned);
        assert_eq!(hash_of(&sug), hash_of(&cloned));
        assert_eq!(cloned.suggest(&()), vec!["a".to_string()]);
    }

    #[test]
    fn clone_entry_preserves_equality() {
        let sug = SuggestionSupplier::<()>::empty();
        let entry = ErrorEntry::new(1, sug, String::from("abc"));
        let cloned = entry.clone();
        assert_eq!(entry.cursor(), cloned.cursor());
        assert_eq!(entry.reason(), cloned.reason());
        assert_eq!(entry, cloned);
    }

    #[test]
    fn debug_format_contains_fields() {
        let entry = ErrorEntry::new(123, SuggestionSupplier::<()>::empty(), "detail");
        let dbg = format!("{:?}", entry);
        assert!(dbg.contains("ErrorEntry"));
        assert!(dbg.contains("cursor"));
        assert!(dbg.contains("123"));
        assert!(dbg.contains("suggestions"));
        assert!(dbg.contains("reason"));
    }
}
