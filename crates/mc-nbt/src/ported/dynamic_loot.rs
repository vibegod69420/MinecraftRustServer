use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

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
    pub fn dynamic() -> LootPoolEntryType {
        LootPoolEntryType {
            id: "dynamic".to_string(),
        }
    }
}

#[derive(Default)]
pub struct LootContext {
    dynamic_providers: Vec<(Identifier, Vec<ItemStack>)>,
    pub dynamic_drop_requests: Vec<Identifier>,
}

impl LootContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_dynamic_drops(&mut self, name: Identifier, stacks: Vec<ItemStack>) {
        self.dynamic_providers.push((name, stacks));
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#addDynamicDrops
    pub fn add_dynamic_drops(&mut self, name: &Identifier, output: &mut dyn FnMut(ItemStack)) {
        self.dynamic_drop_requests.push(name.clone());
        for (provider_name, stacks) in &self.dynamic_providers {
            if provider_name == name {
                for stack in stacks {
                    output(stack.clone());
                }
            }
        }
    }
}

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
pub struct DynamicLoot {
    pub name: Identifier,
    pub weight: i32,
    pub quality: i32,
    pub conditions: Vec<LootItemCondition>,
    pub functions: Vec<LootItemFunction>,
}

impl DynamicLoot {
    // JAVA: net.minecraft.world.level.storage.loot.entries.DynamicLoot#<init>
    fn new(
        name: Identifier,
        weight: i32,
        quality: i32,
        conditions: Vec<LootItemCondition>,
        functions: Vec<LootItemFunction>,
    ) -> Self {
        Self {
            name,
            weight,
            quality,
            conditions,
            functions,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.DynamicLoot#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntries::dynamic()
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.DynamicLoot#createItemStack
    pub fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), context: &mut LootContext) {
        context.add_dynamic_drops(&self.name, output);
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.DynamicLoot#dynamicEntry
    pub fn dynamic_entry(name: Identifier) -> LootPoolSingletonContainerBuilder<DynamicLoot> {
        LootPoolSingletonContainerBuilder::simple_builder(
            move |weight, quality, conditions, functions| {
                DynamicLoot::new(name.clone(), weight, quality, conditions, functions)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_entry_builds_with_defaults() {
        let id = Identifier::with_default_namespace("chests/test");
        let loot = DynamicLoot::dynamic_entry(id.clone()).build();
        assert_eq!(loot.name, id);
        assert_eq!(loot.weight, 1);
        assert_eq!(loot.quality, 0);
        assert!(loot.conditions.is_empty());
        assert!(loot.functions.is_empty());
        assert_eq!(loot.get_type().id, "dynamic");
    }

    #[test]
    fn dynamic_entry_preserves_negative_weight_and_quality() {
        let id = Identifier::with_default_namespace("neg");
        let loot = DynamicLoot::dynamic_entry(id.clone())
            .set_weight(-5)
            .set_quality(i32::MIN)
            .build();
        assert_eq!(loot.weight, -5);
        assert_eq!(loot.quality, i32::MIN);
        assert_eq!(loot.name, id);
    }

    #[test]
    fn create_item_stack_delegates_to_context_dynamic_drops() {
        let name = Identifier::with_default_namespace("contents");
        let loot = DynamicLoot::dynamic_entry(name.clone()).build();
        let mut ctx = LootContext::new();
        ctx.register_dynamic_drops(
            name.clone(),
            vec![ItemStack::new("stone", 1), ItemStack::new("dirt", 2)],
        );
        let mut out = Vec::new();
        loot.create_item_stack(&mut |s| out.push(s), &mut ctx);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].item, "stone");
        assert_eq!(out[1].count, 2);
        assert_eq!(ctx.dynamic_drop_requests, vec![name]);
    }

    #[test]
    fn create_item_stack_no_match_produces_nothing_but_records_request() {
        let loot = DynamicLoot::dynamic_entry(Identifier::with_default_namespace("a")).build();
        let mut ctx = LootContext::new();
        ctx.register_dynamic_drops(
            Identifier::with_default_namespace("b"),
            vec![ItemStack::new("stone", 1)],
        );
        let mut out = Vec::new();
        loot.create_item_stack(&mut |s| out.push(s), &mut ctx);
        assert!(out.is_empty());
        assert_eq!(ctx.dynamic_drop_requests.len(), 1);
        assert_eq!(
            ctx.dynamic_drop_requests[0],
            Identifier::with_default_namespace("a")
        );
    }

    #[test]
    fn identifier_with_default_namespace_uses_minecraft() {
        let id = Identifier::with_default_namespace("path/to");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "path/to");
    }
}
