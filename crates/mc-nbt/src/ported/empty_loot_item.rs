use std::marker::PhantomData;
use std::sync::LazyLock;

/// Minimal stub for unported `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemStack {
    pub count: i32,
}

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub luck: f32,
}

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.predicates.LootItemCondition`.
#[derive(Debug, Clone, Default)]
pub struct LootItemCondition;

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.functions.LootItemFunction`.
#[derive(Debug, Clone, Default)]
pub struct LootItemFunction;

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer`.
#[derive(Debug, Clone, Default)]
pub struct LootPoolEntryContainer;

/// Minimal stub for unported `com.mojang.serialization.MapCodec`.
pub struct MapCodec<T: ?Sized>(PhantomData<fn() -> T>);

impl<T: ?Sized> MapCodec<T> {
    fn stub() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized> Default for MapCodec<T> {
    fn default() -> Self {
        Self::stub()
    }
}

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.entries.LootPoolEntryType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootPoolEntryType {
    pub name: &'static str,
}

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.entries.LootPoolEntries`.
pub struct LootPoolEntries;

impl LootPoolEntries {
    /// Stub for `LootPoolEntries.EMPTY`.
    pub fn empty() -> LootPoolEntryType {
        LootPoolEntryType { name: "empty" }
    }
}

type SingletonFactory<T> =
    Box<dyn Fn(i32, i32, Vec<LootItemCondition>, Vec<LootItemFunction>) -> T>;

/// Minimal stub for `net.minecraft.world.level.storage.loot.entries.LootPoolSingletonContainer.Builder`.
pub struct LootPoolSingletonContainerBuilder<T> {
    pub weight: i32,
    pub quality: i32,
    pub conditions: Vec<LootItemCondition>,
    pub functions: Vec<LootItemFunction>,
    factory: SingletonFactory<T>,
}

impl<T> LootPoolSingletonContainerBuilder<T> {
    /// Builds the configured singleton entry. Stub helper for self-contained testing.
    pub fn build(self) -> T {
        (self.factory)(self.weight, self.quality, self.conditions, self.functions)
    }
}

/// Minimal stub for `LootPoolSingletonContainer.simpleBuilder(...)`.
fn simple_builder<T, F>(factory: F) -> LootPoolSingletonContainerBuilder<T>
where
    F: Fn(i32, i32, Vec<LootItemCondition>, Vec<LootItemFunction>) -> T + 'static,
{
    LootPoolSingletonContainerBuilder {
        weight: 1,
        quality: 0,
        conditions: Vec::new(),
        functions: Vec::new(),
        factory: Box::new(factory),
    }
}

/// Port of `net.minecraft.world.level.storage.loot.entries.EmptyLootItem`.
#[derive(Debug, Clone)]
pub struct EmptyLootItem {
    pub weight: i32,
    pub quality: i32,
    pub conditions: Vec<LootItemCondition>,
    pub functions: Vec<LootItemFunction>,
}

// JAVA: net.minecraft.world.level.storage.loot.entries.EmptyLootItem#CODEC
pub static CODEC: LazyLock<MapCodec<EmptyLootItem>> = LazyLock::new(MapCodec::stub);

impl EmptyLootItem {
    // JAVA: net.minecraft.world.level.storage.loot.entries.EmptyLootItem#EmptyLootItem
    fn new(
        weight: i32,
        quality: i32,
        conditions: Vec<LootItemCondition>,
        functions: Vec<LootItemFunction>,
    ) -> Self {
        Self {
            weight,
            quality,
            conditions,
            functions,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.EmptyLootItem#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntries::empty()
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.EmptyLootItem#createItemStack
    pub fn create_item_stack(&self, _output: &mut dyn FnMut(ItemStack), _context: &LootContext) {}

    // JAVA: net.minecraft.world.level.storage.loot.entries.EmptyLootItem#emptyItem
    pub fn empty_item() -> LootPoolSingletonContainerBuilder<EmptyLootItem> {
        simple_builder(EmptyLootItem::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_type_returns_empty() {
        let item = EmptyLootItem::new(1, 0, Vec::new(), Vec::new());
        assert_eq!(item.get_type(), LootPoolEntries::empty());
        assert_eq!(item.get_type().name, "empty");
    }

    #[test]
    fn create_item_stack_produces_nothing() {
        let item = EmptyLootItem::new(1, 0, Vec::new(), Vec::new());
        let ctx = LootContext {
            luck: f32::from_bits(0x8000_0000), // -0.0 float bits edge case
        };
        let mut count = 0i32;
        item.create_item_stack(&mut |_stack| count = count.wrapping_add(1), &ctx);
        assert_eq!(count, 0);
    }

    #[test]
    fn empty_item_builder_defaults_and_build() {
        let built = EmptyLootItem::empty_item().build();
        assert_eq!(built.weight, 1);
        assert_eq!(built.quality, 0);
        assert!(built.conditions.is_empty());
        assert!(built.functions.is_empty());
    }

    #[test]
    fn preserves_negative_and_extreme_integer_values() {
        let item = EmptyLootItem::new(
            i32::MIN,
            i32::MAX,
            vec![LootItemCondition],
            vec![LootItemFunction, LootItemFunction],
        );
        assert_eq!(item.weight, i32::MIN);
        assert_eq!(item.quality, i32::MAX);
        assert_eq!(item.conditions.len(), 1);
        assert_eq!(item.functions.len(), 2);

        let overflowed_weight = item.weight.wrapping_add(-1);
        assert_eq!(overflowed_weight, i32::MAX);
    }

    #[test]
    fn codec_static_is_initialized() {
        let _codec: &MapCodec<EmptyLootItem> = &CODEC;
        let _default: MapCodec<EmptyLootItem> = MapCodec::default();
    }
}
