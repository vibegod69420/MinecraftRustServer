use std::marker::PhantomData;

/// Placeholder for Mojang serialization `MapCodec`.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T: ?Sized = ()>(PhantomData<fn() -> T>);

impl<T: ?Sized> MapCodec<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Placeholder for a loot item condition codec target.
pub trait LootItemCondition {}

/// Faithful representation of `net.minecraft.world.level.storage.loot.predicates.LootItemConditionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LootItemConditionType {
    pub namespace: &'static str,
    pub path: &'static str,
}

impl LootItemConditionType {
    pub const fn id(&self) -> (&'static str, &'static str) {
        (self.namespace, self.path)
    }
}

/// Port of `net.minecraft.world.level.storage.loot.predicates.LootItemConditions`.
pub struct LootItemConditions;

impl LootItemConditions {
    pub const INVERTED: LootItemConditionType = Self::register("inverted", MapCodec::new());
    pub const ANY_OF: LootItemConditionType = Self::register("any_of", MapCodec::new());
    pub const ALL_OF: LootItemConditionType = Self::register("all_of", MapCodec::new());
    pub const RANDOM_CHANCE: LootItemConditionType = Self::register("random_chance", MapCodec::new());
    pub const RANDOM_CHANCE_WITH_ENCHANTED_BONUS: LootItemConditionType =
        Self::register("random_chance_with_enchanted_bonus", MapCodec::new());
    pub const ENTITY_PROPERTIES: LootItemConditionType =
        Self::register("entity_properties", MapCodec::new());
    pub const KILLED_BY_PLAYER: LootItemConditionType =
        Self::register("killed_by_player", MapCodec::new());
    pub const ENTITY_SCORES: LootItemConditionType =
        Self::register("entity_scores", MapCodec::new());
    pub const BLOCK_STATE_PROPERTY: LootItemConditionType =
        Self::register("block_state_property", MapCodec::new());
    pub const MATCH_TOOL: LootItemConditionType = Self::register("match_tool", MapCodec::new());
    pub const TABLE_BONUS: LootItemConditionType = Self::register("table_bonus", MapCodec::new());
    pub const SURVIVES_EXPLOSION: LootItemConditionType =
        Self::register("survives_explosion", MapCodec::new());
    pub const DAMAGE_SOURCE_PROPERTIES: LootItemConditionType =
        Self::register("damage_source_properties", MapCodec::new());
    pub const LOCATION_CHECK: LootItemConditionType =
        Self::register("location_check", MapCodec::new());
    pub const WEATHER_CHECK: LootItemConditionType =
        Self::register("weather_check", MapCodec::new());
    pub const REFERENCE: LootItemConditionType = Self::register("reference", MapCodec::new());
    pub const TIME_CHECK: LootItemConditionType = Self::register("time_check", MapCodec::new());
    pub const VALUE_CHECK: LootItemConditionType = Self::register("value_check", MapCodec::new());
    pub const ENCHANTMENT_ACTIVE_CHECK: LootItemConditionType =
        Self::register("enchantment_active_check", MapCodec::new());

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemConditions#register
    const fn register(path: &'static str, _codec: MapCodec<()>) -> LootItemConditionType {
        // JAVA: Registry.register(BuiltInRegistries.LOOT_CONDITION_TYPE, Identifier.withDefaultNamespace($$0), new LootItemConditionType($$1))
        LootItemConditionType {
            namespace: "minecraft",
            path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn register_uses_default_namespace() {
        let t = LootItemConditions::INVERTED;
        assert_eq!(t.namespace, "minecraft");
        assert_eq!(t.path, "inverted");
        assert_eq!(t.id(), ("minecraft", "inverted"));
    }

    #[test]
    fn known_condition_paths() {
        assert_eq!(LootItemConditions::BLOCK_STATE_PROPERTY.path, "block_state_property");
        assert_eq!(
            LootItemConditions::RANDOM_CHANCE_WITH_ENCHANTED_BONUS.path,
            "random_chance_with_enchanted_bonus"
        );
        assert_eq!(
            LootItemConditions::ENCHANTMENT_ACTIVE_CHECK.path,
            "enchantment_active_check"
        );
    }

    #[test]
    fn all_registered_paths_are_unique() {
        let all = [
            LootItemConditions::INVERTED,
            LootItemConditions::ANY_OF,
            LootItemConditions::ALL_OF,
            LootItemConditions::RANDOM_CHANCE,
            LootItemConditions::RANDOM_CHANCE_WITH_ENCHANTED_BONUS,
            LootItemConditions::ENTITY_PROPERTIES,
            LootItemConditions::KILLED_BY_PLAYER,
            LootItemConditions::ENTITY_SCORES,
            LootItemConditions::BLOCK_STATE_PROPERTY,
            LootItemConditions::MATCH_TOOL,
            LootItemConditions::TABLE_BONUS,
            LootItemConditions::SURVIVES_EXPLOSION,
            LootItemConditions::DAMAGE_SOURCE_PROPERTIES,
            LootItemConditions::LOCATION_CHECK,
            LootItemConditions::WEATHER_CHECK,
            LootItemConditions::REFERENCE,
            LootItemConditions::TIME_CHECK,
            LootItemConditions::VALUE_CHECK,
            LootItemConditions::ENCHANTMENT_ACTIVE_CHECK,
        ];
        let mut set = HashSet::new();
        for t in all {
            assert_eq!(t.namespace, "minecraft");
            assert!(set.insert(t.path), "duplicate path: {}", t.path);
        }
        assert_eq!(set.len(), 19);
    }

    #[test]
    fn type_equality_and_copy() {
        let a = LootItemConditions::TIME_CHECK;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, LootItemConditions::VALUE_CHECK);
    }
}
