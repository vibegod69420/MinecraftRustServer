use std::path::Path;

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

// JAVA: net.minecraft.util.profiling.EmptyProfileResults
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyProfileResults;

// JAVA: net.minecraft.util.profiling.EmptyProfileResults#EMPTY
pub static EMPTY: EmptyProfileResults = EmptyProfileResults;

impl EmptyProfileResults {
    // JAVA: net.minecraft.util.profiling.EmptyProfileResults#<init>
    fn new() -> Self {
        Self
    }

    /// Returns the singleton empty profile results instance.
    pub fn instance() -> &'static EmptyProfileResults {
        &EMPTY
    }
}

impl ProfileResults for EmptyProfileResults {
    // JAVA: net.minecraft.util.profiling.EmptyProfileResults#getTimes
    fn get_times(&self, _path: &str) -> Vec<ResultField> {
        Vec::new()
    }

    // JAVA: net.minecraft.util.profiling.EmptyProfileResults#saveResults
    fn save_results(&self, _path: &Path) -> bool {
        false
    }

    // JAVA: net.minecraft.util.profiling.EmptyProfileResults#getStartTimeNano
    fn get_start_time_nano(&self) -> i64 {
        0
    }

    // JAVA: net.minecraft.util.profiling.EmptyProfileResults#getStartTimeTicks
    fn get_start_time_ticks(&self) -> i32 {
        0
    }

    // JAVA: net.minecraft.util.profiling.EmptyProfileResults#getEndTimeNano
    fn get_end_time_nano(&self) -> i64 {
        0
    }

    // JAVA: net.minecraft.util.profiling.EmptyProfileResults#getEndTimeTicks
    fn get_end_time_ticks(&self) -> i32 {
        0
    }

    // JAVA: net.minecraft.util.profiling.EmptyProfileResults#getProfilerResults
    fn get_profiler_results(&self) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_singleton_returns_defaults() {
        let e = &EMPTY;
        assert!(e.get_times("root").is_empty());
        assert!(!e.save_results(Path::new("/tmp/nope")));
        assert_eq!(e.get_start_time_nano(), 0);
        assert_eq!(e.get_start_time_ticks(), 0);
        assert_eq!(e.get_end_time_nano(), 0);
        assert_eq!(e.get_end_time_ticks(), 0);
        assert_eq!(e.get_profiler_results(), "");
    }

    #[test]
    fn durations_are_zero() {
        let e = EmptyProfileResults::new();
        assert_eq!(e.get_nano_duration(), 0);
        assert_eq!(e.get_tick_duration(), 0);
    }

    #[test]
    fn instance_returns_static_ref() {
        let a: *const EmptyProfileResults = EmptyProfileResults::instance();
        let b: *const EmptyProfileResults = &EMPTY;
        assert_eq!(a, b);
    }

    #[test]
    fn get_times_ignores_path() {
        let e = EMPTY;
        assert!(e.get_times("").is_empty());
        assert!(e.get_times("root\u{001e}child").is_empty());
    }
}
