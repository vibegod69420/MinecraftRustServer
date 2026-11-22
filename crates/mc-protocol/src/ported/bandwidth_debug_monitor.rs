use std::sync::atomic::{AtomicI32, Ordering};

/// Minimal stand-in for `net.minecraft.util.debugchart.LocalSampleLogger`.
pub trait LocalSampleLogger {
    fn log_sample(&self, value: i32);
}

pub struct BandwidthDebugMonitor<L: LocalSampleLogger> {
    bytes_received: AtomicI32,
    bandwidth_logger: L,
}

impl<L: LocalSampleLogger> BandwidthDebugMonitor<L> {
    // JAVA: net.minecraft.network.BandwidthDebugMonitor#<init>
    pub fn new(bandwidth_logger: L) -> Self {
        Self {
            bytes_received: AtomicI32::new(0),
            bandwidth_logger,
        }
    }

    // JAVA: net.minecraft.network.BandwidthDebugMonitor#onReceive
    pub fn on_receive(&self, bytes: i32) {
        self.bytes_received.fetch_add(bytes, Ordering::SeqCst);
    }

    // JAVA: net.minecraft.network.BandwidthDebugMonitor#tick
    pub fn tick(&self) {
        let value = self.bytes_received.swap(0, Ordering::SeqCst);
        self.bandwidth_logger.log_sample(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockLogger {
        samples: Mutex<Vec<i32>>,
    }

    impl MockLogger {
        fn new() -> Self {
            Self {
                samples: Mutex::new(Vec::new()),
            }
        }

        fn values(&self) -> Vec<i32> {
            self.samples.lock().unwrap().clone()
        }
    }

    impl LocalSampleLogger for MockLogger {
        fn log_sample(&self, value: i32) {
            self.samples.lock().unwrap().push(value);
        }
    }

    #[test]
    fn tick_logs_zero_initially() {
        let mon = BandwidthDebugMonitor::new(MockLogger::new());
        mon.tick();
        assert_eq!(mon.bandwidth_logger.values(), vec![0]);
    }

    #[test]
    fn on_receive_accumulates_and_tick_resets() {
        let mon = BandwidthDebugMonitor::new(MockLogger::new());
        mon.on_receive(100);
        mon.on_receive(50);
        mon.tick();
        assert_eq!(mon.bandwidth_logger.values(), vec![150]);
        mon.tick();
        assert_eq!(mon.bandwidth_logger.values(), vec![150, 0]);
    }

    #[test]
    fn on_receive_handles_negative_values() {
        let mon = BandwidthDebugMonitor::new(MockLogger::new());
        mon.on_receive(100);
        mon.on_receive(-30);
        mon.tick();
        assert_eq!(mon.bandwidth_logger.values(), vec![70]);
    }

    #[test]
    fn on_receive_wraps_on_i32_overflow() {
        let mon = BandwidthDebugMonitor::new(MockLogger::new());
        mon.on_receive(i32::MAX);
        mon.on_receive(1);
        mon.tick();
        assert_eq!(mon.bandwidth_logger.values(), vec![i32::MIN]);
    }
}
