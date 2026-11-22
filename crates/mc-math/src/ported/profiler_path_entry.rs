//! Faithful port of `net.minecraft.util.profiling.ProfilerPathEntry`.
//!
//! Source: `net/minecraft/util/profiling/ProfilerPathEntry.java` (Minecraft Java Edition 1.21.11)

use std::collections::HashMap;

/// A profiler path entry exposing timing and counter metrics.
///
/// Faithful port of the Java interface
/// `net.minecraft.util.profiling.ProfilerPathEntry`.
pub trait ProfilerPathEntry {
    // JAVA: net.minecraft.util.profiling.ProfilerPathEntry#getDuration
    fn get_duration(&self) -> i64;

    // JAVA: net.minecraft.util.profiling.ProfilerPathEntry#getMaxDuration
    fn get_max_duration(&self) -> i64;

    // JAVA: net.minecraft.util.profiling.ProfilerPathEntry#getCount
    fn get_count(&self) -> i64;

    // JAVA: net.minecraft.util.profiling.ProfilerPathEntry#getCounters
    fn get_counters(&self) -> &HashMap<String, i64>;
}

/// Simple concrete implementation of [`ProfilerPathEntry`].
///
/// Stores duration, max duration, count, and named counters as plain fields.
/// This is a self-contained data holder for testing and standalone use;
/// the Java source defines only the interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleProfilerPathEntry {
    duration: i64,
    max_duration: i64,
    count: i64,
    counters: HashMap<String, i64>,
}

impl SimpleProfilerPathEntry {
    /// Creates a new `SimpleProfilerPathEntry` with the given values.
    pub fn new(
        duration: i64,
        max_duration: i64,
        count: i64,
        counters: HashMap<String, i64>,
    ) -> Self {
        Self {
            duration,
            max_duration,
            count,
            counters,
        }
    }

    /// Returns a mutable reference to the counters map.
    pub fn counters_mut(&mut self) -> &mut HashMap<String, i64> {
        &mut self.counters
    }
}

impl Default for SimpleProfilerPathEntry {
    fn default() -> Self {
        Self {
            duration: 0,
            max_duration: 0,
            count: 0,
            counters: HashMap::new(),
        }
    }
}

impl ProfilerPathEntry for SimpleProfilerPathEntry {
    // JAVA: net.minecraft.util.profiling.ProfilerPathEntry#getDuration
    fn get_duration(&self) -> i64 {
        self.duration
    }

    // JAVA: net.minecraft.util.profiling.ProfilerPathEntry#getMaxDuration
    fn get_max_duration(&self) -> i64 {
        self.max_duration
    }

    // JAVA: net.minecraft.util.profiling.ProfilerPathEntry#getCount
    fn get_count(&self) -> i64 {
        self.count
    }

