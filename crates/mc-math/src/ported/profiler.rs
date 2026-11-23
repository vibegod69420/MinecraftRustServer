use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicI32, Ordering};

pub trait ProfilerFiller {
    fn start_tick(&mut self);
    fn end_tick(&mut self);
}

pub type SharedProfilerFiller = Rc<RefCell<dyn ProfilerFiller>>;

#[derive(Debug, Default)]
pub struct InactiveProfiler;

impl InactiveProfiler {
    pub const INSTANCE: InactiveProfiler = InactiveProfiler;
}

impl ProfilerFiller for InactiveProfiler {
    fn start_tick(&mut self) {}
    fn end_tick(&mut self) {}
}

#[derive(Debug, Default)]
pub struct TracyZoneFiller;

impl TracyZoneFiller {
    pub fn new() -> Self {
        Self
    }
}

impl ProfilerFiller for TracyZoneFiller {
    fn start_tick(&mut self) {}
    fn end_tick(&mut self) {}
}

pub struct TracyClient;

impl TracyClient {
    pub fn is_available() -> bool {
        false
    }
}

struct CombinedProfilerFiller {
    first: SharedProfilerFiller,
    second: SharedProfilerFiller,
}

impl ProfilerFiller for CombinedProfilerFiller {
    fn start_tick(&mut self) {
        self.first.borrow_mut().start_tick();
        self.second.borrow_mut().start_tick();
    }

    fn end_tick(&mut self) {
        self.first.borrow_mut().end_tick();
        self.second.borrow_mut().end_tick();
    }
}

thread_local! {
    static TRACY_FILLER: SharedProfilerFiller = Rc::new(RefCell::new(TracyZoneFiller::new()));
    static ACTIVE: RefCell<Option<SharedProfilerFiller>> = const { RefCell::new(None) };
}

static ACTIVE_COUNT: AtomicI32 = AtomicI32::new(0);

pub struct Scope {
    closed: bool,
}

impl Scope {
    // JAVA: net.minecraft.util.profiling.Profiler$Scope#close
    pub fn close(mut self) {
        if !self.closed {
            self.closed = true;
            stop_using();
        }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if !self.closed {
            self.closed = true;
            stop_using();
        }
    }
}

// JAVA: net.minecraft.util.profiling.Profiler#use
pub fn use_profiler(profiler: SharedProfilerFiller) -> Scope {
    start_using(profiler);
    Scope { closed: false }
}

// JAVA: net.minecraft.util.profiling.Profiler#startUsing
fn start_using(profiler: SharedProfilerFiller) {
    let already_active = ACTIVE.with(|a| a.borrow().is_some());
    if already_active {
        panic!("Profiler is already active");
    }
    let decorated = decorate_filler(profiler);
    ACTIVE.with(|a| {
        *a.borrow_mut() = Some(decorated.clone());
    });
    ACTIVE_COUNT.fetch_add(1, Ordering::SeqCst);
    decorated.borrow_mut().start_tick();
}

// JAVA: net.minecraft.util.profiling.Profiler#stopUsing
fn stop_using() {
    let profiler = ACTIVE.with(|a| a.borrow_mut().take());
    let Some(profiler) = profiler else {
        panic!("Profiler was not active");
    };
    ACTIVE_COUNT.fetch_sub(1, Ordering::SeqCst);
    profiler.borrow_mut().end_tick();
}

// JAVA: net.minecraft.util.profiling.Profiler#decorateFiller
fn decorate_filler(profiler: SharedProfilerFiller) -> SharedProfilerFiller {
    Rc::new(RefCell::new(CombinedProfilerFiller {
        first: get_default_filler(),
        second: profiler,
    }))
}

// JAVA: net.minecraft.util.profiling.Profiler#get
pub fn get() -> SharedProfilerFiller {
    if ACTIVE_COUNT.load(Ordering::SeqCst) == 0 {
        return get_default_filler();
    }
    ACTIVE.with(|a| a.borrow().clone().unwrap_or_else(get_default_filler))
}

// JAVA: net.minecraft.util.profiling.Profiler#getDefaultFiller
fn get_default_filler() -> SharedProfilerFiller {
    if TracyClient::is_available() {
        TRACY_FILLER.with(|t| t.clone())
    } else {
        Rc::new(RefCell::new(InactiveProfiler))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct CountingFiller {
        starts: Rc<Cell<i32>>,
        ends: Rc<Cell<i32>>,
    }

    impl ProfilerFiller for CountingFiller {
        fn start_tick(&mut self) {
            self.starts.set(self.starts.get().wrapping_add(1));
        }
        fn end_tick(&mut self) {
            self.ends.set(self.ends.get().wrapping_add(1));
        }
    }

    fn counting_filler() -> (SharedProfilerFiller, Rc<Cell<i32>>, Rc<Cell<i32>>) {
        let starts = Rc::new(Cell::new(0));
        let ends = Rc::new(Cell::new(0));
        let filler = Rc::new(RefCell::new(CountingFiller {
            starts: starts.clone(),
            ends: ends.clone(),
        }));
        (filler, starts, ends)
    }

    #[test]
    fn get_returns_default_when_inactive() {
        let p = get();
        p.borrow_mut().start_tick();
        p.borrow_mut().end_tick();
    }

    #[test]
    fn use_starts_and_stops_profiler() {
        let (filler, starts, ends) = counting_filler();
        {
            let _scope = use_profiler(filler);
            assert_eq!(starts.get(), 1);
            assert_eq!(ends.get(), 0);
        }
        assert_eq!(ends.get(), 1);
    }

    #[test]
    fn double_start_panics() {
        let (f1, _, _) = counting_filler();
        let _scope = use_profiler(f1);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (f2, _, _) = counting_filler();
            let _ = use_profiler(f2);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn stop_without_start_panics() {
        let result = std::panic::catch_unwind(|| {
            stop_using();
        });
        assert!(result.is_err());
    }

    #[test]
    fn scope_close_stops_once() {
        let (filler, starts, ends) = counting_filler();
        let scope = use_profiler(filler);
        assert_eq!(starts.get(), 1);
        scope.close();
        assert_eq!(ends.get(), 1);
    }

    #[test]
    fn get_returns_active_when_set() {
        let (filler, starts, _) = counting_filler();
        let _scope = use_profiler(filler);
        let active = get();
        active.borrow_mut().start_tick();
        // decorated filler includes our counting filler as second
        assert!(starts.get() >= 2);
    }
}
