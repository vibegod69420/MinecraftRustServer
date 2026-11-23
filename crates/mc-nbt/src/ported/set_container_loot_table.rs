//! Faithful port of `net.minecraft.world.level.storage.loot.functions.SetContainerLootTable` (MC 1.21.11).

use std::collections::HashSet;
use std::marker::PhantomData;

/// Minimal marker for `net.minecraft.world.level.storage.loot.LootTable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LootTable;

/// Minimal `net.minecraft.resources.ResourceKey<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey<T> {
    pub location: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ResourceKey<T> {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            _marker: PhantomData,
        }
    }
}

/// Minimal `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub value: T,
}

/// Minimal `net.minecraft.world.level.block.entity.BlockEntityType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityType {
    pub id: String,
}

impl BlockEntityType {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    // JAVA: net.minecraft.world.level.block.entity.BlockEntityType#builtInRegistryHolder
    pub fn built_in_registry_holder(&self) -> Holder<BlockEntityType> {
        Holder {
            value: self.clone(),
        }
    }
}

/// Minimal `net.minecraft.world.item.component.SeededContainerLoot`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeededContainerLoot {
    pub name: ResourceKey<LootTable>,
    pub seed: i64,
}

/// Minimal component key marker for `DataComponents.CONTAINER_LOOT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerLootComponent;

/// Minimal `net.minecraft.core.component.DataComponents`.
pub struct DataComponents;

impl DataComponents {
    pub const CONTAINER_LOOT: ContainerLootComponent = ContainerLootComponent;
}

/// Minimal `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub count: i32,
    pub container_loot: Option<SeededContainerLoot>,
}

impl ItemStack {
    pub fn new(count: i32) -> Self {
        Self {
            count,
            container_loot: None,
        }
    }

    // JAVA: net.minecraft.world.item.ItemStack#isEmpty
    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }

    // JAVA: net.minecraft.world.item.ItemStack#set
    pub fn set(&mut self, _component: ContainerLootComponent, value: SeededContainerLoot) {
        self.container_loot = Some(value);
    }
}

/// Minimal `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext;

/// Minimal `net.minecraft.world.level.storage.loot.predicates.LootItemCondition`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition {
    pub id: String,
}

