use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item: String,
    pub count: i32,
}

impl ItemStack {
    pub fn new(item: impl Into<String>, count: i32) -> Self {
        Self {
            item: item.into(),
            count,
        }
    }

    // JAVA: net.minecraft.world.item.ItemStack#isEmpty
    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item.is_empty() || self.item == "minecraft:air"
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
    pub fn slots() -> LootPoolEntryType {
        LootPoolEntryType {
            id: "slots".to_string(),
        }
    }
}

#[derive(Default)]
pub struct LootContext;

pub mod problem_reporter {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FieldPathElement {
        pub name: String,
    }

    impl FieldPathElement {
        // JAVA: net.minecraft.util.ProblemReporter$FieldPathElement#<init>
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    pub path: Vec<problem_reporter::FieldPathElement>,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.world.level.storage.loot.ValidationContext#forChild
    pub fn for_child(&self, element: problem_reporter::FieldPathElement) -> Self {
        let mut path = self.path.clone();
        path.push(element);
        Self { path }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotCollection {
    pub items: Vec<ItemStack>,
}

impl SlotCollection {
    // JAVA: net.minecraft.world.item.slot.SlotCollection#itemCopies
    pub fn item_copies(&self) -> Vec<ItemStack> {
        self.items.clone()
    }
}

#[derive(Clone)]
pub struct SlotSource {
    pub items: Vec<ItemStack>,
    pub validated_paths: Rc<RefCell<Vec<Vec<problem_reporter::FieldPathElement>>>>,
}

impl fmt::Debug for SlotSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SlotSource")
            .field("items", &self.items)
            .field("validated_paths", &self.validated_paths.borrow())
            .finish()
    }
}

impl PartialEq for SlotSource {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl SlotSource {
    pub fn new(items: Vec<ItemStack>) -> Self {
        Self {
            items,
            validated_paths: Rc::new(RefCell::new(Vec::new())),
        }
    }

    // JAVA: net.minecraft.world.item.slot.SlotSource#provide
    pub fn provide(&self, _context: &LootContext) -> SlotCollection {
        SlotCollection {
            items: self.items.clone(),
        }
    }

    // JAVA: net.minecraft.world.item.slot.SlotSource#validate
    pub fn validate(&self, context: &ValidationContext) {
        self.validated_paths.borrow_mut().push(context.path.clone());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotLoot {
    pub slot_source: SlotSource,
    pub weight: i32,
    pub quality: i32,
    pub conditions: Vec<LootItemCondition>,
    pub functions: Vec<LootItemFunction>,
}

impl SlotLoot {
    // JAVA: net.minecraft.world.level.storage.loot.entries.SlotLoot#<init>
    fn new(
        slot_source: SlotSource,
        weight: i32,
        quality: i32,
        conditions: Vec<LootItemCondition>,
        functions: Vec<LootItemFunction>,
    ) -> Self {
        Self {
            slot_source,
            weight,
            quality,
            conditions,
            functions,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolSingletonContainer#validate
    fn validate_base(&self, _context: &ValidationContext) {
        // Base singleton validation stub; conditions/functions validation is external.
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.SlotLoot#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntries::slots()
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.SlotLoot#createItemStack
    pub fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), context: &LootContext) {
        for stack in self.slot_source.provide(context).item_copies() {
            if !stack.is_empty() {
                output(stack);
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.SlotLoot#validate
    pub fn validate(&self, context: &ValidationContext) {
        self.validate_base(context);
        self.slot_source
            .validate(&context.for_child(problem_reporter::FieldPathElement::new("slot_source")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_type_returns_slots() {
        let loot = SlotLoot::new(SlotSource::new(vec![]), 1, 0, vec![], vec![]);
        assert_eq!(loot.get_type().id, "slots");
    }

    #[test]
    fn create_item_stack_filters_empty_and_yields_copies() {
        let source = SlotSource::new(vec![
            ItemStack::new("minecraft:stone", 1),
            ItemStack::new("minecraft:air", 1),
            ItemStack::new("minecraft:dirt", 0),
            ItemStack::new("", 5),
            ItemStack::new("minecraft:diamond", 64),
        ]);
        let loot = SlotLoot::new(source, 1, 0, vec![], vec![]);
        let ctx = LootContext;
        let mut out = Vec::new();
        loot.create_item_stack(&mut |s| out.push(s), &ctx);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].item, "minecraft:stone");
        assert_eq!(out[1].item, "minecraft:diamond");
        assert_eq!(out[1].count, 64);
    }

    #[test]
    fn create_item_stack_treats_negative_count_as_empty() {
        let source = SlotSource::new(vec![
            ItemStack::new("minecraft:stone", -1),
            ItemStack::new("minecraft:stick", i32::MIN),
            ItemStack::new("minecraft:apple", 1),
        ]);
        let loot = SlotLoot::new(source, 1, 0, vec![], vec![]);
        let ctx = LootContext;
        let mut out = Vec::new();
        loot.create_item_stack(&mut |s| out.push(s), &ctx);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].item, "minecraft:apple");
    }

    #[test]
    fn validate_calls_slot_source_with_child_path() {
        let source = SlotSource::new(vec![ItemStack::new("minecraft:stone", 1)]);
        let loot = SlotLoot::new(source.clone(), 1, 0, vec![], vec![]);
        let ctx = ValidationContext::new();
        loot.validate(&ctx);
        let paths = source.validated_paths.borrow();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 1);
        assert_eq!(paths[0][0].name, "slot_source");
    }

    #[test]
    fn constructor_preserves_negative_weight_and_quality() {
        let loot = SlotLoot::new(
            SlotSource::new(vec![]),
            -10,
            i32::MIN,
            vec![LootItemCondition],
            vec![LootItemFunction],
        );
        assert_eq!(loot.weight, -10);
        assert_eq!(loot.quality, i32::MIN);
        assert_eq!(loot.conditions.len(), 1);
        assert_eq!(loot.functions.len(), 1);
    }
}
