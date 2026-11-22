use std::collections::BTreeSet;

/// Self-contained faithful port of
/// `net.minecraft.world.level.storage.loot.predicates.TimeCheck` (Java 1.21.11).
///
/// `IntRange` and loot context are modeled locally for standalone compilation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextKey(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemConditionType {
    TimeCheck,
}

/// Minimal context surface required by `TimeCheck`.
pub trait TimeCheckContext {
    fn level_day_time(&self) -> i64;
}

/// Minimal faithful `IntRange` test surface used by `TimeCheck`.
/// Java `TimeCheck` calls `value.getReferencedContextParams()` and `value.test(ctx, (int)time)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntRange {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl IntRange {
    pub const fn new(min: Option<i32>, max: Option<i32>) -> Self {
        Self { min, max }
    }

    // JAVA: net.minecraft.world.level.storage.loot.IntRange#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        BTreeSet::new()
    }

    // JAVA: net.minecraft.world.level.storage.loot.IntRange#test(LootContext,int)
    pub fn test<C: TimeCheckContext>(&self, _context: &C, value: i32) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        true
    }
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeCheck {
    period: Option<i64>,
    value: IntRange,
}

impl TimeCheck {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck#<init>
    pub fn new(period: Option<i64>, value: IntRange) -> Self {
        Self { period, value }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::TimeCheck
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        self.value.get_referenced_context_params()
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck#test
    pub fn test<C: TimeCheckContext>(&self, context: &C) -> bool {
        let mut time = context.level_day_time();
        if let Some(period) = self.period {
            // Java long remainder semantics: result has same sign as dividend.
            // For normal Minecraft day times this is non-negative, but we preserve semantics.
            time %= period;
        }
        self.value.test(context, time as i32)
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck#time
    pub fn time(value: IntRange) -> Builder {
        Builder::new(value)
    }

    pub fn period(&self) -> Option<i64> {
        self.period
    }

    pub fn value(&self) -> IntRange {
        self.value
    }
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck$Builder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Builder {
    period: Option<i64>,
    value: IntRange,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck$Builder#<init>
    pub fn new(value: IntRange) -> Self {
        Self {
            period: None,
            value,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck$Builder#setPeriod
    pub fn set_period(mut self, period: i64) -> Self {
        self.period = Some(period);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.TimeCheck$Builder#build
    pub fn build(self) -> TimeCheck {
        TimeCheck::new(self.period, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::{ContextKey, IntRange, LootItemConditionType, TimeCheck, TimeCheckContext};
    use std::collections::BTreeSet;

    #[derive(Debug, Clone, Copy)]
    struct TestContext {
        day_time: i64,
    }

    impl TimeCheckContext for TestContext {
        fn level_day_time(&self) -> i64 {
            self.day_time
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct RangeWithRef {
        min: Option<i32>,
        max: Option<i32>,
    }

    impl RangeWithRef {
        fn as_int_range(self) -> IntRange {
            IntRange::new(self.min, self.max)
        }
    }

    #[test]
    fn no_period_uses_raw_day_time() {
        let cond = TimeCheck::time(IntRange::new(Some(100), Some(200))).build();
        assert_eq!(cond.get_type(), LootItemConditionType::TimeCheck);
        assert!(cond.test(&TestContext { day_time: 150 }));
        assert!(!cond.test(&TestContext { day_time: 99 }));
        assert!(!cond.test(&TestContext { day_time: 201 }));
    }

    #[test]
    fn period_applies_java_long_remainder() {
        let cond = TimeCheck::time(IntRange::new(Some(0), Some(10)))
            .set_period(24000)
            .build();
        assert!(cond.test(&TestContext { day_time: 24005 }));
        assert!(!cond.test(&TestContext { day_time: 24015 }));
    }

    #[test]
    fn negative_day_time_preserves_remainder_sign() {
        let cond = TimeCheck::time(IntRange::new(Some(-5), Some(-1)))
            .set_period(24000)
            .build();
        assert!(cond.test(&TestContext { day_time: -5 }));
        assert!(!cond.test(&TestContext { day_time: 5 }));
    }

    #[test]
    fn referenced_context_params_delegate_to_range() {
        let cond = TimeCheck::new(None, IntRange::new(Some(0), Some(1)));
        let refs: BTreeSet<ContextKey> = cond.get_referenced_context_params();
        assert!(refs.is_empty());

        let _range = RangeWithRef {
            min: Some(0),
            max: Some(1),
        }
        .as_int_range();
        assert!(TimeCheck::time(_range)
            .build()
            .get_referenced_context_params()
            .is_empty());
    }

    #[test]
    fn int_cast_matches_java_narrowing_for_large_values() {
        // Java casts long to int with truncation; Rust `as i32` does the same wrapping truncation.
        let cond = TimeCheck::time(IntRange::new(Some(-1), Some(-1))).build();
        let large = (i32::MAX as i64) + 2; // 2147483649, as i32 == -2147483647? actually MAX+2 wraps
        let expected = large as i32;
        assert_eq!(expected, i32::MIN.wrapping_add(1));
        let cond2 = TimeCheck::time(IntRange::new(Some(expected), Some(expected))).build();
        assert!(cond2.test(&TestContext { day_time: large }));
        assert!(!cond.test(&TestContext { day_time: large }));
    }
}
