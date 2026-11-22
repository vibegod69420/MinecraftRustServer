use std::marker::PhantomData;

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer`.
#[derive(Debug, Clone, Default)]
pub struct LootPoolEntryContainer;

/// Minimal stub for unported `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone)]
pub struct MapCodec<T: ?Sized> {
    _marker: PhantomData<fn() -> *const T>,
}

impl<T: ?Sized> MapCodec<T> {
    /// Creates a stub codec for self-contained porting.
    pub fn stub() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Default for MapCodec<T> {
    fn default() -> Self {
        Self::stub()
    }
}

/// Port of `net.minecraft.world.level.storage.loot.entries.LootPoolEntryType`.
#[derive(Debug, Clone)]
pub struct LootPoolEntryType {
    codec: MapCodec<LootPoolEntryContainer>,
}

impl LootPoolEntryType {
    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryType#LootPoolEntryType
    pub fn new(codec: MapCodec<LootPoolEntryContainer>) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryType#codec
    pub fn codec(&self) -> &MapCodec<LootPoolEntryContainer> {
        &self.codec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_codec_accessor() {
        let codec = MapCodec::<LootPoolEntryContainer>::stub();
        let entry_type = LootPoolEntryType::new(codec);
        let _codec_ref: &MapCodec<LootPoolEntryContainer> = entry_type.codec();
    }

    #[test]
    fn default_codec_is_usable() {
        let codec = MapCodec::<LootPoolEntryContainer>::default();
        let entry_type = LootPoolEntryType::new(codec);
        let _ = entry_type.codec();
    }

    #[test]
    fn clone_preserves_type() {
        let entry_type = LootPoolEntryType::new(MapCodec::stub());
        let cloned = entry_type.clone();
        let _ = (entry_type.codec(), cloned.codec());
    }

    #[test]
    fn debug_format_does_not_panic() {
        let entry_type = LootPoolEntryType::new(MapCodec::stub());
        let dbg = format!("{:?}", entry_type);
        assert!(dbg.contains("LootPoolEntryType"));
    }
}
