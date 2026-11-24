/// JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer
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

#[derive(Debug, Clone, Default)]
pub struct LootItemConditionBuilder;

impl LootItemConditionBuilder {
    pub fn build(&self) -> LootItemCondition {
        LootItemCondition
    }
}

#[derive(Debug, Clone)]
pub struct IndexedFieldPathElement {
    pub field: &'static str,
    pub index: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationContext;

impl ValidationContext {
    // JAVA: net.minecraft.world.level.storage.loot.ValidationContext#forChild
    pub fn for_child(&self, _path: IndexedFieldPathElement) -> ValidationContext {
        ValidationContext
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootPoolEntryType(pub &'static str);

pub trait ComposableEntryContainer {
    fn expand(&self, ctx: &LootContext, consumer: &mut dyn FnMut()) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct AlternativesEntryBuilder;

#[derive(Debug, Clone, Default)]
pub struct EntryGroupBuilder;

#[derive(Debug, Clone, Default)]
pub struct SequentialEntryBuilder;

pub struct LootPoolEntryContainer {
    pub conditions: Vec<LootItemCondition>,
}

impl LootPoolEntryContainer {
    pub fn new(conditions: Vec<LootItemCondition>) -> Self {
        Self { conditions }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#validate
    pub fn validate(&self, ctx: &ValidationContext) {
        for (i, cond) in self.conditions.iter().enumerate() {
            let child = ctx.for_child(IndexedFieldPathElement {
                field: "conditions",
                index: i,
            });
            cond.validate(&child);
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#canRun
    pub fn can_run(&self, ctx: &LootContext) -> bool {
        self.conditions.iter().all(|c| c.test(ctx))
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntryType("unknown")
    }
}

pub struct LootPoolEntryContainerBuilder {
    pub conditions: Vec<LootItemConditionBuilder>,
}

impl LootPoolEntryContainerBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer.Builder#getThis
    pub fn get_this(self) -> Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer.Builder#when
    pub fn when(mut self, builder: LootItemConditionBuilder) -> Self {
        self.conditions.push(builder);
        self
    }

    pub fn unwrap(self) -> Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer.Builder#getConditions
    pub fn get_conditions(&self) -> Vec<LootItemCondition> {
        self.conditions.iter().map(|b| b.build()).collect()
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer.Builder#otherwise
    pub fn otherwise(self, _other: LootPoolEntryContainerBuilder) -> AlternativesEntryBuilder {
        AlternativesEntryBuilder
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer.Builder#append
    pub fn append(self, _other: LootPoolEntryContainerBuilder) -> EntryGroupBuilder {
        EntryGroupBuilder
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer.Builder#then
    pub fn then(self, _other: LootPoolEntryContainerBuilder) -> SequentialEntryBuilder {
        SequentialEntryBuilder
    }
}

impl Default for LootPoolEntryContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_can_run_empty() {
        // JAVA behavior: Util.allOf([]) => true
        let container = LootPoolEntryContainer::new(Vec::new());
        assert!(container.can_run(&LootContext));
    }

    #[test]
    fn test_can_run_all_true() {
        let container = LootPoolEntryContainer::new(vec![LootItemCondition, LootItemCondition]);
        assert!(container.can_run(&LootContext));
    }

    #[test]
    fn test_validate_runs_without_panic() {
        let container = LootPoolEntryContainer::new(vec![LootItemCondition]);
        container.validate(&ValidationContext);
    }

    #[test]
    fn test_builder_when_and_get_conditions() {
        let builder = LootPoolEntryContainerBuilder::new()
            .when(LootItemConditionBuilder)
            .when(LootItemConditionBuilder);
        let conditions = builder.get_conditions();
        assert_eq!(conditions.len(), 2);
    }

    #[test]
    fn test_builder_unwrap_returns_self() {
        let builder = LootPoolEntryContainerBuilder::new().unwrap();
        assert_eq!(builder.conditions.len(), 0);
    }
}
