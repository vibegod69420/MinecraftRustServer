use std::time::SystemTime;

/// Minimal placeholder for `net.minecraft.util.profiling.ProfileResults`.
///
/// This keeps the port self-contained until the real `ProfileResults` type is ported.
pub trait ProfileResults {}

/// Faithful port of `net.minecraft.util.profiling.metrics.storage.RecordedDeviation`.
///
/// The profiler result type is generic so this module remains self-contained without
/// depending on an unported `ProfileResults` implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedDeviation<P> {
    pub timestamp: SystemTime,
    pub tick: i32,
    pub profiler_result_at_tick: P,
}

impl<P> RecordedDeviation<P> {
    // JAVA: net.minecraft.util.profiling.metrics.storage.RecordedDeviation#<init>
    pub fn new(timestamp: SystemTime, tick: i32, profiler_result_at_tick: P) -> Self {
        Self {
            timestamp,
            tick,
            profiler_result_at_tick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyProfilerResult {
        id: i32,
    }

    impl ProfileResults for DummyProfilerResult {}

    #[test]
    fn new_stores_fields() {
        let ts = UNIX_EPOCH + Duration::from_millis(1_234);
        let result = DummyProfilerResult { id: 7 };
        let dev = RecordedDeviation::new(ts, 42, result.clone());

        assert_eq!(dev.timestamp, ts);
        assert_eq!(dev.tick, 42);
        assert_eq!(dev.profiler_result_at_tick, result);
    }

    #[test]
    fn negative_tick_is_preserved() {
        let dev = RecordedDeviation::new(UNIX_EPOCH, -1, DummyProfilerResult { id: 0 });
        assert_eq!(dev.tick, -1);

        let dev_min = RecordedDeviation::new(UNIX_EPOCH, -123_456, DummyProfilerResult { id: 1 });
        assert_eq!(dev_min.tick, -123_456_i32);
    }

    #[test]
    fn tick_i32_edge_cases_and_wrapping_semantics() {
        let min_dev = RecordedDeviation::new(UNIX_EPOCH, i32::MIN, DummyProfilerResult { id: 2 });
        let max_dev = RecordedDeviation::new(UNIX_EPOCH, i32::MAX, DummyProfilerResult { id: 3 });

        assert_eq!(min_dev.tick, i32::MIN);
        assert_eq!(max_dev.tick, i32::MAX);

        // Java int arithmetic wraps in two's complement.
        assert_eq!(max_dev.tick.wrapping_add(1), i32::MIN);
        assert_eq!(min_dev.tick.wrapping_sub(1), i32::MAX);
    }

    #[test]
    fn profiler_result_can_preserve_float_bits() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct FloatBitsResult {
            f32_bits: u32,
            f64_bits: u64,
        }

        impl ProfileResults for FloatBitsResult {}

        let f32_bits = f32::NAN.to_bits();
        let f64_bits = (-0.0_f64).to_bits();
        let result = FloatBitsResult { f32_bits, f64_bits };
        let dev = RecordedDeviation::new(UNIX_EPOCH, 0, result.clone());

        assert_eq!(dev.profiler_result_at_tick, result);
        assert_eq!(dev.profiler_result_at_tick.f32_bits, f32_bits);
        assert_eq!(dev.profiler_result_at_tick.f64_bits, f64_bits);
        assert!(f32::from_bits(dev.profiler_result_at_tick.f32_bits).is_nan());
        assert_eq!(
            f64::from_bits(dev.profiler_result_at_tick.f64_bits).to_bits(),
            (-0.0_f64).to_bits()
        );
    }

    #[test]
    fn timestamp_before_epoch_is_representable() {
        let ts = UNIX_EPOCH - Duration::from_secs(5);
        let dev = RecordedDeviation::new(ts, 10, DummyProfilerResult { id: 4 });
        assert_eq!(dev.timestamp, ts);
        assert_eq!(dev.tick, 10);
    }
}
