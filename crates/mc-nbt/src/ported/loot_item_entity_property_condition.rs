use std::collections::BTreeSet;

/// Self-contained faithful port of
/// `net.minecraft.world.level.storage.loot.predicates.LootItemEntityPropertyCondition` (Java 1.21.11).
///
/// Entity predicate, loot context, entity target, and vec3 are modeled locally.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextKey(pub &'static str);

pub const LOOT_CONTEXT_PARAM_ORIGIN: ContextKey = ContextKey("minecraft:origin");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemConditionType {
    EntityProperties,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

// JAVA: net.minecraft.world.level.storage.loot.LootContext$EntityTarget
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityTarget {
    This,
    Attacker,
    DirectAttacker,
    AttackingPlayer,
}

impl EntityTarget {
    // JAVA: net.minecraft.world.level.storage.loot.LootContext$EntityTarget#contextParam
    pub fn context_param(self) -> ContextKey {
        match self {
            Self::This => ContextKey("minecraft:this_entity"),
            Self::Attacker => ContextKey("minecraft:attacker"),
            Self::DirectAttacker => ContextKey("minecraft:direct_attacker"),
            Self::AttackingPlayer => ContextKey("minecraft:attacking_player"),
        }
    }
}

/// Minimal EntityPredicate surface used by this condition.
pub trait EntityPredicate {
    fn matches(&self, level: &dyn LootLevel, origin: Option<Vec3>, entity: Option<Entity>) -> bool;
}

/// Minimal level surface passed to EntityPredicate.matches.
pub trait LootLevel {}

/// Minimal context surface required by `LootItemEntityPropertyCondition`.
pub trait LootItemEntityPropertyContext {
    fn level(&self) -> &dyn LootLevel;
    fn get_optional_entity(&self, target: EntityTarget) -> Option<Entity>;
    fn get_optional_origin(&self) -> Option<Vec3>;
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemEntityPropertyCondition
#[derive(Debug, Clone)]
pub struct LootItemEntityPropertyCondition<P> {
    predicate: Option<P>,
    entity_target: EntityTarget,
}

impl<P> LootItemEntityPropertyCondition<P> {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemEntityPropertyCondition#<init>
    pub fn new(predicate: Option<P>, entity_target: EntityTarget) -> Self {
        Self {
            predicate,
            entity_target,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemEntityPropertyCondition#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::EntityProperties
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemEntityPropertyCondition#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        let mut set = BTreeSet::new();
        set.insert(LOOT_CONTEXT_PARAM_ORIGIN);
        set.insert(self.entity_target.context_param());
        set
    }

    pub fn predicate(&self) -> Option<&P> {
        self.predicate.as_ref()
    }

    pub fn entity_target(&self) -> EntityTarget {
        self.entity_target
    }
}

impl<P: EntityPredicate> LootItemEntityPropertyCondition<P> {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemEntityPropertyCondition#test
    pub fn test<C: LootItemEntityPropertyContext>(&self, context: &C) -> bool {
        let entity = context.get_optional_entity(self.entity_target);
        let origin = context.get_optional_origin();
        match &self.predicate {
            None => true,
            Some(predicate) => predicate.matches(context.level(), origin, entity),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemEntityPropertyCondition#hasProperties(LootContext.EntityTarget,EntityPredicate)
    pub fn has_properties(entity_target: EntityTarget, predicate: P) -> Self {
        Self::new(Some(predicate), entity_target)
    }
}

impl LootItemEntityPropertyCondition<AlwaysTruePredicate> {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemEntityPropertyCondition#entityPresent
    pub fn entity_present(entity_target: EntityTarget) -> Self {
        Self::has_properties(entity_target, AlwaysTruePredicate)
    }
}

/// Simple predicate equivalent to an empty/default EntityPredicate builder for `entityPresent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AlwaysTruePredicate;

impl EntityPredicate for AlwaysTruePredicate {
    fn matches(&self, _level: &dyn LootLevel, _origin: Option<Vec3>, _entity: Option<Entity>) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AlwaysTruePredicate, ContextKey, Entity, EntityPredicate, EntityTarget, LootItemEntityPropertyCondition,
        LootItemEntityPropertyContext, LootItemConditionType, LootLevel, Vec3, LOOT_CONTEXT_PARAM_ORIGIN,
    };
    use std::collections::BTreeSet;

    struct TestLevel;
    impl LootLevel for TestLevel {}

    struct TestContext {
        level: TestLevel,
        entity: Option<Entity>,
        origin: Option<Vec3>,
    }

    impl LootItemEntityPropertyContext for TestContext {
        fn level(&self) -> &dyn LootLevel {
            &self.level
        }

        fn get_optional_entity(&self, _target: EntityTarget) -> Option<Entity> {
            self.entity
        }

        fn get_optional_origin(&self) -> Option<Vec3> {
            self.origin
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct EntityIsPresentPredicate;
    impl EntityPredicate for EntityIsPresentPredicate {
        fn matches(&self, _level: &dyn LootLevel, _origin: Option<Vec3>, entity: Option<Entity>) -> bool {
            entity.is_some()
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct OriginXPredicate(f64);
    impl EntityPredicate for OriginXPredicate {
        fn matches(&self, _level: &dyn LootLevel, origin: Option<Vec3>, _entity: Option<Entity>) -> bool {
            origin.map(|v| v.x.to_bits() == self.0.to_bits()).unwrap_or(false)
        }
    }

    #[test]
    fn type_and_referenced_params() {
        let cond = LootItemEntityPropertyCondition::<AlwaysTruePredicate>::entity_present(EntityTarget::This);
        assert_eq!(cond.get_type(), LootItemConditionType::EntityProperties);
        let refs = cond.get_referenced_context_params();
        let mut expected = BTreeSet::new();
        expected.insert(LOOT_CONTEXT_PARAM_ORIGIN);
        expected.insert(ContextKey("minecraft:this_entity"));
        assert_eq!(refs, expected);
    }

    #[test]
    fn empty_predicate_passes() {
        let cond = LootItemEntityPropertyCondition::<AlwaysTruePredicate>::new(None, EntityTarget::Attacker);
        let ctx = TestContext {
            level: TestLevel,
            entity: None,
            origin: None,
        };
        assert!(cond.test(&ctx));
    }

    #[test]
    fn predicate_receives_entity_and_origin() {
        let cond = LootItemEntityPropertyCondition::has_properties(EntityTarget::This, EntityIsPresentPredicate);
        let missing = TestContext {
            level: TestLevel,
            entity: None,
            origin: Some(Vec3::new(1.0, 2.0, 3.0)),
        };
        let present = TestContext {
            level: TestLevel,
            entity: Some(Entity(42)),
            origin: None,
        };
        assert!(!cond.test(&missing));
        assert!(cond.test(&present));
    }

    #[test]
    fn origin_float_bits_are_preserved_in_predicate() {
        let cond = LootItemEntityPropertyCondition::has_properties(EntityTarget::This, OriginXPredicate(-0.0));
        let ctx_neg_zero = TestContext {
            level: TestLevel,
            entity: Some(Entity(1)),
            origin: Some(Vec3::new(-0.0, 0.0, 0.0)),
        };
        let ctx_pos_zero = TestContext {
            level: TestLevel,
            entity: Some(Entity(1)),
            origin: Some(Vec3::new(0.0, 0.0, 0.0)),
        };
        assert!(cond.test(&ctx_neg_zero));
        assert!(!cond.test(&ctx_pos_zero));
    }

    #[test]
    fn entity_target_context_param_mapping() {
        assert_eq!(EntityTarget::This.context_param(), ContextKey("minecraft:this_entity"));
        assert_eq!(EntityTarget::Attacker.context_param(), ContextKey("minecraft:attacker"));
        assert_eq!(
            EntityTarget::DirectAttacker.context_param(),
            ContextKey("minecraft:direct_attacker")
        );
        assert_eq!(
            EntityTarget::AttackingPlayer.context_param(),
            ContextKey("minecraft:attacking_player")
        );
    }
}
