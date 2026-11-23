use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair<A, B>(pub A, pub B);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIoStat {
    pub duration: Duration,
    pub path: Option<String>,
    pub bytes: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub total_bytes: i64,
    pub bytes_per_second: f64,
    pub counts: i64,
    pub counts_per_second: f64,
    pub time_spent_in_io: Duration,
    pub top_ten_contributors_by_total_bytes: Vec<Pair<String, i64>>,
}

impl FileIoStat {
    // JAVA: net.minecraft.util.profiling.jfr.stats.FileIOStat#summary
    pub fn summary(total_duration: Duration, stats: Vec<FileIoStat>) -> Summary {
        let total_bytes: i64 = stats.iter().fold(0i64, |acc, s| acc.wrapping_add(s.bytes));
        let seconds = total_duration.as_secs() as f64;
        let bytes_per_second = (total_bytes as f64) / seconds;
        let counts = stats.len() as i64;
        let counts_per_second = (stats.len() as f64) / seconds;
        let time_spent_in_io = stats.iter().fold(Duration::ZERO, |acc, s| {
            acc.checked_add(s.duration).unwrap_or(Duration::MAX)
        });

        let mut by_path: HashMap<String, i64> = HashMap::new();
        for s in &stats {
            if let Some(path) = &s.path {
                let entry = by_path.entry(path.clone()).or_insert(0i64);
                *entry = entry.wrapping_add(s.bytes);
            }
        }

        let mut top: Vec<(String, i64)> = by_path.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        top.truncate(10);
        let top_ten_contributors_by_total_bytes =
            top.into_iter().map(|(k, v)| Pair(k, v)).collect();

        Summary {
            total_bytes,
            bytes_per_second,
            counts,
            counts_per_second,
            time_spent_in_io,
            top_ten_contributors_by_total_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_empty() {
        let s = FileIoStat::summary(Duration::from_secs(2), Vec::new());
        assert_eq!(s.total_bytes, 0);
        assert_eq!(s.bytes_per_second, 0.0);
        assert_eq!(s.counts, 0);
        assert_eq!(s.counts_per_second, 0.0);
        assert_eq!(s.time_spent_in_io, Duration::ZERO);
        assert!(s.top_ten_contributors_by_total_bytes.is_empty());
    }

    #[test]
    fn summary_aggregates_and_sorts_top_ten() {
        let stats = vec![
            FileIoStat { duration: Duration::from_millis(10), path: Some("a".into()), bytes: 100 },
            FileIoStat { duration: Duration::from_millis(20), path: Some("b".into()), bytes: 300 },
            FileIoStat { duration: Duration::from_millis(5), path: Some("a".into()), bytes: 50 },
            FileIoStat { duration: Duration::from_millis(1), path: None, bytes: 999 },
        ];
        let s = FileIoStat::summary(Duration::from_secs(2), stats);
        assert_eq!(s.total_bytes, 1449);
        assert_eq!(s.bytes_per_second, 724.5);
        assert_eq!(s.counts, 4);
        assert_eq!(s.counts_per_second, 2.0);
        assert_eq!(s.time_spent_in_io, Duration::from_millis(36));
        assert_eq!(s.top_ten_contributors_by_total_bytes.len(), 2);
        assert_eq!(s.top_ten_contributors_by_total_bytes[0], Pair("b".into(), 300));
        assert_eq!(s.top_ten_contributors_by_total_bytes[1], Pair("a".into(), 150));
    }

    #[test]
    fn summary_zero_seconds_yields_infinity_rates() {
        let stats = vec![FileIoStat {
            duration: Duration::from_secs(1),
            path: Some("x".into()),
            bytes: 10,
        }];
        let s = FileIoStat::summary(Duration::from_secs(0), stats);
        assert!(s.bytes_per_second.is_infinite());
        assert!(s.counts_per_second.is_infinite());
        assert_eq!(s.total_bytes, 10);
    }

    #[test]
    fn summary_wraps_bytes_like_java_long() {
        let stats = vec![
            FileIoStat { duration: Duration::ZERO, path: Some("p".into()), bytes: i64::MAX },
            FileIoStat { duration: Duration::ZERO, path: Some("p".into()), bytes: 1 },
        ];
        let s = FileIoStat::summary(Duration::from_secs(1), stats);
        assert_eq!(s.total_bytes, i64::MIN);
        assert_eq!(s.top_ten_contributors_by_total_bytes[0], Pair("p".into(), i64::MIN));
    }

    #[test]
    fn summary_limits_to_top_ten() {
        let mut stats = Vec::new();
        for i in 0..12i64 {
            stats.push(FileIoStat {
                duration: Duration::from_millis(1),
                path: Some(format!("p{i}")),
                bytes: i,
            });
        }
        let s = FileIoStat::summary(Duration::from_secs(1), stats);
        assert_eq!(s.top_ten_contributors_by_total_bytes.len(), 10);
        assert_eq!(s.top_ten_contributors_by_total_bytes[0].1, 11);
        assert_eq!(s.top_ten_contributors_by_total_bytes[9].1, 2);
    }
}
