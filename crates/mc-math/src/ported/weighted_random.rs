//! Faithful port of `net.minecraft.util.random.WeightedRandom` (Minecraft Java Edition 1.21.11).
//!
//! Provides weighted random selection from a list of items, matching Java integer
//! semantics and control flow exactly.

/// Minimal random source interface required by weighted random selection.
///
/// Corresponds to the `nextInt(int bound)` method on `net.minecraft.util.RandomSource`.
pub trait RandomSource {
    /// Returns a uniformly distributed random integer in `[0, bound)`.
    ///
    /// Matches Java `RandomSource.nextInt(int bound)`.
    fn next_int(&mut self, bound: i32) -> i32;
}

impl<F: FnMut(i32) -> i32 + ?Sized> RandomSource for F {
    fn next_int(&mut self, bound: i32) -> i32 {
        self(bound)
    }
}

// JAVA: net.minecraft.util.random.WeightedRandom#getTotalWeight
pub fn get_total_weight<T>(items: &[T], weight_fn: impl Fn(&T) -> i32) -> i32 {
    let mut total: i64 = 0i64;
    for item in items {
        total = total.wrapping_add(i64::from(weight_fn(item)));
    }
    if total > 2_147_483_647_i64 {
        panic!("Sum of weights must be <= 2147483647");
    }
    total as i32
}

// JAVA: net.minecraft.util.random.WeightedRandom#getRandomItem
pub fn get_random_item_with_total<'a, T, R: RandomSource + ?Sized>(
    random: &mut R,
    items: &'a [T],
    total_weight: i32,
    weight_fn: impl Fn(&T) -> i32,
) -> Option<&'a T> {
    if total_weight < 0 {
        // JAVA: Util.pauseInIde(...) is an IDE debug hook; observable behavior is throwing.
        panic!("Negative total weight in getRandomItem");
    }
    if total_weight == 0 {
        return None;
    }
    let value: i32 = random.next_int(total_weight);
    get_weighted_item(items, value, weight_fn)
}

// JAVA: net.minecraft.util.random.WeightedRandom#getWeightedItem
pub fn get_weighted_item<'a, T>(
    items: &'a [T],
    mut value: i32,
    weight_fn: impl Fn(&T) -> i32,
) -> Option<&'a T> {
    for item in items {
        value = value.wrapping_sub(weight_fn(item));
        if value < 0 {
            return Some(item);
        }
    }
    None
}

