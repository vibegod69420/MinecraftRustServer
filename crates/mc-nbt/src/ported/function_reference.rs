// JAVA: net.minecraft.world.level.storage.loot.functions.FunctionReference

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKey {
    pub name: String,
}

impl ResourceKey {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

pub trait LootItemCondition {}

#[derive(Default)]
pub struct LootContext {
    pub resolver: BTreeMap<ResourceKey, Box<dyn LootItemFunction>>,
    visited: Vec<ResourceKey>,
}

impl LootContext {
    pub fn new(resolver: BTreeMap<ResourceKey, Box<dyn LootItemFunction>>) -> Self {
        Self { resolver, visited: Vec::new() }
    }

    pub fn push_visited_element(&mut self, entry: ResourceKey) -> bool {
        if self.visited.contains(&entry) {
            false
        } else {
            self.visited.push(entry);
            true
        }
    }

    pub fn pop_visited_element(&mut self, entry: ResourceKey) {
        if let Some(pos) = self.visited.iter().rposition(|x| x == &entry) {
            self.visited.remove(pos);
        }
    }

    pub fn get_resolver(&self) -> &BTreeMap<ResourceKey, Box<dyn LootItemFunction>> {
        &self.resolver
    }
}

pub trait LootItemFunction {
    fn get_type(&self) -> &'static str;
    fn apply(&self, stack: ItemStack, ctx: &mut LootContext) -> ItemStack;
    fn validate(&self, _ctx: &mut ValidationContext) {}
}

#[derive(Clone)]
pub struct ValidationContext {
    pub allows_references: bool,
    pub visited: BTreeSet<ResourceKey>,
    pub resolver: Rc<BTreeMap<ResourceKey, Box<dyn LootItemFunction>>>,
    pub problems: Vec<String>,
}

impl ValidationContext {
    pub fn new(
        allows_references: bool,
        resolver: BTreeMap<ResourceKey, Box<dyn LootItemFunction>>,
    ) -> Self {
        Self {
            allows_references,
            visited: BTreeSet::new(),
            resolver: Rc::new(resolver),
            problems: Vec::new(),
        }
    }

    pub fn allows_references(&self) -> bool {
        self.allows_references
    }

    pub fn has_visited_element(&self, key: &ResourceKey) -> bool {
        self.visited.contains(key)
    }

    pub fn report_problem(&mut self, problem: String) {
        self.problems.push(problem);
    }

    pub fn resolver(&self) -> &BTreeMap<ResourceKey, Box<dyn LootItemFunction>> {
        &self.resolver
    }

    pub fn enter_element(&self, _path: &str, key: ResourceKey) -> Self {
        let mut next = self.clone();
        next.visited.insert(key);
        next
    }
}

pub struct LootItemConditionalFunction {
    pub conditions: Vec<Box<dyn LootItemCondition>>,
}

pub struct FunctionReference {
    pub conditions: Vec<Box<dyn LootItemCondition>>,
    pub name: ResourceKey,
}

impl FunctionReference {
    pub fn new(conditions: Vec<Box<dyn LootItemCondition>>, name: ResourceKey) -> Self {
        Self { conditions, name }
    }

    pub fn get_type(&self) -> &'static str {
        "reference"
    }

    pub fn validate(&self, ctx: &mut ValidationContext) {
        if !ctx.allows_references() {
            ctx.report_problem(format!("reference not allowed: {}", self.name.name));
        } else if ctx.has_visited_element(&self.name) {
            ctx.report_problem(format!("recursive reference: {}", self.name.name));
        } else {
            if let Some(function) = ctx.resolver().get(&self.name) {
                let mut entered = ctx.enter_element("reference", self.name.clone());
                function.validate(&mut entered);
                ctx.problems.extend(entered.problems);
            } else {
                ctx.report_problem(format!("missing reference: {}", self.name.name));
            }
        }
    }

