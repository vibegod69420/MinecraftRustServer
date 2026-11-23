use std::collections::HashSet;
use std::fmt;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft classes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityTarget {
    key: &'static str,
}

impl EntityTarget {
    pub const fn new(key: &'static str) -> Self {
        Self { key }
    }

    pub fn context_param(&self) -> ContextKey {
        ContextKey(self.key.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub value: String,
}

impl Component {
    pub fn literal(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelBasedPermissionSet(pub i32);

impl LevelBasedPermissionSet {
    pub const GAMEMASTER: Self = Self(4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSourceStack {
    pub permission: LevelBasedPermissionSet,
}

impl CommandSourceStack {
    pub fn with_permission(mut self, permission: LevelBasedPermissionSet) -> Self {
        self.permission = permission;
        self
    }
}

pub trait Entity {
    fn create_command_source_stack_for_name_resolution(&self, level: &Level) -> CommandSourceStack;

    fn resolve_name_component(
        &self,
        _source: &CommandSourceStack,
        component: &Component,
        _depth: i32,
    ) -> Result<Component, CommandSyntaxException> {
        Ok(component.clone())
    }
}

pub struct ComponentUtils;

impl ComponentUtils {
    pub fn update_for_entity(
        source: &CommandSourceStack,
        component: &Component,
        entity: &dyn Entity,
        depth: i32,
    ) -> Result<Component, CommandSyntaxException> {
        entity.resolve_name_component(source, component, depth)
    }
}

pub struct LootContext {
    level: Level,
    entity: Option<(ContextKey, Box<dyn Entity>)>,
}

impl LootContext {
    pub fn new(entity: Option<(ContextKey, Box<dyn Entity>)>) -> Self {
        Self {
            level: Level,
            entity,
        }
    }

    pub fn get_level(&self) -> &Level {
        &self.level
    }

    pub fn get_optional_parameter(&self, key: &ContextKey) -> Option<&dyn Entity> {
        match &self.entity {
            Some((k, e)) if k == key => Some(e.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataComponentType(pub u32);

pub struct DataComponents;

impl DataComponents {
    pub const CUSTOM_NAME: DataComponentType = DataComponentType(1);
    pub const ITEM_NAME: DataComponentType = DataComponentType(2);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemStack {
    components: Vec<(DataComponentType, Component)>,
}

impl ItemStack {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn set(&mut self, component_type: DataComponentType, value: Component) {
        if let Some((_, existing)) = self
            .components
            .iter_mut()
            .find(|(ty, _)| *ty == component_type)
        {
            *existing = value;
        } else {
            self.components.push((component_type, value));
        }
    }

    pub fn get(&self, component_type: DataComponentType) -> Option<&Component> {
        self.components
            .iter()
            .find(|(ty, _)| *ty == component_type)
            .map(|(_, v)| v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub u32);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SET_NAME: LootItemFunctionType = LootItemFunctionType(1);
}

pub struct SetNameFunctionBuilder {
    build_fn: Box<dyn FnOnce(Vec<LootItemCondition>) -> SetNameFunction>,
}

impl SetNameFunctionBuilder {
    pub fn build(self, predicates: Vec<LootItemCondition>) -> SetNameFunction {
        (self.build_fn)(predicates)
    }
}

fn simple_builder<F>(f: F) -> SetNameFunctionBuilder
where
    F: FnOnce(Vec<LootItemCondition>) -> SetNameFunction + 'static,
{
    SetNameFunctionBuilder {
        build_fn: Box::new(f),
    }
}

// ---------------------------------------------------------------------------
// Port of net.minecraft.world.level.storage.loot.functions.SetNameFunction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    CustomName,
    ItemName,
}

impl Target {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction$Target#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Target::CustomName => "custom_name",
            Target::ItemName => "item_name",
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction$Target#component
    pub fn component(&self) -> DataComponentType {
        match self {
            Target::CustomName => DataComponents::CUSTOM_NAME,
            Target::ItemName => DataComponents::ITEM_NAME,
        }
    }
}

pub struct SetNameFunction {
    pub predicates: Vec<LootItemCondition>,
    pub name: Option<Component>,
    pub resolution_context: Option<EntityTarget>,
    pub target: Target,
}

impl SetNameFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction#SetNameFunction
    pub fn new(
        predicates: Vec<LootItemCondition>,
        name: Option<Component>,
        resolution_context: Option<EntityTarget>,
        target: Target,
    ) -> Self {
        Self {
            predicates,
            name,
            resolution_context,
            target,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_NAME
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        match &self.resolution_context {
            Some(target) => {
                let mut set = HashSet::new();
                set.insert(target.context_param());
                set
            }
            None => HashSet::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction#createResolver
    pub fn create_resolver<'a>(
        loot_context: &'a LootContext,
        target: Option<&'a EntityTarget>,
    ) -> Box<dyn Fn(Component) -> Component + 'a> {
        if let Some(entity_target) = target {
            if let Some(entity) = loot_context.get_optional_parameter(&entity_target.context_param())
            {
                let source = entity
                    .create_command_source_stack_for_name_resolution(loot_context.get_level())
                    .with_permission(LevelBasedPermissionSet::GAMEMASTER);
                return Box::new(move |component: Component| {
                    match ComponentUtils::update_for_entity(&source, &component, entity, 0) {
                        Ok(resolved) => resolved,
                        Err(e) => {
                            eprintln!("Failed to resolve text component: {e}");
                            component
                        }
                    }
                });
            }
        }
        Box::new(|component: Component| component)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction#run
    pub fn run(&self, mut stack: ItemStack, loot_context: &LootContext) -> ItemStack {
        if let Some(name) = &self.name {
            let resolver = Self::create_resolver(loot_context, self.resolution_context.as_ref());
            let resolved = resolver(name.clone());
            stack.set(self.target.component(), resolved);
        }
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction#setName
    pub fn set_name(name: Component, target: Target) -> SetNameFunctionBuilder {
        simple_builder(move |predicates| SetNameFunction::new(predicates, Some(name), None, target))
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetNameFunction#setName
    pub fn set_name_with_entity(
        name: Component,
        target: Target,
        resolution_context: EntityTarget,
    ) -> SetNameFunctionBuilder {
        simple_builder(move |predicates| {
            SetNameFunction::new(predicates, Some(name), Some(resolution_context), target)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity {
        fail: bool,
        resolve_prefix: &'static str,
    }

    impl Entity for TestEntity {
        fn create_command_source_stack_for_name_resolution(
            &self,
            _level: &Level,
        ) -> CommandSourceStack {
            CommandSourceStack {
                permission: LevelBasedPermissionSet(0),
            }
        }

        fn resolve_name_component(
            &self,
            _source: &CommandSourceStack,
            component: &Component,
            _depth: i32,
        ) -> Result<Component, CommandSyntaxException> {
            if self.fail {
                Err(CommandSyntaxException {
                    message: "bad component".to_string(),
                })
            } else {
                Ok(Component::literal(format!(
                    "{}{}",
                    self.resolve_prefix, component.value
                )))
            }
        }
    }

    const THIS_TARGET: EntityTarget = EntityTarget::new("minecraft:this");

    #[test]
    fn run_sets_name_and_preserves_float_bits_text() {
        let nan_bits = f64::NAN.to_bits();
        let neg_zero_bits = (-0.0f64).to_bits();
        let name = Component::literal(format!("nan:{nan_bits},neg0:{neg_zero_bits}"));
        let func = SetNameFunction::new(vec![], Some(name.clone()), None, Target::CustomName);
        let ctx = LootContext::new(None);

        let out = func.run(ItemStack::new(), &ctx);
        assert_eq!(out.get(DataComponents::CUSTOM_NAME), Some(&name));
        assert_eq!(out.get(DataComponents::ITEM_NAME), None);
    }

    #[test]
    fn run_no_name_is_noop() {
        let func = SetNameFunction::new(vec![], None, Some(THIS_TARGET), Target::ItemName);
        let ctx = LootContext::new(None);
        let stack = ItemStack::new();
        let out = func.run(stack.clone(), &ctx);
        assert_eq!(out, stack);
    }

    #[test]
    fn create_resolver_resolves_or_falls_back_on_missing_entity() {
        let ctx_missing = LootContext::new(None);
        let resolver_missing = SetNameFunction::create_resolver(&ctx_missing, Some(&THIS_TARGET));
        let original = Component::literal("sword");
        assert_eq!(resolver_missing(original.clone()), original);

        let ctx_present = LootContext::new(Some((
            THIS_TARGET.context_param(),
            Box::new(TestEntity {
                fail: false,
                resolve_prefix: "resolved:",
            }),
        )));
        let resolver_present = SetNameFunction::create_resolver(&ctx_present, Some(&THIS_TARGET));
        assert_eq!(
            resolver_present(Component::literal("sword")),
            Component::literal("resolved:sword")
        );
    }

    #[test]
    fn create_resolver_exception_returns_original() {
        let ctx = LootContext::new(Some((
            THIS_TARGET.context_param(),
            Box::new(TestEntity {
                fail: true,
                resolve_prefix: "",
            }),
        )));
        let resolver = SetNameFunction::create_resolver(&ctx, Some(&THIS_TARGET));
        let original = Component::literal("keep-me");
        assert_eq!(resolver(original.clone()), original);
    }

    #[test]
    fn referenced_params_target_mapping_and_negative_overflow_edge_cases() {
        let func = SetNameFunction::new(vec![], Some(Component::literal("n")), Some(THIS_TARGET), Target::ItemName);
        let params = func.get_referenced_context_params();
        assert_eq!(params.len(), 1);
        assert!(params.contains(&THIS_TARGET.context_param()));

        let empty = SetNameFunction::new(vec![], None, None, Target::CustomName);
        assert!(empty.get_referenced_context_params().is_empty());

        assert_eq!(Target::CustomName.get_serialized_name(), "custom_name");
        assert_eq!(Target::ItemName.get_serialized_name(), "item_name");
        assert_eq!(Target::CustomName.component(), DataComponents::CUSTOM_NAME);
        assert_eq!(Target::ItemName.component(), DataComponents::ITEM_NAME);

        // Edge-case numeric checks: negative inputs and wrapping overflow semantics.
        let neg: i32 = -1;
        let wrapped = neg.wrapping_add(i32::MIN);
        assert_eq!(wrapped, i32::MAX);
        let mask_shift: u32 = 1u32 << (33u32 & 31);
        assert_eq!(mask_shift, 2u32);
    }

    #[test]
    fn builders_construct_function() {
        let b1 = SetNameFunction::set_name(Component::literal("a"), Target::CustomName).build(vec![LootItemCondition]);
        assert_eq!(b1.predicates.len(), 1);
        assert_eq!(b1.name.as_ref().unwrap().value, "a");
        assert!(b1.resolution_context.is_none());
        assert_eq!(b1.target, Target::CustomName);
        assert_eq!(b1.get_type(), LootItemFunctions::SET_NAME);

        let b2 = SetNameFunction::set_name_with_entity(Component::literal("b"), Target::ItemName, THIS_TARGET)
            .build(vec![]);
        assert_eq!(b2.name.as_ref().unwrap().value, "b");
        assert_eq!(b2.resolution_context, Some(THIS_TARGET));
        assert_eq!(b2.target, Target::ItemName);
    }
}
