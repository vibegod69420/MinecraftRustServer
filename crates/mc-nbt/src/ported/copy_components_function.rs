use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// Minimal context key used by loot context lookups.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey {
    pub name: String,
}

impl ContextKey {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

/// Typed data component pair (`DataComponentType<T>` + value) used by the copy function.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedDataComponent<C, V> {
    pub type_id: C,
    pub value: V,
}

impl<C, V> TypedDataComponent<C, V> {
    pub fn new(type_id: C, value: V) -> Self {
        Self { type_id, value }
    }
}

/// Minimal source interface corresponding to `DataComponentGetter`.
pub trait DataComponentSource<C: Clone + Eq + Hash, V: Clone> {
    fn as_component_map(&self) -> Option<&dyn DataComponentMap<C, V>>;
    fn get_typed(&self, type_id: &C) -> Option<TypedDataComponent<C, V>>;
}

/// Minimal component map interface corresponding to `DataComponentMap`.
pub trait DataComponentMap<C: Clone + Eq + Hash, V: Clone>: DataComponentSource<C, V> {
    fn filter_entries(&self, predicate: &dyn Fn(&C) -> bool) -> Vec<TypedDataComponent<C, V>>;
}

/// Minimal item stack interface used by `CopyComponentsFunction#run`.
pub trait ItemStackLike<C: Clone + Eq + Hash, V: Clone> {
    fn apply_components(&mut self, components: &[TypedDataComponent<C, V>]);
    fn set_component(&mut self, component: TypedDataComponent<C, V>);
}

/// Minimal loot context interface providing component sources and the global component type set.
pub trait LootContext<C: Clone + Eq + Hash, V: Clone> {
    fn get_component_source(&self, key: &ContextKey) -> Option<&dyn DataComponentSource<C, V>>;
    fn all_data_component_types(&self) -> Vec<C>;
}

/// Minimal loot condition interface for `LootItemConditionalFunction`.
pub trait LootItemCondition<C: Clone + Eq + Hash, V: Clone> {
    fn test(&self, ctx: &dyn LootContext<C, V>) -> bool;
}

/// Minimal function type marker corresponding to `LootItemFunctionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LootItemFunctionType(pub &'static str);

/// Minimal function type registry constants used by this port.
pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const COPY_COMPONENTS: LootItemFunctionType = LootItemFunctionType("copy_components");
}

/// Direct source record: resolves the context parameter itself as a component source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectSource {
    context_param: ContextKey,
}

impl DirectSource {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$DirectSource#<init>
    pub fn new(context_param: ContextKey) -> Self {
        Self { context_param }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$DirectSource#contextParam
    pub fn context_param(&self) -> &ContextKey {
        &self.context_param
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$DirectSource#get
    pub fn resolve<'a, C: Clone + Eq + Hash, V: Clone>(
        &self,
        ctx: &'a dyn LootContext<C, V>,
    ) -> Option<&'a dyn DataComponentSource<C, V>> {
        ctx.get_component_source(&self.context_param)
    }
}

/// Block entity source record: resolves the context parameter's collected components.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockEntitySource {
    context_param: ContextKey,
}

impl BlockEntitySource {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$BlockEntitySource#<init>
    pub fn new(context_param: ContextKey) -> Self {
        Self { context_param }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$BlockEntitySource#contextParam
    pub fn context_param(&self) -> &ContextKey {
        &self.context_param
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$BlockEntitySource#get
    pub fn resolve<'a, C: Clone + Eq + Hash, V: Clone>(
        &self,
        ctx: &'a dyn LootContext<C, V>,
    ) -> Option<&'a dyn DataComponentSource<C, V>> {
        ctx.get_component_source(&self.context_param)
    }
}

/// Minimal `LootContextArg<DataComponentGetter>` representation for this function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LootContextArg {
    Direct(DirectSource),
    BlockEntity(BlockEntitySource),
}

