use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
}

impl ItemStack {
    pub fn is_empty(&self) -> bool {
        self.id.is_empty() || self.count <= 0
    }

    pub fn copy_with_count(&self, count: i32) -> Self {
        Self {
            id: self.id.clone(),
            count,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const FURNACE_SMELT: LootItemFunctionType =
        LootItemFunctionType("minecraft:furnace_smelt");
}

pub struct SingleRecipeInput {
    pub item: ItemStack,
}

impl SingleRecipeInput {
    pub fn new(item: ItemStack) -> Self {
        Self { item }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmeltingRecipe {
    pub result_id: String,
    pub result_count: i32,
}

impl SmeltingRecipe {
    pub fn assemble(&self, _input: &SingleRecipeInput, _registry: &RegistryAccess) -> ItemStack {
        ItemStack {
            id: self.result_id.clone(),
            count: self.result_count,
        }
    }
}

pub struct RecipeHolder<T> {
    pub value: T,
}

impl<T> RecipeHolder<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeType {
    Smelting,
}

#[derive(Debug, Clone, Default)]
pub struct RegistryAccess;

#[derive(Debug, Clone, Default)]
pub struct RecipeAccess {
    pub smelting_recipes: HashMap<String, SmeltingRecipe>,
}

impl RecipeAccess {
    pub fn new() -> Self {
        Self {
            smelting_recipes: HashMap::new(),
        }
    }

    pub fn add_smelting(&mut self, input_id: &str, result_id: &str, result_count: i32) {
        self.smelting_recipes.insert(
            input_id.to_string(),
            SmeltingRecipe {
                result_id: result_id.to_string(),
                result_count,
            },
        );
    }

    pub fn get_recipe_for(
        &self,
        _recipe_type: RecipeType,
        input: &SingleRecipeInput,
        _level: &Level,
    ) -> Option<RecipeHolder<SmeltingRecipe>> {
        self.smelting_recipes
            .get(&input.item.id)
            .map(|r| RecipeHolder { value: r.clone() })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Level {
    pub recipe_access: RecipeAccess,
    pub registry_access: RegistryAccess,
}

impl Level {
    pub fn recipe_access(&self) -> &RecipeAccess {
        &self.recipe_access
    }

    pub fn registry_access(&self) -> &RegistryAccess {
        &self.registry_access
    }
}

#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub level: Level,
}

impl LootContext {
    pub fn get_level(&self) -> &Level {
        &self.level
    }
}

pub struct LootItemConditionalFunctionBuilder<T> {
    build_fn: Box<dyn Fn(Vec<LootItemCondition>) -> T>,
    predicates: Vec<LootItemCondition>,
}

impl<T> LootItemConditionalFunctionBuilder<T> {
    pub fn build(self) -> T {
        (self.build_fn)(self.predicates)
    }
}

pub fn simple_builder<T, F>(f: F) -> LootItemConditionalFunctionBuilder<T>
where
    F: Fn(Vec<LootItemCondition>) -> T + 'static,
{
    LootItemConditionalFunctionBuilder {
        build_fn: Box::new(f),
        predicates: Vec::new(),
    }
}

pub struct SmeltItemFunction {
    pub predicates: Vec<LootItemCondition>,
}

impl SmeltItemFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#<init>
    fn new(predicates: Vec<LootItemCondition>) -> Self {
        Self { predicates }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::FURNACE_SMELT
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#run
    pub fn run(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if stack.is_empty() {
            return stack;
        }
        let input = SingleRecipeInput::new(stack.clone());
        let recipe = ctx.get_level().recipe_access().get_recipe_for(
            RecipeType::Smelting,
            &input,
            ctx.get_level(),
        );
        if let Some(holder) = recipe {
            let result = holder
                .value()
                .assemble(&input, ctx.get_level().registry_access());
            if !result.is_empty() {
                return result.copy_with_count(stack.count);
            }
        }
        eprintln!(
            "Couldn't smelt {} because there is no smelting recipe",
            stack.id
        );
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#smelted
    pub fn smelted() -> LootItemConditionalFunctionBuilder<SmeltItemFunction> {
        simple_builder(SmeltItemFunction::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx() -> LootContext {
        LootContext {
            level: Level {
                recipe_access: RecipeAccess::new(),
                registry_access: RegistryAccess,
            },
        }
    }

    #[test]
    fn smelt_with_recipe_preserves_count() {
        let f = SmeltItemFunction::new(vec![]);
        let mut ctx = empty_ctx();
        ctx.level
            .recipe_access
            .add_smelting("minecraft:iron_ore", "minecraft:iron_ingot", 1);
        let out = f.run(
            ItemStack {
                id: "minecraft:iron_ore".into(),
                count: 3,
            },
            &ctx,
        );
        assert_eq!(out.id, "minecraft:iron_ingot");
        assert_eq!(out.count, 3);
    }

    #[test]
    fn smelt_empty_stack_returns_original() {
        let f = SmeltItemFunction::new(vec![]);
        let stack = ItemStack {
            id: "".into(),
            count: 0,
        };
        let out = f.run(stack.clone(), &empty_ctx());
        assert_eq!(out, stack);
    }

    #[test]
    fn smelt_no_recipe_returns_original() {
        let f = SmeltItemFunction::new(vec![]);
        let stack = ItemStack {
            id: "minecraft:stone".into(),
            count: 5,
        };
        let out = f.run(stack.clone(), &empty_ctx());
        assert_eq!(out, stack);
    }

    #[test]
    fn smelt_empty_result_returns_original() {
        let f = SmeltItemFunction::new(vec![]);
        let mut ctx = empty_ctx();
        ctx.level.recipe_access.smelting_recipes.insert(
            "minecraft:weird".into(),
            SmeltingRecipe {
                result_id: "".into(),
                result_count: 0,
            },
        );
        let stack = ItemStack {
            id: "minecraft:weird".into(),
            count: 2,
        };
        let out = f.run(stack.clone(), &ctx);
        assert_eq!(out, stack);
    }

    #[test]
    fn smelt_negative_count_treated_as_empty() {
        let f = SmeltItemFunction::new(vec![]);
        let mut ctx = empty_ctx();
        ctx.level
            .recipe_access
            .add_smelting("minecraft:iron_ore", "minecraft:iron_ingot", 1);
        let stack = ItemStack {
            id: "minecraft:iron_ore".into(),
            count: -5,
        };
        let out = f.run(stack.clone(), &ctx);
        assert_eq!(out, stack);
    }

    #[test]
    fn smelted_builder_builds_function() {
        let f = SmeltItemFunction::smelted().build();
        assert_eq!(f.get_type(), LootItemFunctions::FURNACE_SMELT);
    }
}
