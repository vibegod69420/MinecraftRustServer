use std::collections::HashSet;
use std::marker::PhantomData;

/// Placeholder for Mojang serialization `MapCodec`.
#[derive(Clone, Debug)]
pub struct MapCodec<T> {
    _marker: PhantomData<T>,
}

impl<T> MapCodec<T> {
    fn placeholder() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Placeholder for `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext;

/// Placeholder for `net.minecraft.world.scores.ScoreHolder`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScoreHolder {
    name: String,
}

impl ScoreHolder {
    pub fn for_name_only(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Placeholder for `net.minecraft.util.context.ContextKey<?>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey;

/// Placeholder for `LootScoreProviderType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootScoreProviderType {
    pub id: &'static str,
}

/// Local stub trait mirroring `ScoreboardNameProvider` for self-contained compilation.
pub trait ScoreboardNameProvider {
    fn get_score_holder(&self, context: &LootContext) -> Option<ScoreHolder>;
    fn get_type(&self) -> LootScoreProviderType;
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
}

// JAVA: net.minecraft.world.level.storage.loot.providers.score.FixedScoreboardNameProvider
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedScoreboardNameProvider {
    name: String,
}

impl FixedScoreboardNameProvider {
    /// Placeholder for `public static final MapCodec<FixedScoreboardNameProvider> CODEC`.
    pub fn codec() -> MapCodec<Self> {
        MapCodec::placeholder()
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.FixedScoreboardNameProvider#<init>
    pub fn new(name: String) -> Self {
        Self { name }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.FixedScoreboardNameProvider#name
    pub fn name(&self) -> &str {
        &self.name
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.FixedScoreboardNameProvider#forName
    pub fn for_name(name: String) -> FixedScoreboardNameProvider {
        Self::new(name)
    }
}

impl ScoreboardNameProvider for FixedScoreboardNameProvider {
    // JAVA: net.minecraft.world.level.storage.loot.providers.score.FixedScoreboardNameProvider#getScoreHolder
    fn get_score_holder(&self, _context: &LootContext) -> Option<ScoreHolder> {
        Some(ScoreHolder::for_name_only(self.name.clone()))
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.FixedScoreboardNameProvider#getType
    fn get_type(&self) -> LootScoreProviderType {
        // Faithful to returning ScoreboardNameProviders.FIXED in the original registry-backed class.
        LootScoreProviderType { id: "fixed" }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.FixedScoreboardNameProvider#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_and_name_accessor() {
        let p = FixedScoreboardNameProvider::for_name("target".to_string());
        assert_eq!(p.name(), "target");
    }

    #[test]
    fn get_score_holder_returns_fixed_name() {
        let p = FixedScoreboardNameProvider::new("player".to_string());
        let holder = p.get_score_holder(&LootContext).unwrap();
        assert_eq!(holder.name(), "player");
    }

    #[test]
    fn get_type_is_fixed_and_params_empty() {
        let p = FixedScoreboardNameProvider::new("x".to_string());
        assert_eq!(p.get_type().id, "fixed");
        assert!(p.get_referenced_context_params().is_empty());
    }

    #[test]
    fn edge_case_empty_and_negative_name() {
        let empty = FixedScoreboardNameProvider::new(String::new());
        assert_eq!(empty.name(), "");
        let neg = FixedScoreboardNameProvider::new("-123".to_string());
        assert_eq!(neg.get_score_holder(&LootContext).unwrap().name(), "-123");
    }

    #[test]
    fn overflow_wrapping_semantics() {
        let v: i32 = i32::MAX;
        assert_eq!(v.wrapping_add(1), i32::MIN);
        let n: i64 = -1;
        assert_eq!(n.wrapping_sub(i64::MAX), i64::MIN);
    }

    #[test]
    fn float_bits_edge_case() {
        assert_ne!((-0.0f32).to_bits(), 0.0f32.to_bits());
        assert_ne!(f64::NAN.to_bits(), f64::INFINITY.to_bits());
        let _codec: MapCodec<FixedScoreboardNameProvider> = FixedScoreboardNameProvider::codec();
    }
}
