use std::cell::RefCell;

/// Minimal item stack representation for loot generation.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
}

/// Source of random integers, matching Java `RandomSource` usage needed by loot pools.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Loot context providing randomness and luck.
/// Randomness uses interior mutability so shared references can advance random state,
/// matching Java's mutable object-reference semantics in a Rust-safe way.
pub struct LootContext<'a> {
    random: RefCell<&'a mut dyn RandomSource>,
    luck: f32,
}

impl<'a> LootContext<'a> {
    pub fn new(random: &'a mut dyn RandomSource, luck: f32) -> Self {
        Self {
            random: RefCell::new(random),
            luck,
        }
    }

    /// Equivalent to Java `LootContext#getRandom()`.
    pub fn get_random(&self) -> std::cell::RefMut<'_, &'a mut dyn RandomSource> {
        self.random.borrow_mut()
    }

    /// Equivalent to Java `LootContext#getLuck()`.
    pub fn get_luck(&self) -> f32 {
        self.luck
    }
}

/// Minimal validation context stub for `validate` traversal.
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    pub path: Vec<String>,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self { path: Vec::new() }
    }

    pub fn for_child(&self, element: impl Into<String>) -> Self {
        let mut child = self.clone();
        child.path.push(element.into());
        child
    }
}

/// Java `LootPoolEntry` behavior required by `LootPool`.
pub trait LootPoolEntry {
    fn get_weight(&self, luck: f32) -> i32;
    fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), ctx: &LootContext<'_>);
}

