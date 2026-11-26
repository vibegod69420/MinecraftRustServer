use std::fmt;

// ---------------------------------------------------------------------------
// Minimal self-contained supporting types
// ---------------------------------------------------------------------------

/// Minimal resource location (namespace:path), used as a key identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal typed resource key wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    location: ResourceLocation,
}

impl ResourceKey {
    pub fn new(location: ResourceLocation) -> Self {
        Self { location }
    }

    pub fn identifier(&self) -> &ResourceLocation {
        &self.location
    }

    pub fn location(&self) -> &ResourceLocation {
        &self.location
    }
}

/// Minimal loot item condition type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootItemConditionType {
    Reference,
}

/// Validation problems reported during condition validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationProblem {
    ReferenceNotAllowed(ResourceLocation),
    RecursiveReference(ResourceLocation),
    MissingReference(ResourceLocation),
}

/// Resolves loot item conditions by resource location.
pub trait ConditionResolver {
    fn get_condition<'s>(&'s self, key: &ResourceLocation) -> Option<&'s dyn LootItemCondition>;
}

/// Minimal validation context used by loot condition validation.
///
/// The `'a` lifetime is the resolver/condition lifetime, allowing resolved
/// conditions to be validated while the context is mutably borrowed.
pub trait ValidationContext<'a> {
    fn allows_references(&self) -> bool;
    fn has_visited_element(&self, key: &ResourceLocation) -> bool;
    fn report_problem(&mut self, problem: ValidationProblem);
    fn resolver(&self) -> &'a dyn ConditionResolver;
    fn enter_element<'b>(&'b mut self, key: &ResourceLocation) -> Box<dyn ValidationContext<'a> + 'b>;
}

/// Opaque visited-entry token for loot context loop detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VisitedEntry(pub usize);

/// Creates a visited entry from a condition reference identity.
/// Mirrors `LootContext.createVisitedEntry(condition)` behavior for loop detection.
pub fn create_visited_entry(condition: &dyn LootItemCondition) -> VisitedEntry {
    let ptr = condition as *const dyn LootItemCondition as *const () as usize;
    VisitedEntry(ptr)
}

/// Minimal loot context used by loot condition testing.
///
/// The `'a` lifetime is the resolver/condition lifetime, allowing resolved
/// conditions to be tested while the context is mutably borrowed.
pub trait LootContext<'a> {
    fn get_resolver(&self) -> &'a dyn ConditionResolver;
    fn push_visited_element(&mut self, entry: VisitedEntry) -> bool;
    fn pop_visited_element(&mut self, entry: VisitedEntry);
}

/// Minimal loot item condition interface.
pub trait LootItemCondition {
    fn get_type(&self) -> LootItemConditionType;
    fn validate<'a>(&self, ctx: &mut dyn ValidationContext<'a>);
    fn test<'a>(&self, ctx: &mut dyn LootContext<'a>) -> bool;
}

/// Default `LootItemCondition#validate` behavior (base interface no-op).
fn default_loot_item_condition_validate<'a>(_ctx: &mut dyn ValidationContext<'a>) {}

// ---------------------------------------------------------------------------
// Ported: net.minecraft.world.level.storage.loot.predicates.ConditionReference
// ---------------------------------------------------------------------------

/// Faithful port of `net.minecraft.world.level.storage.loot.predicates.ConditionReference`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConditionReference {
    pub name: ResourceKey,
}

