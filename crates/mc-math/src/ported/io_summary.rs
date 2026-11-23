use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountAndSize {
    pub total_count: i64,
    pub total_size: i64,
}

impl CountAndSize {
    // JAVA: net.minecraft.util.profiling.jfr.stats.IoSummary$CountAndSize#add
    pub fn add(&self, other: CountAndSize) -> CountAndSize {
        CountAndSize {
            total_count: self.total_count.wrapping_add(other.total_count),
            total_size: self.total_size.wrapping_add(other.total_size),
        }
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.IoSummary$CountAndSize#averageSize
    pub fn average_size(&self) -> f32 {
        (self.total_size as f32) / (self.total_count as f32)
    }
}

#[derive(Clone, Debug)]
pub struct IoSummary<T> {
    total_count_and_size: CountAndSize,
    largest_size_contributors: Vec<(T, CountAndSize)>,
    recording_duration: Duration,
}

impl<T> IoSummary<T> {
    // JAVA: net.minecraft.util.profiling.jfr.stats.IoSummary#<init>
    pub fn new(recording_duration: Duration, mut entries: Vec<(T, CountAndSize)>) -> Self {
        let mut total_count_and_size = CountAndSize {
            total_count: 0,
            total_size: 0,
        };
        for (_, cs) in &entries {
            total_count_and_size = total_count_and_size.add(*cs);
        }

        entries.sort_by(|a, b| {
            b.1.total_size
                .cmp(&a.1.total_size)
                .then_with(|| b.1.total_count.cmp(&a.1.total_count))
        });
        entries.truncate(10);

        Self {
            total_count_and_size,
            largest_size_contributors: entries,
            recording_duration,
        }
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.IoSummary#getCountsPerSecond
    pub fn get_counts_per_second(&self) -> f64 {
        (self.total_count_and_size.total_count as f64) / (self.recording_duration.as_secs() as f64)
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.IoSummary#getSizePerSecond
    pub fn get_size_per_second(&self) -> f64 {
        (self.total_count_and_size.total_size as f64) / (self.recording_duration.as_secs() as f64)
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.IoSummary#getTotalCount
    pub fn get_total_count(&self) -> i64 {
        self.total_count_and_size.total_count
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.IoSummary#getTotalSize
    pub fn get_total_size(&self) -> i64 {
        self.total_count_and_size.total_size
    }

    // JAVA: net.minecraft.util.profiling.jfr.stats.IoSummary#largestSizeContributors
    pub fn largest_size_contributors(&self) -> &[(T, CountAndSize)] {
        &self.largest_size_contributors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn basic_summary_totals_and_rates() {
        let entries = vec![
            ("a", CountAndSize { total_count: 2, total_size: 100 }),
            ("b", CountAndSize { total_count: 3, total_size: 300 }),
        ];
        let summary = IoSummary::new(Duration::from_secs(10), entries);
        assert_eq!(summary.get_total_count(), 5);
        assert_eq!(summary.get_total_size(), 400);
        assert!((summary.get_counts_per_second() - 0.5).abs() < 1e-9);
        assert!((summary.get_size_per_second() - 40.0).abs() < 1e-9);
    }

    #[test]
    fn largest_contributors_sorted_and_limited_to_ten() {
        let mut entries = Vec::new();
        for i in 0..12i64 {
            entries.push((
                i,
                CountAndSize {
                    total_count: 1,
                    total_size: i * 10,
                },
            ));
        }
        let summary = IoSummary::new(Duration::from_secs(1), entries);
        let top = summary.largest_size_contributors();
        assert_eq!(top.len(), 10);
        assert_eq!(top[0].1.total_size, 110);
        assert_eq!(top[9].1.total_size, 20);
    }

    #[test]
    fn largest_contributors_tie_break_by_count_desc() {
        let entries = vec![
            ("a", CountAndSize { total_count: 1, total_size: 100 }),
            ("b", CountAndSize { total_count: 5, total_size: 100 }),
            ("c", CountAndSize { total_count: 3, total_size: 100 }),
        ];
        let summary = IoSummary::new(Duration::from_secs(1), entries);
        let top = summary.largest_size_contributors();
        assert_eq!(top[0].0, "b");
        assert_eq!(top[1].0, "c");
        assert_eq!(top[2].0, "a");
    }

    #[test]
    fn count_and_size_average_float_bits() {
        let cs = CountAndSize {
            total_count: 4,
            total_size: 10,
        };
        let avg = cs.average_size();
        assert_eq!(avg.to_bits(), 2.5f32.to_bits());
    }

    #[test]
    fn count_and_size_add_wrapping() {
        let a = CountAndSize {
            total_count: i64::MAX,
            total_size: i64::MIN,
        };
        let b = CountAndSize {
            total_count: 1,
            total_size: -1,
        };
        let c = a.add(b);
        assert_eq!(c.total_count, i64::MIN);
        assert_eq!(c.total_size, i64::MAX);
    }

    #[test]
    fn zero_duration_rates_are_infinite() {
        let entries = vec![(
            "a",
            CountAndSize {
                total_count: 10,
                total_size: 100,
            },
        )];
        let summary = IoSummary::new(Duration::from_secs(0), entries);
        assert!(summary.get_counts_per_second().is_infinite());
        assert!(summary.get_size_per_second().is_infinite());
    }
}
