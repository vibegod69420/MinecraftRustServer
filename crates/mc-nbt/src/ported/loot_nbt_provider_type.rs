use std::fmt;

/// Placeholder for Mojang `MapCodec<? extends NbtProvider>`.
/// The full codec infrastructure is not ported; this preserves the record field.
#[derive(Clone, Copy)]
pub struct NbtProviderCodec {
    pub type_name: &'static str,
}

impl fmt::Debug for NbtProviderCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NbtProviderCodec")
            .field("type_name", &self.type_name)
            .finish()
    }
}

impl PartialEq for NbtProviderCodec {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name
    }
}

impl Eq for NbtProviderCodec {}

/// Port of `net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LootNbtProviderType {
    pub codec: NbtProviderCodec,
}

impl LootNbtProviderType {
    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType#<init>
    pub fn new(codec: NbtProviderCodec) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType#codec
    pub fn codec(&self) -> &NbtProviderCodec {
        &self.codec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_codec_accessor() {
        let codec = NbtProviderCodec { type_name: "storage" };
        let ty = LootNbtProviderType::new(codec);
        assert_eq!(ty.codec().type_name, "storage");
        assert_eq!(ty.codec, codec);
    }

    #[test]
    fn equality_and_clone() {
        let a = LootNbtProviderType::new(NbtProviderCodec { type_name: "context" });
        let b = a.clone();
        let c = LootNbtProviderType::new(NbtProviderCodec { type_name: "storage" });
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn codec_equality_by_type_name() {
        let a = NbtProviderCodec { type_name: "same" };
        let b = NbtProviderCodec { type_name: "same" };
        let c = NbtProviderCodec { type_name: "other" };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn debug_contains_type_name() {
        let ty = LootNbtProviderType::new(NbtProviderCodec { type_name: "storage" });
        let dbg = format!("{:?}", ty);
        assert!(dbg.contains("storage"));
    }
}
