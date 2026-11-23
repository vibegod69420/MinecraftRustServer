use std::sync::Arc;

/// Minimal self-contained profiler filler interface for this module.
// JAVA: net.minecraft.util.profiling.ProfilerFiller
pub trait ProfilerFiller {
    fn start_tick(&mut self);
    fn end_tick(&mut self);
    fn push(&mut self, name: &str);
    fn pop(&mut self);
}

/// Minimal self-contained profile results placeholder.
// JAVA: net.minecraft.util.profiling.ProfileResults
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfileResults;

/// Minimal self-contained profile collector interface.
// JAVA: net.minecraft.util.profiling.ProfileCollector
pub trait ProfileCollector: ProfilerFiller {
    fn get_results(&self) -> ProfileResults;
}

/// Minimal self-contained inactive profiler.
// JAVA: net.minecraft.util.profiling.InactiveProfiler
#[derive(Debug, Clone, Copy, Default)]
pub struct InactiveProfiler;

impl InactiveProfiler {
    // JAVA: net.minecraft.util.profiling.InactiveProfiler#INSTANCE
    pub const INSTANCE: InactiveProfiler = InactiveProfiler;
}

impl ProfilerFiller for InactiveProfiler {
    // JAVA: net.minecraft.util.profiling.InactiveProfiler#startTick
    fn start_tick(&mut self) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#endTick
    fn end_tick(&mut self) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#push
    fn push(&mut self, _name: &str) {}

    // JAVA: net.minecraft.util.profiling.InactiveProfiler#pop
    fn pop(&mut self) {}
}

impl ProfileCollector for InactiveProfiler {
    // JAVA: net.minecraft.util.profiling.InactiveProfiler#getResults
    fn get_results(&self) -> ProfileResults {
        ProfileResults
    }
}

/// Minimal self-contained active profiler.
// JAVA: net.minecraft.util.profiling.ActiveProfiler
#[derive(Clone)]
pub struct ActiveProfiler {
    real_time: Arc<dyn Fn() -> i64 + Send + Sync>,
    tick_count: Arc<dyn Fn() -> i32 + Send + Sync>,
    suppress_warnings: Arc<dyn Fn() -> bool + Send + Sync>,
}

impl ActiveProfiler {
    // JAVA: net.minecraft.util.profiling.ActiveProfiler#<init>
    pub fn new(
        real_time: Arc<dyn Fn() -> i64 + Send + Sync>,
        tick_count: Arc<dyn Fn() -> i32 + Send + Sync>,
        suppress_warnings: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> Self {
        Self {
            real_time,
            tick_count,
            suppress_warnings,
        }
    }

    pub fn real_time(&self) -> i64 {
        (self.real_time)()
    }

    pub fn tick_count(&self) -> i32 {
        (self.tick_count)()
    }

    pub fn suppress_warnings(&self) -> bool {
        (self.suppress_warnings)()
    }
}

impl ProfilerFiller for ActiveProfiler {
    // JAVA: net.minecraft.util.profiling.ActiveProfiler#startTick
    fn start_tick(&mut self) {}

    // JAVA: net.minecraft.util.profiling.ActiveProfiler#endTick
    fn end_tick(&mut self) {}

    // JAVA: net.minecraft.util.profiling.ActiveProfiler#push
    fn push(&mut self, _name: &str) {}

    // JAVA: net.minecraft.util.profiling.ActiveProfiler#pop
    fn pop(&mut self) {}
}

impl ProfileCollector for ActiveProfiler {
    // JAVA: net.minecraft.util.profiling.ActiveProfiler#getResults
    fn get_results(&self) -> ProfileResults {
        ProfileResults
    }
}

enum ProfilerSlot {
    Inactive(InactiveProfiler),
    Active(ActiveProfiler),
}

impl ProfilerSlot {
    fn as_filler_mut(&mut self) -> &mut dyn ProfilerFiller {
        match self {
            ProfilerSlot::Inactive(p) => p,
            ProfilerSlot::Active(p) => p,
        }
    }

    fn as_collector(&self) -> &dyn ProfileCollector {
        match self {
            ProfilerSlot::Inactive(p) => p,
            ProfilerSlot::Active(p) => p,
        }
    }

