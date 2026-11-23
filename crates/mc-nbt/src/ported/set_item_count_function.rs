// Self-contained faithful port of
// net.minecraft.world.level.storage.loot.functions.SetItemCountFunction

use std::collections::HashSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#getCount
    pub fn get_count(&self) -> i32 {
        self.count
    }

    // JAVA: net.minecraft.world.item.ItemStack#setCount
    pub fn set_count(&mut self, count: i32) {
        self.count = count;
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey(pub String);

pub trait NumberProvider {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getInt
    fn get_int(&self, ctx: &LootContext) -> i32;
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
}

#[derive(Debug, Clone)]
pub struct ConstantNumberProvider {
    pub value: i32,
    pub refs: HashSet<ContextKey>,
}

impl NumberProvider for ConstantNumberProvider {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getInt
    fn get_int(&self, _ctx: &LootContext) -> i32 {
        self.value
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        self.refs.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    SetCount,
}

pub type LootItemConditionalFunctionBuilder<T> =
    Box<dyn FnOnce(Vec<LootItemCondition>) -> T>;

// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#simpleBuilder
pub fn simple_builder<T, F>(f: F) -> LootItemConditionalFunctionBuilder<T>
where
    F: FnOnce(Vec<LootItemCondition>) -> T + 'static,
{
    Box::new(f)
}

pub struct SetItemCountFunction<P: NumberProvider> {
    pub conditions: Vec<LootItemCondition>,
    pub value: P,
    pub add: bool,
}

impl<P: NumberProvider> SetItemCountFunction<P> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#CODEC
    // Codec placeholder: RecordCodecBuilder.mapCodec(commonFields(...).and(group(NumberProviders.CODEC.fieldOf("count"), Codec.BOOL.fieldOf("add").orElse(false))...))
    pub const CODEC_NAME: &'static str = "minecraft:set_count";

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#<init>
    fn new(conditions: Vec<LootItemCondition>, value: P, add: bool) -> Self {
        Self { conditions, value, add }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::SetCount
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        self.value.get_referenced_context_params()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        let base: i32 = if self.add { stack.get_count() } else { 0 };
        let new_count = base.wrapping_add(self.value.get_int(ctx));
        stack.set_count(new_count);
        stack
    }
}

impl<P: NumberProvider + 'static> SetItemCountFunction<P> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#setCount
    pub fn set_count(value: P) -> LootItemConditionalFunctionBuilder<SetItemCountFunction<P>>
    where
        P: Clone,
    {
        let v = value.clone();
        simple_builder(move |conditions| SetItemCountFunction::new(conditions, v, false))
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#setCount
    pub fn set_count_with_add(
        value: P,
        add: bool,
    ) -> LootItemConditionalFunctionBuilder<SetItemCountFunction<P>>
    where
        P: Clone,
    {
        let v = value.clone();
        simple_builder(move |conditions| SetItemCountFunction::new(conditions, v, add))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn const_provider(v: i32) -> ConstantNumberProvider {
        ConstantNumberProvider { value: v, refs: HashSet::new() }
    }

    #[test]
    fn set_count_replaces_value() {
        let f = SetItemCountFunction::set_count(const_provider(5))(vec![]);
        let out = f.run(ItemStack { count: 10 }, &LootContext::default());
        assert_eq!(out.get_count(), 5);
        assert_eq!(f.get_type(), LootItemFunctionType::SetCount);
        assert!(!f.add);
    }

    #[test]
    fn set_count_adds_to_existing() {
        let f = SetItemCountFunction::set_count_with_add(const_provider(3), true)(vec![]);
        let out = f.run(ItemStack { count: 7 }, &LootContext::default());
        assert_eq!(out.get_count(), 10);
        assert!(f.add);
    }

    #[test]
    fn wrapping_overflow_on_add() {
        let f = SetItemCountFunction::set_count_with_add(const_provider(1), true)(vec![]);
        let out = f.run(ItemStack { count: i32::MAX }, &LootContext::default());
        assert_eq!(out.get_count(), i32::MAX.wrapping_add(1));
    }

    #[test]
    fn negative_values_and_wrapping_underflow() {
        let f = SetItemCountFunction::set_count_with_add(const_provider(-1), true)(vec![]);
        let out = f.run(ItemStack { count: i32::MIN }, &LootContext::default());
        assert_eq!(out.get_count(), i32::MIN.wrapping_add(-1));

        let f2 = SetItemCountFunction::set_count(const_provider(-5))(vec![]);
        let out2 = f2.run(ItemStack { count: 100 }, &LootContext::default());
        assert_eq!(out2.get_count(), -5);
    }

    #[test]
    fn referenced_context_params_delegated() {
        let mut refs = HashSet::new();
        refs.insert(ContextKey("this".into()));
        let provider = ConstantNumberProvider { value: 1, refs: refs.clone() };
        let f = SetItemCountFunction::new(vec![], provider, false);
        assert_eq!(f.get_referenced_context_params(), refs);
    }

    #[test]
    fn float_bits_not_applicable_but_integer_bits_preserved() {
        // Ensures exact i32 bit patterns survive through set/get (including sign bit).
        let val: i32 = -0x1234_5678i32;
        let f = SetItemCountFunction::set_count(const_provider(val))(vec![]);
        let out = f.run(ItemStack { count: 0 }, &LootContext::default());
        assert_eq!(out.get_count().to_be_bytes(), val.to_be_bytes());
    }
}