// JAVA: net.minecraft.util.random.WeightedRandom#getRandomItem
pub fn get_random_item<'a, T, R: RandomSource + ?Sized>(
    random: &mut R,
    items: &'a [T],
    weight_fn: impl Fn(&T) -> i32,
) -> Option<&'a T> {
    let total: i32 = get_total_weight(items, &weight_fn);
    get_random_item_with_total(random, items, total, &weight_fn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FixedRandom {
        value: i32,
    }

    impl RandomSource for FixedRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            self.value
        }
    }

    #[derive(Debug)]
    struct BoundCheckingRandom {
        expected_bound: i32,
        return_value: i32,
        called: bool,
    }

    impl BoundCheckingRandom {
        fn new(expected_bound: i32, return_value: i32) -> Self {
            Self {
                expected_bound,
                return_value,
                called: false,
            }
        }
    }

    impl RandomSource for BoundCheckingRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            assert_eq!(bound, self.expected_bound);
            self.called = true;
            self.return_value
        }
    }

    // ---- get_total_weight ----

    #[test]
    fn total_weight_empty() {
        let items: Vec<i32> = Vec::new();
        assert_eq!(get_total_weight(&items, |i| *i), 0);
    }

    #[test]
    fn total_weight_single() {
        let items = vec![42i32];
        assert_eq!(get_total_weight(&items, |i| *i), 42);
    }

    #[test]
    fn total_weight_multiple() {
        let items = vec![1i32, 2, 3, 4, 5];
        assert_eq!(get_total_weight(&items, |i| *i), 15);
    }

    #[test]
    fn total_weight_at_i32_max() {
        let items = vec![i32::MAX];
        assert_eq!(get_total_weight(&items, |i| *i), i32::MAX);
    }

    #[test]
    fn total_weight_exactly_i32_max_split() {
        let items = vec![i32::MAX - 1, 1];
        assert_eq!(get_total_weight(&items, |i| *i), i32::MAX);
    }

    #[test]
    #[should_panic(expected = "Sum of weights must be <= 2147483647")]
    fn total_weight_exceeds_i32_max_panics() {
        let items = vec![i32::MAX, 1];
        let _ = get_total_weight(&items, |i| *i);
    }

    #[test]
    #[should_panic(expected = "Sum of weights must be <= 2147483647")]
    fn total_weight_large_sum_panics() {
        let items = vec![1_000_000_000i32, 1_000_000_000, 1_000_000_000];
        let _ = get_total_weight(&items, |i| *i);
    }

    #[test]
    fn total_weight_negative_weights() {
        let items = vec![10i32, -3, 5];
        assert_eq!(get_total_weight(&items, |i| *i), 12);
    }

    #[test]
    fn total_weight_all_negative() {
        let items = vec![-5i32, -3];
        assert_eq!(get_total_weight(&items, |i| *i), -8);
    }

    #[test]
    fn total_weight_i32_min() {
        let items = vec![i32::MIN, 1];
        assert_eq!(get_total_weight(&items, |i| *i), i32::MIN.wrapping_add(1));
    }

    #[test]
    fn total_weight_zero_weights() {
        let items = vec![0i32, 0, 0];
        assert_eq!(get_total_weight(&items, |i| *i), 0);
    }

    #[test]
    fn total_weight_uses_wrapping_i64_accumulation() {
        // Accumulation is performed as Java long (i64) with wrapping semantics.
        // With i32 weights, the sum check catches overflow before i64 wrap in practice,
        // but we verify correct behavior for large valid sums.
        let items = vec![1_073_741_823i32, 1_073_741_823]; // (2^30 - 1) * 2 = 2^31 - 2
        assert_eq!(get_total_weight(&items, |i| *i), 2_147_483_646);
    }

    // ---- get_weighted_item ----

    #[test]
    fn weighted_item_selects_first() {
        let items = vec![("a", 5i32), ("b", 3), ("c", 2)];
        let result = get_weighted_item(&items, 0, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("a"));
    }

    #[test]
    fn weighted_item_selects_middle() {
        let items = vec![("a", 5i32), ("b", 3), ("c", 2)];
        // 5 - 5 = 0 (not < 0), 0 - 3 = -3 (< 0)
        let result = get_weighted_item(&items, 5, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("b"));
    }

    #[test]
    fn weighted_item_selects_last() {
        let items = vec![("a", 5i32), ("b", 3), ("c", 2)];
        // 9-5=4, 4-3=1, 1-2=-1
        let result = get_weighted_item(&items, 9, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("c"));
    }

    #[test]
    fn weighted_item_boundary_values() {
        let items = vec![("a", 5i32), ("b", 3)];
        assert_eq!(
            get_weighted_item(&items, 4, |(_, w)| *w).map(|(s, _)| *s),
            Some("a")
        );
        assert_eq!(
            get_weighted_item(&items, 5, |(_, w)| *w).map(|(s, _)| *s),
            Some("b")
        );
        assert_eq!(
            get_weighted_item(&items, 7, |(_, w)| *w).map(|(s, _)| *s),
            Some("b")
        );
    }

    #[test]
    fn weighted_item_exhausted_returns_none() {
        let items = vec![("a", 5i32), ("b", 3)];
        // 8-5=3, 3-3=0 -> None
        let result = get_weighted_item(&items, 8, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn weighted_item_value_too_large_returns_none() {
        let items = vec![("a", 1i32)];
        let result = get_weighted_item(&items, 100, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn weighted_item_empty_returns_none() {
        let items: Vec<(i32, i32)> = Vec::new();
        let result = get_weighted_item(&items, 0, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn weighted_item_zero_weight_skipped() {
        let items = vec![("a", 0i32), ("b", 5)];
        // 0-0=0 (not < 0), 0-5=-5 -> b
        let result = get_weighted_item(&items, 0, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("b"));
    }

    #[test]
    fn weighted_item_all_zero_weights_returns_none() {
        let items = vec![("a", 0i32), ("b", 0)];
        let result = get_weighted_item(&items, 0, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn weighted_item_negative_weight() {
        let items = vec![("a", -5i32), ("b", 10)];
        // 0.wrapping_sub(-5) = 5, not < 0; 5.wrapping_sub(10) = -5 -> b
        let result = get_weighted_item(&items, 0, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("b"));
    }

    #[test]
    fn weighted_item_wrapping_sub_i32_min_value() {
        let items = vec![("a", 1i32), ("b", 1)];
        // i32::MIN.wrapping_sub(1) = i32::MAX, not < 0
        // i32::MAX.wrapping_sub(1) = i32::MAX - 1, not < 0
        let result = get_weighted_item(&items, i32::MIN, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn weighted_item_wrapping_sub_i32_min_weight() {
        let items = vec![("a", i32::MIN)];
        // 0.wrapping_sub(i32::MIN) wraps in two's complement to i32::MIN (< 0)
        let expected = 0i32.wrapping_sub(i32::MIN);
        assert!(expected < 0);
        let result = get_weighted_item(&items, 0, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("a"));
    }

    #[test]
    fn weighted_item_wrapping_sub_large_values() {
        let items = vec![("a", i32::MAX), ("b", i32::MAX)];
        // i32::MAX.wrapping_sub(i32::MAX) = 0, not < 0
        // 0.wrapping_sub(i32::MAX) = -2147483647 < 0 -> b
        let result = get_weighted_item(&items, i32::MAX, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("b"));
    }

    // ---- get_random_item_with_total (4-arg overload) ----

    #[test]
    #[should_panic(expected = "Negative total weight in getRandomItem")]
    fn random_item_with_total_negative_panics() {
        let mut rng = FixedRandom { value: 0 };
        let items = vec![("a", 1i32)];
        let _ = get_random_item_with_total(&mut rng, &items, -1, |(_, w)| *w);
    }

    #[test]
    #[should_panic(expected = "Negative total weight in getRandomItem")]
    fn random_item_with_total_i32_min_panics() {
        let mut rng = FixedRandom { value: 0 };
        let items = vec![("a", 1i32)];
        let _ = get_random_item_with_total(&mut rng, &items, i32::MIN, |(_, w)| *w);
    }

    #[test]
    fn random_item_with_total_zero_returns_none() {
        let mut rng = FixedRandom { value: 0 };
        let items = vec![("a", 5i32)];
        let result = get_random_item_with_total(&mut rng, &items, 0, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn random_item_with_total_zero_does_not_call_next_int() {
        struct PanicRandom;
        impl RandomSource for PanicRandom {
            fn next_int(&mut self, _bound: i32) -> i32 {
                panic!("next_int should not be called when total_weight == 0");
            }
        }
        let mut rng = PanicRandom;
        let items = vec![("a", 5i32)];
        let result = get_random_item_with_total(&mut rng, &items, 0, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn random_item_with_total_selects_correctly() {
        let items = vec![("a", 5i32), ("b", 3), ("c", 2)];
        let total = 10i32;

        for (roll, expected) in [(0, "a"), (4, "a"), (5, "b"), (7, "b"), (8, "c"), (9, "c")] {
            let mut rng = FixedRandom { value: roll };
            let result = get_random_item_with_total(&mut rng, &items, total, |(_, w)| *w);
            assert_eq!(
                result.map(|(s, _)| *s),
                Some(expected),
                "roll={roll} should select {expected}"
            );
        }
    }

    #[test]
    fn random_item_with_total_passes_correct_bound() {
        let items = vec![("a", 3i32), ("b", 7)];
        let mut rng = BoundCheckingRandom::new(10, 5);
        let result = get_random_item_with_total(&mut rng, &items, 10, |(_, w)| *w);
        assert!(rng.called);
        assert_eq!(result.map(|(s, _)| *s), Some("b"));
    }

    #[test]
    fn random_item_with_total_single_item() {
        let items = vec![("only", 10i32)];
        let mut rng = FixedRandom { value: 0 };
        assert_eq!(
            get_random_item_with_total(&mut rng, &items, 10, |(_, w)| *w).map(|(s, _)| *s),
            Some("only")
        );
        let mut rng = FixedRandom { value: 9 };
        assert_eq!(
            get_random_item_with_total(&mut rng, &items, 10, |(_, w)| *w).map(|(s, _)| *s),
            Some("only")
        );
    }

    #[test]
    fn random_item_with_total_empty_items_zero_total() {
        let items: Vec<(i32, i32)> = Vec::new();
        let mut rng = FixedRandom { value: 0 };
        let result = get_random_item_with_total(&mut rng, &items, 0, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn random_item_with_total_out_of_range_next_int_returns_none() {
        let items = vec![("a", 5i32), ("b", 5)];
        let mut rng = FixedRandom { value: 100 };
        // 100-5=95, 95-5=90 -> None (faithful: no extra validation)
        let result = get_random_item_with_total(&mut rng, &items, 10, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn random_item_with_total_negative_next_int() {
        let items = vec![("a", 5i32), ("b", 5)];
        let mut rng = FixedRandom { value: -1 };
        // -1 - 5 = -6 < 0 -> "a" (faithful: no extra validation)
        let result = get_random_item_with_total(&mut rng, &items, 10, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("a"));
    }

    // ---- get_random_item (3-arg overload, computes total weight) ----

    #[test]
    fn random_item_basic() {
        let items = vec![("a", 1i32), ("b", 1), ("c", 1)];
        let mut rng = FixedRandom { value: 1 };
        let result = get_random_item(&mut rng, &items, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("b"));
    }

    #[test]
    fn random_item_empty() {
        let items: Vec<(i32, i32)> = Vec::new();
        let mut rng = FixedRandom { value: 0 };
        let result = get_random_item(&mut rng, &items, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    #[should_panic(expected = "Sum of weights must be <= 2147483647")]
    fn random_item_total_overflow_panics() {
        let items = vec![("a", i32::MAX), ("b", 1i32)];
        let mut rng = FixedRandom { value: 0 };
        let _ = get_random_item(&mut rng, &items, |(_, w)| *w);
    }

    #[test]
    #[should_panic(expected = "Negative total weight in getRandomItem")]
    fn random_item_negative_computed_total_panics() {
        let items = vec![("a", -5i32), ("b", -3)];
        let mut rng = FixedRandom { value: 0 };
        let _ = get_random_item(&mut rng, &items, |(_, w)| *w);
    }

    #[test]
    fn random_item_single() {
        let items = vec![("only", 7i32)];
        let mut rng = FixedRandom { value: 3 };
        let result = get_random_item(&mut rng, &items, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("only"));
    }

    #[test]
    fn random_item_uses_computed_total_as_bound() {
        let items = vec![("a", 2i32), ("b", 3), ("c", 5)];
        let mut rng = BoundCheckingRandom::new(10, 4);
        let result = get_random_item(&mut rng, &items, |(_, w)| *w);
        assert!(rng.called);
        assert_eq!(result.map(|(s, _)| *s), Some("b"));
    }

    #[test]
    fn random_item_all_zero_weights_returns_none() {
        let items = vec![("a", 0i32), ("b", 0), ("c", 0)];
        let mut rng = FixedRandom { value: 0 };
        let result = get_random_item(&mut rng, &items, |(_, w)| *w);
        assert_eq!(result, None);
    }

    #[test]
    fn random_item_with_fn_closure_as_random_source() {
        // Verifies the blanket impl for FnMut(i32) -> i32 works as a RandomSource.
        let items = vec![("a", 1i32), ("b", 2), ("c", 3)];
        let mut rng = |_bound: i32| 2i32;
        let result = get_random_item_with_total(&mut rng, &items, 6, |(_, w)| *w);
        assert_eq!(result.map(|(s, _)| *s), Some("b"));
    }
}