    pub fn run(&self, stack: ItemStack, ctx: &mut LootContext) -> ItemStack {
        if !ctx.resolver.contains_key(&self.name) {
            return stack;
        }

        let mut function = ctx.resolver.remove(&self.name).unwrap();
        let entry = self.name.clone();
        let pushed = ctx.push_visited_element(entry.clone());

        let result = if pushed {
            let output = function.apply(stack, ctx);
            ctx.pop_visited_element(entry);
            output
        } else {
            stack
        };

        ctx.resolver.insert(self.name.clone(), function);
        result
    }

    pub fn apply(&self, stack: ItemStack, ctx: &mut LootContext) -> ItemStack {
        self.run(stack, ctx)
    }
}

impl LootItemFunction for FunctionReference {
    fn get_type(&self) -> &'static str {
        FunctionReference::get_type(self)
    }

    fn apply(&self, stack: ItemStack, ctx: &mut LootContext) -> ItemStack {
        self.run(stack, ctx)
    }

    fn validate(&self, ctx: &mut ValidationContext) {
        FunctionReference::validate(self, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AddFunction {
        amount: i32,
    }

    impl LootItemFunction for AddFunction {
        fn get_type(&self) -> &'static str {
            "add"
        }

        fn apply(&self, mut stack: ItemStack, _ctx: &mut LootContext) -> ItemStack {
            stack.value += self.amount;
            stack
        }
    }

    #[test]
    fn missing_reference_returns_original_stack() {
        let reference = FunctionReference::new(Vec::new(), ResourceKey::new("missing"));
        let mut ctx = LootContext::default();
        let stack = ItemStack { value: 5 };
        let out = reference.run(stack.clone(), &mut ctx);
        assert_eq!(out, stack);
    }

    #[test]
    fn reference_applies_target_function() {
        let name = ResourceKey::new("add_one");
        let mut resolver = BTreeMap::new();
        resolver.insert(name.clone(), Box::new(AddFunction { amount: 1 }) as Box<dyn LootItemFunction>);
        let mut ctx = LootContext::new(resolver);

        let reference = FunctionReference::new(Vec::new(), name);
        let out = reference.run(ItemStack { value: 10 }, &mut ctx);
        assert_eq!(out.value, 11);
    }

    #[test]
    fn cycle_is_detected() {
        let name = ResourceKey::new("self");
        let reference = FunctionReference::new(Vec::new(), name.clone());
        let mut resolver = BTreeMap::new();
        resolver.insert(name.clone(), Box::new(reference.clone_placeholder()) as Box<dyn LootItemFunction>);
        let mut ctx = LootContext::new(resolver);

        let out = FunctionReference::new(Vec::new(), name).run(ItemStack { value: 7 }, &mut ctx);
        assert_eq!(out.value, 7);
    }

    #[test]
    fn validation_reports_missing_reference() {
        let name = ResourceKey::new("missing");
        let reference = FunctionReference::new(Vec::new(), name);
        let mut ctx = ValidationContext::new(true, BTreeMap::new());
        reference.validate(&mut ctx);
        assert_eq!(ctx.problems.len(), 1);
    }

    #[test]
    fn validation_reports_recursive_reference() {
        let name = ResourceKey::new("self");
        let reference = FunctionReference::new(Vec::new(), name.clone());
        let mut visited = BTreeSet::new();
        visited.insert(name.clone());
        let mut ctx = ValidationContext::new(true, BTreeMap::new());
        ctx.visited = visited;
        reference.validate(&mut ctx);
        assert_eq!(ctx.problems.len(), 1);
    }

    #[test]
    fn validation_reports_disallowed_references() {
        let reference = FunctionReference::new(Vec::new(), ResourceKey::new("any"));
        let mut ctx = ValidationContext::new(false, BTreeMap::new());
        reference.validate(&mut ctx);
        assert_eq!(ctx.problems.len(), 1);
    }
}

// Test-only helper to avoid cloning the real FunctionReference before it is defined.
trait Placeholder {
    fn clone_placeholder(&self) -> FunctionReference;
}

impl Placeholder for FunctionReference {
    fn clone_placeholder(&self) -> FunctionReference {
        FunctionReference::new(Vec::new(), self.name.clone())
    }
}
