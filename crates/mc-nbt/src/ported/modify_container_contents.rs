// Self-contained faithful port of
// net.minecraft.world.level.storage.loot.functions.ModifyContainerContents

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
    pub container_items: Option<Vec<ItemStack>>,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#isEmpty
    pub fn is_empty(&self) -> bool {
        self.id.is_empty() || self.count <= 0
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    ModifyContents,
}

pub trait LootItemFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction#apply
    fn apply_fn(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack;
    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction#validate
    fn validate_fn(&self, ctx: &mut ValidationContext);
}

#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    pub path: Vec<String>,
    pub problems: Vec<String>,
}

impl ValidationContext {
    // JAVA: net.minecraft.world.level.storage.loot.ValidationContext#forChild (FieldPathElement simplified)
    pub fn for_child(&mut self, field: &str) -> &mut ValidationContext {
        self.path.push(field.to_string());
        self
    }
}

pub trait ContainerComponentManipulator {
    // JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulator#modifyItems
    fn modify_items<F>(&self, stack: &mut ItemStack, f: F)
    where
        F: FnMut(ItemStack) -> ItemStack;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerComponentManipulators {
    Container,
}

impl ContainerComponentManipulator for ContainerComponentManipulators {
    // JAVA: net.minecraft.world.level.storage.loot.ContainerComponentManipulator#modifyItems (stubbed for container contents)
    fn modify_items<F>(&self, stack: &mut ItemStack, mut f: F)
    where
        F: FnMut(ItemStack) -> ItemStack,
    {
        match self {
            ContainerComponentManipulators::Container => {
                if let Some(items) = stack.container_items.as_mut() {
                    for item in items.iter_mut() {
                        let current = std::mem::take(item);
                        *item = f(current);
                    }
                }
            }
        }
    }
}

pub struct ModifyContainerContents<M: LootItemFunction> {
    pub conditions: Vec<LootItemCondition>,
    pub component: ContainerComponentManipulators,
    pub modifier: M,
}

impl<M: LootItemFunction> ModifyContainerContents<M> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ModifyContainerContents#CODEC
    // Codec placeholder: RecordCodecBuilder.mapCodec(commonFields(...).and(group(component, modifier)...))
    pub const CODEC_NAME: &'static str = "minecraft:modify_contents";

    // JAVA: net.minecraft.world.level.storage.loot.functions.ModifyContainerContents#<init>
    pub fn new(
        conditions: Vec<LootItemCondition>,
        component: ContainerComponentManipulators,
        modifier: M,
    ) -> Self {
        Self {
            conditions,
            component,
            modifier,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ModifyContainerContents#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::ModifyContents
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ModifyContainerContents#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if stack.is_empty() {
            return stack;
        }
        let modifier = &self.modifier;
        self.component.modify_items(&mut stack, |item| modifier.apply_fn(item, ctx));
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ModifyContainerContents#validate
    pub fn validate(&self, ctx: &mut ValidationContext) {
        // super.validate($$0);
        self.modifier.validate_fn(ctx.for_child("modifier"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct AddCountModifier(i32);
    impl LootItemFunction for AddCountModifier {
        fn apply_fn(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
            stack.count = stack.count.wrapping_add(self.0);
            stack
        }
        fn validate_fn(&self, ctx: &mut ValidationContext) {
            ctx.problems.push(format!("validated@{:?}", ctx.path));
        }
    }

    #[test]
    fn empty_stack_is_returned_unchanged() {
        let f = ModifyContainerContents::new(vec![], ContainerComponentManipulators::Container, AddCountModifier(1));
        let stack = ItemStack { id: "".into(), count: 0, container_items: Some(vec![ItemStack { id: "a".into(), count: 1, container_items: None }]) };
        let out = f.run(stack.clone(), &LootContext::default());
        assert_eq!(out, stack);
        assert_eq!(f.get_type(), LootItemFunctionType::ModifyContents);
    }

    #[test]
    fn modifies_each_container_item() {
        let f = ModifyContainerContents::new(vec![], ContainerComponentManipulators::Container, AddCountModifier(2));
        let stack = ItemStack {
            id: "chest".into(),
            count: 1,
            container_items: Some(vec![
                ItemStack { id: "a".into(), count: 1, container_items: None },
                ItemStack { id: "b".into(), count: i32::MAX, container_items: None },
            ]),
        };
        let out = f.run(stack, &LootContext::default());
        let items = out.container_items.unwrap();
        assert_eq!(items[0].count, 3);
        assert_eq!(items[1].count, i32::MAX.wrapping_add(2)); // overflow wrapping
    }

    #[test]
    fn no_container_items_is_noop() {
        let f = ModifyContainerContents::new(vec![], ContainerComponentManipulators::Container, AddCountModifier(5));
        let stack = ItemStack { id: "chest".into(), count: 1, container_items: None };
        let out = f.run(stack.clone(), &LootContext::default());
        assert_eq!(out, stack);
    }

    #[test]
    fn validate_delegates_to_modifier_with_child_path() {
        let f = ModifyContainerContents::new(vec![], ContainerComponentManipulators::Container, AddCountModifier(0));
        let mut ctx = ValidationContext::default();
        f.validate(&mut ctx);
        assert_eq!(ctx.problems.len(), 1);
        assert!(ctx.problems[0].contains("modifier"));
    }

    #[test]
    fn negative_count_item_is_empty() {
        let f = ModifyContainerContents::new(vec![], ContainerComponentManipulators::Container, AddCountModifier(1));
        let stack = ItemStack { id: "x".into(), count: -1, container_items: Some(vec![ItemStack { id: "a".into(), count: 1, container_items: None }]) };
        let out = f.run(stack.clone(), &LootContext::default());
        assert_eq!(out, stack);
    }
}
