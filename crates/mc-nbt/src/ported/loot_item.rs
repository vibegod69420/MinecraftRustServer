use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Holder<T: Clone> {
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: String,
}

impl Item {
    // JAVA: net.minecraft.world.item.Item#builtInRegistryHolder
    pub fn built_in_registry_holder(&self) -> Holder<Item> {
        Holder {
            value: self.clone(),
        }
    }
}

pub trait ItemLike {
    // JAVA: net.minecraft.world.level.ItemLike#asItem
    fn as_item(&self) -> Item;
}

impl ItemLike for Item {
    fn as_item(&self) -> Item {
        self.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item: Holder<Item>,
    pub count: i32,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#<init>
    pub fn from_holder(item: Holder<Item>) -> Self {
        Self { item, count: 1 }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LootItemCondition;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LootItemFunction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootPoolEntryType {
    pub id: String,
}

pub struct LootPoolEntries;

impl LootPoolEntries {
    pub fn item() -> LootPoolEntryType {
        LootPoolEntryType {
            id: "item".to_string(),
        }
    }
}

#[derive(Default)]
pub struct LootContext;

pub struct LootPoolSingletonContainerBuilder<T> {
    factory: Box<dyn Fn(i32, i32, Vec<LootItemCondition>, Vec<LootItemFunction>) -> T>,
    pub weight: i32,
    pub quality: i32,
    pub conditions: Vec<LootItemCondition>,
    pub functions: Vec<LootItemFunction>,
}

impl<T> fmt::Debug for LootPoolSingletonContainerBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LootPoolSingletonContainerBuilder")
            .field("weight", &self.weight)
            .field("quality", &self.quality)
            .field("conditions", &self.conditions)
            .field("functions", &self.functions)
            .finish()
    }
}

impl<T> LootPoolSingletonContainerBuilder<T> {
    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolSingletonContainer#simpleBuilder
    pub fn simple_builder<F>(factory: F) -> Self
    where
        F: Fn(i32, i32, Vec<LootItemCondition>, Vec<LootItemFunction>) -> T + 'static,
    {
        Self {
            factory: Box::new(factory),
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn set_weight(mut self, weight: i32) -> Self {
        self.weight = weight;
        self
    }

    pub fn set_quality(mut self, quality: i32) -> Self {
        self.quality = quality;
        self
    }

    pub fn build(self) -> T {
        (self.factory)(
            self.weight,
            self.quality,
            self.conditions,
            self.functions,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootItem {
    pub item: Holder<Item>,
    pub weight: i32,
    pub quality: i32,
    pub conditions: Vec<LootItemCondition>,
    pub functions: Vec<LootItemFunction>,
}

impl LootItem {
    // JAVA: net.minecraft.world.level.storage.loot.entries.LootItem#<init>
    fn new(
        item: Holder<Item>,
        weight: i32,
        quality: i32,
        conditions: Vec<LootItemCondition>,
        functions: Vec<LootItemFunction>,
    ) -> Self {
        Self {
            item,
            weight,
            quality,
            conditions,
            functions,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootItem#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntries::item()
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootItem#createItemStack
    pub fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), _context: &mut LootContext) {
        output(ItemStack::from_holder(self.item.clone()));
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootItem#lootTableItem
    pub fn loot_table_item(item_like: &impl ItemLike) -> LootPoolSingletonContainerBuilder<LootItem> {
        let holder = item_like.as_item().built_in_registry_holder();
        LootPoolSingletonContainerBuilder::simple_builder(
            move |weight, quality, conditions, functions| {
                LootItem::new(holder.clone(), weight, quality, conditions, functions)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loot_table_item_builds_with_defaults() {
        let item = Item {
            id: "minecraft:stone".to_string(),
        };
        let loot = LootItem::loot_table_item(&item).build();
        assert_eq!(loot.item.value.id, "minecraft:stone");
        assert_eq!(loot.weight, 1);
        assert_eq!(loot.quality, 0);
        assert!(loot.conditions.is_empty());
        assert!(loot.functions.is_empty());
        assert_eq!(loot.get_type().id, "item");
    }

    #[test]
    fn loot_table_item_preserves_negative_weight_and_quality() {
        let item = Item {
            id: "minecraft:dirt".to_string(),
        };
        let loot = LootItem::loot_table_item(&item)
            .set_weight(i32::MIN)
            .set_quality(-1)
            .build();
        assert_eq!(loot.weight, i32::MIN);
        assert_eq!(loot.quality, -1);
    }

    #[test]
    fn create_item_stack_outputs_single_item_stack() {
        let item = Item {
            id: "minecraft:diamond".to_string(),
        };
        let loot = LootItem::loot_table_item(&item).build();
        let mut ctx = LootContext;
        let mut out = Vec::new();
        loot.create_item_stack(&mut |s| out.push(s), &mut ctx);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].item.value.id, "minecraft:diamond");
        assert_eq!(out[0].count, 1);
    }

    #[test]
    fn item_like_and_registry_holder_preserve_identity() {
        let item = Item {
            id: "minecraft:apple".to_string(),
        };
        let as_item = item.as_item();
        let holder = as_item.built_in_registry_holder();
        assert_eq!(holder.value, item);
    }

    #[test]
    fn builder_can_set_extreme_weights() {
        let item = Item {
            id: "minecraft:stick".to_string(),
        };
        let loot = LootItem::loot_table_item(&item)
            .set_weight(i32::MAX)
            .set_quality(i32::MAX)
            .build();
        assert_eq!(loot.weight, i32::MAX);
        assert_eq!(loot.quality, i32::MAX);
    }
}
