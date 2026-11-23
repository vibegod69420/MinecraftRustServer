use std::collections::BTreeMap;
use std::time::Instant;

const UNKNOWN_THREAD: &str = "unknown";

#[derive(Clone, Debug)]
pub struct RecordedThread {
    pub java_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RecordedEvent {
    pub start_time: Instant,
    pub thread: Option<RecordedThread>,
    pub allocated: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThreadAllocationStat {
    pub timestamp: Instant,
    pub thread_name: String,
    pub total_bytes: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub allocations_per_second_by_thread: BTreeMap<String, f64>,
}

impl ThreadAllocationStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.ThreadAllocationStat#from
    pub fn from(event: &RecordedEvent) -> Self {
        let thread_name = match &event.thread {
            Some(t) => t
                .java_name
                .clone()
                .unwrap_or_else(|| UNKNOWN_THREAD.to_string()),
            None => UNKNOWN_THREAD.to_string(),
        };
        Self {
            timestamp: event.start_time,
            thread_name,
            total_bytes: event.allocated,
        }
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.ThreadAllocationStat#summary
    pub fn summary(stats: &[ThreadAllocationStat]) -> Summary {
        let mut groups: BTreeMap<String, Vec<&ThreadAllocationStat>> = BTreeMap::new();
        for stat in stats {
            groups
                .entry(stat.thread_name.clone())
                .or_default()
                .push(stat);
        }

        let mut allocations_per_second_by_thread: BTreeMap<String, f64> = BTreeMap::new();
        for (name, group) in &groups {
            if group.len() >= 2 {
                let first = group[0];
                let last = group[group.len() - 1];
                let seconds = duration_seconds_between(&first.timestamp, &last.timestamp);
                let bytes = last.total_bytes.wrapping_sub(first.total_bytes);
                allocations_per_second_by_thread
                    .insert(name.clone(), (bytes as f64) / (seconds as f64));
            }
        }

        Summary {
            allocations_per_second_by_thread,
        }
    }
}

fn duration_seconds_between(start: &Instant, end: &Instant) -> i64 {
    if *end >= *start {
        end.duration_since(*start).as_secs() as i64
    } else {
        -((start.duration_since(*end).as_secs()) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn from_known_and_unknown_threads() {
        let t = Instant::now();
        let known = RecordedEvent {
            start_time: t,
            thread: Some(RecordedThread {
                java_name: Some("main".to_string()),
            }),
            allocated: 1024,
        };
        let stat = ThreadAllocationStat::from(&known);
        assert_eq!(stat.thread_name, "main");
        assert_eq!(stat.total_bytes, 1024);
        assert_eq!(stat.timestamp, t);

        let unknown_null_thread = RecordedEvent {
            start_time: t,
            thread: None,
            allocated: 0,
        };
        assert_eq!(
            ThreadAllocationStat::from(&unknown_null_thread).thread_name,
            "unknown"
        );

        let unknown_null_name = RecordedEvent {
            start_time: t,
            thread: Some(RecordedThread { java_name: None }),
            allocated: 0,
        };
        assert_eq!(
            ThreadAllocationStat::from(&unknown_null_name).thread_name,
            "unknown"
        );
    }

    #[test]
    fn summary_basic_allocation_rate() {
        let t0 = Instant::now();
        let stats = vec![
            ThreadAllocationStat {
                timestamp: t0,
                thread_name: "worker".to_string(),
                total_bytes: 100,
            },
            ThreadAllocationStat {
                timestamp: t0 + Duration::from_secs(10),
                thread_name: "worker".to_string(),
                total_bytes: 1100,
            },
        ];
        let summary = ThreadAllocationStat::summary(&stats);
        let rate = summary
            .allocations_per_second_by_thread
            .get("worker")
            .copied()
            .unwrap();
        assert!((rate - 100.0).abs() < 1e-9);
    }

    #[test]
    fn summary_ignores_single_sample() {
        let t0 = Instant::now();
        let stats = vec![ThreadAllocationStat {
            timestamp: t0,
            thread_name: "solo".to_string(),
            total_bytes: 100,
        }];
        let summary = ThreadAllocationStat::summary(&stats);
        assert!(summary.allocations_per_second_by_thread.is_empty());
    }

    #[test]
    fn summary_wrapping_byte_delta() {
        let t0 = Instant::now();
        let stats = vec![
            ThreadAllocationStat {
                timestamp: t0,
                thread_name: "a".to_string(),
                total_bytes: i64::MAX,
            },
            ThreadAllocationStat {
                timestamp: t0 + Duration::from_secs(2),
                thread_name: "a".to_string(),
                total_bytes: i64::MIN,
            },
        ];
        let summary = ThreadAllocationStat::summary(&stats);
        let rate = summary
            .allocations_per_second_by_thread
            .get("a")
            .copied()
            .unwrap();
        let expected = (i64::MIN.wrapping_sub(i64::MAX) as f64) / 2.0;
        assert_eq!(rate.to_bits(), expected.to_bits());
    }

    #[test]
    fn summary_zero_duration_produces_infinity() {
        let t0 = Instant::now();
        let stats = vec![
            ThreadAllocationStat {
                timestamp: t0,
                thread_name: "a".to_string(),
                total_bytes: 0,
            },
            ThreadAllocationStat {
                timestamp: t0,
                thread_name: "a".to_string(),
                total_bytes: 100,
            },
        ];
        let summary = ThreadAllocationStat::summary(&stats);
        let rate = summary
            .allocations_per_second_by_thread
            .get("a")
            .copied()
            .unwrap();
        assert!(rate.is_infinite() && rate.is_sign_positive());
    }
}
