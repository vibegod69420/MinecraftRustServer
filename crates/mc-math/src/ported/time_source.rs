#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl TimeUnit {
    fn nanos_per(self) -> i64 {
        match self {
            TimeUnit::Nanoseconds => 1,
            TimeUnit::Microseconds => 1_000,
            TimeUnit::Milliseconds => 1_000_000,
            TimeUnit::Seconds => 1_000_000_000,
            TimeUnit::Minutes => 60_000_000_000,
            TimeUnit::Hours => 3_600_000_000_000,
            TimeUnit::Days => 86_400_000_000_000,
        }
    }

    // JAVA: java.util.concurrent.TimeUnit#convert
    pub fn convert(self, duration: i64, source_unit: TimeUnit) -> i64 {
        let src_nanos = source_unit.nanos_per();
        let dst_nanos = self.nanos_per();
        if src_nanos == dst_nanos {
            duration
        } else if src_nanos < dst_nanos {
            duration / (dst_nanos / src_nanos)
        } else {
            let scale = src_nanos / dst_nanos;
            let max = i64::MAX / scale;
            let min = i64::MIN / scale;
            if duration > max {
                i64::MAX
            } else if duration < min {
                i64::MIN
            } else {
                duration.wrapping_mul(scale)
            }
        }
    }
}

pub trait TimeSource {
    // JAVA: net.minecraft.util.TimeSource#get
    fn get(&self, unit: TimeUnit) -> i64;
}

pub trait NanoTimeSource {
    // JAVA: net.minecraft.util.TimeSource$NanoTimeSource#getAsLong
    fn get_as_long(&self) -> i64;
}

impl<T: NanoTimeSource> TimeSource for T {
    // JAVA: net.minecraft.util.TimeSource$NanoTimeSource#get
    fn get(&self, unit: TimeUnit) -> i64 {
        unit.convert(self.get_as_long(), TimeUnit::Nanoseconds)
    }
}

#[cfg(test)]
mod tests {
    use super::{NanoTimeSource, TimeSource, TimeUnit};

    struct FixedNano(i64);

    impl NanoTimeSource for FixedNano {
        fn get_as_long(&self) -> i64 {
            self.0
        }
    }

    #[test]
    fn nano_time_source_converts_units() {
        let src = FixedNano(1_500_000_000);
        assert_eq!(src.get(TimeUnit::Nanoseconds), 1_500_000_000);
        assert_eq!(src.get(TimeUnit::Microseconds), 1_500_000);
        assert_eq!(src.get(TimeUnit::Milliseconds), 1_500);
        assert_eq!(src.get(TimeUnit::Seconds), 1);
    }

    #[test]
    fn nano_time_source_negative_values() {
        let src = FixedNano(-2_000_000_000);
        assert_eq!(src.get(TimeUnit::Seconds), -2);
        assert_eq!(src.get(TimeUnit::Milliseconds), -2_000);
        assert_eq!(src.get(TimeUnit::Nanoseconds), -2_000_000_000);
    }

    #[test]
    fn convert_saturates_on_overflow() {
        assert_eq!(
            TimeUnit::Nanoseconds.convert(i64::MAX, TimeUnit::Seconds),
            i64::MAX
        );
        assert_eq!(
            TimeUnit::Nanoseconds.convert(i64::MIN, TimeUnit::Seconds),
            i64::MIN
        );
    }

    #[test]
    fn convert_identity_and_larger_units() {
        assert_eq!(
            TimeUnit::Milliseconds.convert(5_000, TimeUnit::Milliseconds),
            5_000
        );
        assert_eq!(TimeUnit::Minutes.convert(120, TimeUnit::Seconds), 2);
        assert_eq!(TimeUnit::Hours.convert(2, TimeUnit::Days), 48);
    }
}
