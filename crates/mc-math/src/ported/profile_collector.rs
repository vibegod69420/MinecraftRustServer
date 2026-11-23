use std::collections::HashSet;
use std::path::Path;

// Minimal self-contained stubs for unported dependencies.
pub trait ProfilerFiller {}

#[derive(Debug, Clone)]
pub struct PathEntry;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricCategory(pub String);

// Minimal self-contained ResultField for standalone compilation.
#[derive(Debug, Clone)]
pub struct ResultField {
    pub percentage: f64,
    pub global_percentage: f64,
    pub count: i64,
    pub name: String,
}

// Minimal self-contained ProfileResults trait for standalone compilation.
pub trait ProfileResults {
    // JAVA: net.minecraft.util.profiling.ProfileResults#getTimes
    fn get_times(&self, path: &str) -> Vec<ResultField>;

    // JAVA: net.minecraft.util.profiling.ProfileResults#saveResults
    fn save_results(&self, path: &Path) -> bool;

    // JAVA: net.minecraft.util.profiling.ProfileResults#getStartTimeNano
    fn get_start_time_nano(&self) -> i64;

    // JAVA: net.minecraft.util.profiling.ProfileResults#getStartTimeTicks
    fn get_start_time_ticks(&self) -> i32;

    // JAVA: net.minecraft.util.profiling.ProfileResults#getEndTimeNano
    fn get_end_time_nano(&self) -> i64;

    // JAVA: net.minecraft.util.profiling.ProfileResults#getEndTimeTicks
    fn get_end_time_ticks(&self) -> i32;

    // JAVA: net.minecraft.util.profiling.ProfileResults#getNanoDuration
    fn get_nano_duration(&self) -> i64 {
        self.get_end_time_nano()
            .wrapping_sub(self.get_start_time_nano())
    }

    // JAVA: net.minecraft.util.profiling.ProfileResults#getTickDuration
    fn get_tick_duration(&self) -> i32 {
        self.get_end_time_ticks()
            .wrapping_sub(self.get_start_time_ticks())
    }

    // JAVA: net.minecraft.util.profiling.ProfileResults#getProfilerResults
    fn get_profiler_results(&self) -> String;
}

// JAVA: net.minecraft.util.profiling.ProfileCollector
pub trait ProfileCollector: ProfilerFiller {
    // JAVA: net.minecraft.util.profiling.ProfileCollector#getResults
    fn get_results(&self) -> Box<dyn ProfileResults + '_>;

    // JAVA: net.minecraft.util.profiling.ProfileCollector#getEntry
    fn get_entry(&self, name: &str) -> Option<PathEntry>;

    // JAVA: net.minecraft.util.profiling.ProfileCollector#getChartedPaths
    fn get_charted_paths(&self) -> HashSet<(String, MetricCategory)>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyResults;

    impl ProfileResults for EmptyResults {
        fn get_times(&self, _path: &str) -> Vec<ResultField> {
            Vec::new()
        }
        fn save_results(&self, _path: &Path) -> bool {
            false
        }
        fn get_start_time_nano(&self) -> i64 {
            0
        }
        fn get_start_time_ticks(&self) -> i32 {
            0
        }
        fn get_end_time_nano(&self) -> i64 {
            0
        }
        fn get_end_time_ticks(&self) -> i32 {
            0
        }
        fn get_profiler_results(&self) -> String {
            String::new()
        }
    }

    struct MockCollector;

    impl ProfilerFiller for MockCollector {}

    impl ProfileCollector for MockCollector {
        fn get_results(&self) -> Box<dyn ProfileResults + '_> {
            Box::new(EmptyResults)
        }

        fn get_entry(&self, name: &str) -> Option<PathEntry> {
            if name == "root" {
                Some(PathEntry)
            } else {
                None
            }
        }

        fn get_charted_paths(&self) -> HashSet<(String, MetricCategory)> {
            let mut set = HashSet::new();
            set.insert(("root".to_string(), MetricCategory("tick".to_string())));
            set
        }
    }

    #[test]
    fn mock_collector_get_entry() {
        let c = MockCollector;
        assert!(c.get_entry("root").is_some());
        assert!(c.get_entry("missing").is_none());
    }

    #[test]
    fn mock_collector_charted_paths() {
        let c = MockCollector;
        let paths = c.get_charted_paths();
        assert_eq!(paths.len(), 1);
        assert!(paths.contains(&("root".to_string(), MetricCategory("tick".to_string()))));
    }

    #[test]
    fn mock_collector_results() {
        let c = MockCollector;
        let r = c.get_results();
        assert_eq!(r.get_profiler_results(), "");
        assert_eq!(r.get_nano_duration(), 0);
        assert_eq!(r.get_tick_duration(), 0);
    }

    #[test]
    fn metric_category_hash_eq() {
        let a = MetricCategory("cpu".to_string());
        let b = MetricCategory("cpu".to_string());
        let c = MetricCategory("mem".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);
        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
    }
}
