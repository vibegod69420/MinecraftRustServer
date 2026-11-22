use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataComponentType(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentValue {
    Int(i32),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataComponentPatch {
    pub entries: Vec<(DataComponentType, Option<ComponentValue>)>,
}

impl DataComponentPatch {
    // JAVA: net.minecraft.core.component.DataComponentPatch#builder
    pub fn builder() -> DataComponentPatchBuilder {
        DataComponentPatchBuilder { entries: Vec::new() }
    }
}

pub struct DataComponentPatchBuilder {
    entries: Vec<(DataComponentType, Option<ComponentValue>)>,
}

impl DataComponentPatchBuilder {
    // JAVA: net.minecraft.core.component.DataComponentPatch$Builder#set
    pub fn set(mut self, component: DataComponentType, value: ComponentValue) -> Self {
        self.entries.push((component, Some(value)));
        self
    }

    // JAVA: net.minecraft.core.component.DataComponentPatch$Builder#build
    pub fn build(self) -> DataComponentPatch {
        DataComponentPatch { entries: self.entries }
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
    pub const SET_COMPONENTS: LootItemFunctionType = LootItemFunctionType("set_components");
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub components: BTreeMap<String, Option<ComponentValue>>,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#applyComponentsAndValidate
    pub fn apply_components_and_validate(&mut self, patch: &DataComponentPatch) {
        for (ty, val) in &patch.entries {
            self.components.insert(ty.0.clone(), val.clone());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetComponentsFunction {
    pub conditions: Vec<LootItemCondition>,
    pub components: DataComponentPatch,
}

impl SetComponentsFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetComponentsFunction#<init>
    fn new(conditions: Vec<LootItemCondition>, components: DataComponentPatch) -> Self {
        Self { conditions, components }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetComponentsFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_COMPONENTS
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetComponentsFunction#run
    pub fn run(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
        stack.apply_components_and_validate(&self.components);
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetComponentsFunction#setComponent
    pub fn set_component(
        component: DataComponentType,
        value: ComponentValue,
    ) -> impl FnOnce(Vec<LootItemCondition>) -> Self {
        move |conditions| {
            Self::new(
                conditions,
                DataComponentPatch::builder().set(component, value).build(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> LootContext {
        LootContext
    }

    #[test]
    fn run_applies_components() {
        let patch = DataComponentPatch::builder()
            .set(DataComponentType("damage".into()), ComponentValue::Int(5))
            .build();
        let f = SetComponentsFunction::new(vec![], patch);
        let out = f.run(ItemStack::default(), &ctx());
        assert_eq!(
            out.components.get("damage"),
            Some(&Some(ComponentValue::Int(5)))
        );
    }

    #[test]
    fn get_type_is_set_components() {
        let f = SetComponentsFunction::new(vec![], DataComponentPatch::default());
        assert_eq!(f.get_type(), LootItemFunctions::SET_COMPONENTS);
    }

    #[test]
    fn set_component_builder_creates_function() {
        let builder = SetComponentsFunction::set_component(
            DataComponentType("name".into()),
            ComponentValue::Str("x".into()),
        );
        let f = builder(vec![]);
        let out = f.run(ItemStack::default(), &ctx());
        assert_eq!(
            out.components.get("name"),
            Some(&Some(ComponentValue::Str("x".into())))
        );
    }

    #[test]
    fn run_overwrites_existing_component_and_handles_edge_int() {
        let mut stack = ItemStack::default();
        stack
            .components
            .insert("damage".into(), Some(ComponentValue::Int(1)));
        let patch = DataComponentPatch::builder()
            .set(DataComponentType("damage".into()), ComponentValue::Int(i32::MIN))
            .build();
        let f = SetComponentsFunction::new(vec![], patch);
        let out = f.run(stack, &ctx());
        assert_eq!(
            out.components.get("damage"),
            Some(&Some(ComponentValue::Int(i32::MIN)))
        );
    }
}
