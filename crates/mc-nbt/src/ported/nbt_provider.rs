use crate::NbtTag;
use std::collections::HashSet;

/// Minimal placeholder for `net.minecraft.util.context.ContextKey<?>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey {
    pub name: String,
}

impl ContextKey {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Minimal placeholder for `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext;

/// Minimal self-contained placeholder for `LootNbtProviderType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootNbtProviderType {
    pub codec_id: &'static str,
}

impl LootNbtProviderType {
    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType#<init>
    pub fn new(codec_id: &'static str) -> Self {
        Self { codec_id }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType#codec
    pub fn codec_id(&self) -> &'static str {
        self.codec_id
    }
}

/// Port of `net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider`.
pub trait NbtProvider {
    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider#get
    fn get(&self, ctx: &LootContext) -> Option<NbtTag>;

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider#getType
    fn get_type(&self) -> LootNbtProviderType;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyProvider {
        tag: Option<NbtTag>,
    }

    impl NbtProvider for DummyProvider {
        // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider#get
        fn get(&self, _ctx: &LootContext) -> Option<NbtTag> {
            self.tag.clone()
        }

        // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider#getReferencedContextParams
        fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
            HashSet::new()
        }

        // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider#getType
        fn get_type(&self) -> LootNbtProviderType {
            LootNbtProviderType::new("dummy")
        }
    }

    #[test]
    fn get_returns_some_when_present() {
        let p = DummyProvider { tag: Some(NbtTag::Int(42)) };
        assert_eq!(p.get(&LootContext), Some(NbtTag::Int(42)));
    }

    #[test]
    fn get_returns_none_when_absent() {
        let p = DummyProvider { tag: None };
        assert_eq!(p.get(&LootContext), None);
    }

    #[test]
    fn referenced_context_params_empty() {
        let p = DummyProvider { tag: None };
        assert!(p.get_referenced_context_params().is_empty());
    }

    #[test]
    fn get_type_returns_provider_type() {
        let p = DummyProvider { tag: None };
        let ty = p.get_type();
        assert_eq!(ty.codec_id(), "dummy");
        assert_eq!(ty, LootNbtProviderType::new("dummy"));
    }

    #[test]
    fn context_key_equality() {
        let a = ContextKey::new("origin");
        let b = ContextKey::new("origin");
        let c = ContextKey::new("this");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
