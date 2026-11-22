use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: String,
}

pub trait ItemLike {
    fn as_item(&self) -> &Item;
}

impl ItemLike for Item {
    fn as_item(&self) -> &Item {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

impl<T> Holder<T> {
    // JAVA: net.minecraft.core.Holder#value
    pub fn value(&self) -> &T {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SET_ITEM: LootItemFunctionType = LootItemFunctionType("set_item");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item: Item,
    pub count: i32,
}

impl ItemStack {
    pub fn new(item: Item, count: i32) -> Self {
        Self { item, count }
    }

    // JAVA: net.minecraft.world.item.ItemStack#transmuteCopy
    pub fn transmute_copy(&self, item: &dyn ItemLike) -> Self {
        Self {
            item: item.as_item().clone(),
            count: self.count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetItemFunction {
    pub conditions: Vec<LootItemCondition>,
    pub item: Holder<Item>,
}

impl SetItemFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemFunction#<init>
    fn new(conditions: Vec<LootItemCondition>, item: Holder<Item>) -> Self {
        Self { conditions, item }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_ITEM
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetItemFunction#run
    pub fn run(&self, stack: ItemStack, _ctx: &LootContext) -> ItemStack {
        stack.transmute_copy(self.item.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> LootContext {
        LootContext
    }

    #[test]
    fn run_transmutes_item_preserves_count() {
        let f = SetItemFunction::new(vec![], Holder(Item { id: "minecraft:diamond".into() }));
        let stack = ItemStack::new(Item { id: "minecraft:stone".into() }, 42);
        let out = f.run(stack, &ctx());
        assert_eq!(out.item.id, "minecraft:diamond");
        assert_eq!(out.count, 42);
    }

    #[test]
    fn get_type_is_set_item() {
        let f = SetItemFunction::new(vec![], Holder(Item { id: "minecraft:apple".into() }));
        assert_eq!(f.get_type(), LootItemFunctions::SET_ITEM);
    }

    #[test]
    fn run_preserves_zero_and_negative_counts() {
        let f = SetItemFunction::new(vec![], Holder(Item { id: "minecraft:gold".into() }));
        let zero = f.run(ItemStack::new(Item { id: "a".into() }, 0), &ctx());
        assert_eq!(zero.count, 0);
        let neg = f.run(ItemStack::new(Item { id: "a".into() }, -1), &ctx());
        assert_eq!(neg.count, -1);
        assert_eq!(neg.item.id, "minecraft:gold");
    }

    #[test]
    fn conditions_are_stored() {
        let f = SetItemFunction::new(
            vec![LootItemCondition, LootItemCondition],
            Holder(Item { id: "x".into() }),
        );
        assert_eq!(f.conditions.len(), 2);
    }
}