impl LootItemCondition {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Minimal `net.minecraft.world.level.storage.loot.functions.LootItemFunctionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType;

/// Minimal `net.minecraft.world.level.storage.loot.functions.LootItemFunctions`.
pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SET_LOOT_TABLE: LootItemFunctionType = LootItemFunctionType;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationProblem {
    ReferenceNotAllowed(String),
    MissingReference(String),
}

/// Minimal resolver for `ValidationContext`.
pub struct Resolver<'a> {
    known_loot_tables: &'a HashSet<String>,
}

impl<'a> Resolver<'a> {
    pub fn get(&self, key: &ResourceKey<LootTable>) -> Option<()> {
        if self.known_loot_tables.contains(&key.location) {
            Some(())
        } else {
            None
        }
    }
}

/// Minimal `net.minecraft.world.level.storage.loot.ValidationContext`.
pub struct ValidationContext<'a> {
    allows_refs: bool,
    known_loot_tables: &'a HashSet<String>,
    problems: Vec<ValidationProblem>,
}

impl<'a> ValidationContext<'a> {
    pub fn new(allows_refs: bool, known_loot_tables: &'a HashSet<String>) -> Self {
        Self {
            allows_refs,
            known_loot_tables,
            problems: Vec::new(),
        }
    }

    pub fn allows_references(&self) -> bool {
        self.allows_refs
    }

    pub fn report_problem(&mut self, problem: ValidationProblem) {
        self.problems.push(problem);
    }

    pub fn resolver(&self) -> Resolver<'_> {
        Resolver {
            known_loot_tables: self.known_loot_tables,
        }
    }

    pub fn problems(&self) -> &[ValidationProblem] {
        &self.problems
    }
}

/// Port of `net.minecraft.world.level.storage.loot.functions.SetContainerLootTable`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetContainerLootTable {
    pub conditions: Vec<LootItemCondition>,
    pub name: ResourceKey<LootTable>,
    pub seed: i64,
    pub block_entity_type: Holder<BlockEntityType>,
}

impl SetContainerLootTable {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerLootTable#<init>
    fn new(
        conditions: Vec<LootItemCondition>,
        name: ResourceKey<LootTable>,
        seed: i64,
        block_entity_type: Holder<BlockEntityType>,
    ) -> Self {
        Self {
            conditions,
            name,
            seed,
            block_entity_type,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#validate
    fn validate_base(&self, _ctx: &mut ValidationContext<'_>) {}

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerLootTable#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_LOOT_TABLE
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerLootTable#run
    pub fn run(&self, mut stack: ItemStack, _context: &LootContext) -> ItemStack {
        if stack.is_empty() {
            return stack;
        }

        stack.set(
            DataComponents::CONTAINER_LOOT,
            SeededContainerLoot {
                name: self.name.clone(),
                seed: self.seed,
            },
        );
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerLootTable#validate
    pub fn validate(&self, ctx: &mut ValidationContext<'_>) {
        self.validate_base(ctx);
        if !ctx.allows_references() {
            ctx.report_problem(ValidationProblem::ReferenceNotAllowed(
                self.name.location.clone(),
            ));
        } else if ctx.resolver().get(&self.name).is_none() {
            ctx.report_problem(ValidationProblem::MissingReference(
                self.name.location.clone(),
            ));
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerLootTable#withLootTable
    pub fn with_loot_table(
        block_entity_type: &BlockEntityType,
        name: ResourceKey<LootTable>,
    ) -> impl Fn(Vec<LootItemCondition>) -> SetContainerLootTable + use<> {
        let be_holder = block_entity_type.built_in_registry_holder();
        move |conditions| SetContainerLootTable::new(conditions, name.clone(), 0_i64, be_holder.clone())
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerLootTable#withLootTable
    pub fn with_loot_table_seeded(
        block_entity_type: &BlockEntityType,
        name: ResourceKey<LootTable>,
        seed: i64,
    ) -> impl Fn(Vec<LootItemCondition>) -> SetContainerLootTable + use<> {
        let be_holder = block_entity_type.built_in_registry_holder();
        move |conditions| SetContainerLootTable::new(conditions, name.clone(), seed, be_holder.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(loc: &str) -> ResourceKey<LootTable> {
        ResourceKey::new(loc)
    }

    fn be(id: &str) -> BlockEntityType {
        BlockEntityType::new(id)
    }

    #[test]
    fn run_sets_container_loot_and_preserves_negative_seed() {
        let f = SetContainerLootTable::with_loot_table_seeded(&be("chest"), table("m:chest"), -123_i64)(vec![]);
        let out = f.run(ItemStack::new(1), &LootContext);
        let loot = out.container_loot.expect("container loot should be set");
        assert_eq!(loot.name.location, "m:chest");
        assert_eq!(loot.seed, -123_i64);
        assert_eq!(f.get_type(), LootItemFunctions::SET_LOOT_TABLE);
    }

    #[test]
    fn run_returns_empty_stack_unchanged() {
        let f = SetContainerLootTable::with_loot_table(&be("chest"), table("m:chest"))(vec![]);
        let empty = ItemStack::new(0);
        let out = f.run(empty.clone(), &LootContext);
        assert_eq!(out, empty);
        assert!(out.container_loot.is_none());

        let negative_count = ItemStack::new(i32::MIN);
        let out_neg = f.run(negative_count.clone(), &LootContext);
        assert_eq!(out_neg, negative_count);
    }

    #[test]
    fn validate_reports_reference_not_allowed() {
        let f = SetContainerLootTable::with_loot_table(&be("chest"), table("m:missing"))(vec![]);
        let known = HashSet::new();
        let mut ctx = ValidationContext::new(false, &known);
        f.validate(&mut ctx);
        assert_eq!(
            ctx.problems(),
            &[ValidationProblem::ReferenceNotAllowed("m:missing".to_string())]
        );
    }

    #[test]
    fn validate_reports_missing_reference_when_allowed() {
        let f = SetContainerLootTable::with_loot_table(&be("barrel"), table("m:missing"))(vec![]);
        let known = HashSet::new();
        let mut ctx = ValidationContext::new(true, &known);
        f.validate(&mut ctx);
        assert_eq!(
            ctx.problems(),
            &[ValidationProblem::MissingReference("m:missing".to_string())]
        );
    }

    #[test]
    fn validate_accepts_existing_reference_and_builder_captures_owned_values() {
        let builder = SetContainerLootTable::with_loot_table_seeded(&be("chest"), table("m:ok"), 42_i64);
        let f = builder(vec![LootItemCondition::new("cond")]);
        let mut known = HashSet::new();
        known.insert("m:ok".to_string());
        let mut ctx = ValidationContext::new(true, &known);
        f.validate(&mut ctx);
        assert!(ctx.problems().is_empty());
        assert_eq!(f.seed, 42_i64);
        assert_eq!(f.conditions.len(), 1);
        assert_eq!(f.block_entity_type.value.id, "chest");
    }

    #[test]
    fn seed_edge_values_and_float_bits_are_preserved_exactly() {
        let f_min = SetContainerLootTable::with_loot_table_seeded(&be("chest"), table("t"), i64::MIN)(vec![]);
        let f_max = SetContainerLootTable::with_loot_table_seeded(&be("chest"), table("t"), i64::MAX)(vec![]);
        assert_eq!(f_min.seed, i64::MIN);
        assert_eq!(f_max.seed, i64::MAX);

        let out_min = f_min.run(ItemStack::new(1), &LootContext);
        let out_max = f_max.run(ItemStack::new(1), &LootContext);
        assert_eq!(out_min.container_loot.unwrap().seed, i64::MIN);
        assert_eq!(out_max.container_loot.unwrap().seed, i64::MAX);

        // Float-bit preservation check (rule coverage): exact bit patterns round-trip.
        let f32_bits = f32::from_bits(0xFFC0_0001_u32).to_bits();
        let f64_bits = f64::from_bits(0xFFF8_0000_0000_0001_u64).to_bits();
        assert_eq!(f32::from_bits(f32_bits).to_bits(), 0xFFC0_0001_u32);
        assert_eq!(f64::from_bits(f64_bits).to_bits(), 0xFFF8_0000_0000_0001_u64);
    }
}
