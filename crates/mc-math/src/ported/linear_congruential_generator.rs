const MULTIPLIER: i64 = 6_364_136_223_846_793_005i64;
const INCREMENT: i64 = 1_442_695_040_888_963_407i64;

// JAVA: net.minecraft.util.LinearCongruentialGenerator#next
pub fn next(mut value: i64, addend: i64) -> i64 {
    value = value.wrapping_mul(
        value
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(INCREMENT),
    );
    value.wrapping_add(addend)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_zero_value_returns_addend() {
        assert_eq!(next(0, 0), 0);
        assert_eq!(next(0, 5), 5);
        assert_eq!(next(0, -1), -1);
    }

    #[test]
    fn next_one_zero() {
        // 1 * (1 * MULTIPLIER + INCREMENT) + 0
        assert_eq!(next(1, 0), 7_806_831_264_735_756_412i64);
    }

    #[test]
    fn next_wrapping_behavior() {
        // 2 * (2 * MULTIPLIER + INCREMENT) + 1, computed with Java i64 wrapping
        assert_eq!(next(2, 1), -8_551_553_170_254_004_397i64);
    }

    #[test]
    fn next_negative_inputs() {
        // -1 * (-1 * MULTIPLIER + INCREMENT) + 0
        assert_eq!(next(-1, 0), 4_921_441_182_957_829_598i64);
    }

    #[test]
    fn next_extreme_values_no_panic() {
        let a = next(i64::MAX, i64::MIN);
        let b = next(i64::MIN, i64::MAX);
        // Deterministic wrapping results; verify consistency.
        assert_eq!(a, next(i64::MAX, i64::MIN));
        assert_eq!(b, next(i64::MIN, i64::MAX));
        assert_ne!(a, b);
    }
}
