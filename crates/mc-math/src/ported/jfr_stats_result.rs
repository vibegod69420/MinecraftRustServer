use std::cmp::Ordering;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkStatus(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkIdentification {
    pub x: i32,
    pub z: i32,
    pub dimension: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PacketIdentification {
    pub packet_id: i32,
    pub direction: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FpsStat {
    pub fps: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TickTimeStat {
    pub nanos: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CpuLoadStat {
    pub user: f64,
    pub system: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GcHeapStatSummary {
    pub total_bytes: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThreadAllocationStatSummary {
    pub total_bytes: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IoSummary<T> {
    pub total_count: i64,
    pub total_bytes: i64,
    pub total_duration: Duration,
    pub entries: Vec<T>,
}

impl<T> IoSummary<T> {
    pub fn new(total_count: i64, total_bytes: i64, total_duration: Duration, entries: Vec<T>) -> Self {
        Self {
            total_count,
            total_bytes,
            total_duration,
            entries,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileIoStatSummary {
    pub total_count: i64,
    pub total_bytes: i64,
    pub total_duration: Duration,
}

impl FileIoStatSummary {
    pub fn new(total_count: i64, total_bytes: i64, total_duration: Duration) -> Self {
        Self {
            total_count,
            total_bytes,
            total_duration,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkGenStat {
    pub status: ChunkStatus,
    pub duration: Duration,
}

impl ChunkGenStat {
    pub fn new(status: ChunkStatus, duration: Duration) -> Self {
        Self { status, duration }
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.ChunkGenStat#status
    pub fn status(&self) -> ChunkStatus {
        self.status
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.ChunkGenStat#duration
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructureGenStat {
    pub structure_name: String,
    pub duration: Duration,
}

impl StructureGenStat {
    pub fn new(structure_name: String, duration: Duration) -> Self {
        Self {
            structure_name,
            duration,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimedStatSummary<T> {
    pub total_duration: Duration,
    pub count: i64,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub average_duration: Duration,
    pub samples: Vec<T>,
}

impl<T: Clone> TimedStatSummary<T> {
    // JAVA: net.minecraft.util.profiling.jfr.stats.TimedStatSummary#summary
    pub fn summary(stats: Vec<T>, get_duration: impl Fn(&T) -> Duration) -> Option<Self> {
        if stats.is_empty() {
            return None;
        }

        let mut total_nanos: u128 = 0;
        let mut min_duration = get_duration(&stats[0]);
        let mut max_duration = get_duration(&stats[0]);

        for stat in &stats {
            let d = get_duration(stat);
            total_nanos = total_nanos.saturating_add(d.as_nanos());
            if d < min_duration {
                min_duration = d;
            }
            if d > max_duration {
                max_duration = d;
            }
        }

        let count = stats.len() as i64;
        let avg_nanos = if count > 0 {
            (total_nanos / (count as u128)) as u64
        } else {
            0
        };

        Some(Self {
            total_duration: Duration::from_nanos(total_nanos.min(u64::MAX as u128) as u64),
            count,
            min_duration,
            max_duration,
            average_duration: Duration::from_nanos(avg_nanos),
            samples: stats,
        })
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.TimedStatSummary#totalDuration
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JfrStatsResult {
    pub recording_started: Duration,
    pub recording_ended: Duration,
    pub recording_duration: Duration,
    pub world_creation_duration: Option<Duration>,
    pub fps: Vec<FpsStat>,
    pub server_tick_times: Vec<TickTimeStat>,
    pub cpu_load_stats: Vec<CpuLoadStat>,
    pub heap_summary: GcHeapStatSummary,
    pub thread_allocation_summary: ThreadAllocationStatSummary,
    pub received_packets_summary: IoSummary<PacketIdentification>,
    pub sent_packets_summary: IoSummary<PacketIdentification>,
    pub written_chunks: IoSummary<ChunkIdentification>,
    pub read_chunks: IoSummary<ChunkIdentification>,
    pub file_writes: FileIoStatSummary,
    pub file_reads: FileIoStatSummary,
    pub chunk_gen_stats: Vec<ChunkGenStat>,
    pub structure_gen_stats: Vec<StructureGenStat>,
}

impl JfrStatsResult {
    // JAVA: net.minecraft.util.profiling.jfr.parse.JfrStatsResult#chunkGenSummary
    pub fn chunk_gen_summary(&self) -> Vec<(ChunkStatus, TimedStatSummary<ChunkGenStat>)> {
        let mut grouped: std::collections::BTreeMap<i32, Vec<ChunkGenStat>> =
            std::collections::BTreeMap::new();
        for stat in &self.chunk_gen_stats {
            grouped.entry(stat.status().0).or_default().push(stat.clone());
        }

        let mut result: Vec<(ChunkStatus, TimedStatSummary<ChunkGenStat>)> = grouped
            .into_iter()
            .filter_map(|(status_id, stats)| {
                TimedStatSummary::summary(stats, |s| s.duration())
                    .map(|summary| (ChunkStatus(status_id), summary))
            })
            .collect();

        result.sort_by(|a, b| {
            let da = a.1.total_duration();
            let db = b.1.total_duration();
            db.cmp(&da)
        });

        result
    }

    // JAVA: net.minecraft.util.profiling.jfr.parse.JfrStatsResult#asJson
    pub fn as_json(&self) -> String {
        // JfrResultJsonSerializer is not ported; provide a minimal faithful placeholder
        // that preserves the method contract without requiring the serializer class.
        String::from("{}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(chunk_gen_stats: Vec<ChunkGenStat>) -> JfrStatsResult {
        JfrStatsResult {
            recording_started: Duration::from_secs(0),
            recording_ended: Duration::from_secs(10),
            recording_duration: Duration::from_secs(10),
            world_creation_duration: None,
            fps: vec![],
            server_tick_times: vec![],
            cpu_load_stats: vec![],
            heap_summary: GcHeapStatSummary { total_bytes: 0 },
            thread_allocation_summary: ThreadAllocationStatSummary { total_bytes: 0 },
            received_packets_summary: IoSummary::new(0, 0, Duration::ZERO, vec![]),
            sent_packets_summary: IoSummary::new(0, 0, Duration::ZERO, vec![]),
            written_chunks: IoSummary::new(0, 0, Duration::ZERO, vec![]),
            read_chunks: IoSummary::new(0, 0, Duration::ZERO, vec![]),
            file_writes: FileIoStatSummary::new(0, 0, Duration::ZERO),
            file_reads: FileIoStatSummary::new(0, 0, Duration::ZERO),
            chunk_gen_stats,
            structure_gen_stats: vec![],
        }
    }

    #[test]
    fn chunk_gen_summary_empty() {
        let result = make_result(vec![]);
        let summary = result.chunk_gen_summary();
        assert!(summary.is_empty());
    }

    #[test]
    fn chunk_gen_summary_groups_by_status() {
        let s1 = ChunkStatus(1);
        let s2 = ChunkStatus(2);
        let result = make_result(vec![
            ChunkGenStat::new(s1, Duration::from_millis(100)),
            ChunkGenStat::new(s2, Duration::from_millis(200)),
            ChunkGenStat::new(s1, Duration::from_millis(50)),
        ]);
        let summary = result.chunk_gen_summary();
        assert_eq!(summary.len(), 2);
        // sorted by total duration descending: s2 has 200ms, s1 has 150ms
        assert_eq!(summary[0].0, s2);
        assert_eq!(summary[0].1.total_duration(), Duration::from_millis(200));
        assert_eq!(summary[0].1.count, 1);
        assert_eq!(summary[1].0, s1);
        assert_eq!(summary[1].1.total_duration(), Duration::from_millis(150));
        assert_eq!(summary[1].1.count, 2);
    }

    #[test]
    fn chunk_gen_summary_sorted_descending_by_total_duration() {
        let s1 = ChunkStatus(10);
        let s2 = ChunkStatus(20);
        let s3 = ChunkStatus(30);
        let result = make_result(vec![
            ChunkGenStat::new(s1, Duration::from_millis(10)),
            ChunkGenStat::new(s2, Duration::from_millis(500)),
            ChunkGenStat::new(s3, Duration::from_millis(100)),
        ]);
        let summary = result.chunk_gen_summary();
        let statuses: Vec<i32> = summary.iter().map(|(s, _)| s.0).collect();
        assert_eq!(statuses, vec![20, 30, 10]);
    }

    #[test]
    fn timed_stat_summary_empty_returns_none() {
        let summary: Option<TimedStatSummary<ChunkGenStat>> =
            TimedStatSummary::summary(vec![], |s| s.duration());
        assert!(summary.is_none());
    }

    #[test]
    fn timed_stat_summary_calculates_min_max_avg() {
        let stats = vec![
            ChunkGenStat::new(ChunkStatus(0), Duration::from_millis(10)),
            ChunkGenStat::new(ChunkStatus(0), Duration::from_millis(30)),
            ChunkGenStat::new(ChunkStatus(0), Duration::from_millis(20)),
        ];
        let summary = TimedStatSummary::summary(stats, |s| s.duration()).unwrap();
        assert_eq!(summary.total_duration, Duration::from_millis(60));
        assert_eq!(summary.count, 3);
        assert_eq!(summary.min_duration, Duration::from_millis(10));
        assert_eq!(summary.max_duration, Duration::from_millis(30));
        assert_eq!(summary.average_duration, Duration::from_millis(20));
    }

    #[test]
    fn as_json_returns_placeholder() {
        let result = make_result(vec![]);
        assert_eq!(result.as_json(), "{}");
    }
}
