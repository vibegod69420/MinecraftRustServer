use std::collections::HashSet;
use std::fmt;

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
pub struct ContextKey {
    pub id: String,
}

/// Placeholder for `LootScoreProviderType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootScoreProviderType {
    pub id: String,
}

// JAVA: net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProvider
pub trait ScoreboardNameProvider: fmt::Debug {
    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProvider#getScoreHolder
    fn get_score_holder(&self, context: &LootContext) -> Option<ScoreHolder>;

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProvider#getType
    fn get_type(&self) -> LootScoreProviderType;

    // JAVA: net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProvider#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyProvider {
        name: String,
    }

    impl ScoreboardNameProvider for DummyProvider {
        fn get_score_holder(&self, _context: &LootContext) -> Option<ScoreHolder> {
            Some(ScoreHolder::for_name_only(self.name.clone()))
        }

        fn get_type(&self) -> LootScoreProviderType {
            LootScoreProviderType {
                id: "dummy".to_string(),
            }
        }

        fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
            HashSet::new()
        }
    }

    #[test]
    fn get_score_holder_returns_name() {
        let p = DummyProvider {
            name: "target".to_string(),
        };
        let holder = p.get_score_holder(&LootContext).unwrap();
        assert_eq!(holder.name(), "target");
    }

    #[test]
    fn get_type_and_empty_params() {
        let p = DummyProvider {
            name: "x".to_string(),
        };
        assert_eq!(p.get_type().id, "dummy");
        assert!(p.get_referenced_context_params().is_empty());
    }

    #[test]
    fn edge_case_negative_and_empty_name() {
        let p = DummyProvider {
            name: "-1".to_string(),
        };
        assert_eq!(p.get_score_holder(&LootContext).unwrap().name(), "-1");
        let empty = DummyProvider {
            name: String::new(),
        };
        assert_eq!(empty.get_score_holder(&LootContext).unwrap().name(), "");
    }

    #[test]
    fn overflow_wrapping_semantics() {
        let v: i32 = i32::MAX;
        assert_eq!(v.wrapping_add(1), i32::MIN);
        assert_eq!((-5i32).wrapping_mul(3), -15);
    }

    #[test]
    fn float_bits_edge_case() {
        assert_ne!(f64::NAN.to_bits(), f64::INFINITY.to_bits());
        assert_ne!((-0.0f64).to_bits(), 0.0f64.to_bits());
        assert_eq!(f32::NEG_INFINITY.to_bits(), f32::NEG_INFINITY.to_bits());
    }
}
