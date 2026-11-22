use std::marker::PhantomData;

/// Placeholder for unported `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T: ?Sized>(PhantomData<fn() -> T>);

impl<T: ?Sized> MapCodec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

/// Placeholder for unported `net.minecraft.world.item.enchantment.LevelBasedValue`.
#[derive(Clone)]
pub struct LevelBasedValue {
    calculate_fn: fn(i32) -> f32,
}

impl LevelBasedValue {
    // JAVA: net.minecraft.world.item.enchantment.LevelBasedValue#calculate
    pub fn calculate(&self, level: i32) -> f32 {
        (self.calculate_fn)(level)
    }

    /// Test/construction helper for the unported `LevelBasedValue`.
    pub fn from_fn(calculate_fn: fn(i32) -> f32) -> Self {
        Self { calculate_fn }
    }
}

impl std::fmt::Debug for LevelBasedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LevelBasedValue").finish_non_exhaustive()
    }
}

/// Placeholder for unported loot context parameter key.
#[derive(Debug, Clone, Copy)]
pub struct LootContextParam<T>(PhantomData<T>);

/// Placeholder for unported `net.minecraft.world.level.storage.loot.parameters.LootContextParams`.
pub struct LootContextParams;

impl LootContextParams {
    pub const ENCHANTMENT_LEVEL: LootContextParam<i32> = LootContextParam(PhantomData);
}

/// Placeholder for unported `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub enchantment_level: i32,
}

impl LootContext {
    /// Placeholder for `LootContext#getParameter` specialized to int parameters.
    pub fn get_parameter_int(&self, _param: LootContextParam<i32>) -> i32 {
        self.enchantment_level
    }
}

/// Placeholder for unported `net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootNumberProviderType(pub u32);

/// Placeholder for unported `net.minecraft.world.level.storage.loot.providers.number.NumberProviders`.
pub struct NumberProviders;

impl NumberProviders {
    pub const ENCHANTMENT_LEVEL: LootNumberProviderType = LootNumberProviderType(1);
}

/// Placeholder for unported `net.minecraft.world.level.storage.loot.LootContextUser`.
pub trait LootContextUser {}

// JAVA: java.lang.Math#round
fn java_math_round_f32(a: f32) -> i32 {
    if a.is_nan() {
        return 0;
    }
    if a >= i32::MAX as f32 {
        return i32::MAX;
    }
    if a <= i32::MIN as f32 {
        return i32::MIN;
    }
    let v = (a as f64 + 0.5f64).floor();
    if v >= i32::MAX as f64 {
        i32::MAX
    } else if v <= i32::MIN as f64 {
        i32::MIN
    } else {
        v as i32
    }
}

/// Placeholder for unported `net.minecraft.world.level.storage.loot.providers.number.NumberProvider`.
pub trait NumberProvider: LootContextUser {
    fn get_float(&self, ctx: &LootContext) -> f32;
    fn get_int(&self, ctx: &LootContext) -> i32 {
        java_math_round_f32(self.get_float(ctx))
    }
    fn get_type(&self) -> LootNumberProviderType;
}

/// Port of `net.minecraft.world.level.storage.loot.providers.number.EnchantmentLevelProvider`.
#[derive(Debug, Clone)]
pub struct EnchantmentLevelProvider {
    amount: LevelBasedValue,
}

impl EnchantmentLevelProvider {
    /// Placeholder for unported codec; Java initializes via `RecordCodecBuilder.mapCodec(...)`.
    pub const CODEC: MapCodec<EnchantmentLevelProvider> = MapCodec::placeholder();

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.EnchantmentLevelProvider#<init>
    pub fn new(amount: LevelBasedValue) -> Self {
        Self { amount }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.EnchantmentLevelProvider#amount
    pub fn amount(&self) -> &LevelBasedValue {
        &self.amount
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.EnchantmentLevelProvider#getFloat
    pub fn get_float(&self, ctx: &LootContext) -> f32 {
        let i = ctx.get_parameter_int(LootContextParams::ENCHANTMENT_LEVEL);
        self.amount.calculate(i)
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.EnchantmentLevelProvider#getType
    pub fn get_type(&self) -> LootNumberProviderType {
        NumberProviders::ENCHANTMENT_LEVEL
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.EnchantmentLevelProvider#forEnchantmentLevel
    pub fn for_enchantment_level(amount: LevelBasedValue) -> Self {
        Self::new(amount)
    }
}

impl LootContextUser for EnchantmentLevelProvider {}

impl NumberProvider for EnchantmentLevelProvider {
    fn get_float(&self, ctx: &LootContext) -> f32 {
        EnchantmentLevelProvider::get_float(self, ctx)
    }

    fn get_type(&self) -> LootNumberProviderType {
        EnchantmentLevelProvider::get_type(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level_times_two(level: i32) -> f32 {
        level as f32 * 2.0
    }

    fn neg_level(level: i32) -> f32 {
        if level < 0 { -1.5 } else { level as f32 + 0.25 }
    }

    #[test]
    fn get_float_uses_enchantment_level_parameter() {
        let p = EnchantmentLevelProvider::new(LevelBasedValue::from_fn(level_times_two));
        let ctx = LootContext { enchantment_level: 3 };
        assert_eq!(p.get_float(&ctx), 6.0);
    }

    #[test]
    fn get_type_is_enchantment_level() {
        let p = EnchantmentLevelProvider::for_enchantment_level(LevelBasedValue::from_fn(level_times_two));
        assert_eq!(p.get_type(), NumberProviders::ENCHANTMENT_LEVEL);
    }

    #[test]
    fn for_enchantment_level_constructs() {
        let p = EnchantmentLevelProvider::for_enchantment_level(LevelBasedValue::from_fn(neg_level));
        let ctx = LootContext { enchantment_level: -5 };
        assert_eq!(p.get_float(&ctx), -1.5);
    }

    #[test]
    fn negative_and_zero_levels() {
        let p = EnchantmentLevelProvider::new(LevelBasedValue::from_fn(neg_level));
        assert_eq!(p.get_float(&LootContext { enchantment_level: 0 }), 0.25);
        assert_eq!(p.get_float(&LootContext { enchantment_level: 7 }), 7.25);
    }

    #[test]
    fn number_provider_trait_default_get_int() {
        let p = EnchantmentLevelProvider::new(LevelBasedValue::from_fn(|l| l as f32 + 0.5));
        let ctx = LootContext { enchantment_level: 2 };
        // 2.5 rounds to 3 in Java Math.round
        assert_eq!(NumberProvider::get_int(&p, &ctx), 3);
    }

    #[test]
    fn amount_accessor_is_present() {
        let p = EnchantmentLevelProvider::new(LevelBasedValue::from_fn(level_times_two));
        let _ = p.amount();
    }
}
