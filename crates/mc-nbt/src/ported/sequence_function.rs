use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
}

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    pub path: Vec<String>,
}

impl ValidationContext {
    pub fn for_child(&self, element: ProblemReporterIndexedFieldPathElement) -> Self {
        let mut path = self.path.clone();
        path.push(element.to_string());
        Self { path }
    }
}

pub struct ProblemReporterIndexedFieldPathElement {
    pub field: String,
    pub index: usize,
}

impl ProblemReporterIndexedFieldPathElement {
    pub fn new(field: &str, index: usize) -> Self {
        Self {
            field: field.to_string(),
            index,
        }
    }
}

impl std::fmt::Display for ProblemReporterIndexedFieldPathElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.field, self.index)
    }
}

pub trait LootItemFunction {
    fn apply(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack;
    fn validate(&self, _ctx: &ValidationContext) {}
    fn get_type(&self) -> LootItemFunctionType;
}

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SEQUENCE: LootItemFunctionType = LootItemFunctionType("minecraft:sequence");

    pub fn compose(
        functions: &[Arc<dyn LootItemFunction>],
    ) -> Box<dyn Fn(ItemStack, &LootContext) -> ItemStack> {
        let fns = functions.to_vec();
        Box::new(move |mut stack, ctx| {
            for f in &fns {
                stack = f.apply(stack, ctx);
            }
            stack
        })
    }
}

pub struct SequenceFunction {
    pub functions: Vec<Arc<dyn LootItemFunction>>,
    composite_function: Box<dyn Fn(ItemStack, &LootContext) -> ItemStack>,
}

impl SequenceFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SequenceFunction#<init>
    fn new(functions: Vec<Arc<dyn LootItemFunction>>) -> Self {
        let composite_function = LootItemFunctions::compose(&functions);
        Self {
            functions,
            composite_function,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SequenceFunction#of
    pub fn of(functions: Vec<Arc<dyn LootItemFunction>>) -> Self {
        Self::new(functions)
    }
}

impl LootItemFunction for SequenceFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SequenceFunction#apply
    fn apply(&self, stack: ItemStack, ctx: &LootContext) -> ItemStack {
        (self.composite_function)(stack, ctx)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SequenceFunction#validate
    fn validate(&self, ctx: &ValidationContext) {
        for i in 0..self.functions.len() {
            self.functions[i].validate(
                &ctx.for_child(ProblemReporterIndexedFieldPathElement::new("functions", i)),
            );
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SequenceFunction#getType
    fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SEQUENCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct AddCount(i32);
    impl LootItemFunction for AddCount {
        fn apply(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
            stack.count = stack.count.wrapping_add(self.0);
            stack
        }
        fn get_type(&self) -> LootItemFunctionType {
            LootItemFunctionType("test:add")
        }
    }

    struct MulAdd(i32, i32);
    impl LootItemFunction for MulAdd {
        fn apply(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
            stack.count = stack.count.wrapping_mul(self.0).wrapping_add(self.1);
            stack
        }
        fn get_type(&self) -> LootItemFunctionType {
            LootItemFunctionType("test:mul_add")
        }
    }

    struct RecordingFn {
        paths: Arc<Mutex<Vec<Vec<String>>>>,
    }
    impl LootItemFunction for RecordingFn {
        fn apply(&self, stack: ItemStack, _ctx: &LootContext) -> ItemStack {
            stack
        }
        fn validate(&self, ctx: &ValidationContext) {
            self.paths.lock().unwrap().push(ctx.path.clone());
        }
        fn get_type(&self) -> LootItemFunctionType {
            LootItemFunctionType("test:rec")
        }
    }

    #[test]
    fn apply_composes_in_order() {
        let seq = SequenceFunction::of(vec![Arc::new(MulAdd(2, 1)), Arc::new(MulAdd(2, 3))]);
        let out = seq.apply(
            ItemStack {
                id: "x".into(),
                count: 1,
            },
            &LootContext,
        );
        // (1*2+1)=3, (3*2+3)=9
        assert_eq!(out.count, 9);
    }

    #[test]
    fn empty_sequence_identity() {
        let seq = SequenceFunction::of(vec![]);
        let stack = ItemStack {
            id: "a".into(),
            count: 5,
        };
        let out = seq.apply(stack.clone(), &LootContext);
        assert_eq!(out, stack);
    }

    #[test]
    fn validate_propagates_indexed_paths() {
        let p1 = Arc::new(Mutex::new(vec![]));
        let p2 = Arc::new(Mutex::new(vec![]));
        let seq = SequenceFunction::of(vec![
            Arc::new(RecordingFn { paths: p1.clone() }),
            Arc::new(RecordingFn { paths: p2.clone() }),
        ]);
        seq.validate(&ValidationContext::default());
        assert_eq!(p1.lock().unwrap()[0], vec!["functions[0]"]);
        assert_eq!(p2.lock().unwrap()[0], vec!["functions[1]"]);
    }

    #[test]
    fn wrapping_overflow_in_composed_functions() {
        let seq = SequenceFunction::of(vec![
            Arc::new(AddCount(i32::MAX)),
            Arc::new(AddCount(1)),
        ]);
        let out = seq.apply(
            ItemStack {
                id: "x".into(),
                count: 0,
            },
            &LootContext,
        );
        assert_eq!(out.count, i32::MIN);
    }

    #[test]
    fn negative_count_composition() {
        let seq = SequenceFunction::of(vec![Arc::new(AddCount(-10)), Arc::new(MulAdd(2, -1))]);
        let out = seq.apply(
            ItemStack {
                id: "x".into(),
                count: 3,
            },
            &LootContext,
        );
        // (3 + -10) = -7, (-7 * 2 + -1) = -15
        assert_eq!(out.count, -15);
    }

    #[test]
    fn get_type_is_sequence() {
        let seq = SequenceFunction::of(vec![]);
        assert_eq!(seq.get_type(), LootItemFunctions::SEQUENCE);
    }
}
