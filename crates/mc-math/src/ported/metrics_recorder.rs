//! Faithful port of `net.minecraft.util.profiling.metrics.profiling.MetricsRecorder`.
//!
//! Source: `net/minecraft/util/profiling/metrics/profiling/MetricsRecorder.java`

/// Minimal local stand-in for `net.minecraft.util.profiling.ProfilerFiller`.
///
/// The full profiler type is not yet ported; this trait provides the type
/// boundary required by [`MetricsRecorder::get_profiler`].
pub trait ProfilerFiller {}

/// Port of `net.minecraft.util.profiling.metrics.profiling.MetricsRecorder`.
///
/// A recorder controls the lifecycle of metric profiling across ticks.
pub trait MetricsRecorder {
    // JAVA: net.minecraft.util.profiling.metrics.profiling.MetricsRecorder#end
    fn end(&mut self);

    // JAVA: net.minecraft.util.profiling.metrics.profiling.MetricsRecorder#cancel
    fn cancel(&mut self);

    // JAVA: net.minecraft.util.profiling.metrics.profiling.MetricsRecorder#startTick
    fn start_tick(&mut self);

    // JAVA: net.minecraft.util.profiling.metrics.profiling.MetricsRecorder#isRecording
    fn is_recording(&self) -> bool;

    // JAVA: net.minecraft.util.profiling.metrics.profiling.MetricsRecorder#getProfiler
    fn get_profiler(&mut self) -> &mut dyn ProfilerFiller;

