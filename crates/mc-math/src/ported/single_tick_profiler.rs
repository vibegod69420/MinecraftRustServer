use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;

pub trait ProfilerFiller {
    fn start_tick(&mut self);
    fn end_tick(&mut self);
    fn get_results(&self) -> ProfileResults;
    fn is_inactive_collector(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ProfileResults {
    nano_duration: i64,
}

impl ProfileResults {
    pub fn get_nano_duration(&self) -> i64 {
        self.nano_duration
    }

    pub fn save_results(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, format!("duration_ns={}\n", self.nano_duration))
    }
}

#[derive(Debug, Default)]
pub struct InactiveProfiler;

impl ProfilerFiller for InactiveProfiler {
    fn start_tick(&mut self) {}
    fn end_tick(&mut self) {}
    fn get_results(&self) -> ProfileResults {
        ProfileResults { nano_duration: 0 }
    }
    fn is_inactive_collector(&self) -> bool {
        true
    }
}

pub struct ActiveProfiler {
    real_time: Box<dyn Fn() -> i64>,
    _tick: Box<dyn Fn() -> i32>,
    _should_save: Box<dyn Fn() -> bool>,
    start_nanos: i64,
    end_nanos: i64,
}

impl ActiveProfiler {
    pub fn new(
        real_time: Box<dyn Fn() -> i64>,
        tick: Box<dyn Fn() -> i32>,
        should_save: Box<dyn Fn() -> bool>,
    ) -> Self {
        Self {
            real_time,
            _tick: tick,
            _should_save: should_save,
            start_nanos: 0,
            end_nanos: 0,
        }
    }
}

impl ProfilerFiller for ActiveProfiler {
    fn start_tick(&mut self) {
        self.start_nanos = (self.real_time)();
    }

    fn end_tick(&mut self) {
        self.end_nanos = (self.real_time)();
    }

    fn get_results(&self) -> ProfileResults {
        ProfileResults {
            nano_duration: self.end_nanos.saturating_sub(self.start_nanos),
        }
    }

    fn is_inactive_collector(&self) -> bool {
        false
    }
}

struct CombinedProfilerFiller {
    first: Rc<RefCell<dyn ProfilerFiller>>,
    second: Rc<RefCell<dyn ProfilerFiller>>,
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

    fn get_results(&self) -> ProfileResults {
        ProfileResults { nano_duration: 0 }
    }

    fn is_inactive_collector(&self) -> bool {
        false
    }
}

fn combine_fillers(
    a: Rc<RefCell<dyn ProfilerFiller>>,
    b: Rc<RefCell<dyn ProfilerFiller>>,
) -> Rc<RefCell<dyn ProfilerFiller>> {
    Rc::new(RefCell::new(CombinedProfilerFiller { first: a, second: b }))
}

pub struct SharedConstants;

impl SharedConstants {
    pub const DEBUG_MONITOR_TICK_TIMES: bool = false;
    pub const MAXIMUM_TICK_TIME_NANOS: i64 = 0;
}

pub fn time_source() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64
}

fn get_filename_formatted_date_time() -> String {
    let nanos = time_source();
    format!("t{nanos}")
}

pub struct SingleTickProfiler {
    real_time: Rc<dyn Fn() -> i64>,
    save_threshold: i64,
    tick: Rc<Cell<i32>>,
    location: PathBuf,
    profiler: Rc<RefCell<dyn ProfilerFiller>>,
}

impl SingleTickProfiler {
    // JAVA: net.minecraft.util.profiling.SingleTickProfiler#<init>
    pub fn new(real_time: Rc<dyn Fn() -> i64>, name: String, save_threshold: i64) -> Self {
        Self {
            real_time,
            save_threshold,
            tick: Rc::new(Cell::new(0)),
            location: PathBuf::from("debug").join(name),
            profiler: Rc::new(RefCell::new(InactiveProfiler)),
        }
    }

    // JAVA: net.minecraft.util.profiling.SingleTickProfiler#startTick
    pub fn start_tick(&mut self) -> Rc<RefCell<dyn ProfilerFiller>> {
        let rt = self.real_time.clone();
        let tick = self.tick.clone();
        let p: Rc<RefCell<dyn ProfilerFiller>> = Rc::new(RefCell::new(ActiveProfiler::new(
            Box::new(move || rt()),
            Box::new(move || tick.get()),
            Box::new(|| true),
        )));
        self.profiler = p.clone();
        self.tick.set(self.tick.get().wrapping_add(1));
        p
    }

