use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationProblem {
    ReferenceNotAllowed(ResourceKey),
    RecursiveReference(ResourceKey),
    MissingReference(ResourceKey),
    Custom(String),
}

#[derive(Debug, Clone, Copy)]
pub struct InlineLootTablePathElement;

impl InlineLootTablePathElement {
    // JAVA: net.minecraft.world.level.storage.loot.entries.NestedLootTable$1#get
    pub fn get(&self) -> &'static str {
        "->{inline}"
    }
}

pub const INLINE_LOOT_TABLE_PATH_ELEMENT: InlineLootTablePathElement = InlineLootTablePathElement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootPoolEntryType(pub &'static str);

pub struct LootPoolEntries;

impl LootPoolEntries {
    pub const LOOT_TABLE: LootPoolEntryType = LootPoolEntryType("minecraft:loot_table");
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
    pub scale: f32,
}

impl ItemStack {
    pub fn new(id: impl Into<String>, count: i32, scale: f32) -> Self {
        Self {
            id: id.into(),
            count,
            scale,
        }
    }
}

impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.count == other.count && self.scale.to_bits() == other.scale.to_bits()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LootItemCondition {
    pub id: String,
}

impl LootItemCondition {
    pub fn validate(&self, _ctx: &mut ValidationContext<'_>) {}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LootItemFunction {
    pub id: String,
}

impl LootItemFunction {
    pub fn validate(&self, _ctx: &mut ValidationContext<'_>) {}
}

pub trait LootTableResolver {
    fn get(&self, key: &ResourceKey) -> Option<LootTable>;
}

pub struct LootContext<'a> {
    resolver: &'a dyn LootTableResolver,
}

impl<'a> LootContext<'a> {
    pub fn new(resolver: &'a dyn LootTableResolver) -> Self {
        Self { resolver }
    }

    pub fn resolver(&self) -> &dyn LootTableResolver {
        self.resolver
    }
}

pub struct ValidationContext<'a> {
    allows_references: bool,
    visited: HashSet<ResourceKey>,
    problems: Vec<ValidationProblem>,
    resolver: &'a dyn LootTableResolver,
    path: String,
}

impl<'a> ValidationContext<'a> {
    pub fn new(allows_references: bool, resolver: &'a dyn LootTableResolver) -> Self {
        Self {
            allows_references,
            visited: HashSet::new(),
            problems: Vec::new(),
            resolver,
            path: String::new(),
        }
    }

    pub fn allows_references(&self) -> bool {
        self.allows_references
    }

    pub fn report_problem(&mut self, problem: ValidationProblem) {
        self.problems.push(problem);
    }

    pub fn has_visited_element(&self, key: &ResourceKey) -> bool {
        self.visited.contains(key)
    }

    pub fn resolver(&self) -> &dyn LootTableResolver {
        self.resolver
    }

    pub fn enter_element(&self, _path: &str, key: ResourceKey) -> ValidationContext<'a> {
        let mut visited = self.visited.clone();
        visited.insert(key);
        ValidationContext {
            allows_references: self.allows_references,
            visited,
            problems: Vec::new(),
            resolver: self.resolver,
            path: self.path.clone(),
        }
    }

    pub fn for_child(&self, path: &str) -> ValidationContext<'a> {
        ValidationContext {
            allows_references: self.allows_references,
            visited: self.visited.clone(),
            problems: Vec::new(),
            resolver: self.resolver,
            path: format!("{}{}", self.path, path),
        }
    }

    pub fn problems(&self) -> &[ValidationProblem] {
        &self.problems
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn merge_child(&mut self, child: ValidationContext<'_>) {
        self.problems.extend(child.problems);
    }

    pub fn mark_visited(&mut self, key: ResourceKey) {
        self.visited.insert(key);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootTable {
    pub items: Vec<ItemStack>,
    pub report_path_on_validate: bool,
}

impl Default for LootTable {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            report_path_on_validate: false,
        }
    }
}

impl LootTable {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get_random_items_raw(&self, _ctx: &LootContext<'_>, output: &mut dyn FnMut(ItemStack)) {
        for item in &self.items {
            output(item.clone());
        }
    }

