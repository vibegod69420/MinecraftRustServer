use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

/// JAVA: net.minecraft.util.ThreadingDetector
pub struct ThreadingDetector {
    name: String,
    state: Mutex<DetectorState>,
    acquired: Condvar,
    thread_that_failed_to_acquire: Mutex<Option<thread::ThreadId>>,
    thread_that_failed_to_acquire_name: Mutex<Option<String>>,
    full_exception: Mutex<Option<ThreadingException>>,
    released: AtomicBool,
}

struct DetectorState {
    locked: bool,
}

/// JAVA: net.minecraft.ReportedException (minimal self-contained stand-in)
#[derive(Debug, Clone)]
pub struct ReportedException {
    pub message: String,
    pub report: CrashReport,
}

/// JAVA: net.minecraft.CrashReport (minimal self-contained stand-in)
#[derive(Debug, Clone)]
pub struct CrashReport {
    pub title: String,
    pub exception_message: String,
    pub categories: Vec<CrashReportCategory>,
}

/// JAVA: net.minecraft.CrashReportCategory (minimal self-contained stand-in)
#[derive(Debug, Clone)]
pub struct CrashReportCategory {
    pub title: String,
    pub details: Vec<(String, String)>,
}

impl CrashReport {
    // JAVA: net.minecraft.CrashReport#<init>(String, Throwable)
    pub fn new(title: String, exception_message: String) -> Self {
        Self {
            title,
            exception_message,
            categories: Vec::new(),
        }
    }

    // JAVA: net.minecraft.CrashReport#addCategory(String)
    pub fn add_category(&mut self, title: &str) -> &mut CrashReportCategory {
        self.categories.push(CrashReportCategory {
            title: title.to_string(),
            details: Vec::new(),
        });
        let idx = self.categories.len() - 1;
        &mut self.categories[idx]
    }
}

impl CrashReportCategory {
    // JAVA: net.minecraft.CrashReportCategory#setDetail(String, Object)
    pub fn set_detail(&mut self, key: &str, value: String) {
        self.details.push((key.to_string(), value));
    }
}

pub type ThreadingException = ReportedException;

impl ThreadingDetector {
    // JAVA: net.minecraft.util.ThreadingDetector#<init>(String)
    pub fn new(name: String) -> Self {
        Self {
            name,
            state: Mutex::new(DetectorState { locked: false }),
            acquired: Condvar::new(),
            thread_that_failed_to_acquire: Mutex::new(None),
            thread_that_failed_to_acquire_name: Mutex::new(None),
            full_exception: Mutex::new(None),
            released: AtomicBool::new(false),
        }
    }

    // JAVA: net.minecraft.util.ThreadingDetector#checkAndLock()
    pub fn check_and_lock(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.locked {
            state.locked = true;
            self.released.store(false, Ordering::SeqCst);
            return;
        }

        // Contention: record failing thread and wait for release.
        {
            let mut failed = self.thread_that_failed_to_acquire.lock().unwrap();
            *failed = Some(thread::current().id());
        }
        {
            let mut failed_name = self.thread_that_failed_to_acquire_name.lock().unwrap();
            let current = thread::current();
            *failed_name = Some(current.name().unwrap_or("<unnamed>").to_string());
        }

        while state.locked {
            state = self.acquired.wait(state).unwrap();
        }
        // Acquired after contention; throw the stored exception if present.
        let ex = self.full_exception.lock().unwrap().clone();
        if let Some(e) = ex {
            panic!("{}", format_reported_exception(&e));
        }
        // If no exception stored, take the lock.
        state.locked = true;
        self.released.store(false, Ordering::SeqCst);
    }

    // JAVA: net.minecraft.util.ThreadingDetector#checkAndUnlock()
    pub fn check_and_unlock(&self) {
        let mut state = self.state.lock().unwrap();
        let failed_thread_id = *self.thread_that_failed_to_acquire.lock().unwrap();
        let failed_thread_name = self.thread_that_failed_to_acquire_name.lock().unwrap().clone();

        if let Some(_tid) = failed_thread_id {
            let ex = make_threading_exception(&self.name, failed_thread_name.as_deref());
            {
                let mut full = self.full_exception.lock().unwrap();
                *full = Some(ex.clone());
            }
            state.locked = false;
            self.released.store(true, Ordering::SeqCst);
            self.acquired.notify_all();
            panic!("{}", format_reported_exception(&ex));
        }

        state.locked = false;
        self.released.store(true, Ordering::SeqCst);
        self.acquired.notify_all();
    }
}

