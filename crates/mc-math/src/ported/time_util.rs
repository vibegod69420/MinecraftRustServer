pub const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000i64;
pub const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000i64;
pub const MILLISECONDS_PER_SECOND: i64 = 1_000i64;
pub const SECONDS_PER_HOUR: i64 = 3_600i64;
pub const SECONDS_PER_MINUTE: i32 = 60i32;

/// Minimal self-contained representation of `net.minecraft.util.valueproviders.UniformInt`
/// for use by `TimeUtil::range_of_seconds`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformInt {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl UniformInt {
    // JAVA: net.minecraft.util.valueproviders.UniformInt#of
    pub fn of(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }
}

// JAVA: net.minecraft.util.TimeUtil#rangeOfSeconds
pub fn range_of_seconds(min_seconds: i32, max_seconds: i32) -> UniformInt {
    UniformInt::of(
        min_seconds.wrapping_mul(20),
        max_seconds.wrapping_mul(20),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_java_timeunit_values() {
        assert_eq!(NANOSECONDS_PER_SECOND, 1_000_000_000i64);
        assert_eq!(NANOSECONDS_PER_MILLISECOND, 1_000_000i64);
        assert_eq!(MILLISECONDS_PER_SECOND, 1_000i64);
        assert_eq!(SECONDS_PER_HOUR, 3_600i64);
        assert_eq!(SECONDS_PER_MINUTE, 60i32);
    }

    #[test]
    fn range_of_seconds_scales_by_ticks() {
        assert_eq!(
            range_of_seconds(1, 2),
            UniformInt {
                min_inclusive: 20,
                max_inclusive: 40
            }
        );
        assert_eq!(range_of_seconds(0, 0), UniformInt::of(0, 0));
        assert_eq!(range_of_seconds(-1, 3), UniformInt::of(-20, 60));
    }

    #[test]
    fn range_of_seconds_uses_wrapping_int_math() {
        let result = range_of_seconds(i32::MAX, i32::MIN);
        assert_eq!(result.min_inclusive, i32::MAX.wrapping_mul(20));
        assert_eq!(result.max_inclusive, i32::MIN.wrapping_mul(20));
    }

    #[test]
    fn uniform_int_of_stores_bounds() {
        let u = UniformInt::of(-10, 10);
        assert_eq!(u.min_inclusive, -10);
        assert_eq!(u.max_inclusive, 10);
    }
}
