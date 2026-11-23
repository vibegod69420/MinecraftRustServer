use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
    pub weight: f32,
}

impl ItemStack {
    pub fn new(id: impl Into<String>, count: i32, weight: f32) -> Self {
        Self {
            id: id.into(),
            count,
            weight,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.id.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServerLevel;

#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub level: ServerLevel,
}

impl LootContext {
    pub fn get_level(&self) -> &ServerLevel {
        &self.level
    }
}

pub struct LootTable;

impl LootTable {
    pub fn create_stack_splitter<'a>(
        _level: &ServerLevel,
        output: &'a mut dyn FnMut(ItemStack),
    ) -> Box<dyn FnMut(ItemStack) + 'a> {
        Box::new(move |stack: ItemStack| output(stack))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProblemReporter {
    messages: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
}

impl ProblemReporter {
    pub fn report(&self, msg: impl Into<String>) {
        self.messages.borrow_mut().push(msg.into());
    }

    pub fn messages(&self) -> Vec<String> {
        self.messages.borrow().clone()
    }
}

#[derive(Debug, Clone)]
pub struct IndexedFieldPathElement {
    pub name: String,
    pub index: usize,
}

impl IndexedFieldPathElement {
    pub fn new(name: impl Into<String>, index: usize) -> Self {
        Self {
            name: name.into(),
            index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    reporter: ProblemReporter,
    path: String,
}

impl ValidationContext {
    pub fn new(reporter: ProblemReporter) -> Self {
        Self {
            reporter,
            path: String::new(),
        }
    }

    pub fn for_child(&self, element: &IndexedFieldPathElement) -> Self {
        let child_path = if self.path.is_empty() {
            format!("{}[{}]", element.name, element.index)
        } else {
            format!("{}.{}[{}]", self.path, element.name, element.index)
        };
        Self {
            reporter: self.reporter.clone(),
            path: child_path,
        }
    }

    pub fn report(&self, msg: impl Into<String>) {
        let msg = msg.into();
        let full = if self.path.is_empty() {
            msg
        } else {
            format!("{}: {}", self.path, msg)
        };
        self.reporter.report(full);
    }
}

pub trait LootItemCondition: fmt::Debug {
    fn test(&self, ctx: &LootContext) -> bool;
    fn validate(&self, _ctx: &ValidationContext) {}
}

pub trait LootPoolEntry: fmt::Debug {
    fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), ctx: &LootContext);
}

pub trait LootPoolEntryContainer: fmt::Debug {
    fn expand(&self, ctx: &LootContext, output: &mut dyn FnMut(&dyn LootPoolEntry));
    fn validate(&self, ctx: &ValidationContext);
}

pub trait ContainerComponentManipulator: fmt::Debug {
    fn set_contents(&self, stack: &mut ItemStack, contents: Vec<ItemStack>);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub u32);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SET_CONTENTS: LootItemFunctionType = LootItemFunctionType(1);
}

pub trait LootItemFunction: fmt::Debug {
    fn run(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack;
}

#[derive(Debug)]
pub struct SetContainerContents {
    conditions: Vec<Box<dyn LootItemCondition>>,
    component: Box<dyn ContainerComponentManipulator>,
    entries: Vec<Box<dyn LootPoolEntryContainer>>,
}

impl SetContainerContents {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents#<init>
    fn new(
        conditions: Vec<Box<dyn LootItemCondition>>,
        component: Box<dyn ContainerComponentManipulator>,
        entries: Vec<Box<dyn LootPoolEntryContainer>>,
    ) -> Self {
        Self {
            conditions,
            component,
            entries,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_CONTENTS
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents#run
    pub fn run(&self, mut stack: ItemStack, ctx: &LootContext) -> ItemStack {
        if stack.is_empty() {
            return stack;
        }

        let mut items: Vec<ItemStack> = Vec::new();
        for entry_container in &self.entries {
            entry_container.expand(ctx, &mut |pool_entry: &dyn LootPoolEntry| {
                let mut push_output = |s: ItemStack| {
                    items.push(s);
                };
                let mut splitter =
                    LootTable::create_stack_splitter(ctx.get_level(), &mut push_output);
                pool_entry.create_item_stack(&mut *splitter, ctx);
            });
        }
        self.component.set_contents(&mut stack, items);
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents#validate
    pub fn validate(&self, ctx: &ValidationContext) {
        for condition in &self.conditions {
            condition.validate(ctx);
        }

        let mut i: i32 = 0;
        while (i as usize) < self.entries.len() {
            let idx = i as usize;
            let child = ctx.for_child(&IndexedFieldPathElement::new("entries", idx));
            self.entries[idx].validate(&child);
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents#setContents
    pub fn set_contents(component: Box<dyn ContainerComponentManipulator>) -> Builder {
        Builder::new(component)
    }
}

impl LootItemFunction for SetContainerContents {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents#run
    fn run(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack {
        SetContainerContents::run(self, stack, ctx)
    }
}

pub struct Builder {
    conditions: Vec<Box<dyn LootItemCondition>>,
    entries: Vec<Box<dyn LootPoolEntryContainer>>,
    component: Option<Box<dyn ContainerComponentManipulator>>,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents$Builder#<init>
    pub fn new(component: Box<dyn ContainerComponentManipulator>) -> Self {
        Self {
            conditions: Vec::new(),
            entries: Vec::new(),
            component: Some(component),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents$Builder#getThis
    fn get_this(&mut self) -> &mut Self {
        self
    }

    pub fn when(mut self, condition: Box<dyn LootItemCondition>) -> Self {
        self.conditions.push(condition);
        self.get_this();
        self
    }

    pub fn get_conditions(&self) -> &[Box<dyn LootItemCondition>] {
        &self.conditions
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents$Builder#withEntry
    pub fn with_entry(mut self, entry: Box<dyn LootPoolEntryContainer>) -> Self {
        self.entries.push(entry);
        self.get_this();
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetContainerContents$Builder#build
    pub fn build(mut self) -> Box<dyn LootItemFunction> {
        let component = self
            .component
            .take()
            .expect("SetContainerContents.Builder component missing");
        Box::new(SetContainerContents::new(
            self.conditions,
            component,
            self.entries,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug)]
    struct MockCondition {
        valid: bool,
    }

    impl LootItemCondition for MockCondition {
        fn test(&self, _ctx: &LootContext) -> bool {
            true
        }

        fn validate(&self, ctx: &ValidationContext) {
            if !self.valid {
                ctx.report("invalid condition");
            }
        }
    }

    #[derive(Debug, Clone)]
    struct MockEntry {
        items: Vec<ItemStack>,
        validation_msg: Option<String>,
    }

    impl LootPoolEntry for MockEntry {
        fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), _ctx: &LootContext) {
            for item in &self.items {
                output(item.clone());
            }
        }
    }

    impl LootPoolEntryContainer for MockEntry {
        fn expand(&self, _ctx: &LootContext, output: &mut dyn FnMut(&dyn LootPoolEntry)) {
            output(self);
        }

        fn validate(&self, ctx: &ValidationContext) {
            if let Some(msg) = &self.validation_msg {
                ctx.report(msg.clone());
            }
        }
    }

    #[derive(Debug, Default)]
    struct MockComponentState {
        seen_stack_id: String,
        seen_contents: Vec<ItemStack>,
        calls: i32,
    }

    #[derive(Debug, Clone)]
    struct MockComponent {
        state: Rc<RefCell<MockComponentState>>,
    }

    impl MockComponent {
        fn new() -> Self {
            Self {
                state: Rc::new(RefCell::new(MockComponentState::default())),
            }
        }
    }

    impl ContainerComponentManipulator for MockComponent {
        fn set_contents(&self, stack: &mut ItemStack, contents: Vec<ItemStack>) {
            let mut st = self.state.borrow_mut();
            st.seen_stack_id = stack.id.clone();
            st.seen_contents = contents;
            st.calls = st.calls.wrapping_add(1);
            stack.count = stack.count.wrapping_add(0); // preserve value, explicit Java-like int op
        }
    }

    fn ctx() -> LootContext {
        LootContext::default()
    }

    #[test]
    fn empty_stack_returns_unchanged_and_skips_component() {
        let comp = MockComponent::new();
        let comp_state = comp.state.clone();
        let func = SetContainerContents::set_contents(Box::new(comp))
            .with_entry(Box::new(MockEntry {
                items: vec![ItemStack::new("minecraft:stone", 1, 1.0)],
                validation_msg: None,
            }))
            .build();

        let input = ItemStack::new("minecraft:chest", 0, 0.0);
        let out = func.run(input.clone(), &ctx());
        assert_eq!(out.count, 0);
        assert!(out.is_empty());
        assert_eq!(comp_state.borrow().calls, 0);
    }

    #[test]
    fn negative_count_is_empty_edge_case() {
        let comp = MockComponent::new();
        let comp_state = comp.state.clone();
        let func = SetContainerContents::set_contents(Box::new(comp)).build();

        let input = ItemStack::new("minecraft:chest", -1i32, -0.0f32);
        let out = func.run(input, &ctx());
        assert!(out.is_empty());
        assert_eq!(out.count, -1i32);
        assert_eq!(comp_state.borrow().calls, 0);
    }

    #[test]
    fn run_collects_entries_and_sets_contents() {
        let comp = MockComponent::new();
        let comp_state = comp.state.clone();
        let entry1 = MockEntry {
            items: vec![ItemStack::new("minecraft:apple", 2, 1.5)],
            validation_msg: None,
        };
        let entry2 = MockEntry {
            items: vec![
                ItemStack::new("minecraft:bread", 3, 2.0),
                ItemStack::new("minecraft:stick", i32::MAX, 0.0),
            ],
            validation_msg: None,
        };

        let func = SetContainerContents::set_contents(Box::new(comp))
            .with_entry(Box::new(entry1))
            .with_entry(Box::new(entry2))
            .build();

        let input = ItemStack::new("minecraft:chest", 1, 1.0);
        let out = func.run(input, &ctx());
        assert_eq!(out.id, "minecraft:chest");
        let st = comp_state.borrow();
        assert_eq!(st.calls, 1);
        assert_eq!(st.seen_contents.len(), 3);
        assert_eq!(st.seen_contents[0].id, "minecraft:apple");
        assert_eq!(st.seen_contents[1].count, 3);
        assert_eq!(st.seen_contents[2].count, i32::MAX);
    }

    #[test]
    fn float_bits_preserved_through_run() {
        let nan_bits = 0x7FC0_0001u32;
        let neg_zero_bits = (-0.0f32).to_bits();
        let payload_weight = f32::from_bits(nan_bits);

        let comp = MockComponent::new();
        let comp_state = comp.state.clone();
        let entry = MockEntry {
            items: vec![
                ItemStack::new("minecraft:paper", 1, payload_weight),
                ItemStack::new("minecraft:paper", 1, f32::from_bits(neg_zero_bits)),
            ],
            validation_msg: None,
        };

        let func = SetContainerContents::set_contents(Box::new(comp))
            .with_entry(Box::new(entry))
            .build();

        let out = func.run(ItemStack::new("minecraft:chest", 1, 0.0), &ctx());
        assert_eq!(out.id, "minecraft:chest");
        let st = comp_state.borrow();
        assert_eq!(st.seen_contents.len(), 2);
        assert_eq!(st.seen_contents[0].weight.to_bits(), nan_bits);
        assert_eq!(st.seen_contents[1].weight.to_bits(), neg_zero_bits);
        assert_ne!(st.seen_contents[1].weight.to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn validate_reports_indexed_children_and_conditions() {
        let comp = MockComponent::new();
        let cond = MockCondition { valid: false };
        let entry0 = MockEntry {
            items: vec![],
            validation_msg: Some("bad entry".to_string()),
        };
        let entry1 = MockEntry {
            items: vec![],
            validation_msg: None,
        };

        let sc = SetContainerContents::new(
            vec![Box::new(cond)],
            Box::new(comp),
            vec![Box::new(entry0), Box::new(entry1)],
        );

        let reporter = ProblemReporter::default();
        let ctx = ValidationContext::new(reporter.clone());
        sc.validate(&ctx);
        let msgs = reporter.messages();
        assert!(msgs.iter().any(|m| m == "invalid condition"));
        assert!(msgs.iter().any(|m| m == "entries[0]: bad entry"));
        assert!(!msgs.iter().any(|m| m.contains("entries[1]")));
    }

    #[test]
    fn wrapping_index_increment_semantics_for_validate_loop() {
        // Mirrors Java int loop increment semantics: i = i.wrapping_add(1)
        let mut i: i32 = i32::MAX;
        i = i.wrapping_add(1);
        assert_eq!(i, i32::MIN);

        let comp = MockComponent::new();
        let sc = SetContainerContents::new(vec![], Box::new(comp), vec![]);
        let reporter = ProblemReporter::default();
        let vctx = ValidationContext::new(reporter.clone());
        sc.validate(&vctx);
        assert!(reporter.messages().is_empty());
        assert_eq!(sc.get_type(), LootItemFunctions::SET_CONTENTS);
    }
}
