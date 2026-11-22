//! Method 2 RNG call-trace instrumentation matching the Java oracle trace format.

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RngTraceEvent {
    pub seq: u64,
    pub stream_id: u64,
    pub impl_name: &'static str,
    pub method: &'static str,
    pub arg: i64,
    pub result: i64,
}

pub trait RngTracer: Send + Sync {
    fn record(&self, event: RngTraceEvent);
}

#[derive(Debug, Default)]
pub struct VecRngTracer {
    seq: AtomicU64,
    events: std::sync::Mutex<Vec<RngTraceEvent>>,
}

impl VecRngTracer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> Vec<RngTraceEvent> {
        self.events.lock().expect("tracer lock poisoned").clone()
    }

    pub fn clear(&self) {
        self.events.lock().expect("tracer lock poisoned").clear();
        self.seq.store(0, Ordering::Relaxed);
    }
}

impl RngTracer for VecRngTracer {
    fn record(&self, mut event: RngTraceEvent) {
        event.seq = self.seq.fetch_add(1, Ordering::Relaxed) + 1;
        self.events.lock().expect("tracer lock poisoned").push(event);
    }
}
