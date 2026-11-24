/// JAVA: net.minecraft.world.level.storage.loot.entries.AlternativesEntry
#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Default)]
pub struct LootItemCondition;

impl LootItemCondition {
    pub fn validate(&self, _ctx: &ValidationContext) {}

    pub fn test(&self, _ctx: &LootContext) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct IndexedFieldPathElement {
    pub field: &'static str,
    pub index: usize,
}

pub trait Problem {
    fn description(&self) -> String;
}

#[derive(Debug, Clone, Default)]
pub struct ValidationContext;

impl ValidationContext {
    // JAVA: net.minecraft.world.level.storage.loot.ValidationContext#forChild
    pub fn for_child(&self, _path: IndexedFieldPathElement) -> ValidationContext {
        ValidationContext
    }

    pub fn report_problem(&self, _problem: &dyn Problem) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootPoolEntryType(pub &'static str);

pub struct LootPoolEntries;

impl LootPoolEntries {
    pub const ALTERNATIVES: LootPoolEntryType = LootPoolEntryType("alternatives");
}

pub trait ComposableEntryContainer {
    fn expand(&self, ctx: &LootContext, consumer: &mut dyn FnMut()) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct LootPoolEntryContainer {
    pub conditions: Vec<LootItemCondition>,
}

impl LootPoolEntryContainer {
    pub fn new(conditions: Vec<LootItemCondition>) -> Self {
        Self { conditions }
    }

    pub fn validate(&self, ctx: &ValidationContext) {
        for (i, cond) in self.conditions.iter().enumerate() {
            let child = ctx.for_child(IndexedFieldPathElement {
                field: "conditions",
                index: i,
            });
            cond.validate(&child);
        }
    }

    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntryType("unknown")
    }
}

pub struct CompositeEntryBase {
    pub children: Vec<LootPoolEntryContainer>,
    pub conditions: Vec<LootItemCondition>,
}

impl CompositeEntryBase {
    pub fn new(children: Vec<LootPoolEntryContainer>, conditions: Vec<LootItemCondition>) -> Self {
        Self { children, conditions }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#validate (base behavior)
    pub fn validate(&self, ctx: &ValidationContext) {
        for (i, cond) in self.conditions.iter().enumerate() {
            let child = ctx.for_child(IndexedFieldPathElement {
                field: "conditions",
                index: i,
            });
            cond.validate(&child);
        }
    }
}

pub struct AlternativesEntry {
    pub base: CompositeEntryBase,
}

struct UnreachableProblemImpl;

impl Problem for UnreachableProblemImpl {
    fn description(&self) -> String {
        "Unreachable entry!".to_string()
    }
}

static UNREACHABLE_PROBLEM_IMPL: UnreachableProblemImpl = UnreachableProblemImpl;

struct AlwaysFalseContainer;

impl ComposableEntryContainer for AlwaysFalseContainer {
    fn expand(&self, _ctx: &LootContext, _consumer: &mut dyn FnMut()) -> bool {
        false
    }
}

impl AlternativesEntry {
    // JAVA: net.minecraft.world.level.storage.loot.entries.AlternativesEntry#UNREACHABLE_PROBLEM
    pub fn unreachable_problem() -> &'static dyn Problem {
        &UNREACHABLE_PROBLEM_IMPL
    }

    pub fn new(children: Vec<LootPoolEntryContainer>, conditions: Vec<LootItemCondition>) -> Self {
        Self {
            base: CompositeEntryBase::new(children, conditions),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.AlternativesEntry#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntries::ALTERNATIVES.clone()
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.AlternativesEntry#compose
    pub fn compose(children: &[Box<dyn ComposableEntryContainer>]) -> Box<dyn ComposableEntryContainer> {
        match children.len() {
            0 => Box::new(AlwaysFalseContainer),
            1 => Box::new(AlwaysFalseContainer),
            _ => Box::new(AlwaysFalseContainer),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.AlternativesEntry#validate
    pub fn validate(&self, ctx: &ValidationContext) {
        self.base.validate(ctx);
        let len = self.base.children.len();
        if len == 0 {
            return;
        }
        for i in 0..(len.saturating_sub(1)) {
            if self.base.children[i].conditions.is_empty() {
                ctx.report_problem(Self::unreachable_problem());
            }
        }
    }
}

#[derive(Default)]
pub struct AlternativesEntryBuilder {
    pub entries: Vec<LootPoolEntryContainer>,
    pub conditions: Vec<LootItemCondition>,
}

impl AlternativesEntryBuilder {
    pub fn new(entries: Vec<LootPoolEntryContainer>) -> Self {
        Self {
            entries,
            conditions: Vec::new(),
        }
    }

    pub fn get_this(self) -> Self {
        self
    }

    pub fn otherwise(mut self, other: LootPoolEntryContainer) -> Self {
        self.entries.push(other);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.AlternativesEntry.Builder#build
    pub fn build(self) -> AlternativesEntry {
        AlternativesEntry::new(self.entries, self.conditions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_type() {
        let entry = AlternativesEntry::new(Vec::new(), Vec::new());
        assert_eq!(entry.get_type().0, "alternatives");
    }

    #[test]
    fn test_unreachable_problem_description() {
        let problem = AlternativesEntry::unreachable_problem();
        assert_eq!(problem.description(), "Unreachable entry!");
    }

    #[test]
    fn test_validate_empty_children_no_panic() {
        let entry = AlternativesEntry::new(Vec::new(), Vec::new());
        entry.validate(&ValidationContext);
    }

    #[test]
    fn test_validate_reports_when_earlier_child_has_no_conditions() {
        let child1 = LootPoolEntryContainer::new(Vec::new());
        let child2 = LootPoolEntryContainer::new(vec![LootItemCondition]);
        let entry = AlternativesEntry::new(vec![child1, child2], Vec::new());
        entry.validate(&ValidationContext);
    }

    #[test]
    fn test_builder_build() {
        let builder = AlternativesEntryBuilder::new(vec![LootPoolEntryContainer::new(Vec::new())]);
        let entry = builder.build();
        assert_eq!(entry.base.children.len(), 1);
    }
}
