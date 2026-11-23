//! Port of `net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction` (1.21.11).
//! Self-contained, minimal traits/types for faithful behavior.

use std::sync::Arc;

/// JAVA: net.minecraft.world.level.storage.loot.parameters.LootContextParams (minimal)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

pub mod loot_context_params {
    use super::ContextKey;
    pub const ATTACKING_ENTITY: ContextKey = ContextKey("attacking_entity");
}

/// Minimal condition trait (not evaluated by run, stored for structure).
pub trait LootItemCondition: Send + Sync {
    fn test(&self, _ctx: &LootContext) -> bool {
        true
    }
}

/// Minimal enchantment holder.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnchantmentHolder(pub String);

impl EnchantmentHolder {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

/// Minimal living-entity view used by this function.
pub trait LivingEntityLike: Send + Sync {
    // JAVA: net.minecraft.world.item.enchantment.EnchantmentHelper#getEnchantmentLevel
    fn get_enchantment_level(&self, enchantment: &EnchantmentHolder) -> i32;
}

/// Minimal loot context.
pub struct LootContext {
    pub attacking_living: Option<Arc<dyn LivingEntityLike>>,
}

impl LootContext {
    pub fn new(attacking_living: Option<Arc<dyn LivingEntityLike>>) -> Self {
        Self { attacking_living }
    }

    pub fn get_optional_attacking_entity(&self) -> Option<&dyn LivingEntityLike> {
        self.attacking_living.as_deref()
    }

    // JAVA-like parameter lookup used in EnchantedCountIncreaseFunction.
    pub fn get_optional_parameter(&self, key: &ContextKey) -> Option<&dyn LivingEntityLike> {
        if key == &loot_context_params::ATTACKING_ENTITY {
            self.get_optional_attacking_entity()
        } else {
            None
        }
    }
}

/// Minimal number provider.
pub trait NumberProvider: Send + Sync {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getFloat
    fn get_float(&self, ctx: &LootContext) -> f32;

    fn get_referenced_context_params(&self) -> Vec<ContextKey> {
        Vec::new()
    }
}

/// Minimal item stack model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
}

impl ItemStack {
    pub fn new(count: i32) -> Self {
        Self { count }
    }

    // JAVA: net.minecraft.world.item.ItemStack#grow
    pub fn grow(&mut self, delta: i32) {
        self.count = self.count.wrapping_add(delta);
    }

    // JAVA: net.minecraft.world.item.ItemStack#limitSize
    pub fn limit_size(&mut self, limit: i32) {
        if self.count > limit {
            self.count = limit;
        }
    }
}

fn java_round_f32(x: f32) -> i32 {
    // JAVA: Math.round(float): floor(x + 0.5f), ties to positive infinity in effect for this form.
    let v = x + 0.5f32;
    if v.is_nan() {
        return 0;
    }
    let f = v.floor();
    if f >= (i32::MAX as f32) {
        return i32::MAX;
    }
    if f <= (i32::MIN as f32) {
        return i32::MIN;
    }
    f as i32
}

pub const NO_LIMIT: i32 = 0;

/// JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction
pub struct EnchantedCountIncreaseFunction {
    pub conditions: Vec<Arc<dyn LootItemCondition>>,
    pub enchantment: EnchantmentHolder,
    pub value: Arc<dyn NumberProvider>,
    pub limit: i32,
}

impl EnchantedCountIncreaseFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#<init>
    pub fn new(
        conditions: Vec<Arc<dyn LootItemCondition>>,
        enchantment: EnchantmentHolder,
        value: Arc<dyn NumberProvider>,
        limit: i32,
    ) -> Self {
        Self {
            conditions,
            enchantment,
            value,
            limit,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#hasLimit
    fn has_limit(&self) -> bool {
        self.limit > 0
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> Vec<ContextKey> {
        let mut v = vec![loot_context_params::ATTACKING_ENTITY];
        for k in self.value.get_referenced_context_params() {
            if !v.iter().any(|x| x == &k) {
                v.push(k);
            }
        }
        v
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        let attacker_opt = ctx.get_optional_parameter(&loot_context_params::ATTACKING_ENTITY);
        if let Some(attacker) = attacker_opt {
            let lvl: i32 = attacker.get_enchantment_level(&self.enchantment);
            if lvl == 0 {
                return stack;
            }
            let vf: f32 = self.value.get_float(ctx);
            // Match Java order: lvl * this.value.getFloat(ctx)
            let inc_f: f32 = (lvl as f32) * vf;
            let rounded: i32 = java_round_f32(inc_f);
            stack.grow(rounded);
            if self.has_limit() {
                stack.limit_size(self.limit);
            }
        }
        stack
    }
}

/// JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction.Builder
pub struct Builder {
    pub conditions: Vec<Arc<dyn LootItemCondition>>,
    pub enchantment: EnchantmentHolder,
    pub count: Arc<dyn NumberProvider>,
    pub limit: i32,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction.Builder#<init>
    pub fn new(enchantment: EnchantmentHolder, count: Arc<dyn NumberProvider>) -> Self {
        Self {
            conditions: Vec::new(),
            enchantment,
            count,
            limit: NO_LIMIT,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction.Builder#setLimit
    pub fn set_limit(mut self, limit: i32) -> Self {
        self.limit = limit;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction.Builder#build
    pub fn build(self) -> EnchantedCountIncreaseFunction {
        EnchantedCountIncreaseFunction {
            conditions: self.conditions,
            enchantment: self.enchantment,
            value: self.count,
            limit: self.limit,
        }
    }
}

// JAVA: net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#lootingMultiplier (adapted)
pub fn looting_multiplier(count: Arc<dyn NumberProvider>) -> Builder {
    Builder::new(EnchantmentHolder::new("minecraft:looting"), count)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ConstFloatProvider(f32);
    impl NumberProvider for ConstFloatProvider {
        fn get_float(&self, _ctx: &LootContext) -> f32 {
            self.0
        }
    }

    struct TestLiving {
        lvl: i32,
    }
    impl LivingEntityLike for TestLiving {
        fn get_enchantment_level(&self, _enchantment: &EnchantmentHolder) -> i32 {
            self.lvl
        }
    }

    #[test]
    fn no_attacker_no_change() {
        let f = EnchantedCountIncreaseFunction::new(
            vec![],
            EnchantmentHolder::new("looting"),
            Arc::new(ConstFloatProvider(1.0)),
            NO_LIMIT,
        );
        let ctx = LootContext::new(None);
        let out = f.run(ItemStack::new(5), &ctx);
        assert_eq!(out.count, 5);
    }

    #[test]
    fn zero_level_no_change() {
        let f = EnchantedCountIncreaseFunction::new(
            vec![],
            EnchantmentHolder::new("looting"),
            Arc::new(ConstFloatProvider(10.0)),
            NO_LIMIT,
        );
        let ctx = LootContext::new(Some(Arc::new(TestLiving { lvl: 0 })));
        let out = f.run(ItemStack::new(3), &ctx);
        assert_eq!(out.count, 3);
    }

    #[test]
    fn rounds_and_grows() {
        // lvl 2, value 0.6 => 1.2 => round 1
        let f = EnchantedCountIncreaseFunction::new(
            vec![],
            EnchantmentHolder::new("looting"),
            Arc::new(ConstFloatProvider(0.6)),
            NO_LIMIT,
        );
        let ctx = LootContext::new(Some(Arc::new(TestLiving { lvl: 2 })));
        let out = f.run(ItemStack::new(10), &ctx);
        assert_eq!(out.count, 11);
    }

    #[test]
    fn limit_applied() {
        // lvl 3, value 5.0 => +15, limit 12 => count becomes 12 if started at 10
        let f = EnchantedCountIncreaseFunction::new(
            vec![],
            EnchantmentHolder::new("looting"),
            Arc::new(ConstFloatProvider(5.0)),
            12,
        );
        let ctx = LootContext::new(Some(Arc::new(TestLiving { lvl: 3 })));
        let out = f.run(ItemStack::new(10), &ctx);
        assert_eq!(out.count, 12);
    }

    #[test]
    fn negative_and_overflow_wrapping() {
        // negative delta via rounding: lvl 1, value -2.3 => -2.3 => round floor(-2.3+0.5)=floor(-1.8)=-2
        let f = EnchantedCountIncreaseFunction::new(
            vec![],
            EnchantmentHolder::new("looting"),
            Arc::new(ConstFloatProvider(-2.3)),
            NO_LIMIT,
        );
        let ctx = LootContext::new(Some(Arc::new(TestLiving { lvl: 1 })));
        let out = f.run(ItemStack::new(5), &ctx);
        assert_eq!(out.count, 3);

        // wrapping_add behavior on overflow
        let f2 = EnchantedCountIncreaseFunction::new(
            vec![],
            EnchantmentHolder::new("looting"),
            Arc::new(ConstFloatProvider(1.0)),
            NO_LIMIT,
        );
        let ctx2 = LootContext::new(Some(Arc::new(TestLiving { lvl: 1 })));
        let out2 = f2.run(ItemStack::new(i32::MAX), &ctx2);
        assert_eq!(out2.count, i32::MIN);
    }
}