// JAVA: net.minecraft.util.ThreadingDetector#makeThreadingException(String, @Nullable Thread)
pub fn make_threading_exception(name: &str, other_thread_name: Option<&str>) -> ReportedException {
    let current = thread::current();
    let current_name = current.name().unwrap_or("<unnamed>").to_string();
    let current_trace = stack_trace(&current_name);

    let mut traces = Vec::new();
    traces.push(current_trace);
    if let Some(other) = other_thread_name {
        // We can't capture another thread's stack in portable Rust; include its name.
        if other != current_name {
            traces.push(format!("{}: \n\tat <stack unavailable>", other));
        }
    }
    let combined = traces.join("\n");
    let msg = format!("Accessing {} from multiple threads", name);
    let mut report = CrashReport::new(msg.clone(), format!("IllegalStateException: {}", msg));
    {
        let cat = report.add_category("Thread dumps");
        cat.set_detail("Thread dumps", combined.clone());
    }
    eprintln!("Thread dumps: \n{}", combined);
    ReportedException {
        message: msg,
        report,
    }
}

// JAVA: net.minecraft.util.ThreadingDetector#stackTrace(Thread)
fn stack_trace(thread_name: &str) -> String {
    let bt = std::backtrace::Backtrace::force_capture();
    let bt_str = format!("{}", bt);
    let frames: String = bt_str
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n\tat ");
    format!("{}: \n\tat {}", thread_name, frames)
}

fn format_reported_exception(e: &ReportedException) -> String {
    let mut out = format!("ReportedException: {}\n", e.message);
    out.push_str(&format!("CrashReport: {}\n", e.report.title));
    out.push_str(&format!("Cause: {}\n", e.report.exception_message));
    for cat in &e.report.categories {
        out.push_str(&format!("Category: {}\n", cat.title));
        for (k, v) in &cat.details {
            out.push_str(&format!("  {}: {}\n", k, v));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn lock_unlock_no_contention() {
        let det = ThreadingDetector::new("test".to_string());
        det.check_and_lock();
        det.check_and_unlock();
        // Should be reusable
        det.check_and_lock();
        det.check_and_unlock();
    }

    #[test]
    fn make_threading_exception_contains_name() {
        let ex = make_threading_exception("myResource", Some("other-thread"));
        assert!(ex.message.contains("Accessing myResource from multiple threads"));
        assert_eq!(ex.report.categories.len(), 1);
        assert_eq!(ex.report.categories[0].title, "Thread dumps");
        let detail = &ex.report.categories[0].details[0];
        assert_eq!(detail.0, "Thread dumps");
        assert!(detail.1.contains("other-thread"));
    }

    #[test]
    fn make_threading_exception_without_other_thread() {
        let ex = make_threading_exception("res", None);
        assert!(ex.message.contains("Accessing res from multiple threads"));
        let detail = &ex.report.categories[0].details[0].1;
        assert!(!detail.is_empty());
    }

    #[test]
    fn contention_triggers_exception() {
        let det = Arc::new(ThreadingDetector::new("contended".to_string()));
        let det2 = Arc::clone(&det);

        det.check_and_lock();

        let handle = thread::spawn(move || {
            // This should contend and eventually panic with ReportedException
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                det2.check_and_lock();
            }));
            assert!(result.is_err());
        });

        // Give the spawned thread time to contend
        thread::sleep(Duration::from_millis(50));
        // Unlocking should detect contention and panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            det.check_and_unlock();
        }));
        assert!(result.is_err());

        let _ = handle.join();
    }

    #[test]
    fn crash_report_category_set_detail() {
        let mut report = CrashReport::new("title".to_string(), "cause".to_string());
        let cat = report.add_category("Cat");
        cat.set_detail("k", "v".to_string());
        assert_eq!(report.categories[0].details[0], ("k".to_string(), "v".to_string()));
    }
}
