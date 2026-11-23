use std::time::Duration;

/// A point-in-time JVM heap statistic sampled from a JFR GC event.
///
/// // JAVA: net.minecraft.util.profiling.jfr.stats.GcHeapStat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GcHeapStat {
    /// Milliseconds since the UNIX epoch, representing the JFR event start time.
    pub timestamp_millis: i64,
    pub heap_used: i64,
    pub timing: Timing,
}

/// // JAVA: net.minecraft.util.profiling.jfr.stats.GcHeapStat$Timing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timing {
    BeforeGc,
    AfterGc,
}

/// // JAVA: net.minecraft.util.profiling.jfr.stats.GcHeapStat$Summary
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub duration: Duration,
    pub gc_total_duration: Duration,
    pub total_gcs: i32,
    pub allocation_rate_bytes_per_second: f64,
}

/// Minimal event view matching the fields read by `GcHeapStat.from`.
///
/// This keeps the port self-contained without depending on a JFR consumer crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedGcHeapEvent {
    pub start_time_millis: i64,
    pub heap_used: i64,
    /// Expected values in practice: `"before gc"` / `"after gc"` (case-insensitive),
    /// matching Java `RecordedEvent#getString("when")`.
    pub when: String,
}

impl GcHeapStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.GcHeapStat#from
    pub fn from(event: &RecordedGcHeapEvent) -> Self {
        let timing = if event.when.eq_ignore_ascii_case("before gc") {
            Timing::BeforeGc
        } else {
            Timing::AfterGc
        };
        Self {
            timestamp_millis: event.start_time_millis,
            heap_used: event.heap_used,
            timing,
        }
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.GcHeapStat#summary
    pub fn summary(duration: Duration, stats: &[GcHeapStat], gc_total_duration: Duration, total_gcs: i32) -> Summary {
        Summary {
            duration,
            gc_total_duration,
            total_gcs,
            allocation_rate_bytes_per_second: Self::calculate_allocation_rate_per_second(stats),
        }
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.GcHeapStat#calculateAllocationRatePerSecond
    fn calculate_allocation_rate_per_second(stats: &[GcHeapStat]) -> f64 {
        let mut total: i64 = 0;

        let mut before_gc: Vec<&GcHeapStat> = Vec::new();
        let mut after_gc: Vec<&GcHeapStat> = Vec::new();
        for stat in stats {
            match stat.timing {
                Timing::BeforeGc => before_gc.push(stat),
                Timing::AfterGc => after_gc.push(stat),
            }
        }

        let mut i: usize = 1;
        while i < before_gc.len() {
            let before = before_gc[i];
            let after = after_gc[i - 1];
            total = total.wrapping_add(before.heap_used.wrapping_sub(after.heap_used));
            i += 1;
        }

        let start_ms = stats[1].timestamp_millis;
        let end_ms = stats[stats.len() - 1].timestamp_millis;
        let duration_secs = Duration::from_millis((end_ms - start_ms) as u64).as_secs();
        (total as f64) / (duration_secs as f64)
    }
}

impl Summary {
    // JAVA: net.minecraft.util.profiling.jfr.stats.GcHeapStat$Summary#gcOverHead
    pub fn gc_over_head(&self) -> f32 {
        (self.gc_total_duration.as_millis() as f32) / (self.duration.as_millis() as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stat(timestamp_millis: i64, heap_used: i64, timing: Timing) -> GcHeapStat {
        GcHeapStat {
            timestamp_millis,
            heap_used,
            timing,
        }
    }

    #[test]
    fn from_maps_timing_case_insensitively() {
        let before = GcHeapStat::from(&RecordedGcHeapEvent {
            start_time_millis: 1000,
            heap_used: 500,
            when: "Before GC".to_string(),
        });
        assert_eq!(before.timing, Timing::BeforeGc);
        assert_eq!(before.timestamp_millis, 1000);
        assert_eq!(before.heap_used, 500);

        let after = GcHeapStat::from(&RecordedGcHeapEvent {
            start_time_millis: 2000,
            heap_used: 300,
            when: "AFTER GC".to_string(),
        });
        assert_eq!(after.timing, Timing::AfterGc);

        let unknown_defaults_to_after = GcHeapStat::from(&RecordedGcHeapEvent {
            start_time_millis: 3000,
            heap_used: 100,
            when: "something else".to_string(),
        });
        assert_eq!(unknown_defaults_to_after.timing, Timing::AfterGc);
    }

    #[test]
    fn calculate_allocation_rate_per_second_matches_java_formula() {
        // Interleaved order as typically received from JFR; grouping preserves encounter order.
        let stats = vec![
            stat(0_000, 100, Timing::BeforeGc),
            stat(0_100, 20, Timing::AfterGc),
            stat(1_000, 220, Timing::BeforeGc),
            stat(1_100, 30, Timing::AfterGc),
            stat(3_000, 330, Timing::BeforeGc),
            stat(3_100, 40, Timing::AfterGc),
        ];

        let summary = GcHeapStat::summary(
            Duration::from_millis(10_000),
            &stats,
            Duration::from_millis(500),
            2,
        );

        // total = (220 - 20) + (330 - 30) = 500
        // duration = between(stats[1]=100ms, stats[5]=3100ms) = 3s -> getSeconds() == 3
        assert_eq!(summary.total_gcs, 2);
        assert_eq!(summary.gc_total_duration, Duration::from_millis(500));
        assert_eq!(summary.duration, Duration::from_millis(10_000));
        assert!((summary.allocation_rate_bytes_per_second - (500.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn gc_over_head_uses_millis_as_float() {
        let summary = Summary {
            duration: Duration::from_millis(2000),
            gc_total_duration: Duration::from_millis(500),
            total_gcs: 1,
            allocation_rate_bytes_per_second: 0.0,
        };
        let overhead = summary.gc_over_head();
        assert!((overhead - 0.25_f32).abs() < 1e-6);
    }

    #[test]
    fn allocation_rate_handles_negative_deltas_and_wrapping_arithmetic() {
        // Negative delta: heapUsed decreased more than expected (possible in edge cases);
        // Java uses long subtraction, so wrapping_sub matches two's-complement behavior.
        let stats = vec![
            stat(0_000, 0, Timing::BeforeGc),
            stat(0_100, 100, Timing::AfterGc),
            stat(2_000, 50, Timing::BeforeGc), // 50 - 100 = -50
            stat(2_100, 10, Timing::AfterGc),
            stat(4_000, 20, Timing::BeforeGc), // 20 - 10 = 10
            stat(4_100, 5, Timing::AfterGc),
        ];

        let rate = GcHeapStat::calculate_allocation_rate_per_second(&stats);
        // total = -50 + 10 = -40, duration between 100ms and 4100ms = 4s
        assert!((rate - (-40.0 / 4.0)).abs() < 1e-9);
    }

    #[test]
    fn allocation_rate_large_values_wrapping_add_sub() {
        let stats = vec![
            stat(0, i64::MAX - 10, Timing::BeforeGc),
            stat(1_000, -100, Timing::AfterGc),
            stat(3_000, i64::MAX, Timing::BeforeGc),
            stat(3_100, 0, Timing::AfterGc),
            stat(5_000, 0, Timing::BeforeGc),
            stat(5_100, 0, Timing::AfterGc),
        ];
        // Just ensure it computes without panicking and uses wrapping semantics.
        let rate = GcHeapStat::calculate_allocation_rate_per_second(&stats);
        assert!(rate.is_finite() || rate.is_infinite() || rate.is_nan());
        // Verify the wrapping total directly:
        // i=1: (i64::MAX) - (-100) wraps, i=2: 0 - 0
        let expected_total = i64::MAX.wrapping_sub(-100_i64);
        let expected = (expected_total as f64) / (4.0_f64); // between 1000ms and 5100ms = 4s
        assert_eq!(rate.to_bits(), expected.to_bits());
    }
}
