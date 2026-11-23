//! Port of `net.minecraft.world.level.storage.loot.functions.CopyBlockState` (1.21.11).
//!
//! This module is a self-contained, faithful-enough runtime port of the loot function logic.
//! It focuses on the data model and `run`/builder semantics described by the decompiled source.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Property {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct StateDefinition {
    pub properties: Vec<Property>,
}

impl StateDefinition {
    pub fn get_property(&self, name: &str) -> Option<Property> {
        self.properties.iter().find(|p| p.name == name).cloned()
    }

    pub fn get_properties(&self) -> &Vec<Property> {
        &self.properties
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub state_definition: StateDefinition,
}

impl Block {
    pub fn built_in_registry_holder(self) -> Holder<Block> {
        Holder(self)
    }
}

#[derive(Clone, Debug)]
pub struct Holder<T>(pub T);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockState {
    pub map: BTreeMap<String, String>,
}

impl BlockState {
    pub fn has_property(&self, prop: &Property) -> bool {
        self.map.contains_key(&prop.name)
    }

    pub fn get_string(&self, prop: &Property) -> Option<&String> {
        self.map.get(&prop.name)
    }
}

pub type BlockItemStateProperties = BlockState;

impl BlockItemStateProperties {
    pub fn empty() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn with_from(&self, prop: &Property, src: &BlockState) -> Self {
        let mut m = self.map.clone();
        if let Some(v) = src.get_string(prop) {
            m.insert(prop.name.clone(), v.clone());
        }
        Self { map: m }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ItemStack {
    pub block_state: Option<BlockItemStateProperties>,
}

impl ItemStack {
    pub fn update_block_state<F>(&mut self, default: BlockItemStateProperties, f: F)
    where
        F: FnOnce(BlockItemStateProperties) -> BlockItemStateProperties,
    {
        let cur = self.block_state.clone().unwrap_or(default);
        let next = f(cur);
        self.block_state = Some(next);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LootContextParamKey(pub &'static str);

pub struct LootContextParams;

impl LootContextParams {
    pub const BLOCK_STATE: LootContextParamKey = LootContextParamKey("block_state");
}

#[derive(Default, Clone, Debug)]
pub struct LootContext {
    pub block_state: Option<BlockState>,
}

impl LootContext {
    pub fn get_optional_parameter(&self, key: &LootContextParamKey) -> Option<&BlockState> {
        if key.0 == "block_state" {
            self.block_state.as_ref()
        } else {
            None
        }
    }
}

pub trait LootItemCondition {
    fn test(&self, ctx: &LootContext) -> bool;
}

pub struct TrueCondition;

impl LootItemCondition for TrueCondition {
    fn test(&self, _ctx: &LootContext) -> bool {
        true
    }
}

pub struct FalseCondition;

impl LootItemCondition for FalseCondition {
    fn test(&self, _ctx: &LootContext) -> bool {
        false
    }
}

pub struct LootItemConditionalFunction {
    pub conditions: Vec<Box<dyn LootItemCondition>>,
}

impl LootItemConditionalFunction {
    pub fn new(conditions: Vec<Box<dyn LootItemCondition>>) -> Self {
        Self { conditions }
    }

    pub fn should_run(&self, ctx: &LootContext) -> bool {
        for c in &self.conditions {
            if !c.test(ctx) {
                return false;
            }
        }
        true
    }
}

pub struct CopyBlockState {
    pub base: LootItemConditionalFunction,
    pub block: Holder<Block>,
    pub properties: Vec<Property>,
}

impl CopyBlockState {
    /// JAVA: net.minecraft.world.level.storage.loot.functions.CopyBlockState#apply
    pub fn apply(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if self.base.should_run(ctx) {
            self.run(stack, ctx)
        } else {
            stack
        }
    }

    /// JAVA: net.minecraft.world.level.storage.loot.functions.CopyBlockState#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if let Some(block_state) = ctx.get_optional_parameter(&LootContextParams::BLOCK_STATE) {
            let props_to_copy = self.properties.clone();
            stack.update_block_state(BlockItemStateProperties::empty(), |mut props| {
                for prop in props_to_copy.iter() {
                    if block_state.has_property(prop) {
                        props = props.with_from(prop, block_state);
                    }
                }
                props
            });
        }
        stack
    }
}

/// JAVA: net.minecraft.world.level.storage.loot.functions.CopyBlockState#copyState
pub fn copy_state(block: Block) -> CopyBlockStateBuilder {
    CopyBlockStateBuilder::new(block)
}

pub struct CopyBlockStateBuilder {
    block: Holder<Block>,
    properties: Vec<Property>,
}

impl CopyBlockStateBuilder {
    pub fn new(block: Block) -> Self {
        Self {
            block: block.built_in_registry_holder(),
            properties: Vec::new(),
        }
    }

    /// JAVA: net.minecraft.world.level.storage.loot.functions.CopyBlockState.Builder#copy
    pub fn copy(&mut self, prop: Property) -> &mut Self {
        let def_props = &self.block.0.state_definition.properties;
        if !def_props.iter().any(|p| p == &prop) {
            panic!("Property {} is not present on block", prop.name);
        }
        if !self.properties.iter().any(|p| p == &prop) {
            self.properties.push(prop);
        }
        self
    }

    /// JAVA: net.minecraft.world.level.storage.loot.functions.CopyBlockState.Builder#getThis
    pub fn get_this(&mut self) -> &mut Self {
        self
    }

    /// JAVA: net.minecraft.world.level.storage.loot.functions.CopyBlockState.Builder#build
    pub fn build(self, conditions: Vec<Box<dyn LootItemCondition>>) -> CopyBlockState {
        CopyBlockState {
            base: LootItemConditionalFunction::new(conditions),
            block: self.block,
            properties: self.properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block_with_props(names: &[&str]) -> Block {
        let props = names
            .iter()
            .map(|n| Property { name: (*n).to_string() })
            .collect::<Vec<_>>();
        Block {
            state_definition: StateDefinition { properties: props },
        }
    }

    #[test]
    fn test_copy_single_property_when_present() {
        let block = make_block_with_props(&["facing", "waterlogged"]);
        let facing = Property {
            name: "facing".to_string(),
        };
        let mut builder = copy_state(block.clone());
        builder.copy(facing.clone());
        let fn_copy = builder.build(vec![Box::new(TrueCondition)]);

        let mut bs_map = BTreeMap::new();
        bs_map.insert("facing".to_string(), "north".to_string());
        bs_map.insert("waterlogged".to_string(), "true".to_string());
        let ctx = LootContext {
            block_state: Some(BlockState { map: bs_map }),
        };

        let stack_in = ItemStack { block_state: None };
        let stack_out = fn_copy.apply(stack_in, &ctx);
        let out_props = stack_out.block_state.expect("block_state should be set");
        assert_eq!(out_props.map.get("facing").map(String::as_str), Some("north"));
        assert_eq!(out_props.map.get("waterlogged"), None);
    }

    #[test]
    fn test_no_copy_when_block_state_missing() {
        let block = make_block_with_props(&["facing"]);
        let facing = Property {
            name: "facing".to_string(),
        };
        let mut builder = copy_state(block);
        builder.copy(facing);
        let fn_copy = builder.build(vec![Box::new(TrueCondition)]);

        let ctx = LootContext { block_state: None };
        let stack_in = ItemStack { block_state: None };
        let stack_out = fn_copy.apply(stack_in, &ctx);
        assert!(stack_out.block_state.is_none());
    }

    #[test]
    fn test_copy_ignores_property_not_in_source_block_state() {
        let block = make_block_with_props(&["facing", "power"]);
        let facing = Property {
            name: "facing".to_string(),
        };
        let power = Property {
            name: "power".to_string(),
        };
        let mut builder = copy_state(block);
        builder.copy(facing.clone());
        builder.copy(power.clone());
        let fn_copy = builder.build(vec![Box::new(TrueCondition)]);

        let mut bs_map = BTreeMap::new();
        bs_map.insert("facing".to_string(), "south".to_string());
        // power not present
        let ctx = LootContext {
            block_state: Some(BlockState { map: bs_map }),
        };

        let stack_in = ItemStack { block_state: None };
        let stack_out = fn_copy.apply(stack_in, &ctx);
        let out_props = stack_out.block_state.expect("block_state should be set");
        assert_eq!(out_props.map.get("facing").map(String::as_str), Some("south"));
        assert_eq!(out_props.map.get("power"), None);
    }

    #[test]
    fn test_conditions_prevent_run() {
        let block = make_block_with_props(&["facing"]);
        let facing = Property {
            name: "facing".to_string(),
        };
        let mut builder = copy_state(block);
        builder.copy(facing);
        let fn_copy = builder.build(vec![Box::new(FalseCondition)]);

        let mut bs_map = BTreeMap::new();
        bs_map.insert("facing".to_string(), "east".to_string());
        let ctx = LootContext {
            block_state: Some(BlockState { map: bs_map }),
        };

        let stack_in = ItemStack { block_state: None };
        let stack_out = fn_copy.apply(stack_in, &ctx);
        assert!(stack_out.block_state.is_none());
    }

    #[test]
    fn test_builder_rejects_property_not_on_block() {
        let block = make_block_with_props(&["facing"]);
        let not_present = Property {
            name: "power".to_string(),
        };
        let mut builder = copy_state(block);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            builder.copy(not_present);
        }));
        assert!(result.is_err());
    }
}
