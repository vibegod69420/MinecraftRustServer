use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const LIMIT_COUNT: LootItemFunctionType = LootItemFunctionType("limit_count");
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IntRange {
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub referenced: BTreeSet<ContextKey>,
}

impl IntRange {
    // JAVA: net.minecraft.world.level.storage.loot.IntRange#clamp
    pub fn clamp(&self, _ctx: &LootContext, value: i32) -> i32 {
        let mut v = value;
        if let Some(min) = self.min {
            if v < min {
                v = min;
            }
        }
        if let Some(max) = self.max {
            if v > max {
                v = max;
            }
        }
        v
    }

    // JAVA: net.minecraft.world.level.storage.loot.IntRange#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        self.referenced.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
}

impl ItemStack {
    pub fn new(count: i32) -> Self {
        Self { count }
    }

    // JAVA: net.minecraft.world.item.ItemStack#getCount
    pub fn get_count(&self) -> i32 {
        self.count
    }

    // JAVA: net.minecraft.world.item.ItemStack#setCount
    pub fn set_count(&mut self, count: i32) {
        self.count = count;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitCount {
    pub conditions: Vec<LootItemCondition>,
    pub limiter: IntRange,
}

impl LimitCount {
    // JAVA: net.minecraft.world.level.storage.loot.functions.LimitCount#<init>
    fn new(conditions: Vec<LootItemCondition>, limiter: IntRange) -> Self {
        Self { conditions, limiter }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LimitCount#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::LIMIT_COUNT
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LimitCount#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        self.limiter.get_referenced_context_params()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LimitCount#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        let count = self.limiter.clamp(ctx, stack.get_count());
        stack.set_count(count);
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LimitCount#limitCount
    pub fn limit_count(limiter: IntRange) -> impl FnOnce(Vec<LootItemCondition>) -> Self {
        move |conditions| Self::new(conditions, limiter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> LootContext {
        LootContext
    }

    #[test]
    fn run_clamps_to_max() {
        let f = LimitCount::new(
            vec![],
            IntRange { min: None, max: Some(5), referenced: BTreeSet::new() },
        );
        let out = f.run(ItemStack::new(10), &ctx());
        assert_eq!(out.get_count(), 5);
    }

    #[test]
    fn run_clamps_to_min() {
        let f = LimitCount::new(
            vec![],
            IntRange { min: Some(3), max: None, referenced: BTreeSet::new() },
        );
        let out = f.run(ItemStack::new(1), &ctx());
        assert_eq!(out.get_count(), 3);
    }

    #[test]
    fn run_clamps_range_and_negative_values() {
        let f = LimitCount::new(
            vec![],
            IntRange { min: Some(-2), max: Some(4), referenced: BTreeSet::new() },
        );
        assert_eq!(f.run(ItemStack::new(-10), &ctx()).get_count(), -2);
        assert_eq!(f.run(ItemStack::new(10), &ctx()).get_count(), 4);
        assert_eq!(f.run(ItemStack::new(0), &ctx()).get_count(), 0);
    }

    #[test]
    fn get_type_and_referenced_params_delegate() {
        let mut refs = BTreeSet::new();
        refs.insert(ContextKey("foo".into()));
        let f = LimitCount::new(
            vec![],
            IntRange { min: Some(0), max: Some(1), referenced: refs.clone() },
        );
        assert_eq!(f.get_type(), LootItemFunctions::LIMIT_COUNT);
        assert_eq!(f.get_referenced_context_params(), refs);
    }

    #[test]
    fn builder_creates_function() {
        let builder = LimitCount::limit_count(IntRange {
            min: Some(1),
            max: Some(2),
            referenced: BTreeSet::new(),
        });
        let f = builder(vec![LootItemCondition]);
        assert_eq!(f.conditions.len(), 1);
        assert_eq!(f.limiter.min, Some(1));
        assert_eq!(f.limiter.max, Some(2));
    }
}
