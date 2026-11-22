use std::collections::HashSet;
use std::fmt;
use std::marker::PhantomData;

/// Minimal self-contained stub for `net.minecraft.util.context.ContextKey`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

/// Minimal self-contained stub for `net.minecraft.world.level.storage.loot.parameters.LootContextParams`.
pub struct LootContextParams;

impl LootContextParams {
    /// Stub for `LootContextParams.LAST_DAMAGE_PLAYER`.
    pub const LAST_DAMAGE_PLAYER: ContextKey = ContextKey("last_damage_player");
}

/// Minimal self-contained stub for `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Default, Clone)]
pub struct LootContext {
    params: HashSet<ContextKey>,
}

impl LootContext {
    /// Stub constructor for self-contained compilation.
    pub fn new() -> Self {
        Self {
            params: HashSet::new(),
        }
    }

    /// Stub constructor with parameters for testing.
    pub fn with_params(params: HashSet<ContextKey>) -> Self {
        Self { params }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#hasParameter
    pub fn has_parameter(&self, key: &ContextKey) -> bool {
        self.params.contains(key)
    }
}

/// Minimal self-contained stub for
/// `net.minecraft.world.level.storage.loot.predicates.LootItemConditionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemConditionType(pub &'static str);

/// Stub for `net.minecraft.world.level.storage.loot.predicates.LootItemConditions#KILLED_BY_PLAYER`.
pub const KILLED_BY_PLAYER_TYPE: LootItemConditionType = LootItemConditionType("killed_by_player");

/// Minimal self-contained stub for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T>(PhantomData<fn() -> T>);

impl<T> MapCodec<T> {
    // JAVA: com.mojang.serialization.MapCodec#unit
    pub fn unit(_instance: T) -> Self {
        Self(PhantomData)
    }
}

/// Minimal self-contained stub for
/// `net.minecraft.world.level.storage.loot.predicates.LootItemCondition`.
pub trait LootItemCondition: fmt::Debug {
    fn get_type(&self) -> LootItemConditionType;
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
    fn test(&self, ctx: &LootContext) -> bool;
}

/// Minimal self-contained stub for
/// `net.minecraft.world.level.storage.loot.predicates.LootItemCondition$Builder`.
pub trait LootItemConditionBuilder: fmt::Debug {
    fn build(&self) -> Box<dyn LootItemCondition>;
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition
#[derive(Debug, Clone, Copy)]
pub struct LootItemKilledByPlayerCondition;

impl LootItemKilledByPlayerCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition#INSTANCE
    const INSTANCE: LootItemKilledByPlayerCondition = LootItemKilledByPlayerCondition;

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition#CODEC
    pub fn codec() -> MapCodec<LootItemKilledByPlayerCondition> {
        MapCodec::unit(Self::INSTANCE)
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition#<init>
    fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition#killedByPlayer
    pub fn killed_by_player() -> KilledByPlayerBuilder {
        KilledByPlayerBuilder
    }
}

impl LootItemCondition for LootItemKilledByPlayerCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition#getType
    fn get_type(&self) -> LootItemConditionType {
        KILLED_BY_PLAYER_TYPE
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = HashSet::new();
        set.insert(LootContextParams::LAST_DAMAGE_PLAYER);
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition#test
    fn test(&self, ctx: &LootContext) -> bool {
        ctx.has_parameter(&LootContextParams::LAST_DAMAGE_PLAYER)
    }
}

/// Builder returned by `killed_by_player()`, equivalent to Java lambda `() -> INSTANCE`.
#[derive(Debug, Clone, Copy)]
pub struct KilledByPlayerBuilder;

impl LootItemConditionBuilder for KilledByPlayerBuilder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition#killedByPlayer
    fn build(&self) -> Box<dyn LootItemCondition> {
        Box::new(LootItemKilledByPlayerCondition::INSTANCE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_returns_true_when_last_damage_player_present() {
        let mut params = HashSet::new();
        params.insert(LootContextParams::LAST_DAMAGE_PLAYER);
        let ctx = LootContext::with_params(params);
        let cond = LootItemKilledByPlayerCondition::new();
        assert!(cond.test(&ctx));
    }

    #[test]
    fn test_returns_false_when_missing_parameter() {
        let ctx = LootContext::new();
        let cond = LootItemKilledByPlayerCondition::new();
        assert!(!cond.test(&ctx));
    }

    #[test]
    fn get_type_is_killed_by_player() {
        let cond = LootItemKilledByPlayerCondition::new();
        assert_eq!(cond.get_type(), KILLED_BY_PLAYER_TYPE);
    }

    #[test]
    fn referenced_context_params_contains_last_damage_player() {
        let cond = LootItemKilledByPlayerCondition::new();
        let params = cond.get_referenced_context_params();
        assert_eq!(params.len(), 1);
        assert!(params.contains(&LootContextParams::LAST_DAMAGE_PLAYER));
    }

    #[test]
    fn killed_by_player_builder_returns_instance_behavior() {
        let built = LootItemKilledByPlayerCondition::killed_by_player().build();
        let mut params = HashSet::new();
        params.insert(LootContextParams::LAST_DAMAGE_PLAYER);
        let ctx = LootContext::with_params(params);
        assert!(built.test(&ctx));
        assert_eq!(built.get_type(), KILLED_BY_PLAYER_TYPE);
    }

    #[test]
    fn codec_stub_is_constructible() {
        let _codec = LootItemKilledByPlayerCondition::codec();
    }
}