impl LootContextArg {
    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg#contextParam
    pub fn context_param(&self) -> &ContextKey {
        match self {
            Self::Direct(s) => s.context_param(),
            Self::BlockEntity(s) => s.context_param(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextArg#get
    pub fn get<'a, C: Clone + Eq + Hash, V: Clone>(
        &self,
        ctx: &'a dyn LootContext<C, V>,
    ) -> Option<&'a dyn DataComponentSource<C, V>> {
        match self {
            Self::Direct(s) => s.resolve(ctx),
            Self::BlockEntity(s) => s.resolve(ctx),
        }
    }
}

/// Faithful port of `CopyComponentsFunction`.
pub struct CopyComponentsFunction<C: Clone + Eq + Hash, V: Clone> {
    conditions: Vec<Box<dyn LootItemCondition<C, V>>>,
    source: LootContextArg,
    include: Option<Vec<C>>,
    exclude: Option<Vec<C>>,
    baked_predicate: Box<dyn Fn(&C) -> bool>,
}

impl<C: Clone + Eq + Hash + Debug + 'static, V: Clone + 'static> CopyComponentsFunction<C, V> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction#<init>
    pub fn new(
        conditions: Vec<Box<dyn LootItemCondition<C, V>>>,
        source: LootContextArg,
        include: Option<Vec<C>>,
        exclude: Option<Vec<C>>,
    ) -> Self {
        let include = include.map(|v| v.to_vec());
        let exclude = exclude.map(|v| v.to_vec());

        let include_for_pred = include.clone();
        let exclude_for_pred = exclude.clone();
        let baked_predicate: Box<dyn Fn(&C) -> bool> = Box::new(move |t: &C| {
            if let Some(ref ex) = exclude_for_pred {
                if ex.contains(t) {
                    return false;
                }
            }
            if let Some(ref inc) = include_for_pred {
                if !inc.contains(t) {
                    return false;
                }
            }
            true
        });

        Self {
            conditions,
            source,
            include,
            exclude,
            baked_predicate,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::COPY_COMPONENTS
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = HashSet::new();
        set.insert(self.source.context_param().clone());
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction#run
    pub fn run<'a, S: ItemStackLike<C, V> + ?Sized>(
        &self,
        stack: &'a mut S,
        ctx: &dyn LootContext<C, V>,
    ) -> &'a mut S {
        if let Some(source) = self.source.get(ctx) {
            if let Some(map) = source.as_component_map() {
                let filtered = map.filter_entries(&*self.baked_predicate);
                stack.apply_components(&filtered);
            } else {
                let exclude_list: &[C] = self.exclude.as_deref().unwrap_or(&[]);
                let types: Vec<C> = match &self.include {
                    Some(inc) => inc.clone(),
                    None => ctx.all_data_component_types(),
                };
                for type_id in &types {
                    if !exclude_list.contains(type_id) {
                        if let Some(typed) = source.get_typed(type_id) {
                            stack.set_component(typed);
                        }
                    }
                }
            }
        }
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction#copyComponentsFromEntity
    pub fn copy_components_from_entity(key: ContextKey) -> Builder<C, V> {
        Builder::new(LootContextArg::Direct(DirectSource::new(key)))
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction#copyComponentsFromBlockEntity
    pub fn copy_components_from_block_entity(key: ContextKey) -> Builder<C, V> {
        Builder::new(LootContextArg::BlockEntity(BlockEntitySource::new(key)))
    }

    pub fn conditions(&self) -> &[Box<dyn LootItemCondition<C, V>>] {
        &self.conditions
    }

    pub fn source(&self) -> &LootContextArg {
        &self.source
    }

    pub fn include(&self) -> Option<&[C]> {
        self.include.as_deref()
    }

    pub fn exclude(&self) -> Option<&[C]> {
        self.exclude.as_deref()
    }
}

/// Builder for `CopyComponentsFunction`.
pub struct Builder<C: Clone + Eq + Hash, V: Clone> {
    conditions: Vec<Box<dyn LootItemCondition<C, V>>>,
    source: LootContextArg,
    include: Option<Vec<C>>,
    exclude: Option<Vec<C>>,
}

impl<C: Clone + Eq + Hash + Debug + 'static, V: Clone + 'static> Builder<C, V> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$Builder#<init>
    pub fn new(source: LootContextArg) -> Self {
        Self {
            conditions: Vec::new(),
            source,
            include: None,
            exclude: None,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$Builder#include
    pub fn include(mut self, component_type: C) -> Self {
        self.include
            .get_or_insert_with(Vec::new)
            .push(component_type);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$Builder#exclude
    pub fn exclude(mut self, component_type: C) -> Self {
        self.exclude
            .get_or_insert_with(Vec::new)
            .push(component_type);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$Builder#getThis
    fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$Builder#getConditions
    pub fn get_conditions(&self) -> &[Box<dyn LootItemCondition<C, V>>] {
        &self.conditions
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction$Builder#build
    pub fn build(self) -> CopyComponentsFunction<C, V> {
        CopyComponentsFunction::new(self.conditions, self.source, self.include, self.exclude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestType(&'static str);

    #[derive(Debug, Clone, PartialEq)]
    enum TestValue {
        Int(i32),
        Long(i64),
        FloatBits(u32),
        DoubleBits(u64),
    }

    struct TestMapSource {
        entries: BTreeMap<&'static str, TestValue>,
    }

    impl TestMapSource {
        fn new(entries: Vec<(&'static str, TestValue)>) -> Self {
            let mut map = BTreeMap::new();
            for (k, v) in entries {
                map.insert(k, v);
            }
            Self { entries: map }
        }
    }

    impl DataComponentSource<TestType, TestValue> for TestMapSource {
        fn as_component_map(&self) -> Option<&dyn DataComponentMap<TestType, TestValue>> {
            Some(self)
        }

        fn get_typed(&self, type_id: &TestType) -> Option<TypedDataComponent<TestType, TestValue>> {
            self.entries
                .get(type_id.0)
                .cloned()
                .map(|value| TypedDataComponent::new(type_id.clone(), value))
        }
    }

    impl DataComponentMap<TestType, TestValue> for TestMapSource {
        fn filter_entries(
            &self,
            predicate: &dyn Fn(&TestType) -> bool,
        ) -> Vec<TypedDataComponent<TestType, TestValue>> {
            self.entries
                .iter()
                .filter(|(k, _)| predicate(&TestType(k)))
                .map(|(k, v)| TypedDataComponent::new(TestType(k), v.clone()))
                .collect()
        }
    }

    struct TestGetterSource {
        entries: BTreeMap<&'static str, TestValue>,
    }

    impl TestGetterSource {
        fn new(entries: Vec<(&'static str, TestValue)>) -> Self {
            let mut map = BTreeMap::new();
            for (k, v) in entries {
                map.insert(k, v);
            }
            Self { entries: map }
        }
    }

    impl DataComponentSource<TestType, TestValue> for TestGetterSource {
        fn as_component_map(&self) -> Option<&dyn DataComponentMap<TestType, TestValue>> {
            None
        }

        fn get_typed(&self, type_id: &TestType) -> Option<TypedDataComponent<TestType, TestValue>> {
            self.entries
                .get(type_id.0)
                .cloned()
                .map(|value| TypedDataComponent::new(type_id.clone(), value))
        }
    }

    #[derive(Default)]
    struct TestStack {
        components: BTreeMap<&'static str, TestValue>,
    }

    impl ItemStackLike<TestType, TestValue> for TestStack {
        fn apply_components(&mut self, components: &[TypedDataComponent<TestType, TestValue>]) {
            for c in components {
                self.components.insert(c.type_id.0, c.value.clone());
            }
        }

        fn set_component(&mut self, component: TypedDataComponent<TestType, TestValue>) {
            self.components.insert(component.type_id.0, component.value);
        }
    }

    struct TestContext {
        source: Option<Box<dyn DataComponentSource<TestType, TestValue>>>,
        all_types: Vec<TestType>,
        key: ContextKey,
    }

    impl TestContext {
        fn new(
            key: ContextKey,
            source: Option<Box<dyn DataComponentSource<TestType, TestValue>>>,
            all_types: Vec<TestType>,
        ) -> Self {
            Self {
                source,
                all_types,
                key,
            }
        }
    }

    impl LootContext<TestType, TestValue> for TestContext {
        fn get_component_source(
            &self,
            key: &ContextKey,
        ) -> Option<&dyn DataComponentSource<TestType, TestValue>> {
            if &self.key == key {
                self.source.as_deref()
            } else {
                None
            }
        }

        fn all_data_component_types(&self) -> Vec<TestType> {
            self.all_types.clone()
        }
    }

    #[test]
    fn run_copies_from_component_map_with_include_exclude() {
        let key = ContextKey::new("src");
        let source = TestMapSource::new(vec![
            ("a", TestValue::Int(1)),
            ("b", TestValue::Int(i32::MIN)),
            ("c", TestValue::Long(i64::MAX)),
        ]);
        let ctx = TestContext::new(key.clone(), Some(Box::new(source)), vec![]);
        let func = CopyComponentsFunction::copy_components_from_block_entity(key)
            .include(TestType("a"))
            .include(TestType("b"))
            .exclude(TestType("b"))
            .build();

        let mut stack = TestStack::default();
        func.run(&mut stack, &ctx);

        assert_eq!(stack.components.get("a"), Some(&TestValue::Int(1)));
        assert!(!stack.components.contains_key("b"));
        assert!(!stack.components.contains_key("c"));
        assert_eq!(
            func.get_referenced_context_params(),
            HashSet::from([ContextKey::new("src")])
        );
        assert_eq!(func.get_type(), LootItemFunctions::COPY_COMPONENTS);
    }

    #[test]
    fn run_copies_from_getter_uses_registry_when_include_absent() {
        let key = ContextKey::new("src");
        let source = TestGetterSource::new(vec![
            ("a", TestValue::Int(-7)),
            ("b", TestValue::Int(2)),
            ("c", TestValue::Int(3)),
        ]);
        let ctx = TestContext::new(
            key.clone(),
            Some(Box::new(source)),
            vec![TestType("a"), TestType("b"), TestType("c"), TestType("d")],
        );
        let func = CopyComponentsFunction::copy_components_from_entity(key)
            .exclude(TestType("b"))
            .build();

        let mut stack = TestStack::default();
        func.run(&mut stack, &ctx);

        assert_eq!(stack.components.get("a"), Some(&TestValue::Int(-7)));
        assert!(!stack.components.contains_key("b"));
        assert_eq!(stack.components.get("c"), Some(&TestValue::Int(3)));
        assert!(!stack.components.contains_key("d"));
    }

    #[test]
    fn run_preserves_float_and_double_bits_including_nan_and_neg_zero() {
        let key = ContextKey::new("src");
        let nan_f32 = f32::from_bits(0x7FC0_0001);
        let neg_zero_f64 = f64::from_bits(0x8000_0000_0000_0000);
        let source = TestMapSource::new(vec![
            ("f", TestValue::FloatBits(nan_f32.to_bits())),
            ("d", TestValue::DoubleBits(neg_zero_f64.to_bits())),
        ]);
        let ctx = TestContext::new(key.clone(), Some(Box::new(source)), vec![]);
        let func = CopyComponentsFunction::<TestType, TestValue>::copy_components_from_entity(key)
            .build();

        let mut stack = TestStack::default();
        func.run(&mut stack, &ctx);

        assert_eq!(
            stack.components.get("f"),
            Some(&TestValue::FloatBits(0x7FC0_0001))
        );
        assert_eq!(
            stack.components.get("d"),
            Some(&TestValue::DoubleBits(0x8000_0000_0000_0000))
        );
    }

    #[test]
    fn run_no_source_or_missing_context_param_is_noop() {
        let key = ContextKey::new("src");
        let ctx_missing = TestContext::new(ContextKey::new("other"), None, vec![TestType("a")]);
        let func = CopyComponentsFunction::<TestType, TestValue>::copy_components_from_entity(
            key.clone(),
        )
        .include(TestType("a"))
        .build();

        let mut stack = TestStack::default();
        stack.components.insert("keep", TestValue::Int(42));
        func.run(&mut stack, &ctx_missing);
        assert_eq!(stack.components.get("keep"), Some(&TestValue::Int(42)));
        assert_eq!(stack.components.len(), 1);

        let ctx_none = TestContext::new(key, None, vec![TestType("a")]);
        func.run(&mut stack, &ctx_none);
        assert_eq!(stack.components.get("keep"), Some(&TestValue::Int(42)));
        assert_eq!(stack.components.len(), 1);
    }

    #[test]
    fn getter_branch_respects_include_order_and_negative_overflow_values() {
        let key = ContextKey::new("src");
        let neg_one_i32_as_bits: i32 = -1;
        let overflowed_i32 = i32::MAX.wrapping_add(1); // -2147483648
        let source = TestGetterSource::new(vec![
            ("x", TestValue::Int(overflowed_i32)),
            ("y", TestValue::Long(i64::MIN)),
            ("z", TestValue::Int(neg_one_i32_as_bits)),
        ]);
        let ctx = TestContext::new(key.clone(), Some(Box::new(source)), vec![TestType("x")]);
        let func = CopyComponentsFunction::copy_components_from_block_entity(key)
            .include(TestType("z"))
            .include(TestType("x"))
            .include(TestType("y"))
            .build();

        let mut stack = TestStack::default();
        func.run(&mut stack, &ctx);

        assert_eq!(
            stack.components.get("x"),
            Some(&TestValue::Int(i32::MIN))
        );
        assert_eq!(stack.components.get("y"), Some(&TestValue::Long(i64::MIN)));
        assert_eq!(stack.components.get("z"), Some(&TestValue::Int(-1)));
    }
}
