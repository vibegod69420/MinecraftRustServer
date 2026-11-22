use std::fmt;
use std::marker::PhantomData;

/// Minimal local stub for `com.mojang.serialization.MapCodec<T>`.
///
/// This is a self-contained placeholder that preserves the record field shape
/// of `LootItemConditionType` without requiring the external codec framework.
pub struct MapCodec<T: ?Sized>(PhantomData<fn() -> *const T>);

impl<T: ?Sized> MapCodec<T> {
    /// Creates a new `MapCodec` stub.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized> Default for MapCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Clone for MapCodec<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for MapCodec<T> {}

impl<T: ?Sized> fmt::Debug for MapCodec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MapCodec(..)")
    }
}

impl<T: ?Sized> PartialEq for MapCodec<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T: ?Sized> Eq for MapCodec<T> {}

/// Minimal local stub for
/// `net.minecraft.world.level.storage.loot.predicates.LootItemCondition`.
pub trait LootItemCondition: fmt::Debug {}

/// Port of `net.minecraft.world.level.storage.loot.predicates.LootItemConditionType`.
///
/// Java record: `LootItemConditionType(MapCodec<? extends LootItemCondition> codec)`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LootItemConditionType {
    codec: MapCodec<dyn LootItemCondition>,
}

impl LootItemConditionType {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemConditionType#<init>
    pub fn new(codec: MapCodec<dyn LootItemCondition>) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemConditionType#codec
    pub fn codec(&self) -> &MapCodec<dyn LootItemCondition> {
        &self.codec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyCondition {
        value: i32,
        bits: u32,
    }

    impl LootItemCondition for DummyCondition {}

    #[test]
    fn new_and_codec_accessor() {
        let codec: MapCodec<dyn LootItemCondition> = MapCodec::new();
        let condition_type = LootItemConditionType::new(codec);
        // Accessor returns the stored codec stub.
        let _ = condition_type.codec();
        assert_eq!(condition_type.codec(), &MapCodec::new());
    }

    #[test]
    fn clone_preserves_equality() {
        let condition_type = LootItemConditionType::new(MapCodec::new());
        let cloned = condition_type.clone();
        assert_eq!(condition_type, cloned);
    }

    #[test]
    fn debug_contains_type_name() {
        let condition_type = LootItemConditionType::new(MapCodec::new());
        let dbg = format!("{:?}", condition_type);
        assert!(dbg.contains("LootItemConditionType"));
        assert!(dbg.contains("MapCodec"));
    }

    #[test]
    fn map_codec_copy_and_default() {
        let codec: MapCodec<dyn LootItemCondition> = MapCodec::default();
        let copied = codec;
        let another = codec;
        assert_eq!(copied, another);
    }

    #[test]
    fn dummy_condition_edge_values_are_representable() {
        // The loot condition type itself has no numeric fields, but stubbed
        // condition implementations must preserve Java-relevant edge values.
        let neg = DummyCondition {
            value: i32::MIN,
            bits: (-1.0f32).to_bits(),
        };
        let overflow_like = DummyCondition {
            value: i32::MAX.wrapping_add(1),
            bits: f32::NAN.to_bits(),
        };
        let neg_zero = DummyCondition {
            value: -0i32,
            bits: (-0.0f32).to_bits(),
        };

        assert_eq!(neg.value, i32::MIN);
        assert_eq!(neg.bits, 0xBF80_0000);
        assert_eq!(overflow_like.value, i32::MIN);
        assert!(f32::from_bits(overflow_like.bits).is_nan());
        assert_eq!(neg_zero.value, 0);
        assert_eq!(neg_zero.bits, 0x8000_0000);
        assert_ne!(neg_zero.bits, 0.0f32.to_bits());
    }
}
