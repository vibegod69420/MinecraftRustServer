use std::time::{SystemTime, UNIX_EPOCH};

/// Minimal month/day value matching Java `java.time.MonthDay` usage in `SpecialDates`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthDay {
    pub month: u8,
    pub day: u8,
}

impl MonthDay {
    pub const fn of(month: u8, day: u8) -> Self {
        Self { month, day }
    }
}

pub const HALLOWEEN: MonthDay = MonthDay::of(10, 31);
pub const CHRISTMAS_RANGE: [MonthDay; 3] = [
    MonthDay::of(12, 24),
    MonthDay::of(12, 25),
    MonthDay::of(12, 26),
];
pub const CHRISTMAS: MonthDay = MonthDay::of(12, 24);
pub const NEW_YEAR: MonthDay = MonthDay::of(1, 1);

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    // Howard Hinnant civil_from_days algorithm (proleptic Gregorian calendar).
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m, d)
}

// JAVA: net.minecraft.util.SpecialDates#dayNow
pub fn day_now() -> MonthDay {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = secs / 86_400;
    let (_year, month, day) = civil_from_days(days);
    MonthDay::of(month as u8, day as u8)
}

// JAVA: net.minecraft.util.SpecialDates#isHalloween
pub fn is_halloween() -> bool {
    HALLOWEEN == day_now()
}

// JAVA: net.minecraft.util.SpecialDates#isExtendedChristmas
pub fn is_extended_christmas() -> bool {
    let now = day_now();
    CHRISTMAS_RANGE.iter().any(|d| *d == now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_java_dates() {
        assert_eq!(HALLOWEEN, MonthDay::of(10, 31));
        assert_eq!(CHRISTMAS, MonthDay::of(12, 24));
        assert_eq!(NEW_YEAR, MonthDay::of(1, 1));
        assert_eq!(
            CHRISTMAS_RANGE,
            [
                MonthDay::of(12, 24),
                MonthDay::of(12, 25),
                MonthDay::of(12, 26)
            ]
        );
    }

    #[test]
    fn christmas_range_contains_only_expected_days() {
        assert!(CHRISTMAS_RANGE.contains(&MonthDay::of(12, 24)));
        assert!(CHRISTMAS_RANGE.contains(&MonthDay::of(12, 25)));
        assert!(CHRISTMAS_RANGE.contains(&MonthDay::of(12, 26)));
        assert!(!CHRISTMAS_RANGE.contains(&MonthDay::of(12, 23)));
        assert!(!CHRISTMAS_RANGE.contains(&MonthDay::of(12, 27)));
        assert!(!CHRISTMAS_RANGE.contains(&MonthDay::of(1, 1)));
    }

    #[test]
    fn day_now_returns_valid_month_day() {
        let now = day_now();
        assert!((1..=12).contains(&now.month));
        assert!((1..=31).contains(&now.day));
    }

    #[test]
    fn civil_from_days_known_epoch_and_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(365), (1971, 1, 1)); // 1970 not leap
        assert_eq!(civil_from_days(19631), (2023, 10, 1)); // 2023-10-01
    }

    #[test]
    fn month_day_equality_is_exact() {
        assert_eq!(MonthDay::of(10, 31), MonthDay::of(10, 31));
        assert_ne!(MonthDay::of(10, 31), MonthDay::of(10, 30));
        assert_ne!(MonthDay::of(10, 31), MonthDay::of(11, 31));
    }
}
