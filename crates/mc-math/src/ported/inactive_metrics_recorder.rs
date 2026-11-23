/// Minimal stub for `net.minecraft.util.profiling.ProfilerFiller`.
pub trait ProfilerFiller {}

/// Minimal stub for `net.minecraft.util.profiling.InactiveProfiler`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InactiveProfiler;

impl InactiveProfiler {
    pub const INSTANCE: InactiveProfiler = InactiveProfiler;
}

impl ProfilerFiller for InactiveProfiler {}

/// Minimal stub for `net.minecraft.util.profiling.metrics.profiling.MetricsRecorder`.
pub trait MetricsRecorder {
    fn end(&mut self);
    fn cancel(&mut self);
    fn start_tick(&mut self);
    fn is_recording(&self) -> bool;
    fn get_profiler(&self) -> &'static dyn ProfilerFiller;
    fn end_tick(&mut self);
}

/// Port of `net.minecraft.util.profiling.metrics.profiling.InactiveMetricsRecorder`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InactiveMetricsRecorder;

impl InactiveMetricsRecorder {
    pub const INSTANCE: InactiveMetricsRecorder = InactiveMetricsRecorder;
}

impl MetricsRecorder for InactiveMetricsRecorder {
    // JAVA: net.minecraft.util.profiling.metrics.profiling.InactiveMetricsRecorder#end
    fn end(&mut self) {}

    // JAVA: net.minecraft.util.profiling.metrics.profiling.InactiveMetricsRecorder#cancel
    fn cancel(&mut self) {}

    // JAVA: net.minecraft.util.profiling.metrics.profiling.InactiveMetricsRecorder#startTick
    fn start_tick(&mut self) {}

    // JAVA: net.minecraft.util.profiling.metrics.profiling.InactiveMetricsRecorder#isRecording
    fn is_recording(&self) -> bool {
        false
    }

    // JAVA: net.minecraft.util.profiling.metrics.profiling.InactiveMetricsRecorder#getProfiler
    fn get_profiler(&self) -> &'static dyn ProfilerFiller {
        &InactiveProfiler::INSTANCE
    }

    // JAVA: net.minecraft.util.profiling.metrics.profiling.InactiveMetricsRecorder#endTick
    fn end_tick(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_recording_is_false() {
        let rec = InactiveMetricsRecorder::INSTANCE;
        assert!(!rec.is_recording());
    }

    #[test]
    fn methods_do_not_panic() {
        let mut rec = InactiveMetricsRecorder;
        rec.start_tick();
        rec.end_tick();
        rec.end();
        rec.cancel();
        assert!(!rec.is_recording());
    }

    #[test]
    fn get_profiler_returns_inactive_profiler_instance() {
        let rec = InactiveMetricsRecorder;
        let p = rec.get_profiler();
        let inactive: &dyn ProfilerFiller = &InactiveProfiler::INSTANCE;
        assert!(std::ptr::eq(
            p as *const dyn ProfilerFiller as *const (),
            inactive as *const dyn ProfilerFiller as *const ()
        ));
    }

    #[test]
    fn instance_constants_are_usable() {
        let rec = InactiveMetricsRecorder::INSTANCE;
        let prof = InactiveProfiler::INSTANCE;
        let _ = rec;
        let _ = prof;
        assert!(!InactiveMetricsRecorder::INSTANCE.is_recording());
    }
}
