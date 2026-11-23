use std::collections::BTreeMap;
use std::time::Duration;

pub trait TimedStat {
    fn duration(&self) -> Duration;
}

fn evaluate_percentiles_nanos(values_nanos: &[i64]) -> BTreeMap<i32, f64> {
    if values_nanos.is_empty() {
        return BTreeMap::new();
    }
    let mut sorted = values_nanos.to_vec();
    sorted.sort_unstable();
    let mut out = BTreeMap::new();
    for &p in &[50_i32, 75, 90, 95, 99] {
        let rank = ((p as f64 / 100.0) * sorted.len() as f64).ceil() as usize;
        let idx = rank.saturating_sub(1).min(sorted.len() - 1);
        out.insert(p, sorted[idx] as f64);
    }
    out
}

#[derive(Debug, Clone)]
pub struct TimedStatSummary<T> {
    pub fastest: T,
    pub slowest: T,
    pub second_slowest: Option<T>,
    pub count: i32,
    pub percentiles_nanos: BTreeMap<i32, f64>,
    pub total_duration: Duration,
}

impl<T: TimedStat + Clone> TimedStatSummary<T> {
    // JAVA: net.minecraft.util.profiling.jfr.stats.TimedStatSummary#summary
    pub fn summary(stats: Vec<T>) -> Option<TimedStatSummary<T>> {
        if stats.is_empty() {
            return None;
        }

        let mut sorted = stats;
        sorted.sort_by(|a, b| a.duration().cmp(&b.duration()));

        let total_duration = sorted.iter().map(|s| s.duration()).fold(Duration::ZERO, |acc, d| {
            acc.checked_add(d).unwrap_or(Duration::MAX)
        });

        let fastest = sorted.first().expect("non-empty").clone();
        let slowest = sorted.last().expect("non-empty").clone();
        let second_slowest = if sorted.len() > 1 {
            Some(sorted[sorted.len() - 2].clone())
        } else {
            None
        };
        let count = sorted.len() as i32;
        let nanos: Vec<i64> = sorted
            .iter()
            .map(|s| s.duration().as_nanos().min(i64::MAX as u128) as i64)
            .collect();
        let percentiles_nanos = evaluate_percentiles_nanos(&nanos);

        Some(TimedStatSummary {
            fastest,
            slowest,
            second_slowest,
            count,
            percentiles_nanos,
            total_duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct S(Duration);
    impl TimedStat for S {
        fn duration(&self) -> Duration {
            self.0
        }
    }

    #[test]
    fn summary_empty_returns_none() {
        let out: Option<TimedStatSummary<S>> = TimedStatSummary::summary(Vec::new());
        assert!(out.is_none());
    }

    #[test]
    fn summary_single_element() {
        let s = TimedStatSummary::summary(vec![S(Duration::from_nanos(123))]).unwrap();
        assert_eq!(s.fastest.0, Duration::from_nanos(123));
        assert_eq!(s.slowest.0, Duration::from_nanos(123));
        assert!(s.second_slowest.is_none());
        assert_eq!(s.count, 1);
        assert_eq!(s.total_duration, Duration::from_nanos(123));
        assert_eq!(s.percentiles_nanos.get(&50), Some(&123.0));
    }

    #[test]
    fn summary_multiple_elements_sorted_and_totals() {
        let s = TimedStatSummary::summary(vec![
            S(Duration::from_millis(30)),
            S(Duration::from_millis(10)),
            S(Duration::from_millis(20)),
        ])
        .unwrap();
        assert_eq!(s.fastest.0, Duration::from_millis(10));
        assert_eq!(s.slowest.0, Duration::from_millis(30));
        assert_eq!(s.second_slowest.as_ref().unwrap().0, Duration::from_millis(20));
        assert_eq!(s.count, 3);
        assert_eq!(s.total_duration, Duration::from_millis(60));
    }

    #[test]
    fn summary_two_elements_has_second_slowest() {
        let s = TimedStatSummary::summary(vec![
            S(Duration::from_secs(5)),
            S(Duration::from_secs(1)),
        ])
        .unwrap();
        assert_eq!(s.fastest.0, Duration::from_secs(1));
        assert_eq!(s.slowest.0, Duration::from_secs(5));
        assert_eq!(s.second_slowest.unwrap().0, Duration::from_secs(1));
        assert_eq!(s.count, 2);
    }

    #[test]
    fn summary_handles_zero_durations() {
        let s = TimedStatSummary::summary(vec![S(Duration::ZERO), S(Duration::ZERO)]).unwrap();
        assert_eq!(s.total_duration, Duration::ZERO);
        assert_eq!(s.percentiles_nanos.get(&99), Some(&0.0));
    }
}
