use crate::NbtTag;
use std::collections::{BTreeMap, HashSet};

/// Minimal placeholder for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }

    fn as_storage_key(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

/// Minimal placeholder for `net.minecraft.util.context.ContextKey<?>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey {
    pub name: String,
}

/// Minimal placeholder for `net.minecraft.world.level.storage.loot.LootContext`.
/// Contains command storage data to support `StorageNbtProvider#get`.
#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub command_storage: BTreeMap<String, NbtTag>,
}

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
}

/// Placeholder for `NbtProviders.STORAGE`.
pub const STORAGE_PROVIDER_TYPE: LootNbtProviderType = LootNbtProviderType {
    codec_id: "storage",
};

/// Port of `net.minecraft.world.level.storage.loot.providers.nbt.StorageNbtProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageNbtProvider {
    id: Identifier,
}

impl StorageNbtProvider {
    /// Placeholder for `StorageNbtProvider#CODEC` (`MapCodec<StorageNbtProvider>`).
    pub const CODEC_ID: &'static str = "storage";

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.StorageNbtProvider#<init>
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.StorageNbtProvider#id
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.StorageNbtProvider#getType
    pub fn get_type(&self) -> LootNbtProviderType {
        STORAGE_PROVIDER_TYPE.clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.StorageNbtProvider#get
    pub fn get(&self, ctx: &LootContext) -> Option<NbtTag> {
        ctx.command_storage.get(&self.id.as_storage_key()).cloned()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.StorageNbtProvider#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_tag_from_command_storage() {
        let id = Identifier::with_default_namespace("test");
        let provider = StorageNbtProvider::new(id.clone());
        let mut ctx = LootContext::default();
        ctx.command_storage.insert(id.as_storage_key(), NbtTag::String("hello".to_string()));
        assert_eq!(provider.get(&ctx), Some(NbtTag::String("hello".to_string())));
    }

    #[test]
    fn get_returns_none_when_missing() {
        let provider = StorageNbtProvider::new(Identifier::with_default_namespace("missing"));
        let ctx = LootContext::default();
        assert_eq!(provider.get(&ctx), None);
    }

    #[test]
    fn get_type_is_storage() {
        let provider = StorageNbtProvider::new(Identifier::with_default_namespace("x"));
        assert_eq!(provider.get_type(), STORAGE_PROVIDER_TYPE);
        assert_eq!(provider.get_type().codec_id, "storage");
    }

    #[test]
    fn referenced_context_params_empty() {
        let provider = StorageNbtProvider::new(Identifier::with_default_namespace("x"));
        assert!(provider.get_referenced_context_params().is_empty());
    }

    #[test]
    fn id_accessor_and_equality() {
        let id = Identifier::with_default_namespace("abc");
        let a = StorageNbtProvider::new(id.clone());
        let b = StorageNbtProvider::new(id.clone());
        let c = StorageNbtProvider::new(Identifier::with_default_namespace("def"));
        assert_eq!(a.id(), &id);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
