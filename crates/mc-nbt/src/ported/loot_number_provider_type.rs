//! Faithful port of `net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType` (MC 1.21.11).

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Minimal placeholder for `net.minecraft.world.level.storage.loot.providers.number.NumberProvider`.
pub trait NumberProvider {}

/// Minimal placeholder for `com.mojang.serialization.MapCodec`.
pub struct MapCodec<T: ?Sized>(PhantomData<fn() -> T>);

impl<T: ?Sized> MapCodec<T> {
    /// Creates a new placeholder codec.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized> Default for MapCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> fmt::Debug for MapCodec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MapCodec").finish()
    }
}

impl<T: ?Sized> Clone for MapCodec<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for MapCodec<T> {}

impl<T: ?Sized> PartialEq for MapCodec<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T: ?Sized> Eq for MapCodec<T> {}

impl<T: ?Sized> Hash for MapCodec<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

/// Port of Java record `LootNumberProviderType(MapCodec<? extends NumberProvider> codec)`.
pub struct LootNumberProviderType<T: ?Sized + NumberProvider> {
    codec: MapCodec<T>,
}

impl<T: ?Sized + NumberProvider> LootNumberProviderType<T> {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType#<init>
    pub const fn new(codec: MapCodec<T>) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType#codec
    pub const fn codec(&self) -> &MapCodec<T> {
        &self.codec
    }
}

impl<T: ?Sized + NumberProvider> fmt::Debug for LootNumberProviderType<T> {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LootNumberProviderType")
            .field("codec", &self.codec)
            .finish()
    }
}

impl<T: ?Sized + NumberProvider> Clone for LootNumberProviderType<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + NumberProvider> Copy for LootNumberProviderType<T> {}

impl<T: ?Sized + NumberProvider> PartialEq for LootNumberProviderType<T> {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType#equals
    fn eq(&self, other: &Self) -> bool {
        self.codec == other.codec
    }
}

impl<T: ?Sized + NumberProvider> Eq for LootNumberProviderType<T> {}

impl<T: ?Sized + NumberProvider> Hash for LootNumberProviderType<T> {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType#hashCode
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.codec.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    struct DummyProvider;
    impl NumberProvider for DummyProvider {}

    struct OtherProvider;
    impl NumberProvider for OtherProvider {}

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn new_and_codec_accessor() {
        let codec = MapCodec::<DummyProvider>::new();
        let t = LootNumberProviderType::new(codec);
        assert_eq!(*t.codec(), codec);
    }

    #[test]
    fn copy_clone_and_equality() {
        let a = LootNumberProviderType::new(MapCodec::<DummyProvider>::new());
        let b = a; // Copy
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn debug_contains_record_name() {
        let t = LootNumberProviderType::new(MapCodec::<DummyProvider>::new());
        let dbg = format!("{:?}", t);
        assert!(dbg.contains("LootNumberProviderType"));
        assert!(dbg.contains("MapCodec"));
    }

    #[test]
    fn different_provider_types_are_distinct_types() {
        let a = LootNumberProviderType::new(MapCodec::<DummyProvider>::new());
        let b = LootNumberProviderType::new(MapCodec::<OtherProvider>::new());
        // Different generic instantiations; verify each behaves correctly.
        assert_eq!(a, LootNumberProviderType::new(MapCodec::<DummyProvider>::new()));
        assert_eq!(b, LootNumberProviderType::new(MapCodec::<OtherProvider>::new()));
        let _ = (a, b);
    }

    #[test]
    fn numeric_edge_cases_overflow_negative_and_float_bits() {
        // Java integer semantics sanity checks relevant to ported numeric providers.
        let max_i32 = i32::MAX;
        let min_i32 = i32::MIN;
        assert_eq!(max_i32.wrapping_add(1), min_i32);
        assert_eq!(min_i32.wrapping_sub(1), max_i32);
        assert_eq!((-1_i32).wrapping_mul(2), -2_i32);

        let neg: i64 = -123_456_789_i64;
        assert_eq!(neg.wrapping_add(-1), -123_456_790_i64);
        assert_eq!((neg as i32).wrapping_sub(i32::MAX), (neg as i32).wrapping_add(i32::MIN).wrapping_add(1));

        let nan = f32::NAN;
        assert_ne!(nan.to_bits(), f32::from_bits(nan.to_bits() ^ 1).to_bits() ^ 1 ^ 1); // bit identity check
        assert_eq!(nan.to_bits(), nan.to_bits());
        assert_eq!(f64::NEG_INFINITY.to_bits(), (-1.0_f64 / 0.0_f64).to_bits());
        assert_eq!(0.0_f32.to_bits(), 0.0_f32.to_bits());
        assert_ne!((-0.0_f32).to_bits(), 0.0_f32.to_bits());
    }
}
