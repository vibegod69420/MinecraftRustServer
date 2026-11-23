use std::time::{Duration, SystemTime};

/// Minimal view over a JFR `RecordedEvent` for `TickTimeStat` extraction.
pub trait TickTimeEvent {
    fn get_start_time(&self) -> SystemTime;
    fn get_duration(&self, name: &str) -> Duration;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickTimeStat {
    pub timestamp: SystemTime,
    pub current_average: Duration,
}

impl TickTimeStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.TickTimeStat#from
    pub fn from<E: TickTimeEvent + ?Sized>(event: &E) -> Self {
        Self {
            timestamp: event.get_start_time(),
            current_average: event.get_duration("averageTickDuration"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    struct MockEvent {
        start: SystemTime,
        avg: Duration,
    }

    impl TickTimeEvent for MockEvent {
        fn get_start_time(&self) -> SystemTime {
            self.start
        }

        fn get_duration(&self, name: &str) -> Duration {
            assert_eq!(name, "averageTickDuration");
            self.avg
        }
    }

    #[test]
    fn from_basic_values() {
        let ts = UNIX_EPOCH + Duration::from_millis(1_234);
        let avg = Duration::from_micros(50_000);
        let stat = TickTimeStat::from(&MockEvent { start: ts, avg });
        assert_eq!(stat.timestamp, ts);
        assert_eq!(stat.current_average, avg);
    }

    #[test]
    fn from_zero_duration() {
        let stat = TickTimeStat::from(&MockEvent {
            start: UNIX_EPOCH,
            avg: Duration::ZERO,
        });
        assert_eq!(stat.timestamp, UNIX_EPOCH);
        assert_eq!(stat.current_average, Duration::ZERO);
    }

    #[test]
    fn from_large_duration() {
        let avg = Duration::new(u64::MAX / 2, 999_999_999);
        let stat = TickTimeStat::from(&MockEvent {
            start: UNIX_EPOCH,
            avg,
        });
        assert_eq!(stat.current_average, avg);
    }

    #[test]
    fn from_pre_epoch_timestamp() {
        let ts = UNIX_EPOCH
            .checked_sub(Duration::from_secs(10))
            .expect("pre-epoch timestamp");
        let stat = TickTimeStat::from(&MockEvent {
            start: ts,
            avg: Duration::from_nanos(1),
        });
        assert_eq!(stat.timestamp, ts);
        assert_eq!(stat.current_average, Duration::from_nanos(1));
    }
}
