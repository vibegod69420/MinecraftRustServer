use std::marker::PhantomData;
use std::sync::LazyLock;

/// Placeholder for unported `com.mojang.serialization.MapCodec<T>`.
#[derive(Debug)]
pub struct MapCodec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> MapCodec<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for MapCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder for unported `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
}

impl ItemStack {
    pub fn empty() -> Self {
        Self { count: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }
}

/// Placeholder for unported `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LootContext;

/// Placeholder for unported `net.minecraft.world.level.storage.loot.predicates.LootItemCondition`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition {
    pub id: i32,
}

/// Placeholder for unported
/// `net.minecraft.world.level.storage.loot.functions.LootItemFunctionType<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemFunctionType<T> {
    pub name: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> LootItemFunctionType<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

/// Placeholder for
/// `net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction.Builder<?>`.
#[derive(Debug, Clone)]
pub struct SimpleConditionalBuilder<T> {
    predicates: Vec<LootItemCondition>,
    build_fn: fn(Vec<LootItemCondition>) -> T,
}

impl<T> SimpleConditionalBuilder<T> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction$Builder#build
    pub fn build(self) -> T {
        (self.build_fn)(self.predicates)
    }

    pub fn with_predicate(mut self, predicate: LootItemCondition) -> Self {
        self.predicates.push(predicate);
        self
    }
}

// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#simpleBuilder
pub fn simple_builder<T>(f: fn(Vec<LootItemCondition>) -> T) -> SimpleConditionalBuilder<T> {
    SimpleConditionalBuilder {
        predicates: Vec::new(),
        build_fn: f,
    }
}

/// Placeholder for unported `net.minecraft.world.level.storage.loot.functions.LootItemFunctions`.
pub struct LootItemFunctions;

impl LootItemFunctions {
    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunctions#DISCARD
    pub fn discard() -> &'static LootItemFunctionType<DiscardItem> {
        static DISCARD: LazyLock<LootItemFunctionType<DiscardItem>> =
            LazyLock::new(|| LootItemFunctionType::new("minecraft:discard"));
        &DISCARD
    }
}

/// Port of `net.minecraft.world.level.storage.loot.functions.DiscardItem`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscardItem {
    predicates: Vec<LootItemCondition>,
}

// JAVA: net.minecraft.world.level.storage.loot.functions.DiscardItem#CODEC
pub static CODEC: LazyLock<MapCodec<DiscardItem>> = LazyLock::new(MapCodec::new);

impl DiscardItem {
    // JAVA: net.minecraft.world.level.storage.loot.functions.DiscardItem#<init>
    pub fn new(predicates: Vec<LootItemCondition>) -> Self {
        Self { predicates }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.DiscardItem#getType
    pub fn get_type(&self) -> &'static LootItemFunctionType<DiscardItem> {
        LootItemFunctions::discard()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.DiscardItem#run
    pub fn run(&self, _stack: ItemStack, _context: &LootContext) -> ItemStack {
        ItemStack::empty()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.DiscardItem#discardItem
    pub fn discard_item() -> SimpleConditionalBuilder<DiscardItem> {
        simple_builder(DiscardItem::new)
    }

    pub fn predicates(&self) -> &[LootItemCondition] {
        &self.predicates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_returns_empty_for_normal_stack() {
        let f = DiscardItem::new(Vec::new());
        let ctx = LootContext;
        let out = f.run(ItemStack { count: 64 }, &ctx);
        assert_eq!(out, ItemStack::empty());
        assert!(out.is_empty());
    }

    #[test]
    fn run_returns_empty_for_negative_and_overflow_counts() {
        let f = DiscardItem::new(Vec::new());
        let ctx = LootContext;
        let neg = f.run(ItemStack { count: -1 }, &ctx);
        let min = f.run(ItemStack { count: i32::MIN }, &ctx);
        let max = f.run(ItemStack { count: i32::MAX }, &ctx);
        assert!(neg.is_empty());
        assert!(min.is_empty());
        assert!(max.is_empty());
    }

    #[test]
    fn run_returns_empty_for_float_bit_counts() {
        let f = DiscardItem::new(Vec::new());
        let ctx = LootContext;
        let nan_bits = f32::NAN.to_bits() as i32;
        let neg_zero_bits = (-0.0f32).to_bits() as i32;
        let inf_bits = f32::INFINITY.to_bits() as i32;
        let out_nan = f.run(ItemStack { count: nan_bits }, &ctx);
        let out_neg_zero = f.run(ItemStack { count: neg_zero_bits }, &ctx);
        let out_inf = f.run(ItemStack { count: inf_bits }, &ctx);
        assert!(out_nan.is_empty());
        assert!(out_neg_zero.is_empty());
        assert!(out_inf.is_empty());
    }

    #[test]
    fn get_type_is_discard() {
        let f = DiscardItem::new(Vec::new());
        let ty = f.get_type();
        assert_eq!(ty.name, "minecraft:discard");
        assert_eq!(ty, LootItemFunctions::discard());
    }

    #[test]
    fn discard_item_builder_builds_function() {
        let built = DiscardItem::discard_item()
            .with_predicate(LootItemCondition { id: 1 })
            .with_predicate(LootItemCondition { id: -2 })
            .build();
        assert_eq!(
            built.predicates(),
            &[LootItemCondition { id: 1 }, LootItemCondition { id: -2 }]
        );
    }

    #[test]
    fn constructor_stores_predicates_and_codec_exists() {
        let preds = vec![
            LootItemCondition { id: i32::MAX },
            LootItemCondition { id: i32::MIN },
        ];
        let f = DiscardItem::new(preds.clone());
        assert_eq!(f.predicates(), &preds[..]);
        let _codec: &MapCodec<DiscardItem> = &CODEC;
    }
}
