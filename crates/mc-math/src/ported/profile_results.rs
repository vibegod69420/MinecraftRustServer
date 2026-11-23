use std::path::Path;

pub const PATH_SEPARATOR: char = '\u{001e}';

// Minimal self-contained ResultField for standalone compilation.
#[derive(Debug, Clone)]
pub struct ResultField {
    pub percentage: f64,
    pub global_percentage: f64,
    pub count: i64,
    pub name: String,
}

impl ResultField {
    // JAVA: net.minecraft.util.profiling.ResultField#<init>
    pub fn new(name: String, percentage: f64, global_percentage: f64, count: i64) -> Self {
        Self {
            percentage,
            global_percentage,
            count,
            name,
        }
    }
}

// JAVA: net.minecraft.util.profiling.ProfileResults
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

// JAVA: net.minecraft.util.profiling.ProfileResults#demanglePath
pub fn demangle_path(path: &str) -> String {
    path.replace(PATH_SEPARATOR, ".")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockResults {
        start_nano: i64,
        end_nano: i64,
        start_ticks: i32,
        end_ticks: i32,
    }

    impl ProfileResults for MockResults {
        fn get_times(&self, _path: &str) -> Vec<ResultField> {
            vec![ResultField::new("root".to_string(), 100.0, 100.0, 1)]
        }

        fn save_results(&self, _path: &Path) -> bool {
            true
        }

        fn get_start_time_nano(&self) -> i64 {
            self.start_nano
        }

        fn get_start_time_ticks(&self) -> i32 {
            self.start_ticks
        }

        fn get_end_time_nano(&self) -> i64 {
            self.end_nano
        }

        fn get_end_time_ticks(&self) -> i32 {
            self.end_ticks
        }

        fn get_profiler_results(&self) -> String {
            "ok".to_string()
        }
    }

    #[test]
    fn demangle_path_replaces_separator() {
        let input = format!("root\u{001e}tick\u{001e}level");
        assert_eq!(demangle_path(&input), "root.tick.level");
        assert_eq!(demangle_path("plain"), "plain");
        assert_eq!(demangle_path(""), "");
    }

    #[test]
    fn default_durations_compute_difference() {
        let r = MockResults {
            start_nano: 100,
            end_nano: 250,
            start_ticks: 10,
            end_ticks: 20,
        };
        assert_eq!(r.get_nano_duration(), 150);
        assert_eq!(r.get_tick_duration(), 10);
    }

    #[test]
    fn durations_wrap_on_overflow() {
        let r = MockResults {
            start_nano: i64::MAX,
            end_nano: i64::MIN,
            start_ticks: i32::MAX,
            end_ticks: i32::MIN,
        };
        assert_eq!(
            r.get_nano_duration(),
            i64::MIN.wrapping_sub(i64::MAX)
        );
        assert_eq!(
            r.get_tick_duration(),
            i32::MIN.wrapping_sub(i32::MAX)
        );
    }

    #[test]
    fn trait_methods_delegate_correctly() {
        let r = MockResults {
            start_nano: 0,
            end_nano: 0,
            start_ticks: 0,
            end_ticks: 0,
        };
        assert_eq!(r.get_times("root").len(), 1);
        assert!(r.save_results(Path::new("/tmp/test")));
        assert_eq!(r.get_profiler_results(), "ok");
    }
}