    // JAVA: net.minecraft.util.profiling.metrics.profiling.MetricsRecorder#endTick
    fn end_tick(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal mock profiler implementing the local [`ProfilerFiller`] boundary.
    #[derive(Debug, Default)]
    struct MockProfiler;

    impl ProfilerFiller for MockProfiler {}

    /// Mock implementation of [`MetricsRecorder`] for exercising the interface contract.
    #[derive(Debug)]
    struct MockRecorder {
        recording: bool,
        ended: bool,
        cancelled: bool,
        tick_starts: i32,
        tick_ends: i32,
        profiler: MockProfiler,
    }

    impl MockRecorder {
        fn new() -> Self {
            Self {
                recording: false,
                ended: false,
                cancelled: false,
                tick_starts: 0,
                tick_ends: 0,
                profiler: MockProfiler,
            }
        }
    }

    impl MetricsRecorder for MockRecorder {
        fn end(&mut self) {
            self.recording = false;
            self.ended = true;
        }

        fn cancel(&mut self) {
            self.recording = false;
            self.cancelled = true;
        }

        fn start_tick(&mut self) {
            self.recording = true;
            self.tick_starts = self.tick_starts.wrapping_add(1);
        }

        fn is_recording(&self) -> bool {
            self.recording
        }

        fn get_profiler(&mut self) -> &mut dyn ProfilerFiller {
            &mut self.profiler
        }

        fn end_tick(&mut self) {
            self.tick_ends = self.tick_ends.wrapping_add(1);
        }
    }

    fn exercise_recorder<R: MetricsRecorder>(rec: &mut R) {
        assert!(!rec.is_recording());
        rec.start_tick();
        assert!(rec.is_recording());
        {
            let _profiler: &mut dyn ProfilerFiller = rec.get_profiler();
        }
        rec.end_tick();
        assert!(rec.is_recording());
        rec.end();
        assert!(!rec.is_recording());
    }

    #[test]
    fn lifecycle_end() {
        let mut rec = MockRecorder::new();
        assert!(!rec.is_recording());
        assert!(!rec.ended);
        assert!(!rec.cancelled);

        rec.start_tick();
        assert!(rec.is_recording());

        rec.end_tick();
        assert!(rec.is_recording());

        rec.end();
        assert!(!rec.is_recording());
        assert!(rec.ended);
        assert!(!rec.cancelled);
    }

    #[test]
    fn lifecycle_cancel() {
        let mut rec = MockRecorder::new();
        rec.start_tick();
        assert!(rec.is_recording());

        rec.cancel();
        assert!(!rec.is_recording());
        assert!(rec.cancelled);
        assert!(!rec.ended);
    }

    #[test]
    fn multiple_tick_cycles() {
        let mut rec = MockRecorder::new();
        for _ in 0..5 {
            rec.start_tick();
            assert!(rec.is_recording());
            rec.end_tick();
        }
        assert_eq!(rec.tick_starts, 5);
        assert_eq!(rec.tick_ends, 5);
        rec.end();
        assert!(!rec.is_recording());
    }

    #[test]
    fn trait_object_usage() {
        let mut rec = MockRecorder::new();
        let r: &mut dyn MetricsRecorder = &mut rec;
        assert!(!r.is_recording());
        r.start_tick();
        assert!(r.is_recording());
        {
            let _p: &mut dyn ProfilerFiller = r.get_profiler();
        }
        r.end_tick();
        r.end();
        assert!(!r.is_recording());
    }

    #[test]
    fn generic_usage() {
        let mut rec = MockRecorder::new();
        exercise_recorder(&mut rec);
        assert!(rec.ended);
    }

    #[test]
    fn get_profiler_is_accessible_across_calls() {
        let mut rec = MockRecorder::new();
        {
            let _p1: &mut dyn ProfilerFiller = rec.get_profiler();
        }
        {
            let _p2: &mut dyn ProfilerFiller = rec.get_profiler();
        }
        rec.start_tick();
        assert!(rec.is_recording());
        rec.end();
        assert!(!rec.is_recording());
    }

    #[test]
    fn metrics_recorder_trait_is_object_safe() {
        fn _accept_ref(_r: &dyn MetricsRecorder) {}
        fn _accept_mut(_r: &mut dyn MetricsRecorder) {}
    }

    #[test]
    fn wrapping_overflow_edge_cases() {
        let mut rec = MockRecorder::new();

        rec.tick_starts = i32::MAX;
        rec.start_tick();
        assert_eq!(rec.tick_starts, i32::MIN);
        assert!(rec.is_recording());

        rec.tick_ends = i32::MAX;
        rec.end_tick();
        assert_eq!(rec.tick_ends, i32::MIN);

        // Verify explicit wrapping behavior for Java integer semantics.
        assert_eq!(i32::MAX.wrapping_add(1), i32::MIN);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
        assert_eq!(i32::MAX.wrapping_mul(2), -2);
    }

    #[test]
    fn negative_input_edge_cases() {
        let mut rec = MockRecorder::new();

        rec.tick_starts = -1;
        rec.start_tick();
        assert_eq!(rec.tick_starts, 0);

        rec.tick_starts = i32::MIN;
        rec.start_tick();
        assert_eq!(rec.tick_starts, i32::MIN.wrapping_add(1));

        rec.tick_ends = -100;
        rec.end_tick();
        assert_eq!(rec.tick_ends, -99);

        // Negative shift masks as Java would require in numeric contexts.
        let shift_mask_31: i32 = -1 & 31;
        assert_eq!(shift_mask_31, 31);
        let shift_mask_63: i64 = -1_i64 & 63;
        assert_eq!(shift_mask_63, 63);
    }

    #[test]
    fn float_bit_edge_cases() {
        // Verify float bit patterns match expected IEEE-754/Java-visible bit layouts.
        let neg_zero_f32 = (-0.0_f32).to_bits();
        let pos_zero_f32 = 0.0_f32.to_bits();
        assert_eq!(neg_zero_f32, 0x8000_0000_u32);
        assert_eq!(pos_zero_f32, 0x0000_0000_u32);
        assert_ne!(neg_zero_f32, pos_zero_f32);

        let nan_f32_bits = f32::NAN.to_bits();
        assert_eq!(nan_f32_bits & 0x7F80_0000, 0x7F80_0000);
        assert_ne!(nan_f32_bits & 0x007F_FFFF, 0);

        let neg_inf_f32 = f32::NEG_INFINITY.to_bits();
        let pos_inf_f32 = f32::INFINITY.to_bits();
        assert_eq!(neg_inf_f32, 0xFF80_0000_u32);
        assert_eq!(pos_inf_f32, 0x7F80_0000_u32);

        let neg_zero_f64 = (-0.0_f64).to_bits();
        let pos_zero_f64 = 0.0_f64.to_bits();
        assert_eq!(neg_zero_f64, 0x8000_0000_0000_0000_u64);
        assert_eq!(pos_zero_f64, 0x0000_0000_0000_0000_u64);
        assert_ne!(neg_zero_f64, pos_zero_f64);

        let nan_f64_bits = f64::NAN.to_bits();
        assert_eq!(nan_f64_bits & 0x7FF0_0000_0000_0000, 0x7FF0_0000_0000_0000);
        assert_ne!(nan_f64_bits & 0x000F_FFFF_FFFF_FFFF, 0);

        let neg_inf_f64 = f64::NEG_INFINITY.to_bits();
        let pos_inf_f64 = f64::INFINITY.to_bits();
        assert_eq!(neg_inf_f64, 0xFFF0_0000_0000_0000_u64);
        assert_eq!(pos_inf_f64, 0x7FF0_0000_0000_0000_u64);
    }
}
