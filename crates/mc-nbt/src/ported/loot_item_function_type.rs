use std::marker::PhantomData;

/// Placeholder for unported `com.mojang.serialization.MapCodec<T>`.
#[derive(Debug)]
pub struct MapCodec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> MapCodec<T> {
    /// Creates a placeholder codec.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for MapCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for MapCodec<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T> PartialEq for MapCodec<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Eq for MapCodec<T> {}

/// Placeholder bound for unported `net.minecraft.world.level.storage.loot.functions.LootItemFunction`.
pub trait LootItemFunction {}

/// Port of `net.minecraft.world.level.storage.loot.functions.LootItemFunctionType<T>`.
#[derive(Debug)]
pub struct LootItemFunctionType<T: LootItemFunction> {
    codec: MapCodec<T>,
}

impl<T: LootItemFunction> LootItemFunctionType<T> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunctionType#<init>
    pub fn new(codec: MapCodec<T>) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunctionType#codec
    pub fn codec(&self) -> &MapCodec<T> {
        &self.codec
    }
}

impl<T: LootItemFunction> Clone for LootItemFunctionType<T> {
    fn clone(&self) -> Self {
        Self {
            codec: self.codec.clone(),
        }
    }
}

impl<T: LootItemFunction> PartialEq for LootItemFunctionType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.codec == other.codec
    }
}

impl<T: LootItemFunction> Eq for LootItemFunctionType<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyFunction;
    impl LootItemFunction for DummyFunction {}

    #[test]
    fn new_and_codec_accessor() {
        let codec = MapCodec::<DummyFunction>::new();
        let ty = LootItemFunctionType::new(codec.clone());
        assert_eq!(ty.codec(), &codec);
    }

    #[test]
    fn clone_and_eq() {
        let ty = LootItemFunctionType::new(MapCodec::<DummyFunction>::new());
        let ty2 = ty.clone();
        assert_eq!(ty, ty2);
    }

    #[test]
    fn default_codec_eq() {
        let a = MapCodec::<DummyFunction>::default();
        let b = MapCodec::<DummyFunction>::new();
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_instances_same_placeholder_codec() {
        let t1 = LootItemFunctionType::new(MapCodec::<DummyFunction>::new());
        let t2 = LootItemFunctionType::new(MapCodec::<DummyFunction>::new());
        // Placeholder codecs are structurally equal.
        assert_eq!(t1, t2);
    }
}
