use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
}

impl ItemStack {
    pub fn new(id: impl Into<String>, count: i32) -> Self {
        Self {
            id: id.into(),
            count,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootContext;

pub trait ItemPredicate: fmt::Debug {
    fn test(&self, stack: &ItemStack) -> bool;
}

pub trait LootItemCondition: fmt::Debug {
    fn test(&self, ctx: &LootContext) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct ProblemReporter;

impl ProblemReporter {
    pub fn field_path_element(_name: &str) -> ProblemReporter {
        ProblemReporter
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidationContext;

impl ValidationContext {
    pub fn for_child(&self, _reporter: ProblemReporter) -> ValidationContext {
        ValidationContext
    }
}

pub trait LootItemFunction: fmt::Debug {
    fn apply(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack;
    fn validate(&self, ctx: &ValidationContext);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const FILTERED: LootItemFunctionType = LootItemFunctionType("filtered");
}

#[derive(Debug)]
pub struct LootItemConditionalFunctionBase {
    conditions: Vec<Box<dyn LootItemCondition>>,
}

impl LootItemConditionalFunctionBase {
    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#<init>
    pub fn new(conditions: Vec<Box<dyn LootItemCondition>>) -> Self {
        Self { conditions }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#getConditions
    pub fn get_conditions(&self) -> &Vec<Box<dyn LootItemCondition>> {
        &self.conditions
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#validate
    pub fn validate(&self, _ctx: &ValidationContext) {}

    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#apply
    pub fn can_run(&self, ctx: &LootContext) -> bool {
        self.conditions.iter().all(|c| c.test(ctx))
    }
}

pub struct FilteredFunction {
    base: LootItemConditionalFunctionBase,
    filter: Box<dyn ItemPredicate>,
    on_pass: Option<Box<dyn LootItemFunction>>,
    on_fail: Option<Box<dyn LootItemFunction>>,
}

impl fmt::Debug for FilteredFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilteredFunction")
            .field("conditions_len", &self.base.conditions.len())
            .field("filter", &self.filter)
            .field("has_on_pass", &self.on_pass.is_some())
            .field("has_on_fail", &self.on_fail.is_some())
            .finish()
    }
}

impl FilteredFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction#<init>
    pub fn new(
        conditions: Vec<Box<dyn LootItemCondition>>,
        filter: Box<dyn ItemPredicate>,
        on_pass: Option<Box<dyn LootItemFunction>>,
        on_fail: Option<Box<dyn LootItemFunction>>,
    ) -> Self {
        Self {
            base: LootItemConditionalFunctionBase::new(conditions),
            filter,
            on_pass,
            on_fail,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::FILTERED
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction#run
    pub fn run(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack {
        let selected = if self.filter.test(&stack) {
            &self.on_pass
        } else {
            &self.on_fail
        };
        match selected {
            Some(func) => func.apply(stack, ctx),
            None => stack,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction#validate
    pub fn validate(&self, ctx: &ValidationContext) {
        self.base.validate(ctx);
        if let Some(func) = &self.on_pass {
            func.validate(&ctx.for_child(ProblemReporter::field_path_element("on_pass")));
        }
        if let Some(func) = &self.on_fail {
            func.validate(&ctx.for_child(ProblemReporter::field_path_element("on_fail")));
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction#filtered
    pub fn filtered(filter: Box<dyn ItemPredicate>) -> Builder {
        Builder::new(filter)
    }
}

impl LootItemFunction for FilteredFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#apply
    fn apply(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if !self.base.can_run(ctx) {
            return stack;
        }
        self.run(stack, ctx)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction#validate
    fn validate(&self, ctx: &ValidationContext) {
        FilteredFunction::validate(self, ctx)
    }
}

pub struct Builder {
    conditions: Vec<Box<dyn LootItemCondition>>,
    item_predicate: Box<dyn ItemPredicate>,
    on_pass: Option<Box<dyn LootItemFunction>>,
    on_fail: Option<Box<dyn LootItemFunction>>,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction$Builder#<init>
    pub fn new(item_predicate: Box<dyn ItemPredicate>) -> Self {
        Self {
            conditions: Vec::new(),
            item_predicate,
            on_pass: None,
            on_fail: None,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction$Builder#getThis
    pub fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction$Builder#onPass
    pub fn on_pass(mut self, func: Option<Box<dyn LootItemFunction>>) -> Self {
        self.on_pass = func;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction$Builder#onFail
    pub fn on_fail(mut self, func: Option<Box<dyn LootItemFunction>>) -> Self {
        self.on_fail = func;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FilteredFunction$Builder#build
    pub fn build(self) -> Box<dyn LootItemFunction> {
        Box::new(FilteredFunction::new(
            self.conditions,
            self.item_predicate,
            self.on_pass,
            self.on_fail,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[derive(Debug)]
    struct IdPredicate(String);
    impl ItemPredicate for IdPredicate {
        fn test(&self, stack: &ItemStack) -> bool {
            stack.id == self.0
        }
    }

    #[derive(Debug)]
    struct SetCountFn(i32);
    impl LootItemFunction for SetCountFn {
        fn apply(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
            stack.count = self.0;
            stack
        }
        fn validate(&self, _ctx: &ValidationContext) {}
    }

    #[derive(Debug)]
    struct ValidateCounter(Arc<AtomicUsize>);
    impl LootItemFunction for ValidateCounter {
        fn apply(&self, stack: ItemStack, _ctx: &LootContext) -> ItemStack {
            stack
        }
        fn validate(&self, _ctx: &ValidationContext) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[derive(Debug)]
    struct FalseCondition;
    impl LootItemCondition for FalseCondition {
        fn test(&self, _ctx: &LootContext) -> bool {
            false
        }
    }

    #[test]
    fn run_selects_pass_or_fail_branch() {
        let ctx = LootContext::default();
        let f = FilteredFunction::new(
            vec![],
            Box::new(IdPredicate("minecraft:stone".into())),
            Some(Box::new(SetCountFn(10))),
            Some(Box::new(SetCountFn(20))),
        );
        let pass = f.run(ItemStack::new("minecraft:stone", 1), &ctx);
        let fail = f.run(ItemStack::new("minecraft:dirt", 1), &ctx);
        assert_eq!(pass.count, 10);
        assert_eq!(fail.count, 20);
    }

    #[test]
    fn missing_branch_returns_original_with_negative_count() {
        let ctx = LootContext::default();
        let f = FilteredFunction::new(vec![], Box::new(IdPredicate("a".into())), None, None);
        let original = ItemStack::new("b", -7);
        let out = f.run(original.clone(), &ctx);
        assert_eq!(out, original);
    }

    #[test]
    fn extreme_int_counts_and_overflow_adjacent_values_preserved() {
        let ctx = LootContext::default();
        let f = FilteredFunction::new(vec![], Box::new(IdPredicate("no".into())), None, None);
        let min = f.run(ItemStack::new("x", i32::MIN), &ctx);
        let max = f.run(ItemStack::new("x", i32::MAX), &ctx);
        let near_overflow = f.run(ItemStack::new("x", i32::MAX.wrapping_add(0)), &ctx);
        assert_eq!(min.count, i32::MIN);
        assert_eq!(max.count, i32::MAX);
        assert_eq!(near_overflow.count, i32::MAX);
    }

    #[test]
    fn float_bit_patterns_preserved_in_int_payload() {
        let ctx = LootContext::default();
        let nan_bits = f32::NAN.to_bits() as i32;
        let neg_zero_bits = (-0.0f32).to_bits() as i32;
        let inf_bits = f32::INFINITY.to_bits() as i32;
        let f = FilteredFunction::new(vec![], Box::new(IdPredicate("no".into())), None, None);
        assert_eq!(f.run(ItemStack::new("a", nan_bits), &ctx).count as u32, f32::NAN.to_bits());
        assert_eq!(f.run(ItemStack::new("a", neg_zero_bits), &ctx).count as u32, (-0.0f32).to_bits());
        assert_eq!(f.run(ItemStack::new("a", inf_bits), &ctx).count as u32, f32::INFINITY.to_bits());
    }

    #[test]
    fn validate_reports_both_present_branches() {
        let pass_hits = Arc::new(AtomicUsize::new(0));
        let fail_hits = Arc::new(AtomicUsize::new(0));
        let f = FilteredFunction::new(
            vec![],
            Box::new(IdPredicate("a".into())),
            Some(Box::new(ValidateCounter(pass_hits.clone()))),
            Some(Box::new(ValidateCounter(fail_hits.clone()))),
        );
        f.validate(&ValidationContext::default());
        assert_eq!(pass_hits.load(Ordering::SeqCst), 1);
        assert_eq!(fail_hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn builder_and_condition_gate_apply() {
        let ctx = LootContext::default();
        let func = FilteredFunction::filtered(Box::new(IdPredicate("ok".into())))
            .on_pass(Some(Box::new(SetCountFn(7))))
            .on_fail(Some(Box::new(SetCountFn(9))))
            .build();
        let out = func.apply(ItemStack::new("ok", 1), &ctx);
        assert_eq!(out.count, 7);

        let gated = FilteredFunction::new(
            vec![Box::new(FalseCondition)],
            Box::new(IdPredicate("ok".into())),
            Some(Box::new(SetCountFn(7))),
            Some(Box::new(SetCountFn(9))),
        );
        let gated_out = LootItemFunction::apply(&gated, ItemStack::new("ok", 1), &ctx);
        assert_eq!(gated_out.count, 1);
    }
}
