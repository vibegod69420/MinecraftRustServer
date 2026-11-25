// JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition#class
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Enchantment {
    pub id: String,
}

impl Enchantment {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl PartialEq for Enchantment {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Enchantment {}

impl std::hash::Hash for Enchantment {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct Holder<T> {
    pub value: Arc<T>,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(value),
        }
    }
}

impl<T: PartialEq> PartialEq for Holder<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.value == *other.value
    }
}

impl<T: Eq> Eq for Holder<T> {}

impl<T: std::hash::Hash> std::hash::Hash for Holder<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Clone, Debug, Default)]
pub struct LivingEntity {
    pub enchantments: BTreeMap<String, i32>,
}

impl LivingEntity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enchantment(mut self, id: impl Into<String>, level: i32) -> Self {
        self.enchantments.insert(id.into(), level);
        self
    }
}

#[derive(Clone, Debug)]
pub enum Entity {
    Living(LivingEntity),
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ContextKey {
    AttackingEntity,
}

pub struct LootContextParams;

impl LootContextParams {
    pub const ATTACKING_ENTITY: ContextKey = ContextKey::AttackingEntity;
}

#[derive(Clone, Debug)]
pub struct SimpleRandom {
    state: u64,
}

impl SimpleRandom {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_float(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) as u32 as f32) / (u32::MAX as f32 + 1.0)
    }
}

#[derive(Clone, Debug)]
pub struct LootContext {
    pub attacking_entity: Option<Entity>,
    pub random: SimpleRandom,
}

impl LootContext {
    pub fn new(attacking_entity: Option<Entity>) -> Self {
        Self {
            attacking_entity,
            random: SimpleRandom::new(1),
        }
    }

    pub fn with_seed(attacking_entity: Option<Entity>, seed: u64) -> Self {
        Self {
            attacking_entity,
            random: SimpleRandom::new(seed),
        }
    }

    pub fn get_optional_parameter(&self, _key: ContextKey) -> Option<&Entity> {
        self.attacking_entity.as_ref()
    }

    pub fn get_random(&mut self) -> &mut SimpleRandom {
        &mut self.random
    }
}

pub struct EnchantmentHelper;

impl EnchantmentHelper {
    pub fn get_enchantment_level(
        enchantment: &Holder<Enchantment>,
        entity: &LivingEntity,
    ) -> i32 {
        entity
            .enchantments
            .get(&enchantment.value.id)
            .copied()
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Linear {
    pub base: f32,
    pub per_level_above_first: f32,
}

impl Linear {
    pub fn new(base: f32, per_level_above_first: f32) -> Self {
        Self {
            base,
            per_level_above_first,
        }
    }

    pub fn calculate(&self, level: i32) -> f32 {
        self.base + self.per_level_above_first * (level as f32 - 1.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LevelBasedValue {
    Linear {
        base: f32,
        per_level_above_first: f32,
    },
}

impl LevelBasedValue {
    pub fn calculate(&self, level: i32) -> f32 {
        match self {
            Self::Linear {
                base,
                per_level_above_first,
            } => base + per_level_above_first * (level as f32 - 1.0),
        }
    }
}

impl From<Linear> for LevelBasedValue {
    fn from(value: Linear) -> Self {
        Self::Linear {
            base: value.base,
            per_level_above_first: value.per_level_above_first,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LootItemRandomChanceWithEnchantedBonusCondition {
    pub unenchanted_chance: f32,
    pub enchanted_chance: LevelBasedValue,
    pub enchantment: Holder<Enchantment>,
}

impl LootItemRandomChanceWithEnchantedBonusCondition {
    pub fn new(
        unenchanted_chance: f32,
        enchanted_chance: LevelBasedValue,
        enchantment: Holder<Enchantment>,
    ) -> Self {
        Self {
            unenchanted_chance,
            enchanted_chance,
            enchantment,
        }
    }

    pub fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        let mut set = BTreeSet::new();
        set.insert(LootContextParams::ATTACKING_ENTITY);
        set
    }

    pub fn test(&self, ctx: &mut LootContext) -> bool {
        let entity = ctx.get_optional_parameter(LootContextParams::ATTACKING_ENTITY);

        let level = match entity {
            Some(Entity::Living(living)) => {
                EnchantmentHelper::get_enchantment_level(&self.enchantment, living)
            }
            _ => 0,
        };

        let chance = if level > 0 {
            self.enchanted_chance.calculate(level)
        } else {
            self.unenchanted_chance
        };

        ctx.get_random().next_float() < chance
    }

    pub fn random_chance_and_looting_boost(
        unenchanted_chance: f32,
        bonus: f32,
    ) -> Self {
        Self::new(
            unenchanted_chance,
            LevelBasedValue::Linear {
                base: unenchanted_chance + bonus,
                per_level_above_first: bonus,
            },
            Holder::new(Enchantment::new("minecraft:looting")),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_entity_uses_unenchanted_chance_zero() {
        let cond = LootItemRandomChanceWithEnchantedBonusCondition::new(
            0.0,
            LevelBasedValue::Linear {
                base: 1.0,
                per_level_above_first: 0.0,
            },
            Holder::new(Enchantment::new("minecraft:looting")),
        );
        let mut ctx = LootContext::new(None);
        assert!(!cond.test(&mut ctx));
    }

    #[test]
    fn no_entity_uses_unenchanted_chance_one() {
        let cond = LootItemRandomChanceWithEnchantedBonusCondition::new(
            1.0,
            LevelBasedValue::Linear {
                base: 1.0,
                per_level_above_first: 0.0,
            },
            Holder::new(Enchantment::new("minecraft:looting")),
        );
        let mut ctx = LootContext::new(None);
        assert!(cond.test(&mut ctx));
    }

    #[test]
    fn enchanted_level_uses_enchanted_chance() {
        let living = LivingEntity::new().with_enchantment("minecraft:looting", 1);
        let cond = LootItemRandomChanceWithEnchantedBonusCondition::new(
            0.0,
            LevelBasedValue::Linear {
                base: 1.0,
                per_level_above_first: 0.0,
            },
            Holder::new(Enchantment::new("minecraft:looting")),
        );
        let mut ctx = LootContext::new(Some(Entity::Living(living)));
        assert!(cond.test(&mut ctx));
    }

    #[test]
    fn referenced_context_params_contains_attacking_entity() {
        let cond = LootItemRandomChanceWithEnchantedBonusCondition::random_chance_and_looting_boost(
            0.5,
            0.1,
        );
        let params = cond.get_referenced_context_params();
        assert!(params.contains(&LootContextParams::ATTACKING_ENTITY));
    }

    #[test]
    fn linear_calculate_matches_java_formula() {
        let value = Linear::new(0.5, 0.25);
        assert_eq!(value.calculate(1), 0.5);
        assert_eq!(value.calculate(2), 0.75);
    }
}
