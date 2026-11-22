use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

// ---------------------------------------------------------------------------
// Self-contained stubs for external Minecraft types referenced by this class.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }

    pub fn to_language_key(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

/// Minimal stub for a DataFixerUpper `MapCodec<? extends ScoreboardNameProvider>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapCodecMeta {
    pub type_name: &'static str,
}

/// Minimal stub for a DataFixerUpper `Codec<ScoreboardNameProvider>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecMeta {
    pub description: &'static str,
}

/// Minimal stub for
/// `net.minecraft.world.level.storage.loot.providers.score.LootScoreProviderType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootScoreProviderType {
    pub id: Identifier,
    pub codec: MapCodecMeta,
}

/// Minimal stand-in for `BuiltInRegistries.LOOT_SCORE_PROVIDER_TYPE` registration.
fn loot_score_provider_registry() -> &'static Mutex<BTreeMap<String, LootScoreProviderType>> {
    static REGISTRY: LazyLock<Mutex<BTreeMap<String, LootScoreProviderType>>> =
        LazyLock::new(|| Mutex::new(BTreeMap::new()));
    &REGISTRY
}

// ---------------------------------------------------------------------------
// Port of net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProviders
// ---------------------------------------------------------------------------

pub struct ScoreboardNameProviders;

impl ScoreboardNameProviders {
    /// Structural metadata for
    /// `private static final Codec<ScoreboardNameProvider> TYPED_CODEC`.
    pub fn typed_codec_meta() -> CodecMeta {
        CodecMeta {
            description: "BuiltInRegistries.LOOT_SCORE_PROVIDER_TYPE.byNameCodec().dispatch(ScoreboardNameProvider::getType, LootScoreProviderType::codec)",
        }
    }

    /// Structural metadata for
    /// `public static final Codec<ScoreboardNameProvider> CODEC`.
    pub fn codec_meta() -> CodecMeta {
        CodecMeta {
            description: "Codec.lazyInitialized(() -> Codec.either(ContextScoreboardNameProvider.INLINE_CODEC, TYPED_CODEC).xmap(Either::unwrap, ...))",
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProviders#register
    fn register(name: &str, codec: MapCodecMeta) -> LootScoreProviderType {
        let id = Identifier::with_default_namespace(name);
        let provider_type = LootScoreProviderType { id: id.clone(), codec };
        let mut registry = loot_score_provider_registry()
            .lock()
            .expect("loot score provider registry poisoned");
        registry.insert(id.to_language_key(), provider_type.clone());
        provider_type
    }

    /// Accessor for
    /// `public static final LootScoreProviderType FIXED`.
    pub fn fixed() -> LootScoreProviderType {
        static FIXED: LazyLock<LootScoreProviderType> = LazyLock::new(|| {
            ScoreboardNameProviders::register(
                "fixed",
                MapCodecMeta {
                    type_name: "FixedScoreboardNameProvider",
                },
            )
        });
        FIXED.clone()
    }

    /// Accessor for
    /// `public static final LootScoreProviderType CONTEXT`.
    pub fn context() -> LootScoreProviderType {
        static CONTEXT: LazyLock<LootScoreProviderType> = LazyLock::new(|| {
            ScoreboardNameProviders::register(
                "context",
                MapCodecMeta {
                    type_name: "ContextScoreboardNameProvider",
                },
            )
        });
        CONTEXT.clone()
    }

    /// Test/helper lookup in the self-contained registry stand-in.
    pub fn get_registered(id: &str) -> Option<LootScoreProviderType> {
        loot_score_provider_registry()
            .lock()
            .expect("loot score provider registry poisoned")
            .get(id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_creates_namespaced_identifier_and_stores_type() {
        let ty = ScoreboardNameProviders::context();
        assert_eq!(ty.id, Identifier::with_default_namespace("context"));
        assert_eq!(ty.codec.type_name, "ContextScoreboardNameProvider");

        let looked_up = ScoreboardNameProviders::get_registered("minecraft:context");
        assert_eq!(looked_up, Some(ty));
    }

    #[test]
    fn fixed_and_context_are_distinct_registered_types() {
        let fixed = ScoreboardNameProviders::fixed();
        let context = ScoreboardNameProviders::context();
        assert_ne!(fixed, context);
        assert_eq!(fixed.id.path, "fixed");
        assert_eq!(context.id.path, "context");
        assert_eq!(fixed.codec.type_name, "FixedScoreboardNameProvider");
        assert_eq!(context.codec.type_name, "ContextScoreboardNameProvider");
    }

    #[test]
    fn codec_metadata_describes_java_structure() {
        let typed = ScoreboardNameProviders::typed_codec_meta();
        assert!(typed.description.contains("byNameCodec().dispatch"));
        assert!(typed.description.contains("ScoreboardNameProvider::getType"));

        let codec = ScoreboardNameProviders::codec_meta();
        assert!(codec.description.contains("Codec.lazyInitialized"));
        assert!(codec.description.contains("Codec.either"));
        assert!(codec.description.contains("ContextScoreboardNameProvider.INLINE_CODEC"));
    }

    #[test]
    fn identifier_default_namespace_and_key_format() {
        let id = Identifier::with_default_namespace("fixed");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "fixed");
        assert_eq!(id.to_language_key(), "minecraft:fixed");
    }

    #[test]
    fn registry_lookup_missing_returns_none() {
        assert_eq!(
            ScoreboardNameProviders::get_registered("minecraft:does_not_exist"),
            None
        );
    }
}
