use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemConditionType(pub &'static str);

pub struct LootItemConditions;
impl LootItemConditions {
    pub const VALUE_CHECK: LootItemConditionType = LootItemConditionType("value_check");
}

#[derive(Debug, Default, Clone)]
pub struct LootContext {
    pub int_params: std::collections::HashMap<&'static str, i32>,
}

pub trait LootContextUser {
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
}

pub trait LootItemCondition: LootContextUser + fmt::Debug {
    fn get_type(&self) -> LootItemConditionType;
    fn test(&self, ctx: &LootContext) -> bool;
}

pub trait LootItemConditionBuilder: fmt::Debug {
    fn build(&self) -> Box<dyn LootItemCondition>;
}

pub trait NumberProvider: fmt::Debug {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getInt
    fn get_int(&self, ctx: &LootContext) -> i32;
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
}

#[derive(Debug, Clone)]
pub struct ConstantNumberProvider {
    pub value: i32,
}

impl NumberProvider for ConstantNumberProvider {
    fn get_int(&self, _ctx: &LootContext) -> i32 {
        self.value
    }
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        HashSet::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntRange {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl IntRange {
    // JAVA: net.minecraft.world.level.storage.loot.IntRange#exact
    pub fn exact(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.IntRange#lowerBound
    pub fn lower_bound(min: i32) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.IntRange#test
    pub fn test(&self, _ctx: &LootContext, value: i32) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        true
    }

    // JAVA: net.minecraft.world.level.storage.loot.IntRange#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        HashSet::new()
    }
}

#[derive(Debug, Clone)]
pub struct ValueCheckCondition {
    pub provider: Box<dyn NumberProviderClone>,
    pub range: IntRange,
}

trait NumberProviderClone: NumberProvider {
    fn clone_box(&self) -> Box<dyn NumberProviderClone>;
}

impl<T: NumberProvider + Clone + 'static> NumberProviderClone for T {
    fn clone_box(&self) -> Box<dyn NumberProviderClone> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn NumberProviderClone> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl ValueCheckCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.ValueCheckCondition#<init>
    pub fn new(provider: Box<dyn NumberProviderClone>, range: IntRange) -> Self {
        Self { provider, range }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ValueCheckCondition#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditions::VALUE_CHECK
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ValueCheckCondition#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = self.provider.get_referenced_context_params();
        set.extend(self.range.get_referenced_context_params());
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ValueCheckCondition#test
    pub fn test(&self, ctx: &LootContext) -> bool {
        self.range.test(ctx, self.provider.get_int(ctx))
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ValueCheckCondition#hasValue
    pub fn has_value(
        provider: Box<dyn NumberProviderClone>,
        range: IntRange,
    ) -> Box<dyn LootItemConditionBuilder> {
        Box::new(ValueCheckConditionBuilder { provider, range })
    }
}

impl LootContextUser for ValueCheckCondition {
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        ValueCheckCondition::get_referenced_context_params(self)
    }
}

impl LootItemCondition for ValueCheckCondition {
    fn get_type(&self) -> LootItemConditionType {
        ValueCheckCondition::get_type(self)
    }
    fn test(&self, ctx: &LootContext) -> bool {
        ValueCheckCondition::test(self, ctx)
    }
}

#[derive(Debug, Clone)]
struct ValueCheckConditionBuilder {
    provider: Box<dyn NumberProviderClone>,
    range: IntRange,
}

impl LootItemConditionBuilder for ValueCheckConditionBuilder {
    fn build(&self) -> Box<dyn LootItemCondition> {
        Box::new(ValueCheckCondition::new(self.provider.clone_box(), self.range))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        let cond = ValueCheckCondition::new(
            Box::new(ConstantNumberProvider { value: 5 }),
            IntRange::exact(5),
        );
        let ctx = LootContext::default();
        assert!(cond.test(&ctx));
        assert_eq!(cond.get_type(), LootItemConditions::VALUE_CHECK);
    }

    #[test]
    fn exact_mismatch() {
        let cond = ValueCheckCondition::new(
            Box::new(ConstantNumberProvider { value: 4 }),
            IntRange::exact(5),
        );
        assert!(!cond.test(&LootContext::default()));
    }

    #[test]
    fn lower_bound_includes_negative() {
        let cond = ValueCheckCondition::new(
            Box::new(ConstantNumberProvider { value: -1 }),
            IntRange::lower_bound(-5),
        );
        assert!(cond.test(&LootContext::default()));
        let cond2 = ValueCheckCondition::new(
            Box::new(ConstantNumberProvider { value: -10 }),
            IntRange::lower_bound(-5),
        );
        assert!(!cond2.test(&LootContext::default()));
    }

    #[test]
    fn range_with_i32_extremes() {
        let cond = ValueCheckCondition::new(
            Box::new(ConstantNumberProvider { value: i32::MIN }),
            IntRange { min: Some(i32::MIN), max: Some(i32::MAX) },
        );
        assert!(cond.test(&LootContext::default()));
        let cond2 = ValueCheckCondition::new(
            Box::new(ConstantNumberProvider { value: i32::MAX }),
            IntRange::exact(i32::MAX),
        );
        assert!(cond2.test(&LootContext::default()));
    }

    #[test]
    fn has_value_builder() {
        let builder = ValueCheckCondition::has_value(
            Box::new(ConstantNumberProvider { value: 10 }),
            IntRange { min: Some(0), max: Some(20) },
        );
        let cond = builder.build();
        assert!(cond.test(&LootContext::default()));
        assert!(cond.get_referenced_context_params().is_empty());
    }
}