    fn is_active(&self) -> bool {
        matches!(self, ProfilerSlot::Active(_))
    }
}

/// Faithful port of `net.minecraft.util.profiling.ContinuousProfiler`.
///
/// Java stores suppliers and a `ProfileCollector` field. Rust uses closures in
/// `Arc` for the suppliers and an enum slot to keep the module self-contained
/// without `unsafe` mutable statics.
// JAVA: net.minecraft.util.profiling.ContinuousProfiler
pub struct ContinuousProfiler {
    real_time: Arc<dyn Fn() -> i64 + Send + Sync>,
    tick_count: Arc<dyn Fn() -> i32 + Send + Sync>,
    suppress_warnings: Arc<dyn Fn() -> bool + Send + Sync>,
    profiler: ProfilerSlot,
}

impl ContinuousProfiler {
    // JAVA: net.minecraft.util.profiling.ContinuousProfiler#<init>
    pub fn new(
        real_time: Arc<dyn Fn() -> i64 + Send + Sync>,
        tick_count: Arc<dyn Fn() -> i32 + Send + Sync>,
        suppress_warnings: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> Self {
        Self {
            real_time,
            tick_count,
            suppress_warnings,
            profiler: ProfilerSlot::Inactive(InactiveProfiler::INSTANCE),
        }
    }

    // JAVA: net.minecraft.util.profiling.ContinuousProfiler#isEnabled
    pub fn is_enabled(&self) -> bool {
        self.profiler.is_active()
    }

    // JAVA: net.minecraft.util.profiling.ContinuousProfiler#disable
    pub fn disable(&mut self) {
        self.profiler = ProfilerSlot::Inactive(InactiveProfiler::INSTANCE);
    }

    // JAVA: net.minecraft.util.profiling.ContinuousProfiler#enable
    pub fn enable(&mut self) {
        self.profiler = ProfilerSlot::Active(ActiveProfiler::new(
            Arc::clone(&self.real_time),
            Arc::clone(&self.tick_count),
            Arc::clone(&self.suppress_warnings),
        ));
    }

    // JAVA: net.minecraft.util.profiling.ContinuousProfiler#getFiller
    pub fn get_filler(&mut self) -> &mut dyn ProfilerFiller {
        self.profiler.as_filler_mut()
    }

    // JAVA: net.minecraft.util.profiling.ContinuousProfiler#getResults
    pub fn get_results(&self) -> ProfileResults {
        self.profiler.as_collector().get_results()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering};

    fn make_profiler() -> (
        ContinuousProfiler,
        Arc<AtomicI64>,
        Arc<AtomicI32>,
        Arc<AtomicBool>,
    ) {
        let rt = Arc::new(AtomicI64::new(0));
        let tc = Arc::new(AtomicI32::new(0));
        let sw = Arc::new(AtomicBool::new(false));
        let rt_c = Arc::clone(&rt);
        let tc_c = Arc::clone(&tc);
        let sw_c = Arc::clone(&sw);
        let profiler = ContinuousProfiler::new(
            Arc::new(move || rt_c.load(Ordering::Relaxed)),
            Arc::new(move || tc_c.load(Ordering::Relaxed)),
            Arc::new(move || sw_c.load(Ordering::Relaxed)),
        );
        (profiler, rt, tc, sw)
    }

    #[test]
    fn starts_disabled() {
        let (profiler, _, _, _) = make_profiler();
        assert!(!profiler.is_enabled());
        assert_eq!(profiler.get_results(), ProfileResults);
    }

    #[test]
    fn enable_disable_toggles_state() {
        let (mut profiler, _, _, _) = make_profiler();
        profiler.enable();
        assert!(profiler.is_enabled());
        profiler.disable();
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn enable_captures_suppliers() {
        let (mut profiler, rt, tc, sw) = make_profiler();
        rt.store(123, Ordering::Relaxed);
        tc.store(7, Ordering::Relaxed);
        sw.store(true, Ordering::Relaxed);
        profiler.enable();
        match &profiler.profiler {
            ProfilerSlot::Active(active) => {
                assert_eq!(active.real_time(), 123);
                assert_eq!(active.tick_count(), 7);
                assert!(active.suppress_warnings());
            }
            ProfilerSlot::Inactive(_) => panic!("expected active profiler"),
        }
    }

    #[test]
    fn get_filler_is_usable_when_inactive_and_active() {
        let (mut profiler, _, _, _) = make_profiler();
        profiler.get_filler().start_tick();
        profiler.get_filler().push("root");
        profiler.get_filler().pop();
        profiler.get_filler().end_tick();

        profiler.enable();
        profiler.get_filler().start_tick();
        profiler.get_filler().push("root");
        profiler.get_filler().pop();
        profiler.get_filler().end_tick();
        assert!(profiler.is_enabled());
    }

    #[test]
    fn reenable_replaces_active_profiler() {
        let (mut profiler, rt, _, _) = make_profiler();
        rt.store(1, Ordering::Relaxed);
        profiler.enable();
        let first_time = match &profiler.profiler {
            ProfilerSlot::Active(a) => a.real_time(),
            _ => panic!("expected active"),
        };
        rt.store(2, Ordering::Relaxed);
        profiler.enable();
        let second_time = match &profiler.profiler {
            ProfilerSlot::Active(a) => a.real_time(),
            _ => panic!("expected active"),
        };
        assert_eq!(first_time, 1);
        assert_eq!(second_time, 2);
    }
}
