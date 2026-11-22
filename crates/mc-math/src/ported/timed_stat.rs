//! Faithful port of `net.minecraft.util.profiling.jfr.stats.TimedStat`.

use std::time::Duration;

/// Faithful port of `net.minecraft.util.profiling.jfr.stats.TimedStat`.
///
/// A profiling statistic that exposes the total duration it measured.
pub trait TimedStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.TimedStat#duration
    fn duration(&self) -> Duration;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MockTimedStat {
        duration: Duration,
    }

    impl MockTimedStat {
        fn new(duration: Duration) -> Self {
            Self { duration }
        }
    }

    impl TimedStat for MockTimedStat {
        fn duration(&self) -> Duration {
            self.duration
        }
    }

    fn duration_via_ref(stat: &dyn TimedStat) -> Duration {
        stat.duration()
    }

    fn duration_via_generic<T: TimedStat + ?Sized>(stat: &T) -> Duration {
        stat.duration()
    }

    fn clamp_signed_duration(secs: i64, nanos: i32) -> Duration {
        if secs < 0 || nanos < 0 {
            Duration::ZERO
        } else {
            Duration::new(secs as u64, nanos as u32)
        }
    }

    #[test]
    fn zero_duration() {
        let stat = MockTimedStat::new(Duration::ZERO);
        assert_eq!(stat.duration(), Duration::ZERO);
        assert_eq!(stat.duration().as_secs(), 0u64);
        assert_eq!(stat.duration().subsec_nanos(), 0u32);
        assert_eq!(stat.duration().as_nanos(), 0u128);
    }

    #[test]
    fn one_nanosecond() {
        let stat = MockTimedStat::new(Duration::from_nanos(1));
        assert_eq!(stat.duration(), Duration::from_nanos(1));
        assert_eq!(stat.duration().as_nanos(), 1u128);
    }

    #[test]
    fn one_second() {
        let stat = MockTimedStat::new(Duration::from_secs(1));
        assert_eq!(stat.duration().as_secs(), 1u64);
        assert_eq!(stat.duration().subsec_nanos(), 0u32);
    }

    #[test]
    fn subsecond_nanos_boundary() {
        let stat = MockTimedStat::new(Duration::new(0, 999_999_999));
        assert_eq!(stat.duration().as_secs(), 0u64);
        assert_eq!(stat.duration().subsec_nanos(), 999_999_999u32);

        let stat2 = MockTimedStat::new(Duration::new(5, 123_456_789));
        assert_eq!(stat2.duration().as_secs(), 5u64);
        assert_eq!(stat2.duration().subsec_nanos(), 123_456_789u32);
    }

    #[test]
    fn max_duration() {
        let stat = MockTimedStat::new(Duration::MAX);
        assert_eq!(stat.duration(), Duration::MAX);
        assert_eq!(stat.duration().as_secs(), u64::MAX);
        assert_eq!(stat.duration().subsec_nanos(), 999_999_999u32);
    }

    #[test]
    fn large_nanos_conversion() {
        let nanos = u64::MAX;
        let stat = MockTimedStat::new(Duration::from_nanos(nanos));
        let expected_secs = nanos / 1_000_000_000u64;
        let expected_subsec_nanos = (nanos % 1_000_000_000u64) as u32;
        assert_eq!(stat.duration().as_secs(), expected_secs);
        assert_eq!(stat.duration().subsec_nanos(), expected_subsec_nanos);
    }

    #[test]
    fn nanos_overflow_edge_cases_saturating() {
        let a = MockTimedStat::new(Duration::from_secs(u64::MAX));
        let b = Duration::from_secs(1);
        let sum = a.duration().saturating_add(b);
        assert_eq!(sum, Duration::MAX);

        let c = MockTimedStat::new(Duration::new(u64::MAX, 999_999_999));
        let sum2 = c.duration().saturating_add(Duration::from_nanos(1));
        assert_eq!(sum2, Duration::MAX);

        let d = MockTimedStat::new(Duration::ZERO);
        let diff = d.duration().saturating_sub(Duration::from_nanos(1));
        assert_eq!(diff, Duration::ZERO);
    }

    #[test]
    fn negative_inputs_clamped_to_zero() {
        assert_eq!(clamp_signed_duration(-1, 0), Duration::ZERO);
        assert_eq!(clamp_signed_duration(0, -1), Duration::ZERO);
        assert_eq!(clamp_signed_duration(-100, -999_999_999), Duration::ZERO);
        assert_eq!(clamp_signed_duration(i64::MIN, i32::MIN), Duration::ZERO);
        assert_eq!(clamp_signed_duration(-1, 1), Duration::ZERO);
        assert_eq!(clamp_signed_duration(1, -1), Duration::ZERO);

        let stat = MockTimedStat::new(clamp_signed_duration(-5, 500_000_000));
        assert_eq!(stat.duration(), Duration::ZERO);
    }

    #[test]
    fn negative_i64_nanos_input() {
        let inputs: [i64; 5] = [-1i64, -1_000_000_000i64, i64::MIN, -42i64, -999_999_999i64];
        for nanos in inputs {
            let dur = if nanos < 0 {
                Duration::ZERO
            } else {
                Duration::from_nanos(nanos as u64)
            };
            let stat = MockTimedStat::new(dur);
            assert_eq!(stat.duration(), Duration::ZERO, "negative nanos {nanos} should clamp");
        }

        let non_negative: i64 = 1_500_000_000i64;
        let dur = Duration::from_nanos(non_negative as u64);
        let stat = MockTimedStat::new(dur);
        assert_eq!(stat.duration().as_secs(), 1u64);
        assert_eq!(stat.duration().subsec_nanos(), 500_000_000u32);
    }

    #[test]
    fn float_bits_f64() {
        let stat = MockTimedStat::new(Duration::from_nanos(1_500_000_000));
        let secs_f64 = stat.duration().as_secs_f64();
        assert_eq!(secs_f64.to_bits(), 1.5f64.to_bits());

        let zero = MockTimedStat::new(Duration::ZERO);
        let zero_f64 = zero.duration().as_secs_f64();
        assert_eq!(zero_f64.to_bits(), 0.0f64.to_bits());
        assert_ne!(zero_f64.to_bits(), (-0.0f64).to_bits());

        let one_sec = MockTimedStat::new(Duration::from_secs(1));
        assert_eq!(one_sec.duration().as_secs_f64().to_bits(), 1.0f64.to_bits());

        let max_stat = MockTimedStat::new(Duration::MAX);
        let max_f64 = max_stat.duration().as_secs_f64();
        assert!(max_f64.is_finite());
        assert!(max_f64 > 0.0f64);
        assert_eq!(max_f64.to_bits(), max_f64.to_bits());
    }

    #[test]
    fn float_bits_f32() {
        let stat = MockTimedStat::new(Duration::from_secs(2));
        let secs_f32 = stat.duration().as_secs_f32();
        assert_eq!(secs_f32.to_bits(), 2.0f32.to_bits());

        let half = MockTimedStat::new(Duration::from_nanos(500_000_000));
        let half_f32 = half.duration().as_secs_f32();
        assert_eq!(half_f32.to_bits(), 0.5f32.to_bits());

        let zero = MockTimedStat::new(Duration::ZERO);
        let zero_f32 = zero.duration().as_secs_f32();
        assert_eq!(zero_f32.to_bits(), 0.0f32.to_bits());
        assert_ne!(zero_f32.to_bits(), (-0.0f32).to_bits());

        let quarter = MockTimedStat::new(Duration::from_nanos(250_000_000));
        assert_eq!(quarter.duration().as_secs_f32().to_bits(), 0.25f32.to_bits());
    }

    #[test]
    fn trait_object_and_generic_access() {
        let stat = MockTimedStat::new(Duration::from_millis(250));
        assert_eq!(duration_via_ref(&stat), Duration::from_millis(250));
        assert_eq!(duration_via_generic(&stat), Duration::from_millis(250));

        let boxed: Box<dyn TimedStat> = Box::new(MockTimedStat::new(Duration::from_micros(1234)));
        assert_eq!(boxed.duration(), Duration::from_micros(1234));
    }

    #[test]
    fn integer_semantics_wrapping_and_masked_shifts() {
        // Java integer arithmetic wraps on overflow; masked shifts use & 31 / & 63.
        let raw_nanos: i64 = i64::MAX;
        let wrapped_i64 = raw_nanos.wrapping_add(1i64);
        assert_eq!(wrapped_i64, i64::MIN);

        let raw_a: i32 = i32::MAX;
        let raw_b: i32 = 1i32;
        let wrapped_i32 = raw_a.wrapping_add(raw_b);
        assert_eq!(wrapped_i32, i32::MIN);

        let wrapped_sub: i32 = i32::MIN.wrapping_sub(1i32);
        assert_eq!(wrapped_sub, i32::MAX);

        let wrapped_mul: i32 = 1_000_000i32.wrapping_mul(3_000i32);
        let expected_mul = ((1_000_000i64 * 3_000i64) as i32).wrapping_add(0i32);
        assert_eq!(wrapped_mul, expected_mul);

        let shift_amount_32: u32 = 40u32;
        let masked_shift_32: u32 = 1u32.wrapping_shl(shift_amount_32 & 31u32);
        assert_eq!(masked_shift_32, 1u32 << 8u32);

        let shift_amount_64: u64 = 72u64;
        let masked_shift_64: u64 = 1u64.wrapping_shl((shift_amount_64 & 63u64) as u32);
        assert_eq!(masked_shift_64, 1u64 << 8u64);

        // Use wrapped signed value as a clamped duration input boundary.
        let clamped_secs = if wrapped_i64 < 0i64 {
            0u64
        } else {
            (wrapped_i64 / 1_000_000_000i64) as u64
        };
        let stat = MockTimedStat::new(Duration::from_secs(clamped_secs));
        assert_eq!(stat.duration().as_secs(), 0u64);
    }

    #[test]
    fn duration_component_round_trip() {
        let cases: [(u64, u32); 6] = [
            (0u64, 0u32),
            (0u64, 1u32),
            (1u64, 0u32),
            (123u64, 456_789_012u32),
            (u64::MAX, 0u32),
            (u64::MAX, 999_999_999u32),
        ];

        for (secs, subsec_nanos) in cases {
            let dur = Duration::new(secs, subsec_nanos);
            let stat = MockTimedStat::new(dur);
            assert_eq!(stat.duration().as_secs(), secs);
            assert_eq!(stat.duration().subsec_nanos(), subsec_nanos);
            assert_eq!(stat.duration(), dur);
        }
    }
}