    // JAVA: net.minecraft.util.profiling.SingleTickProfiler#endTick
    pub fn end_tick(&mut self) {
        if !self.profiler.borrow().is_inactive_collector() {
            let results = self.profiler.borrow().get_results();
            self.profiler = Rc::new(RefCell::new(InactiveProfiler));
            if results.get_nano_duration() >= self.save_threshold {
                let filename = format!(
                    "tick-results-{}.txt",
                    get_filename_formatted_date_time()
                );
                let path = self.location.join(filename);
                let _ = results.save_results(&path);
                eprintln!("Recorded long tick -- wrote info to: {}", path.display());
            }
        }
    }

    // JAVA: net.minecraft.util.profiling.SingleTickProfiler#createTickProfiler
    pub fn create_tick_profiler(name: &str) -> Option<SingleTickProfiler> {
        if SharedConstants::DEBUG_MONITOR_TICK_TIMES {
            Some(SingleTickProfiler::new(
                Rc::new(time_source),
                name.to_string(),
                SharedConstants::MAXIMUM_TICK_TIME_NANOS,
            ))
        } else {
            None
        }
    }

    // JAVA: net.minecraft.util.profiling.SingleTickProfiler#decorateFiller
    pub fn decorate_filler(
        filler: Rc<RefCell<dyn ProfilerFiller>>,
        single_tick: Option<&mut SingleTickProfiler>,
    ) -> Rc<RefCell<dyn ProfilerFiller>> {
        match single_tick {
            Some(stp) => combine_fillers(stp.start_tick(), filler),
            None => filler,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_tick_increments_tick() {
        let tick_val = Rc::new(Cell::new(0));
        let mut p = SingleTickProfiler {
            real_time: Rc::new(|| 0),
            save_threshold: i64::MAX,
            tick: tick_val.clone(),
            location: PathBuf::from("debug/test"),
            profiler: Rc::new(RefCell::new(InactiveProfiler)),
        };
        let _ = p.start_tick();
        assert_eq!(tick_val.get(), 1);
        let _ = p.start_tick();
        assert_eq!(tick_val.get(), 2);
    }

    #[test]
    fn tick_wraps_on_overflow() {
        let tick_val = Rc::new(Cell::new(i32::MAX));
        let mut p = SingleTickProfiler {
            real_time: Rc::new(|| 0),
            save_threshold: i64::MAX,
            tick: tick_val.clone(),
            location: PathBuf::from("debug/test"),
            profiler: Rc::new(RefCell::new(InactiveProfiler)),
        };
        let _ = p.start_tick();
        assert_eq!(tick_val.get(), i32::MIN);
    }

    #[test]
    fn end_tick_noop_when_inactive() {
        let mut p = SingleTickProfiler {
            real_time: Rc::new(|| 0),
            save_threshold: 0,
            tick: Rc::new(Cell::new(0)),
            location: std::env::temp_dir().join("mc-math-stp-noop"),
            profiler: Rc::new(RefCell::new(InactiveProfiler)),
        };
        p.end_tick();
    }

    #[test]
    fn end_tick_saves_when_over_threshold() {
        let time = Rc::new(Cell::new(0i64));
        let time_clone = time.clone();
        let tmp = std::env::temp_dir().join(format!(
            "mc-math-stp-save-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&tmp);
        let mut p = SingleTickProfiler {
            real_time: Rc::new(move || time_clone.get()),
            save_threshold: 100,
            tick: Rc::new(Cell::new(0)),
            location: tmp.clone(),
            profiler: Rc::new(RefCell::new(InactiveProfiler)),
        };
        let _ = p.start_tick();
        time.set(200);
        p.profiler.borrow_mut().end_tick();
        p.end_tick();
        let exists = tmp.exists();
        let _ = fs::remove_dir_all(&tmp);
        assert!(exists);
    }

    #[test]
    fn decorate_filler_none_returns_original() {
        let filler: Rc<RefCell<dyn ProfilerFiller>> =
            Rc::new(RefCell::new(InactiveProfiler));
        let result = SingleTickProfiler::decorate_filler(filler.clone(), None);
        assert!(Rc::ptr_eq(&filler, &result));
    }

    #[test]
    fn decorate_filler_some_starts_tick() {
        let tick_val = Rc::new(Cell::new(5));
        let mut stp = SingleTickProfiler {
            real_time: Rc::new(|| 0),
            save_threshold: i64::MAX,
            tick: tick_val.clone(),
            location: PathBuf::from("debug/test"),
            profiler: Rc::new(RefCell::new(InactiveProfiler)),
        };
        let filler: Rc<RefCell<dyn ProfilerFiller>> =
            Rc::new(RefCell::new(InactiveProfiler));
        let _combined = SingleTickProfiler::decorate_filler(filler, Some(&mut stp));
        assert_eq!(tick_val.get(), 6);
    }
}
