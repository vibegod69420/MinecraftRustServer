use std::collections::HashSet;
use std::fmt::Debug;

/// JAVA: net.minecraft.world.level.storage.loot.LootContext
/// Stub for self-contained compilation; replace with real type when wired.
#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub damage_source: Option<DamageSource>,
    pub origin: Option<Vec3>,
}

impl LootContext {
    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getOptionalParameter
    pub fn get_optional_parameter_damage_source(&self) -> Option<&DamageSource> {
        self.damage_source.as_ref()
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getOptionalParameter
    pub fn get_optional_parameter_origin(&self) -> Option<&Vec3> {
        self.origin.as_ref()
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getLevel
    pub fn get_level(&self) -> &LootLevel {
        &LootLevel
    }
}

/// Stub for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LootLevel;

/// Stub for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Stub for self-contained compilation.
#[derive(Debug, Clone, Default)]
pub struct DamageSource;

/// JAVA: net.minecraft.util.context.ContextKey
/// Stub for self-contained compilation; replace with real type when wired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams
pub struct LootContextParams;

impl LootContextParams {
    // JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#ORIGIN
    pub const ORIGIN: ContextKey = ContextKey("minecraft:origin");
    // JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams#DAMAGE_SOURCE
    pub const DAMAGE_SOURCE: ContextKey = ContextKey("minecraft:damage_source");
}

/// JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemConditionType
/// Stub for self-contained compilation; replace with real type when wired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LootItemConditionType(pub u32);

// JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemConditions
pub struct LootItemConditions;

impl LootItemConditions {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemConditions#DAMAGE_SOURCE_PROPERTIES
    pub const DAMAGE_SOURCE_PROPERTIES: LootItemConditionType = LootItemConditionType(4);
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemCondition
pub trait LootItemCondition: Debug + Send + Sync {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemCondition#getType
    fn get_type(&self) -> LootItemConditionType;

    // JAVA: java.util.function.Predicate#test
    fn test(&self, ctx: &LootContext) -> bool;
}

// JAVA: net.minecraft.world.level.storage.loot.LootContextUser
pub trait LootContextUser {
    // JAVA: net.minecraft.world.level.storage.loot.LootContextUser#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemCondition$Builder
pub trait LootItemConditionBuilder: Debug {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemCondition$Builder#build
    fn build(&self) -> Box<dyn LootItemCondition>;
}

/// JAVA: net.minecraft.advancements.criterion.DamageSourcePredicate
/// Stub for self-contained compilation; replace with real predicate when wired.
#[derive(Debug, Clone, Default)]
pub struct DamageSourcePredicate {
    pub matches: bool,
}

impl DamageSourcePredicate {
    // JAVA: net.minecraft.advancements.criterion.DamageSourcePredicate#matches
    pub fn matches(&self, _level: &LootLevel, _origin: &Vec3, _source: &DamageSource) -> bool {
        self.matches
    }
}

/// JAVA: net.minecraft.advancements.criterion.DamageSourcePredicate$Builder
#[derive(Debug, Clone, Default)]
pub struct DamageSourcePredicateBuilder {
    pub matches: bool,
}

impl DamageSourcePredicateBuilder {
    // JAVA: net.minecraft.advancements.criterion.DamageSourcePredicate$Builder#build
    pub fn build(&self) -> DamageSourcePredicate {
        DamageSourcePredicate {
            matches: self.matches,
        }
    }
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition
#[derive(Debug, Clone)]
pub struct DamageSourceCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition#predicate
    pub predicate: Option<DamageSourcePredicate>,
}

impl DamageSourceCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition#CODEC
    // Codec is a serialization concern; represented as a marker in this self-contained port.
    pub const CODEC_NAME: &'static str = "damage_source_properties";

    // JAVA: net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition#hasDamageSource
    pub fn has_damage_source(
        builder: DamageSourcePredicateBuilder,
    ) -> DamageSourceConditionBuilder {
        DamageSourceConditionBuilder {
            predicate: Some(builder.build()),
        }
    }
}

impl LootItemCondition for DamageSourceCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition#getType
    fn get_type(&self) -> LootItemConditionType {
        LootItemConditions::DAMAGE_SOURCE_PROPERTIES
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition#test
    fn test(&self, ctx: &LootContext) -> bool {
        let damage_source = ctx.get_optional_parameter_damage_source();
        let origin = ctx.get_optional_parameter_origin();
        match (origin, damage_source) {
            (Some(origin), Some(source)) => match &self.predicate {
                None => true,
                Some(predicate) => predicate.matches(ctx.get_level(), origin, source),
            },
            _ => false,
        }
    }
}

impl LootContextUser for DamageSourceCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = HashSet::new();
        set.insert(LootContextParams::ORIGIN);
        set.insert(LootContextParams::DAMAGE_SOURCE);
        set
    }
}

#[derive(Debug, Clone)]
pub struct DamageSourceConditionBuilder {
    predicate: Option<DamageSourcePredicate>,
}

impl LootItemConditionBuilder for DamageSourceConditionBuilder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemCondition$Builder#build
    fn build(&self) -> Box<dyn LootItemCondition> {
        Box::new(DamageSourceCondition {
            predicate: self.predicate.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with_params() -> LootContext {
        LootContext {
            damage_source: Some(DamageSource),
            origin: Some(Vec3 { x: 1.0, y: 2.0, z: 3.0 }),
        }
    }

    #[test]
    fn type_and_referenced_params() {
        let cond = DamageSourceCondition { predicate: None };
        assert_eq!(
            cond.get_type(),
            LootItemConditions::DAMAGE_SOURCE_PROPERTIES
        );
        let params = cond.get_referenced_context_params();
        assert_eq!(params.len(), 2);
        assert!(params.contains(&LootContextParams::ORIGIN));
        assert!(params.contains(&LootContextParams::DAMAGE_SOURCE));
    }

    #[test]
    fn test_missing_params_returns_false() {
        let cond = DamageSourceCondition { predicate: None };
        assert!(!cond.test(&LootContext::default()));
        assert!(!cond.test(&LootContext {
            damage_source: Some(DamageSource),
            origin: None,
        }));
        assert!(!cond.test(&LootContext {
            damage_source: None,
            origin: Some(Vec3::default()),
        }));
    }

    #[test]
    fn test_empty_predicate_matches_when_params_present() {
        let cond = DamageSourceCondition { predicate: None };
        assert!(cond.test(&ctx_with_params()));
    }

    #[test]
    fn test_predicate_delegation() {
        let cond_true = DamageSourceCondition {
            predicate: Some(DamageSourcePredicate { matches: true }),
        };
        let cond_false = DamageSourceCondition {
            predicate: Some(DamageSourcePredicate { matches: false }),
        };
        let ctx = ctx_with_params();
        assert!(cond_true.test(&ctx));
        assert!(!cond_false.test(&ctx));
    }

    #[test]
    fn has_damage_source_builder() {
        let builder = DamageSourcePredicateBuilder { matches: true };
        let cond = DamageSourceCondition::has_damage_source(builder).build();
        assert_eq!(cond.get_type(), LootItemConditions::DAMAGE_SOURCE_PROPERTIES);
        assert!(cond.test(&ctx_with_params()));

        let builder_false = DamageSourcePredicateBuilder { matches: false };
        let cond_false = DamageSourceCondition::has_damage_source(builder_false).build();
        assert!(!cond_false.test(&ctx_with_params()));
    }
}
