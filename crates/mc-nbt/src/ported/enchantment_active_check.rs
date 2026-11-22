use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootContextParam {
    EnchantmentActive,
}

impl LootContextParam {
    pub fn key(&self) -> ContextKey {
        match self {
            Self::EnchantmentActive => ContextKey("minecraft:enchantment_active"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemConditionType {
    EnchantmentActiveCheck,
}

#[derive(Debug, Clone, Default)]
pub struct LootContext {
    enchantment_active: Option<bool>,
}

impl LootContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enchantment_active(mut self, active: bool) -> Self {
        self.enchantment_active = Some(active);
        self
    }

    pub fn get_parameter_bool(&self, param: LootContextParam) -> bool {
        match param {
            LootContextParam::EnchantmentActive => self
                .enchantment_active
                .expect("Missing LootContext parameter: ENCHANTMENT_ACTIVE"),
        }
    }
}

pub trait LootItemCondition {
    fn get_type(&self) -> LootItemConditionType;
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
    fn test(&self, ctx: &LootContext) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantmentActiveCheck {
    active: bool,
}

impl EnchantmentActiveCheck {
    pub fn new(active: bool) -> Self {
        Self { active }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EnchantmentActiveCheck#active
    pub fn active(&self) -> bool {
        self.active
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EnchantmentActiveCheck#enchantmentActiveCheck
    pub fn enchantment_active_check() -> Box<dyn Fn() -> Self> {
        Box::new(|| EnchantmentActiveCheck::new(true))
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EnchantmentActiveCheck#enchantmentInactiveCheck
    pub fn enchantment_inactive_check() -> Box<dyn Fn() -> Self> {
        Box::new(|| EnchantmentActiveCheck::new(false))
    }
}

impl LootItemCondition for EnchantmentActiveCheck {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.EnchantmentActiveCheck#getType
    fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::EnchantmentActiveCheck
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EnchantmentActiveCheck#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = HashSet::new();
        set.insert(LootContextParam::EnchantmentActive.key());
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.EnchantmentActiveCheck#test
    fn test(&self, ctx: &LootContext) -> bool {
        ctx.get_parameter_bool(LootContextParam::EnchantmentActive) == self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_matches_true() {
        let cond = EnchantmentActiveCheck::new(true);
        let ctx = LootContext::new().with_enchantment_active(true);
        assert!(cond.test(&ctx));
    }

    #[test]
    fn test_active_mismatch() {
        let cond = EnchantmentActiveCheck::new(true);
        let ctx = LootContext::new().with_enchantment_active(false);
        assert!(!cond.test(&ctx));
    }

    #[test]
    fn test_inactive_matches_false() {
        let cond = EnchantmentActiveCheck::new(false);
        let ctx = LootContext::new().with_enchantment_active(false);
        assert!(cond.test(&ctx));
    }

    #[test]
    fn test_inactive_mismatch() {
        let cond = EnchantmentActiveCheck::new(false);
        let ctx = LootContext::new().with_enchantment_active(true);
        assert!(!cond.test(&ctx));
    }

    #[test]
    fn test_builders_type_and_referenced_params() {
        let active = (EnchantmentActiveCheck::enchantment_active_check())();
        let inactive = (EnchantmentActiveCheck::enchantment_inactive_check())();
        assert!(active.active());
        assert!(!inactive.active());
        assert_eq!(
            active.get_type(),
            LootItemConditionType::EnchantmentActiveCheck
        );
        let params = active.get_referenced_context_params();
        assert_eq!(params.len(), 1);
        assert!(params.contains(&LootContextParam::EnchantmentActive.key()));
    }

    #[test]
    #[should_panic(expected = "Missing LootContext parameter")]
    fn test_missing_parameter_panics() {
        let cond = EnchantmentActiveCheck::new(true);
        let ctx = LootContext::new();
        let _ = cond.test(&ctx);
    }
}
