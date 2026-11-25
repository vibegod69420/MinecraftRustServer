use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootPoolEntryType {
    Sequence,
    Group,
    Other,
}

pub struct LootPoolEntries;

impl LootPoolEntries {
    pub const SEQUENCE: LootPoolEntryType = LootPoolEntryType::Sequence;
    pub const GROUP: LootPoolEntryType = LootPoolEntryType::Group;

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#CODEC
    pub fn codec() -> Codec<Box<dyn LootPoolEntryContainer>> {
        Codec {
            _marker: PhantomData,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootPoolEntry;

#[derive(Clone, Default)]
pub struct LootContext {
    pub seed: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Problem {
    pub description: &'static str,
}

pub const NO_CHILDREN_PROBLEM: Problem = Problem {
    description: "Empty children list",
};

pub struct ValidationContext {
    pub problems: Vec<String>,
}

impl ValidationContext {
    // JAVA: net.minecraft.util.ValidationContext#<init>
    pub fn new() -> Self {
        Self {
            problems: Vec::new(),
        }
    }

    // JAVA: net.minecraft.util.ValidationContext#reportProblem
    pub fn report_problem(&mut self, problem: &str) {
        self.problems.push(problem.to_string());
    }

    // JAVA: net.minecraft.util.ValidationContext#forChild
    pub fn for_child(&mut self, _field: &str, _index: usize) -> &mut Self {
        self
    }
}

pub trait LootItemCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemCondition#test
    fn test(&self, ctx: &LootContext) -> bool;

    // JAVA: clone helper for boxed conditions
    fn clone_box(&self) -> Box<dyn LootItemCondition>;
}

pub trait ComposableEntryContainer {
    // JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#expand
    fn expand(&self, ctx: &LootContext, consumer: &mut dyn FnMut(LootPoolEntry)) -> bool;

    // JAVA: clone helper for boxed composed containers
    fn clone_composable(&self) -> Box<dyn ComposableEntryContainer>;
}

pub trait LootPoolEntryContainer {
    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#getType
    fn get_type(&self) -> LootPoolEntryType;

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#conditions
    fn conditions(&self) -> &[Box<dyn LootItemCondition>];

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#expand
    fn expand(&self, ctx: &LootContext, consumer: &mut dyn FnMut(LootPoolEntry)) -> bool;

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#canRun
    fn can_run(&self, ctx: &LootContext) -> bool {
        self.conditions().iter().all(|condition| condition.test(ctx))
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer#validate
    fn validate(&self, ctx: &mut ValidationContext);

    // JAVA: clone helper for boxed entries
    fn clone_entry(&self) -> Box<dyn LootPoolEntryContainer>;
}

pub struct AlwaysTrueContainer;

impl ComposableEntryContainer for AlwaysTrueContainer {
    // JAVA: net.minecraft.world.level.storage.loot.entries.CompositeEntryBase#ALWAYS_TRUE
    fn expand(&self, _ctx: &LootContext, _consumer: &mut dyn FnMut(LootPoolEntry)) -> bool {
        true
    }

    // JAVA: clone helper for AlwaysTrueContainer
    fn clone_composable(&self) -> Box<dyn ComposableEntryContainer> {
        Box::new(AlwaysTrueContainer)
    }
}

pub struct CompositeEntryBase {
    pub entry_type: LootPoolEntryType,
    pub children: Vec<Box<dyn LootPoolEntryContainer>>,
    pub conditions: Vec<Box<dyn LootItemCondition>>,
    pub composed: Box<dyn ComposableEntryContainer>,
}

impl CompositeEntryBase {
    // JAVA: net.minecraft.world.level.storage.loot.entries.CompositeEntryBase#<init>
    pub fn new<F>(
        entry_type: LootPoolEntryType,
        children: Vec<Box<dyn LootPoolEntryContainer>>,
        conditions: Vec<Box<dyn LootItemCondition>>,
        compose: F,
    ) -> Self
    where
        F: FnOnce(&[Box<dyn LootPoolEntryContainer>]) -> Box<dyn ComposableEntryContainer>,
    {
        let composed = compose(&children);
        Self {
            entry_type,
            children,
            conditions,
            composed,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.CompositeEntryBase#createCodec
    pub fn create_codec<T>() -> MapCodec<T> {
        MapCodec::default()
    }
}

impl LootPoolEntryContainer for CompositeEntryBase {
    // JAVA: net.minecraft.world.level.storage.loot.entries.CompositeEntryBase#getType
    fn get_type(&self) -> LootPoolEntryType {
        self.entry_type
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.CompositeEntryBase#conditions
    fn conditions(&self) -> &[Box<dyn LootItemCondition>] {
        &self.conditions
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.CompositeEntryBase#expand
    fn expand(&self, ctx: &LootContext, consumer: &mut dyn FnMut(LootPoolEntry)) -> bool {
        if !self.can_run(ctx) {
            return false;
        }
        self.composed.expand(ctx, consumer)
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.CompositeEntryBase#validate
    fn validate(&self, ctx: &mut ValidationContext) {
        if self.children.is_empty() {
            ctx.report_problem(NO_CHILDREN_PROBLEM.description);
        }

        for (index, child) in self.children.iter().enumerate() {
            child.validate(ctx.for_child("children", index));
        }
    }

    // JAVA: clone helper for CompositeEntryBase
    fn clone_entry(&self) -> Box<dyn LootPoolEntryContainer> {
        Box::new(Self {
            entry_type: self.entry_type,
            children: self.children.iter().map(|child| child.clone_entry()).collect(),
            conditions: self.conditions.iter().map(|condition| condition.clone_box()).collect(),
            composed: self.composed.clone_composable(),
        })
    }
}

pub struct MapCodec<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for MapCodec<T> {
    // JAVA: default MapCodec constructor
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub struct Codec<T> {
    _marker: PhantomData<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct TestEntry {
        result: bool,
        calls: Rc<Cell<usize>>,
    }

    impl LootPoolEntryContainer for TestEntry {
        fn get_type(&self) -> LootPoolEntryType {
            LootPoolEntryType::Other
        }

        fn conditions(&self) -> &[Box<dyn LootItemCondition>] {
            &[]
        }

        fn expand(&self, _ctx: &LootContext, _consumer: &mut dyn FnMut(LootPoolEntry)) -> bool {
            self.calls.set(self.calls.get() + 1);
            self.result
        }

        fn validate(&self, _ctx: &mut ValidationContext) {}

        fn clone_entry(&self) -> Box<dyn LootPoolEntryContainer> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn composite_validate_reports_empty_children() {
        let entry = CompositeEntryBase::new(
            LootPoolEntryType::Other,
            Vec::new(),
            Vec::new(),
            |_children| Box::new(AlwaysTrueContainer),
        );
        let mut ctx = ValidationContext::new();
        entry.validate(&mut ctx);
        assert_eq!(ctx.problems, vec!["Empty children list".to_string()]);
    }

    #[test]
    fn composite_expand_runs_composed_container() {
        let calls = Rc::new(Cell::new(0usize));
        let child = Box::new(TestEntry {
            result: true,
            calls: calls.clone(),
        });
        let entry = CompositeEntryBase::new(
            LootPoolEntryType::Other,
            vec![child],
            Vec::new(),
            |children| {
                let child = children[0].clone_entry();
                Box::new(DelegateForTest { child })
            },
        );
        let mut ctx = LootContext::default();
        let expanded = entry.expand(&mut ctx, &mut |_entry| {});
        assert!(expanded);
    }

    struct DelegateForTest {
        child: Box<dyn LootPoolEntryContainer>,
    }

    impl ComposableEntryContainer for DelegateForTest {
        fn expand(&self, ctx: &LootContext, consumer: &mut dyn FnMut(LootPoolEntry)) -> bool {
            self.child.expand(ctx, consumer)
        }

        fn clone_composable(&self) -> Box<dyn ComposableEntryContainer> {
            Box::new(DelegateForTest {
                child: self.child.clone_entry(),
            })
        }
    }
}
