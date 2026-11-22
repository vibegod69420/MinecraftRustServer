use std::collections::BTreeSet;
use std::fmt;

// ---------------------------------------------------------------------------
// Self-contained stubs for external Minecraft types referenced by this class.
// These are minimal placeholders so the module compiles in isolation.
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextKey {
    pub name: String,
}

impl ContextKey {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl fmt::Debug for ContextKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContextKey({:?})", self.name)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ScoreHolder {
    pub name: String,
}

impl ScoreHolder {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl fmt::Debug for ScoreHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ScoreHolder({:?})", self.name)
    }
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.LootContext.EntityTarget`.
/// Holds the context parameter key used by `contextParam()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityTarget {
    param: ContextKey,
}

impl EntityTarget {
    pub fn new(param: ContextKey) -> Self {
        Self { param }
    }

    /// Mirrors `LootContext.EntityTarget#contextParam()`.
    pub fn context_param(&self) -> &ContextKey {
        &self.param
    }
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext {
    params: Vec<(ContextKey, ScoreHolder)>,
}

impl LootContext {
    pub fn empty() -> Self {
        Self { params: Vec::new() }
    }

    pub fn with_parameter(mut self, key: ContextKey, holder: ScoreHolder) -> Self {
        self.params.push((key, holder));
        self
    }

    /// Mirrors `LootContext#getOptionalParameter(ContextKey)`.
    pub fn get_optional_parameter(&self, key: &ContextKey) -> Option<ScoreHolder> {
        self.params
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.providers.score.LootScoreProviderType`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LootScoreProviderType {
    pub name: String,
}

impl LootScoreProviderType {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProvider`.
pub trait ScoreboardNameProvider: fmt::Debug {
    fn get_type(&self) -> LootScoreProviderType;
    fn get_score_holder(&self, context: &LootContext) -> Option<ScoreHolder>;
    fn get_referenced_context_params(&self) -> BTreeSet<ContextKey>;
}

/// Minimal codec metadata placeholders (DataFixerUpper codecs are not available in mc-nbt).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapCodecMeta {
    pub type_name: &'static str,
    pub fields: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecMeta {
    pub description: &'static str,
}

// ---------------------------------------------------------------------------
// Port of net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextScoreboardNameProvider {
    target: EntityTarget,
}

impl ContextScoreboardNameProvider {
    /// Structural metadata for
    /// `public static final MapCodec<ContextScoreboardNameProvider> CODEC`.
    pub fn codec_meta() -> MapCodecMeta {
        MapCodecMeta {
            type_name: "ContextScoreboardNameProvider",
            fields: &["target"],
        }
    }

    /// Structural metadata for
    /// `public static final Codec<ContextScoreboardNameProvider> INLINE_CODEC`.
    pub fn inline_codec_meta() -> CodecMeta {
        CodecMeta {
            description: "EntityTarget.CODEC.xmap(ContextScoreboardNameProvider::new, ContextScoreboardNameProvider::target)",
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#<init>
    pub fn new(target: EntityTarget) -> Self {
        Self { target }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#target
    pub fn target(&self) -> &EntityTarget {
        &self.target
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#forTarget
    pub fn for_target(target: EntityTarget) -> Box<dyn ScoreboardNameProvider> {
        Box::new(Self::new(target))
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#getType
    pub fn get_type(&self) -> LootScoreProviderType {
        // Mirrors `return ScoreboardNameProviders.CONTEXT;`
        LootScoreProviderType::new("context")
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#getScoreHolder
    pub fn get_score_holder(&self, context: &LootContext) -> Option<ScoreHolder> {
        context.get_optional_parameter(self.target.context_param())
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        let mut set = BTreeSet::new();
        set.insert(self.target.context_param().clone());
        set
    }
}

impl ScoreboardNameProvider for ContextScoreboardNameProvider {
    fn get_type(&self) -> LootScoreProviderType {
        ContextScoreboardNameProvider::get_type(self)
    }

    fn get_score_holder(&self, context: &LootContext) -> Option<ScoreHolder> {
        ContextScoreboardNameProvider::get_score_holder(self, context)
    }

    fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        ContextScoreboardNameProvider::get_referenced_context_params(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(name: &str) -> EntityTarget {
        EntityTarget::new(ContextKey::new(name))
    }

    #[test]
    fn for_target_and_accessors() {
        let provider = ContextScoreboardNameProvider::for_target(target("this_entity"));
        assert_eq!(provider.get_type(), LootScoreProviderType::new("context"));
        let params = provider.get_referenced_context_params();
        assert_eq!(params.len(), 1);
        assert!(params.contains(&ContextKey::new("this_entity")));
    }

    #[test]
    fn get_score_holder_present_and_missing() {
        let key = ContextKey::new("killer");
        let provider = ContextScoreboardNameProvider::new(target("killer"));
        let ctx_missing = LootContext::empty();
        assert_eq!(provider.get_score_holder(&ctx_missing), None);

        let ctx_present =
            LootContext::empty().with_parameter(key.clone(), ScoreHolder::new("player1"));
        assert_eq!(
            provider.get_score_holder(&ctx_present),
            Some(ScoreHolder::new("player1"))
        );
    }

    #[test]
    fn get_score_holder_uses_latest_matching_parameter() {
        let key = ContextKey::new("this_entity");
        let provider = ContextScoreboardNameProvider::new(target("this_entity"));
        let ctx = LootContext::empty()
            .with_parameter(key.clone(), ScoreHolder::new("first"))
            .with_parameter(key, ScoreHolder::new("last"));
        assert_eq!(
            provider.get_score_holder(&ctx),
            Some(ScoreHolder::new("last"))
        );
    }

    #[test]
    fn referenced_context_params_is_singleton_set() {
        let provider = ContextScoreboardNameProvider::new(target("direct_killer"));
        let params = provider.get_referenced_context_params();
        let mut expected = BTreeSet::new();
        expected.insert(ContextKey::new("direct_killer"));
        assert_eq!(params, expected);
    }

    #[test]
    fn codec_metadata_is_structurally_correct() {
        let codec = ContextScoreboardNameProvider::codec_meta();
        assert_eq!(codec.type_name, "ContextScoreboardNameProvider");
        assert_eq!(codec.fields, &["target"]);

        let inline = ContextScoreboardNameProvider::inline_codec_meta();
        assert!(inline.description.contains("EntityTarget.CODEC.xmap"));
        assert!(inline.description.contains("ContextScoreboardNameProvider::new"));
    }

    #[test]
    fn record_equality_and_target_accessor() {
        let a = ContextScoreboardNameProvider::new(target("this_entity"));
        let b = ContextScoreboardNameProvider::new(target("this_entity"));
        let c = ContextScoreboardNameProvider::new(target("killer"));
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.target(), &target("this_entity"));
    }
}