    pub fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.report_path_on_validate {
            ctx.report_problem(ValidationProblem::Custom(ctx.path().to_string()));
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootTableContents {
    Reference(ResourceKey),
    Inline(LootTable),
}

#[derive(Debug, Clone)]
pub struct NestedLootTable {
    pub weight: i32,
    pub quality: i32,
    pub conditions: Vec<LootItemCondition>,
    pub functions: Vec<LootItemFunction>,
    pub contents: LootTableContents,
}

impl NestedLootTable {
    // JAVA: net.minecraft.world.level.storage.loot.entries.NestedLootTable#NestedLootTable
    fn new(
        contents: LootTableContents,
        weight: i32,
        quality: i32,
        conditions: Vec<LootItemCondition>,
        functions: Vec<LootItemFunction>,
    ) -> Self {
        Self {
            weight,
            quality,
            conditions,
            functions,
            contents,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.NestedLootTable#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntries::LOOT_TABLE
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.NestedLootTable#createItemStack
    pub fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), ctx: &LootContext<'_>) {
        let table = match &self.contents {
            LootTableContents::Reference(key) => {
                ctx.resolver().get(key).unwrap_or_else(LootTable::empty)
            }
            LootTableContents::Inline(table) => table.clone(),
        };
        table.get_random_items_raw(ctx, output);
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.NestedLootTable#validate
    pub fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if let LootTableContents::Reference(key) = &self.contents {
            let key = key.clone();
            if !ctx.allows_references() {
                ctx.report_problem(ValidationProblem::ReferenceNotAllowed(key));
                return;
            }
            if ctx.has_visited_element(&key) {
                ctx.report_problem(ValidationProblem::RecursiveReference(key));
                return;
            }
        }

        self.validate_base(ctx);

        match &self.contents {
            LootTableContents::Reference(key) => {
                let key = key.clone();
                if let Some(table) = ctx.resolver().get(&key) {
                    let mut child = ctx.enter_element("reference", key);
                    table.validate(&mut child);
                    ctx.merge_child(child);
                } else {
                    ctx.report_problem(ValidationProblem::MissingReference(key));
                }
            }
            LootTableContents::Inline(table) => {
                let mut child = ctx.for_child(INLINE_LOOT_TABLE_PATH_ELEMENT.get());
                table.validate(&mut child);
                ctx.merge_child(child);
            }
        }
    }

    fn validate_base(&self, ctx: &mut ValidationContext<'_>) {
        for condition in &self.conditions {
            condition.validate(ctx);
        }
        for function in &self.functions {
            function.validate(ctx);
        }
    }
}

#[derive(Debug, Clone)]
pub struct NestedLootTableBuilder {
    contents: LootTableContents,
    weight: i32,
    quality: i32,
    conditions: Vec<LootItemCondition>,
    functions: Vec<LootItemFunction>,
}

impl NestedLootTableBuilder {
    pub fn weight(mut self, weight: i32) -> Self {
        self.weight = weight;
        self
    }

    pub fn quality(mut self, quality: i32) -> Self {
        self.quality = quality;
        self
    }

    pub fn when(mut self, condition: LootItemCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn apply(mut self, function: LootItemFunction) -> Self {
        self.functions.push(function);
        self
    }

    pub fn build(self) -> NestedLootTable {
        NestedLootTable::new(
            self.contents,
            self.weight,
            self.quality,
            self.conditions,
            self.functions,
        )
    }
}

// JAVA: net.minecraft.world.level.storage.loot.entries.NestedLootTable#lootTableReference
pub fn loot_table_reference(key: ResourceKey) -> NestedLootTableBuilder {
    NestedLootTableBuilder {
        contents: LootTableContents::Reference(key),
        weight: 1,
        quality: 0,
        conditions: Vec::new(),
        functions: Vec::new(),
    }
}

// JAVA: net.minecraft.world.level.storage.loot.entries.NestedLootTable#inlineLootTable
pub fn inline_loot_table(table: LootTable) -> NestedLootTableBuilder {
    NestedLootTableBuilder {
        contents: LootTableContents::Inline(table),
        weight: 1,
        quality: 0,
        conditions: Vec::new(),
        functions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct EmptyResolver;

    impl LootTableResolver for EmptyResolver {
        fn get(&self, _key: &ResourceKey) -> Option<LootTable> {
            None
        }
    }

    struct MapResolver(HashMap<ResourceKey, LootTable>);

    impl LootTableResolver for MapResolver {
        fn get(&self, key: &ResourceKey) -> Option<LootTable> {
            self.0.get(key).cloned()
        }
    }

    #[test]
    fn create_item_stack_resolves_reference_and_falls_back_to_empty() {
        let key = ResourceKey("minecraft:chests/test".to_string());
        let table = LootTable {
            items: vec![ItemStack::new("minecraft:diamond", 1, 1.0)],
            report_path_on_validate: false,
        };
        let mut map = HashMap::new();
        map.insert(key.clone(), table);
        let resolver = MapResolver(map);
        let ctx = LootContext::new(&resolver);

        let entry = loot_table_reference(key).build();
        let mut out = Vec::new();
        entry.create_item_stack(&mut |item| out.push(item), &ctx);
        assert_eq!(out, vec![ItemStack::new("minecraft:diamond", 1, 1.0)]);

        let missing = loot_table_reference(ResourceKey("minecraft:missing".to_string())).build();
        let empty_ctx = LootContext::new(&EmptyResolver);
        let mut out2 = Vec::new();
        missing.create_item_stack(&mut |item| out2.push(item), &empty_ctx);
        assert!(out2.is_empty());
    }

    #[test]
    fn create_item_stack_inline_table_and_type() {
        let table = LootTable {
            items: vec![
                ItemStack::new("minecraft:apple", 2, 1.0),
                ItemStack::new("minecraft:bread", -1, 0.5),
            ],
            report_path_on_validate: false,
        };
        let entry = inline_loot_table(table.clone()).build();
        let ctx = LootContext::new(&EmptyResolver);
        let mut out = Vec::new();
        entry.create_item_stack(&mut |item| out.push(item), &ctx);
        assert_eq!(out, table.items);
        assert_eq!(entry.get_type(), LootPoolEntries::LOOT_TABLE);
    }

    #[test]
    fn validate_reports_missing_recursive_and_disallowed_reference() {
        let key = ResourceKey("minecraft:missing".to_string());
        let resolver = EmptyResolver;

        let mut ctx = ValidationContext::new(true, &resolver);
        loot_table_reference(key.clone()).build().validate(&mut ctx);
        assert_eq!(
            ctx.problems(),
            &[ValidationProblem::MissingReference(key.clone())]
        );

        let mut ctx2 = ValidationContext::new(false, &resolver);
        let disallowed_key = ResourceKey("minecraft:no_refs".to_string());
        loot_table_reference(disallowed_key.clone())
            .build()
            .validate(&mut ctx2);
        assert_eq!(
            ctx2.problems(),
            &[ValidationProblem::ReferenceNotAllowed(disallowed_key)]
        );

        let mut ctx3 = ValidationContext::new(true, &resolver);
        ctx3.mark_visited(key.clone());
        loot_table_reference(key.clone()).build().validate(&mut ctx3);
        assert_eq!(
            ctx3.problems(),
            &[ValidationProblem::RecursiveReference(key)]
        );
    }

    #[test]
    fn validate_inline_table_uses_inline_path() {
        let table = LootTable {
            items: Vec::new(),
            report_path_on_validate: true,
        };
        let entry = inline_loot_table(table).build();
        let resolver = EmptyResolver;
        let mut ctx = ValidationContext::new(true, &resolver);
        entry.validate(&mut ctx);
        assert_eq!(
            ctx.problems(),
            &[ValidationProblem::Custom("->{inline}".to_string())]
        );
        assert_eq!(INLINE_LOOT_TABLE_PATH_ELEMENT.get(), "->{inline}");
    }

    #[test]
    fn negative_and_boundary_integer_values_preserved() {
        let entry = loot_table_reference(ResourceKey("minecraft:test".to_string()))
            .weight(i32::MIN)
            .quality(i32::MAX)
            .build();
        assert_eq!(entry.weight, i32::MIN);
        assert_eq!(entry.quality, i32::MAX);

        let entry2 = NestedLootTable::new(
            LootTableContents::Reference(ResourceKey("x".to_string())),
            -1i32,
            i32::wrapping_add(i32::MAX, 1),
            vec![LootItemCondition {
                id: "c".to_string(),
            }],
            vec![LootItemFunction {
                id: "f".to_string(),
            }],
        );
        assert_eq!(entry2.weight, -1i32);
        assert_eq!(entry2.quality, i32::MIN);
        assert_eq!(entry2.conditions.len(), 1);
        assert_eq!(entry2.functions.len(), 1);
    }

    #[test]
    fn float_bits_preserved_in_item_stack() {
        let nan_bits: u32 = 0x7FC0_0001;
        let neg_zero_bits: u32 = 0x8000_0000;
        let inf_bits: u32 = 0x7F80_0000;

        let nan_item = ItemStack::new("a", 1, f32::from_bits(nan_bits));
        let neg_zero_item = ItemStack::new("b", 0, f32::from_bits(neg_zero_bits));
        let inf_item = ItemStack::new("c", i32::MIN, f32::from_bits(inf_bits));

        assert_eq!(nan_item.scale.to_bits(), nan_bits);
        assert_eq!(neg_zero_item.scale.to_bits(), neg_zero_bits);
        assert_eq!(inf_item.scale.to_bits(), inf_bits);

        // Bitwise equality: NaN payloads must match exactly.
        assert_eq!(nan_item, ItemStack::new("a", 1, f32::from_bits(nan_bits)));
        assert_ne!(nan_item, ItemStack::new("a", 1, f32::from_bits(0x7FC0_0002)));
    }
}
