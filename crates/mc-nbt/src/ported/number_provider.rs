/// Placeholder for unported `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext;

/// Placeholder for unported `net.minecraft.world.level.storage.loot.LootContextUser`.
pub trait LootContextUser {}

/// Placeholder for unported `net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootNumberProviderType(pub u32);

// JAVA: java.lang.Math#round
fn java_math_round_f32(a: f32) -> i32 {
    // Java Math.round(float): closest int, ties to +Infinity.
    // NaN -> 0, <= Integer.MIN_VALUE -> MIN, >= Integer.MAX_VALUE -> MAX.
    if a.is_nan() {
        return 0;
    }
    if a >= i32::MAX as f32 {
        return i32::MAX;
    }
    if a <= i32::MIN as f32 {
        return i32::MIN;
    }
    // (int)Math.floor(a + 0.5f) in Java float arithmetic; use f64 to preserve the f32 value.
    let v = (a as f64 + 0.5f64).floor();
    if v >= i32::MAX as f64 {
        i32::MAX
    } else if v <= i32::MIN as f64 {
        i32::MIN
    } else {
        v as i32
    }
}

/// Port of `net.minecraft.world.level.storage.loot.providers.number.NumberProvider`.
pub trait NumberProvider: LootContextUser {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getFloat
    fn get_float(&self, ctx: &LootContext) -> f32;

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getInt
    fn get_int(&self, ctx: &LootContext) -> i32 {
        java_math_round_f32(self.get_float(ctx))
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getType
    fn get_type(&self) -> LootNumberProviderType;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedProvider(f32);
    impl LootContextUser for FixedProvider {}
    impl NumberProvider for FixedProvider {
        fn get_float(&self, _ctx: &LootContext) -> f32 {
            self.0
        }
        fn get_type(&self) -> LootNumberProviderType {
            LootNumberProviderType(0)
        }
    }

    #[test]
    fn get_int_rounds_positive_and_negative() {
        let ctx = LootContext;
        assert_eq!(FixedProvider(2.5f32).get_int(&ctx), 3);
        assert_eq!(FixedProvider(2.4f32).get_int(&ctx), 2);
        assert_eq!(FixedProvider(-2.5f32).get_int(&ctx), -2); // Java ties to +Infinity
        assert_eq!(FixedProvider(-2.6f32).get_int(&ctx), -3);
    }

    #[test]
    fn get_int_handles_nan_and_infinities() {
        let ctx = LootContext;
        assert_eq!(FixedProvider(f32::NAN).get_int(&ctx), 0);
        assert_eq!(FixedProvider(f32::INFINITY).get_int(&ctx), i32::MAX);
        assert_eq!(FixedProvider(f32::NEG_INFINITY).get_int(&ctx), i32::MIN);
    }

    #[test]
    fn get_int_handles_negative_zero_and_small_values() {
        let ctx = LootContext;
        assert_eq!(FixedProvider(-0.0f32).get_int(&ctx), 0);
        assert_eq!(FixedProvider(0.0f32).get_int(&ctx), 0);
        assert_eq!(FixedProvider(-0.1f32).get_int(&ctx), 0);
        assert_eq!(FixedProvider(-0.9f32).get_int(&ctx), -1);
    }

    #[test]
    fn get_int_handles_large_overflow_bounds() {
        let ctx = LootContext;
        assert_eq!(FixedProvider(i32::MAX as f32).get_int(&ctx), i32::MAX);
        assert_eq!(FixedProvider(i32::MIN as f32).get_int(&ctx), i32::MIN);
        assert_eq!(FixedProvider(3_000_000_000.0f32).get_int(&ctx), i32::MAX);
    }

    #[test]
    fn java_math_round_f32_float_bits_edge() {
        // Ensure -0.0 is treated as zero value for rounding.
        let ctx = LootContext;
        let neg_zero = f32::from_bits(0x8000_0000);
        assert_eq!(neg_zero.to_bits(), 0x8000_0000);
        assert_eq!(FixedProvider(neg_zero).get_int(&ctx), 0);
    }
}
