use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Key wrapper for descending `i32` order, matching Java's `Comparator.reverseOrder()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DescI32(pub i32);

impl PartialOrd for DescI32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DescI32 {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

/// Reverse-sorted-by-key map, matching Java's
/// `Int2DoubleRBTreeMap(Comparator.reverseOrder())`.
pub type PercentileMap = BTreeMap<DescI32, f64>;

/// Default percentile indexes: 50, 75, 90, 99 (scale 100).
pub const DEFAULT_INDEXES: [i32; 4] = [50, 75, 90, 99];
const SCALE: i64 = 100;

// JAVA: net.minecraft.util.profiling.jfr.Percentiles#evaluate
pub fn evaluate_longs(values: &[i64]) -> PercentileMap {
    if values.is_empty() {
        return BTreeMap::new();
    }
    let mut sorted_vals: Vec<f64> = values.iter().map(|&v| v as f64).collect();
    sorted_vals.sort_by(|a, b| a.total_cmp(b));
    sorted(&compute(&sorted_vals))
}

// JAVA: net.minecraft.util.profiling.jfr.Percentiles#evaluate
pub fn evaluate_ints(values: &[i32]) -> PercentileMap {
    if values.is_empty() {
        return BTreeMap::new();
    }
    let mut sorted_vals: Vec<f64> = values.iter().map(|&v| v as f64).collect();
    sorted_vals.sort_by(|a, b| a.total_cmp(b));
    sorted(&compute(&sorted_vals))
}

// JAVA: net.minecraft.util.profiling.jfr.Percentiles#evaluate
pub fn evaluate_doubles(values: &[f64]) -> PercentileMap {
    if values.is_empty() {
        return BTreeMap::new();
    }
    let mut sorted_vals: Vec<f64> = values.to_vec();
    sorted_vals.sort_by(|a, b| a.total_cmp(b));
    sorted(&compute(&sorted_vals))
}

// JAVA: net.minecraft.util.profiling.jfr.Percentiles#sorted
fn sorted(map: &BTreeMap<i32, f64>) -> PercentileMap {
    let mut out = BTreeMap::new();
    for (&k, &v) in map {
        out.insert(DescI32(k), v);
    }
    out
}

fn compute(sorted_vals: &[f64]) -> BTreeMap<i32, f64> {
    let mut map = BTreeMap::new();
    for &idx in &DEFAULT_INDEXES {
        map.insert(idx, quantile(sorted_vals, idx as i64, SCALE));
    }
    map
}

/// Linear-interpolated quantile, matching Guava Quantiles.scale(scale).indexes(index).
fn quantile(sorted_vals: &[f64], index: i64, scale: i64) -> f64 {
    let n = sorted_vals.len() as i64;
    if n <= 1 {
        return sorted_vals[0];
    }
    let pos = index.wrapping_mul(n.wrapping_sub(1));
    let lo = (pos / scale) as usize;
    let rem = pos % scale;
    if rem == 0 {
        sorted_vals[lo]
    } else {
        let hi = lo + 1;
        let frac = rem as f64 / scale as f64;
        sorted_vals[lo] + (sorted_vals[hi] - sorted_vals[lo]) * frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_return_empty_map() {
        assert!(evaluate_longs(&[]).is_empty());
        assert!(evaluate_ints(&[]).is_empty());
        assert!(evaluate_doubles(&[]).is_empty());
    }

    #[test]
    fn single_element_all_percentiles_same() {
        let m = evaluate_longs(&[42]);
        for &idx in &DEFAULT_INDEXES {
            assert_eq!(m.get(&DescI32(idx)), Some(&42.0));
        }
    }

    #[test]
    fn known_integer_percentiles() {
        let vals: Vec<i64> = (0..=10).collect();
        let m = evaluate_longs(&vals);
        assert_eq!(m.get(&DescI32(50)), Some(&5.0));
        assert_eq!(m.get(&DescI32(75)), Some(&7.5));
        assert_eq!(m.get(&DescI32(90)), Some(&9.0));
        assert!((m.get(&DescI32(99)).unwrap() - 9.9).abs() < 1e-12);
    }

    #[test]
    fn descending_key_iteration_order() {
        let m = evaluate_ints(&[1, 2, 3, 4, 5]);
        let keys: Vec<i32> = m.keys().map(|k| k.0).collect();
        assert_eq!(keys, vec![99, 90, 75, 50]);
    }

    #[test]
    fn negative_and_double_values() {
        let m = evaluate_longs(&[-10, -5, 0, 5, 10]);
        assert_eq!(m.get(&DescI32(50)), Some(&0.0));

        let d = evaluate_doubles(&[1.5, 2.5, 3.5, 4.5]);
        // 50th: pos=50*3=150, lo=1, rem=50 -> 2.5 + (3.5-2.5)*0.5 = 3.0
        assert_eq!(d.get(&DescI32(50)), Some(&3.0));
        // 75th: pos=75*3=225, lo=2, rem=25 -> 3.5 + (4.5-3.5)*0.25 = 3.75
        assert_eq!(d.get(&DescI32(75)), Some(&3.75));
    }
}
