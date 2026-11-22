use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// Minimal placeholder for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    fn to_key(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

/// Minimal self-contained placeholder for `LootNbtProviderType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootNbtProviderType {
    pub registry_id: String,
    pub codec_id: String,
}

impl LootNbtProviderType {
    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType#<init>
    pub fn new(codec_id: String) -> Self {
        Self {
            registry_id: String::new(),
            codec_id,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType#codec
    pub fn codec_id(&self) -> &str {
        &self.codec_id
    }
}

fn registry() -> &'static Mutex<BTreeMap<String, LootNbtProviderType>> {
    static REG: OnceLock<Mutex<BTreeMap<String, LootNbtProviderType>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(BTreeMap::new()))
}

// JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProviders#register
fn register(name: &str, codec_id: &str) -> LootNbtProviderType {
    let key = Identifier::with_default_namespace(name).to_key();
    let ty = LootNbtProviderType {
        registry_id: key.clone(),
        codec_id: codec_id.to_string(),
    };
    registry()
        .lock()
        .expect("nbt provider registry lock poisoned")
        .insert(key, ty.clone());
    ty
}

/// Port of `net.minecraft.world.level.storage.loot.providers.nbt.NbtProviders`.
///
/// Codec dispatch fields (`TYPED_CODEC`, `CODEC`) are represented as placeholders
/// because the Mojang codec/registry infrastructure is not ported in this crate.
pub struct NbtProviders;

impl NbtProviders {
    /// Placeholder for `NbtProviders#TYPED_CODEC`.
    pub const TYPED_CODEC_ID: &'static str = "TYPED_CODEC";
    /// Placeholder for `NbtProviders#CODEC`.
    pub const CODEC_ID: &'static str = "CODEC";

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProviders#STORAGE
    pub fn storage() -> LootNbtProviderType {
        static VAL: OnceLock<LootNbtProviderType> = OnceLock::new();
        VAL.get_or_init(|| register("storage", "StorageNbtProvider.CODEC"))
            .clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.nbt.NbtProviders#CONTEXT
    pub fn context() -> LootNbtProviderType {
        static VAL: OnceLock<LootNbtProviderType> = OnceLock::new();
        VAL.get_or_init(|| register("context", "ContextNbtProvider.MAP_CODEC"))
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_type_registered() {
        let ty = NbtProviders::storage();
        assert_eq!(ty.registry_id, "minecraft:storage");
        assert_eq!(ty.codec_id(), "StorageNbtProvider.CODEC");
    }

    #[test]
    fn context_type_registered() {
        let ty = NbtProviders::context();
        assert_eq!(ty.registry_id, "minecraft:context");
        assert_eq!(ty.codec_id(), "ContextNbtProvider.MAP_CODEC");
    }

    #[test]
    fn register_inserts_into_registry() {
        let ty = register("custom_test", "Custom.CODEC");
        assert_eq!(ty.registry_id, "minecraft:custom_test");
        let reg = registry().lock().unwrap();
        assert_eq!(reg.get("minecraft:custom_test"), Some(&ty));
    }

    #[test]
    fn register_overwrites_same_key() {
        let _ = register("overwrite_test", "A.CODEC");
        let ty2 = register("overwrite_test", "B.CODEC");
        let reg = registry().lock().unwrap();
        assert_eq!(
            reg.get("minecraft:overwrite_test").map(|t| t.codec_id.as_str()),
            Some("B.CODEC")
        );
        assert_eq!(ty2.codec_id, "B.CODEC");
    }

    #[test]
    fn identifier_with_default_namespace() {
        let id = Identifier::with_default_namespace("path");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "path");
        assert_eq!(id.to_key(), "minecraft:path");
    }
}