/// Java `LootPoolEntryContainer` behavior required by `LootPool`.
pub trait LootPoolEntryContainer {
    fn expand(&self, ctx: &LootContext<'_>, output: &mut dyn FnMut(Box<dyn LootPoolEntry>));
    fn validate(&self, ctx: &ValidationContext);
}

/// Java `LootItemCondition` behavior required by `LootPool`.
pub trait LootItemCondition {
    fn test(&self, ctx: &LootContext<'_>) -> bool;
    fn validate(&self, ctx: &ValidationContext);
}

/// Java `LootItemFunction` behavior required by `LootPool`.
pub trait LootItemFunction {
    fn apply(&self, stack: ItemStack, ctx: &LootContext<'_>) -> ItemStack;
    fn validate(&self, ctx: &ValidationContext);
}

/// Java `NumberProvider` behavior required by `LootPool`.
pub trait NumberProvider {
    fn get_int(&self, ctx: &LootContext<'_>) -> i32;
    fn get_float(&self, ctx: &LootContext<'_>) -> f32;
    fn validate(&self, ctx: &ValidationContext);
}

/// Minimal constant number provider, matching Java `ConstantValue` usage in defaults.
#[derive(Debug, Clone, Copy)]
pub struct ConstantValue {
    value: f32,
}

impl ConstantValue {
    pub fn exactly(value: f32) -> Self {
        Self { value }
    }
}

impl NumberProvider for ConstantValue {
    fn get_int(&self, _ctx: &LootContext<'_>) -> i32 {
        mth_floor(self.value)
    }

    fn get_float(&self, _ctx: &LootContext<'_>) -> f32 {
        self.value
    }

    fn validate(&self, _ctx: &ValidationContext) {}
}

/// Java `Mth.floor(float)`.
fn mth_floor(v: f32) -> i32 {
    let i = v as i32;
    if v < i as f32 {
        i.wrapping_sub(1)
    } else {
        i
    }
}

/// Port of `net.minecraft.world.level.storage.loot.LootPool`.
pub struct LootPool {
    entries: Vec<Box<dyn LootPoolEntryContainer>>,
    conditions: Vec<Box<dyn LootItemCondition>>,
    functions: Vec<Box<dyn LootItemFunction>>,
    rolls: Box<dyn NumberProvider>,
    bonus_rolls: Box<dyn NumberProvider>,
}

impl LootPool {
    // JAVA: net.minecraft.world.level.storage.loot.LootPool#<init>
    pub fn new(
        entries: Vec<Box<dyn LootPoolEntryContainer>>,
        conditions: Vec<Box<dyn LootItemCondition>>,
        functions: Vec<Box<dyn LootItemFunction>>,
        rolls: Box<dyn NumberProvider>,
        bonus_rolls: Box<dyn NumberProvider>,
    ) -> Self {
        Self {
            entries,
            conditions,
            functions,
            rolls,
            bonus_rolls,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool#addRandomItem
    fn add_random_item(&self, output: &mut dyn FnMut(ItemStack), ctx: &LootContext<'_>) {
        let luck = ctx.get_luck();
        let mut entries: Vec<Box<dyn LootPoolEntry>> = Vec::new();
        let mut total_weight: i32 = 0;

        for container in &self.entries {
            container.expand(ctx, &mut |entry| {
                let weight = entry.get_weight(luck);
                if weight > 0 {
                    total_weight = total_weight.wrapping_add(weight);
                    entries.push(entry);
                }
            });
        }

        let size = entries.len() as i32;
        if total_weight != 0 && size != 0 {
            if size == 1 {
                entries[0].create_item_stack(output, ctx);
            } else {
                let mut j = ctx.get_random().next_int(total_weight);
                for entry in &entries {
                    j = j.wrapping_sub(entry.get_weight(luck));
                    if j < 0 {
                        entry.create_item_stack(output, ctx);
                        return;
                    }
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool#addRandomItems
    pub fn add_random_items(&self, output: &mut dyn FnMut(ItemStack), ctx: &LootContext<'_>) {
        if self.conditions.iter().all(|c| c.test(ctx)) {
            let mut decorated = |stack: ItemStack| {
                let mut s = stack;
                for f in &self.functions {
                    s = f.apply(s, ctx);
                }
                output(s);
            };

            let rolls = self.rolls.get_int(ctx);
            let bonus = self.bonus_rolls.get_float(ctx);
            let luck = ctx.get_luck();
            let count = rolls.wrapping_add(mth_floor(bonus * luck));

            for _ in 0..count {
                self.add_random_item(&mut decorated, ctx);
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool#validate
    pub fn validate(&self, ctx: &ValidationContext) {
        for (i, condition) in self.conditions.iter().enumerate() {
            condition.validate(&ctx.for_child(format!("conditions[{i}]")));
        }
        for (i, function) in self.functions.iter().enumerate() {
            function.validate(&ctx.for_child(format!("functions[{i}]")));
        }
        for (i, entry) in self.entries.iter().enumerate() {
            entry.validate(&ctx.for_child(format!("entries[{i}]")));
        }
        self.rolls.validate(&ctx.for_child("rolls"));
        self.bonus_rolls.validate(&ctx.for_child("bonus_rolls"));
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool#lootPool
    pub fn loot_pool() -> Builder {
        Builder::new()
    }
}

/// Port of `net.minecraft.world.level.storage.loot.LootPool$Builder`.
pub struct Builder {
    entries: Vec<Box<dyn LootPoolEntryContainer>>,
    conditions: Vec<Box<dyn LootItemCondition>>,
    functions: Vec<Box<dyn LootItemFunction>>,
    rolls: Box<dyn NumberProvider>,
    bonus_rolls: Box<dyn NumberProvider>,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.LootPool$Builder#<init>
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            conditions: Vec::new(),
            functions: Vec::new(),
            rolls: Box::new(ConstantValue::exactly(1.0)),
            bonus_rolls: Box::new(ConstantValue::exactly(0.0)),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool$Builder#setRolls
    pub fn set_rolls(mut self, rolls: Box<dyn NumberProvider>) -> Self {
        self.rolls = rolls;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool$Builder#unwrap
    pub fn unwrap(self) -> Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool$Builder#setBonusRolls
    pub fn set_bonus_rolls(mut self, bonus_rolls: Box<dyn NumberProvider>) -> Self {
        self.bonus_rolls = bonus_rolls;
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool$Builder#add
    pub fn add(mut self, entry: Box<dyn LootPoolEntryContainer>) -> Self {
        self.entries.push(entry);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool$Builder#when
    pub fn when(mut self, condition: Box<dyn LootItemCondition>) -> Self {
        self.conditions.push(condition);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool$Builder#apply
    pub fn apply(mut self, function: Box<dyn LootItemFunction>) -> Self {
        self.functions.push(function);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootPool$Builder#build
    pub fn build(self) -> LootPool {
        LootPool::new(
            self.entries,
            self.conditions,
            self.functions,
            self.rolls,
            self.bonus_rolls,
        )
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        values: std::collections::VecDeque<i32>,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self {
                values: values.into(),
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            self.values.pop_front().expect("unexpected random call")
        }
    }

    #[derive(Clone)]
    struct FixedEntry {
        weight: i32,
        id: &'static str,
    }

    impl LootPoolEntry for FixedEntry {
        fn get_weight(&self, _luck: f32) -> i32 {
            self.weight
        }

        fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), _ctx: &LootContext<'_>) {
            output(ItemStack {
                id: self.id.to_string(),
                count: 1,
            });
        }
    }

    struct SimpleContainer {
        items: Vec<(i32, &'static str)>,
    }

    impl LootPoolEntryContainer for SimpleContainer {
        fn expand(&self, _ctx: &LootContext<'_>, output: &mut dyn FnMut(Box<dyn LootPoolEntry>)) {
            for &(weight, id) in &self.items {
                output(Box::new(FixedEntry { weight, id }));
            }
        }

        fn validate(&self, _ctx: &ValidationContext) {}
    }

    struct FalseCondition;
    impl LootItemCondition for FalseCondition {
        fn test(&self, _ctx: &LootContext<'_>) -> bool {
            false
        }
        fn validate(&self, _ctx: &ValidationContext) {}
    }

    struct AddSuffix(&'static str);
    impl LootItemFunction for AddSuffix {
        fn apply(&self, mut stack: ItemStack, _ctx: &LootContext<'_>) -> ItemStack {
            stack.id.push_str(self.0);
            stack
        }
        fn validate(&self, _ctx: &ValidationContext) {}
    }

    #[test]
    fn basic_single_entry_no_random_consumed() {
        let mut rng = SeqRandom::new(vec![]);
        let ctx = LootContext::new(&mut rng, 0.0);
        let pool = LootPool::new(
            vec![Box::new(SimpleContainer {
                items: vec![(1, "apple")],
            })],
            vec![],
            vec![],
            Box::new(ConstantValue::exactly(1.0)),
            Box::new(ConstantValue::exactly(0.0)),
        );
        let mut out = Vec::new();
        pool.add_random_items(&mut |s| out.push(s), &ctx);
        assert_eq!(
            out,
            vec![ItemStack {
                id: "apple".to_string(),
                count: 1
            }]
        );
    }

    #[test]
    fn condition_blocks_and_function_order() {
        let mut rng = SeqRandom::new(vec![]);
        let ctx = LootContext::new(&mut rng, 0.0);
        let blocked = LootPool::new(
            vec![Box::new(SimpleContainer {
                items: vec![(1, "x")],
            })],
            vec![Box::new(FalseCondition)],
            vec![],
            Box::new(ConstantValue::exactly(1.0)),
            Box::new(ConstantValue::exactly(0.0)),
        );
        let mut out = Vec::new();
        blocked.add_random_items(&mut |s| out.push(s), &ctx);
        assert!(out.is_empty());

        let pool = LootPool::new(
            vec![Box::new(SimpleContainer {
                items: vec![(1, "a")],
            })],
            vec![],
            vec![Box::new(AddSuffix("b")), Box::new(AddSuffix("c"))],
            Box::new(ConstantValue::exactly(1.0)),
            Box::new(ConstantValue::exactly(0.0)),
        );
        let mut out2 = Vec::new();
        pool.add_random_items(&mut |s| out2.push(s), &ctx);
        assert_eq!(out2[0].id, "abc");
    }

    #[test]
    fn weighted_selection_and_negative_rolls() {
        let mut rng = SeqRandom::new(vec![5]);
        let ctx = LootContext::new(&mut rng, 0.0);
        let pool = LootPool::new(
            vec![Box::new(SimpleContainer {
                items: vec![(3, "a"), (7, "b")],
            })],
            vec![],
            vec![],
            Box::new(ConstantValue::exactly(1.0)),
            Box::new(ConstantValue::exactly(0.0)),
        );
        let mut out = Vec::new();
        pool.add_random_items(&mut |s| out.push(s), &ctx);
        assert_eq!(out[0].id, "b");

        let mut rng2 = SeqRandom::new(vec![]);
        let ctx2 = LootContext::new(&mut rng2, 0.0);
        let no_rolls = LootPool::new(
            vec![Box::new(SimpleContainer {
                items: vec![(1, "x")],
            })],
            vec![],
            vec![],
            Box::new(ConstantValue::exactly(-1.0)),
            Box::new(ConstantValue::exactly(0.0)),
        );
        let mut out2 = Vec::new();
        no_rolls.add_random_items(&mut |s| out2.push(s), &ctx2);
        assert!(out2.is_empty());
    }

    #[test]
    fn bonus_rolls_float_order_and_mth_floor() {
        assert_eq!(mth_floor(1.9), 1);
        assert_eq!(mth_floor(-0.1), -1);
        assert_eq!(mth_floor(-1.0), -1);
        assert_eq!(mth_floor(-0.0f32), 0);
        assert_eq!(mth_floor(f32::NAN), 0);

        let mut rng = SeqRandom::new(vec![]);
        let ctx = LootContext::new(&mut rng, 2.0);
        // rolls=1 + floor(bonus=1.5 * luck=2.0)=3 => 4
        let pool = LootPool::new(
            vec![Box::new(SimpleContainer {
                items: vec![(1, "x")],
            })],
            vec![],
            vec![],
            Box::new(ConstantValue::exactly(1.0)),
            Box::new(ConstantValue::exactly(1.5)),
        );
        let mut out = Vec::new();
        pool.add_random_items(&mut |s| out.push(s), &ctx);
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn wrapping_weight_overflow_and_validate_paths() {
        let mut rng = SeqRandom::new(vec![0]);
        let ctx = LootContext::new(&mut rng, 0.0);
        let pool = LootPool::new(
            vec![Box::new(SimpleContainer {
                items: vec![(i32::MAX, "a"), (1, "b")],
            })],
            vec![],
            vec![],
            Box::new(ConstantValue::exactly(1.0)),
            Box::new(ConstantValue::exactly(0.0)),
        );
        let mut out = Vec::new();
        pool.add_random_items(&mut |s| out.push(s), &ctx);
        // total_weight wraps to i32::MIN (!= 0), behavior follows Java wrapping int semantics
        assert_eq!(out.len(), 1);

        struct RecCond;
        impl LootItemCondition for RecCond {
            fn test(&self, _ctx: &LootContext<'_>) -> bool {
                true
            }
            fn validate(&self, ctx: &ValidationContext) {
                assert_eq!(ctx.path.last().unwrap(), "conditions[0]");
            }
        }
        struct RecNum;
        impl NumberProvider for RecNum {
            fn get_int(&self, _ctx: &LootContext<'_>) -> i32 { 1 }
            fn get_float(&self, _ctx: &LootContext<'_>) -> f32 { 0.0 }
            fn validate(&self, ctx: &ValidationContext) {
                assert_eq!(ctx.path.last().unwrap(), "rolls");
            }
        }
        let p2 = LootPool::new(vec![], vec![Box::new(RecCond)], vec![], Box::new(RecNum), Box::new(ConstantValue::exactly(0.0)));
        p2.validate(&ValidationContext::new());
    }

    #[test]
    fn builder_defaults_and_float_bits() {
        let mut rng = SeqRandom::new(vec![]);
        let ctx = LootContext::new(&mut rng, 0.0);
        let pool = LootPool::loot_pool()
            .add(Box::new(SimpleContainer {
                items: vec![(1, "z")],
            }))
            .set_rolls(Box::new(ConstantValue::exactly(2.0)))
            .set_bonus_rolls(Box::new(ConstantValue::exactly(f32::from_bits(0x3F80_0001)))) // 1.0000001
            .build();
        let mut out = Vec::new();
        pool.add_random_items(&mut |s| out.push(s), &ctx);
        // bonus * luck(0.0) preserves float op order; 0.0 * x = 0.0, floor 0, rolls 2
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|s| s.id == "z"));
    }
}
