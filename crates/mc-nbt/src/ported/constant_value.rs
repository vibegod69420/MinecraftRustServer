use std::marker::PhantomData;

/// Placeholder for unported `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T: ?Sized>(PhantomData<fn() -> T>);

impl<T: ?Sized> MapCodec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

/// Placeholder for unported `com.mojang.serialization.Codec`.
#[derive(Debug, Clone, Copy)]
pub struct Codec<T: ?Sized>(PhantomData<fn() -> T>);

impl<T: ?Sized> Codec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

/// Placeholder for unported `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext;

/// Placeholder for unported `net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootNumberProviderType(pub u32);

/// Placeholder for unported `net.minecraft.world.level.storage.loot.providers.number.NumberProviders`.
pub struct NumberProviders;

impl NumberProviders {
    pub const CONSTANT: LootNumberProviderType = LootNumberProviderType(0);
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

// JAVA: java.lang.Float#floatToIntBits
fn java_float_to_int_bits(f: f32) -> i32 {
    if f.is_nan() {
        0x7fc0_0000u32 as i32
    } else {
        f.to_bits() as i32
    }
}

// JAVA: java.lang.Float#compare
fn java_float_compare(a: f32, b: f32) -> i32 {
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    let a_bits = java_float_to_int_bits(a);
    let b_bits = java_float_to_int_bits(b);
    if a_bits == b_bits {
        0
    } else if a_bits < b_bits {
        -1
    } else {
        1
    }
}

/// Port of `net.minecraft.world.level.storage.loot.providers.number.ConstantValue`.
#[derive(Debug, Clone, Copy)]
pub struct ConstantValue {
    value: f32,
}

impl ConstantValue {
    /// Placeholder for unported codec; Java initializes via `RecordCodecBuilder.mapCodec(...)`.
    pub const CODEC: MapCodec<ConstantValue> = MapCodec::placeholder();
    /// Placeholder for unported codec; Java initializes via `Codec.FLOAT.xmap(...)`.
    pub const INLINE_CODEC: Codec<ConstantValue> = Codec::placeholder();

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#<init>
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#value
    pub fn value(&self) -> f32 {
        self.value
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getType
    pub fn get_type(&self) -> LootNumberProviderType {
        NumberProviders::CONSTANT
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getFloat
    pub fn get_float(&self, _ctx: &LootContext) -> f32 {
        self.value
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#exactly
    pub fn exactly(value: f32) -> Self {
        Self::new(value)
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#hashCode
    pub fn hash_code(&self) -> i32 {
        if self.value != 0.0f32 {
            java_float_to_int_bits(self.value)
        } else {
            0
        }
    }
}

impl LootContextUser for ConstantValue {}

impl NumberProvider for ConstantValue {
    fn get_float(&self, ctx: &LootContext) -> f32 {
        ConstantValue::get_float(self, ctx)
    }

    fn get_type(&self) -> LootNumberProviderType {
        ConstantValue::get_type(self)
    }
}

impl PartialEq for ConstantValue {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#equals
    fn eq(&self, other: &Self) -> bool {
        java_float_compare(other.value, self.value) == 0
    }
}

impl Eq for ConstantValue {}

impl std::hash::Hash for ConstantValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash_code().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of<T: Hash>(v: &T) -> u64 {
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        h.finish()
    }

    #[test]
    fn get_float_and_exactly() {
        let ctx = LootContext;
        let c = ConstantValue::exactly(1.25f32);
        assert_eq!(c.get_float(&ctx), 1.25f32);
        assert_eq!(c.value(), 1.25f32);
    }

    #[test]
    fn get_type_is_constant() {
        assert_eq!(ConstantValue::new(0.0).get_type(), NumberProviders::CONSTANT);
    }

    #[test]
    fn equals_distinguishes_negative_zero_and_treats_nans_equal() {
        let pz = ConstantValue::new(0.0f32);
        let nz = ConstantValue::new(f32::from_bits(0x8000_0000));
        assert_ne!(pz, nz);
        assert_eq!(java_float_compare(-0.0f32, 0.0f32), -1);

        let n1 = ConstantValue::new(f32::NAN);
        let n2 = ConstantValue::new(f32::from_bits(0x7fc0_1234)); // another NaN payload
        assert_eq!(n1, n2);
    }

    #[test]
    fn hash_code_matches_java_behavior() {
        assert_eq!(ConstantValue::new(0.0f32).hash_code(), 0);
        assert_eq!(ConstantValue::new(f32::from_bits(0x8000_0000)).hash_code(), 0);
        assert_eq!(ConstantValue::new(1.0f32).hash_code(), 1.0f32.to_bits() as i32);
        assert_eq!(ConstantValue::new(f32::NAN).hash_code(), 0x7fc0_0000u32 as i32);
    }

    #[test]
    fn hash_trait_uses_java_hash_code() {
        let c = ConstantValue::new(42.5f32);
        assert_eq!(hash_of(&c), hash_of(&c.hash_code()));
    }

    #[test]
    fn negative_and_large_float_bits() {
        let neg = ConstantValue::new(-123.75f32);
        assert_eq!(neg.get_float(&LootContext), -123.75f32);
        assert_eq!(neg.hash_code(), (-123.75f32).to_bits() as i32);
        let big = ConstantValue::new(f32::MAX);
        assert_eq!(big.value().to_bits(), f32::MAX.to_bits());
    }
}