    // JAVA: net.minecraft.util.profiling.ProfilerPathEntry#getCounters
    fn get_counters(&self) -> &HashMap<String, i64> {
        &self.counters
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_entry(
        duration: i64,
        max_duration: i64,
        count: i64,
        counters: HashMap<String, i64>,
    ) -> SimpleProfilerPathEntry {
        SimpleProfilerPathEntry::new(duration, max_duration, count, counters)
    }

    #[test]
    fn test_basic_values() {
        let mut counters = HashMap::new();
        counters.insert("tick".to_string(), 100);
        counters.insert("render".to_string(), 200);

        let entry = make_entry(1_000, 2_000, 5, counters.clone());

        assert_eq!(entry.get_duration(), 1_000);
        assert_eq!(entry.get_max_duration(), 2_000);
        assert_eq!(entry.get_count(), 5);
        assert_eq!(entry.get_counters(), &counters);
        assert_eq!(entry.get_counters().len(), 2);
    }

    #[test]
    fn test_negative_values() {
        let mut counters = HashMap::new();
        counters.insert("min".to_string(), i64::MIN);
        counters.insert("neg_one".to_string(), -1);
        counters.insert("neg_large".to_string(), i64::MIN + 1);

        let entry = make_entry(i64::MIN, -1, -100, counters);

        assert_eq!(entry.get_duration(), i64::MIN);
        assert_eq!(entry.get_max_duration(), -1);
        assert_eq!(entry.get_count(), -100);
        assert_eq!(entry.get_counters().get("min").copied(), Some(i64::MIN));
        assert_eq!(entry.get_counters().get("neg_one").copied(), Some(-1));
        assert_eq!(
            entry.get_counters().get("neg_large").copied(),
            Some(i64::MIN + 1)
        );
    }

    #[test]
    fn test_overflow_edge_values() {
        let mut counters = HashMap::new();
        counters.insert("max".to_string(), i64::MAX);
        counters.insert("min".to_string(), i64::MIN);
        counters.insert("zero".to_string(), 0);

        let entry = make_entry(i64::MAX, i64::MAX, i64::MAX, counters);

        assert_eq!(entry.get_duration(), i64::MAX);
        assert_eq!(entry.get_max_duration(), i64::MAX);
        assert_eq!(entry.get_count(), i64::MAX);
        assert_eq!(entry.get_counters().get("max").copied(), Some(i64::MAX));
        assert_eq!(entry.get_counters().get("min").copied(), Some(i64::MIN));
        assert_eq!(entry.get_counters().get("zero").copied(), Some(0));
    }

    #[test]
    fn test_min_max_boundaries() {
        let mut counters = HashMap::new();
        counters.insert("i64_min".to_string(), i64::MIN);
        counters.insert("i64_max".to_string(), i64::MAX);

        let entry = make_entry(i64::MIN, i64::MAX, i64::MIN, counters);

        assert_eq!(entry.get_duration(), i64::MIN);
        assert_eq!(entry.get_max_duration(), i64::MAX);
        assert_eq!(entry.get_count(), i64::MIN);

        let c = entry.get_counters();
        assert_eq!(c.get("i64_min").copied(), Some(i64::MIN));
        assert_eq!(c.get("i64_max").copied(), Some(i64::MAX));
    }

    #[test]
    fn test_empty_counters() {
        let entry = make_entry(0, 0, 0, HashMap::new());

        assert_eq!(entry.get_duration(), 0);
        assert_eq!(entry.get_max_duration(), 0);
        assert_eq!(entry.get_count(), 0);
        assert!(entry.get_counters().is_empty());
    }

    #[test]
    fn test_missing_counter_returns_none() {
        let entry = make_entry(1, 2, 3, HashMap::new());
        assert_eq!(entry.get_counters().get("nonexistent").copied(), None);
    }

    #[test]
    fn test_trait_object_usage() {
        let mut counters = HashMap::new();
        counters.insert("a".to_string(), 42);
        let entry = make_entry(10, 20, 3, counters);

        let obj: &dyn ProfilerPathEntry = &entry;
        assert_eq!(obj.get_duration(), 10);
        assert_eq!(obj.get_max_duration(), 20);
        assert_eq!(obj.get_count(), 3);
        assert_eq!(obj.get_counters().get("a").copied(), Some(42));
    }

    #[test]
    fn test_generic_trait_bound() {
        fn read_entry<E: ProfilerPathEntry>(e: &E) -> (i64, i64, i64, usize) {
            (
                e.get_duration(),
                e.get_max_duration(),
                e.get_count(),
                e.get_counters().len(),
            )
        }

        let mut counters = HashMap::new();
        counters.insert("x".to_string(), -999);
        let entry = make_entry(-10, -5, -1, counters);

        assert_eq!(read_entry(&entry), (-10, -5, -1, 1));
    }

    #[test]
    fn test_counters_mut_modification() {
        let mut counters = HashMap::new();
        counters.insert("initial".to_string(), 1);
        let mut entry = make_entry(0, 0, 0, counters);

        entry.counters_mut().insert("added".to_string(), i64::MIN);
        entry.counters_mut().insert("initial".to_string(), i64::MAX);

        assert_eq!(entry.get_counters().get("added").copied(), Some(i64::MIN));
        assert_eq!(
            entry.get_counters().get("initial").copied(),
            Some(i64::MAX)
        );
        assert_eq!(entry.get_counters().len(), 2);
    }

    #[test]
    fn test_clone_and_equality() {
        let mut counters = HashMap::new();
        counters.insert("k".to_string(), i64::MIN);
        let entry = make_entry(i64::MIN, i64::MAX, 0, counters);
        let cloned = entry.clone();

        assert_eq!(entry, cloned);
        assert_eq!(cloned.get_duration(), i64::MIN);
        assert_eq!(cloned.get_counters().get("k").copied(), Some(i64::MIN));
    }

    #[test]
    fn test_default() {
        let entry = SimpleProfilerPathEntry::default();
        assert_eq!(entry.get_duration(), 0);
        assert_eq!(entry.get_max_duration(), 0);
        assert_eq!(entry.get_count(), 0);
        assert!(entry.get_counters().is_empty());
    }

    #[test]
    fn test_large_counter_map() {
        let mut counters = HashMap::new();
        for i in 0..256i64 {
            counters.insert(format!("counter_{i}"), i.wrapping_sub(128));
        }
        let entry = make_entry(1, 2, 256, counters);

        assert_eq!(entry.get_counters().len(), 256);
        for i in 0..256i64 {
            let key = format!("counter_{i}");
            assert_eq!(
                entry.get_counters().get(&key).copied(),
                Some(i.wrapping_sub(128))
            );
        }
    }
}