impl ConditionReference {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionReference#ConditionReference
    pub fn new(name: ResourceKey) -> Self {
        Self { name }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionReference#name
    pub fn name(&self) -> &ResourceKey {
        &self.name
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionReference#conditionReference
    pub fn condition_reference(name: ResourceKey) -> impl Fn() -> ConditionReference {
        move || ConditionReference::new(name.clone())
    }
}

impl LootItemCondition for ConditionReference {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionReference#getType
    fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::Reference
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionReference#validate
    fn validate<'a>(&self, ctx: &mut dyn ValidationContext<'a>) {
        let key = self.name.identifier().clone();
        if !ctx.allows_references() {
            ctx.report_problem(ValidationProblem::ReferenceNotAllowed(key));
        } else if ctx.has_visited_element(&key) {
            ctx.report_problem(ValidationProblem::RecursiveReference(key));
        } else {
            // LootItemCondition.super.validate(ctx)
            default_loot_item_condition_validate(ctx);
            let resolved = ctx.resolver().get_condition(&key);
            match resolved {
                Some(condition) => {
                    let mut entered = ctx.enter_element(&key);
                    condition.validate(entered.as_mut());
                }
                None => {
                    ctx.report_problem(ValidationProblem::MissingReference(key));
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionReference#test
    fn test<'a>(&self, ctx: &mut dyn LootContext<'a>) -> bool {
        let key = self.name.identifier().clone();
        let condition = ctx.get_resolver().get_condition(&key);
        match condition {
            None => {
                eprintln!("Tried using unknown condition table called {}", key);
                false
            }
            Some(condition) => {
                let entry = create_visited_entry(condition);
                if ctx.push_visited_element(entry) {
                    let result = condition.test(ctx);
                    ctx.pop_visited_element(entry);
                    result
                } else {
                    eprintln!("Detected infinite loop in loot tables");
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    struct AlwaysTrue;

    impl LootItemCondition for AlwaysTrue {
        fn get_type(&self) -> LootItemConditionType {
            LootItemConditionType::Reference
        }
        fn validate<'a>(&self, _ctx: &mut dyn ValidationContext<'a>) {}
        fn test<'a>(&self, _ctx: &mut dyn LootContext<'a>) -> bool {
            true
        }
    }

    struct AlwaysFalse;

    impl LootItemCondition for AlwaysFalse {
        fn get_type(&self) -> LootItemConditionType {
            LootItemConditionType::Reference
        }
        fn validate<'a>(&self, _ctx: &mut dyn ValidationContext<'a>) {}
        fn test<'a>(&self, _ctx: &mut dyn LootContext<'a>) -> bool {
            false
        }
    }

    struct MockResolver {
        conditions: HashMap<ResourceLocation, Box<dyn LootItemCondition>>,
    }

    impl ConditionResolver for MockResolver {
        fn get_condition<'s>(&'s self, key: &ResourceLocation) -> Option<&'s dyn LootItemCondition> {
            self.conditions.get(key).map(|b| b.as_ref())
        }
    }

    struct MockLootContext<'a> {
        resolver: &'a dyn ConditionResolver,
        visited: HashSet<VisitedEntry>,
    }

    impl<'a> LootContext<'a> for MockLootContext<'a> {
        fn get_resolver(&self) -> &'a dyn ConditionResolver {
            self.resolver
        }
        fn push_visited_element(&mut self, entry: VisitedEntry) -> bool {
            if self.visited.contains(&entry) {
                return false;
            }
            self.visited.insert(entry);
            true
        }
        fn pop_visited_element(&mut self, entry: VisitedEntry) {
            self.visited.remove(&entry);
        }
    }

    struct FailPushLootContext<'a> {
        resolver: &'a dyn ConditionResolver,
    }

    impl<'a> LootContext<'a> for FailPushLootContext<'a> {
        fn get_resolver(&self) -> &'a dyn ConditionResolver {
            self.resolver
        }
        fn push_visited_element(&mut self, _entry: VisitedEntry) -> bool {
            false
        }
        fn pop_visited_element(&mut self, _entry: VisitedEntry) {}
    }

    struct MockValidationContext<'a> {
        allows_refs: bool,
        visited: HashSet<ResourceLocation>,
        problems: Rc<RefCell<Vec<ValidationProblem>>>,
        resolver: &'a dyn ConditionResolver,
    }

    impl<'a> ValidationContext<'a> for MockValidationContext<'a> {
        fn allows_references(&self) -> bool {
            self.allows_refs
        }
        fn has_visited_element(&self, key: &ResourceLocation) -> bool {
            self.visited.contains(key)
        }
        fn report_problem(&mut self, problem: ValidationProblem) {
            self.problems.borrow_mut().push(problem);
        }
        fn resolver(&self) -> &'a dyn ConditionResolver {
            self.resolver
        }
        fn enter_element<'b>(
            &'b mut self,
            key: &ResourceLocation,
        ) -> Box<dyn ValidationContext<'a> + 'b> {
            let mut visited = self.visited.clone();
            visited.insert(key.clone());
            Box::new(MockValidationContext {
                allows_refs: self.allows_refs,
                visited,
                problems: Rc::clone(&self.problems),
                resolver: self.resolver,
            })
        }
    }

    #[test]
    fn test_missing_reference_returns_false() {
        let resolver = MockResolver {
            conditions: HashMap::new(),
        };
        let mut ctx = MockLootContext {
            resolver: &resolver,
            visited: HashSet::new(),
        };
        let cond = ConditionReference::new(ResourceKey::new(ResourceLocation::new(
            "minecraft",
            "missing",
        )));
        assert!(!cond.test(&mut ctx));
    }

    #[test]
    fn test_infinite_loop_returns_false() {
        let key = ResourceLocation::new("minecraft", "self");
        let mut conditions = HashMap::new();
        conditions.insert(key.clone(), Box::new(AlwaysTrue) as Box<dyn LootItemCondition>);
        let resolver = MockResolver { conditions };
        let mut ctx = FailPushLootContext { resolver: &resolver };
        let cond = ConditionReference::new(ResourceKey::new(key));
        assert!(!cond.test(&mut ctx));
    }

    #[test]
    fn test_successful_reference_delegates_and_pops() {
        let key = ResourceLocation::new("minecraft", "ok");
        let mut conditions = HashMap::new();
        conditions.insert(key.clone(), Box::new(AlwaysTrue) as Box<dyn LootItemCondition>);
        let resolver = MockResolver { conditions };
        let mut ctx = MockLootContext {
            resolver: &resolver,
            visited: HashSet::new(),
        };
        let cond = ConditionReference::new(ResourceKey::new(key));
        assert!(cond.test(&mut ctx));
        assert!(ctx.visited.is_empty());

        let key_false = ResourceLocation::new("minecraft", "no");
        let mut conditions2 = HashMap::new();
        conditions2.insert(
            key_false.clone(),
            Box::new(AlwaysFalse) as Box<dyn LootItemCondition>,
        );
        let resolver2 = MockResolver { conditions: conditions2 };
        let mut ctx2 = MockLootContext {
            resolver: &resolver2,
            visited: HashSet::new(),
        };
        let cond2 = ConditionReference::new(ResourceKey::new(key_false));
        assert!(!cond2.test(&mut ctx2));
        assert!(ctx2.visited.is_empty());
    }

    #[test]
    fn test_validate_reports_missing_and_not_allowed_and_recursive() {
        let resolver = MockResolver {
            conditions: HashMap::new(),
        };

        let problems = Rc::new(RefCell::new(Vec::new()));
        let mut ctx = MockValidationContext {
            allows_refs: false,
            visited: HashSet::new(),
            problems: Rc::clone(&problems),
            resolver: &resolver,
        };
        let key_na = ResourceLocation::new("minecraft", "na");
        ConditionReference::new(ResourceKey::new(key_na.clone())).validate(&mut ctx);
        assert_eq!(
            *problems.borrow(),
            vec![ValidationProblem::ReferenceNotAllowed(key_na)]
        );

        let problems = Rc::new(RefCell::new(Vec::new()));
        let key_rec = ResourceLocation::new("minecraft", "rec");
        let mut visited = HashSet::new();
        visited.insert(key_rec.clone());
        let mut ctx = MockValidationContext {
            allows_refs: true,
            visited,
            problems: Rc::clone(&problems),
            resolver: &resolver,
        };
        ConditionReference::new(ResourceKey::new(key_rec.clone())).validate(&mut ctx);
        assert_eq!(
            *problems.borrow(),
            vec![ValidationProblem::RecursiveReference(key_rec)]
        );

        let problems = Rc::new(RefCell::new(Vec::new()));
        let key_missing = ResourceLocation::new("minecraft", "missing");
        let mut ctx = MockValidationContext {
            allows_refs: true,
            visited: HashSet::new(),
            problems: Rc::clone(&problems),
            resolver: &resolver,
        };
        ConditionReference::new(ResourceKey::new(key_missing.clone())).validate(&mut ctx);
        assert_eq!(
            *problems.borrow(),
            vec![ValidationProblem::MissingReference(key_missing)]
        );
    }

    #[test]
    fn test_validate_resolves_and_enters_element() {
        let target_key = ResourceLocation::new("minecraft", "target");
        let mut conditions = HashMap::new();
        conditions.insert(
            target_key.clone(),
            Box::new(AlwaysTrue) as Box<dyn LootItemCondition>,
        );
        let resolver = MockResolver { conditions };
        let problems = Rc::new(RefCell::new(Vec::new()));
        let mut ctx = MockValidationContext {
            allows_refs: true,
            visited: HashSet::new(),
            problems: Rc::clone(&problems),
            resolver: &resolver,
        };
        let cond = ConditionReference::new(ResourceKey::new(target_key));
        cond.validate(&mut ctx);
        assert!(problems.borrow().is_empty());
    }

    #[test]
    fn test_get_type_builder_and_edge_case_names() {
        let key = ResourceKey::new(ResourceLocation::new("", ""));
        let cond = ConditionReference::condition_reference(key.clone())();
        assert_eq!(cond.get_type(), LootItemConditionType::Reference);
        assert_eq!(cond.name().identifier(), key.identifier());
        assert_eq!(cond.name().location().to_string(), ":");

        // Edge-case: negative-like and large numeric strings in identifiers (no numeric logic in class,
        // but ensures key handling is robust for unusual inputs).
        let key2 = ResourceKey::new(ResourceLocation::new("ns", "-1/2147483648/NaN"));
        let cond2 = ConditionReference::new(key2.clone());
        assert_eq!(cond2.name().identifier().path, "-1/2147483648/NaN");
    }
}
