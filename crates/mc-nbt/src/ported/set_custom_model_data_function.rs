use std::collections::BTreeSet;

/// Minimal context key for loot parameter tracking.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContextKey(pub String);

/// Minimal loot context for number provider evaluation.
#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub params: std::collections::BTreeMap<String, f32>,
}

/// Minimal number provider abstraction used by this function.
pub trait NumberProvider: std::fmt::Debug {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getFloat
    fn get_float(&self, ctx: &LootContext) -> f32;
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getInt
    fn get_int(&self, ctx: &LootContext) -> i32;
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getReferencedContextParams
    fn get_referenced_context_params(&self) -> Vec<ContextKey>;
}

/// Constant number provider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstantValue(pub f32);

impl ConstantValue {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#<init>
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

impl NumberProvider for ConstantValue {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getFloat
    fn get_float(&self, _ctx: &LootContext) -> f32 {
        self.0
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getInt
    fn get_int(&self, _ctx: &LootContext) -> i32 {
        // Java/Rust f32->i32 cast: saturating, NaN -> 0, trunc toward zero.
        self.0 as i32
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getReferencedContextParams
    fn get_referenced_context_params(&self) -> Vec<ContextKey> {
        Vec::new()
    }
}

/// Minimal list operation used by `ListOperation.StandAlone`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListOperation {
    ReplaceAll,
    Append,
    Insert { offset: usize },
    ReplaceSection { offset: usize, size: Option<usize> },
}

impl ListOperation {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation#ReplaceAll
    pub fn replace_all() -> Self {
        Self::ReplaceAll
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation#Append
    pub fn append() -> Self {
        Self::Append
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation#Insert
    pub fn insert(offset: usize) -> Self {
        Self::Insert { offset }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation#ReplaceSection
    pub fn replace_section(offset: usize, size: Option<usize>) -> Self {
        Self::ReplaceSection { offset, size }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation#apply
    pub fn apply<T: Clone>(&self, base: &[T], values: &[T]) -> Vec<T> {
        match *self {
            Self::ReplaceAll => values.to_vec(),
            Self::Append => {
                let mut out = base.to_vec();
                out.extend_from_slice(values);
                out
            }
            Self::Insert { offset } => {
                let idx = offset.min(base.len());
                let mut out = Vec::with_capacity(base.len().saturating_add(values.len()));
                out.extend_from_slice(&base[..idx]);
                out.extend_from_slice(values);
                out.extend_from_slice(&base[idx..]);
                out
            }
            Self::ReplaceSection { offset, size } => {
                let idx = offset.min(base.len());
                let remove_count = size
                    .unwrap_or(values.len())
                    .min(base.len().saturating_sub(idx));
                let mut out = Vec::with_capacity(
                    base.len()
                        .saturating_sub(remove_count)
                        .saturating_add(values.len()),
                );
                out.extend_from_slice(&base[..idx]);
                out.extend_from_slice(values);
                out.extend_from_slice(&base[idx.saturating_add(remove_count)..]);
                out
            }
        }
    }
}

/// Standalone list operation with its own values.
#[derive(Debug, Clone)]
pub struct ListOperationStandAlone<T> {
    operation: ListOperation,
    values: Vec<T>,
}

impl<T> ListOperationStandAlone<T> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation$StandAlone#<init>
    pub fn new(operation: ListOperation, values: Vec<T>) -> Self {
        Self { operation, values }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation$StandAlone#value
    pub fn value(&self) -> &Vec<T> {
        &self.values
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation$StandAlone#operation
    pub fn operation(&self) -> &ListOperation {
        &self.operation
    }
}

impl<T: Clone> ListOperationStandAlone<T> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ListOperation$StandAlone#apply
    pub fn apply(&self, base: &[T]) -> Vec<T> {
        self.operation.apply(base, &self.values)
    }
}

/// Placeholder for loot item conditions in the conditional function base.
#[derive(Debug, Clone, Default)]
pub struct LootItemCondition;

/// Minimal custom model data component.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomModelData {
    pub floats: Vec<f32>,
    pub flags: Vec<bool>,
    pub strings: Vec<String>,
    pub colors: Vec<i32>,
}

impl CustomModelData {
    pub const EMPTY: Self = Self {
        floats: Vec::new(),
        flags: Vec::new(),
        strings: Vec::new(),
        colors: Vec::new(),
    };

    // JAVA: net.minecraft.world.item.component.CustomModelData#<init>
    pub fn new(floats: Vec<f32>, flags: Vec<bool>, strings: Vec<String>, colors: Vec<i32>) -> Self {
        Self {
            floats,
            flags,
            strings,
            colors,
        }
    }
}

/// Minimal item stack supporting the custom model data component.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemStack {
    custom_model_data: Option<CustomModelData>,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#<init>
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.world.item.ItemStack#getOrDefault
    pub fn get_or_default_custom_model_data(&self) -> CustomModelData {
        self.custom_model_data
            .clone()
            .unwrap_or(CustomModelData::EMPTY)
    }

    // JAVA: net.minecraft.world.item.ItemStack#set
    pub fn set_custom_model_data(&mut self, data: CustomModelData) {
        self.custom_model_data = Some(data);
    }
}

/// Minimal function type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub &'static str);

/// Port of `SetCustomModelDataFunction`.
pub struct SetCustomModelDataFunction {
    pub conditions: Vec<LootItemCondition>,
    pub floats: Option<ListOperationStandAlone<Box<dyn NumberProvider>>>,
    pub flags: Option<ListOperationStandAlone<bool>>,
    pub strings: Option<ListOperationStandAlone<String>>,
    pub colors: Option<ListOperationStandAlone<Box<dyn NumberProvider>>>,
}

impl std::fmt::Debug for SetCustomModelDataFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetCustomModelDataFunction")
            .field("conditions", &self.conditions)
            .field("floats", &self.floats)
            .field("flags", &self.flags)
            .field("strings", &self.strings)
            .field("colors", &self.colors)
            .finish()
    }
}

impl SetCustomModelDataFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomModelDataFunction#<init>
    pub fn new(
        conditions: Vec<LootItemCondition>,
        floats: Option<ListOperationStandAlone<Box<dyn NumberProvider>>>,
        flags: Option<ListOperationStandAlone<bool>>,
        strings: Option<ListOperationStandAlone<String>>,
        colors: Option<ListOperationStandAlone<Box<dyn NumberProvider>>>,
    ) -> Self {
        Self {
            conditions,
            floats,
            flags,
            strings,
            colors,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomModelDataFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> Vec<ContextKey> {
        let mut set = BTreeSet::new();
        if let Some(ref sa) = self.floats {
            for p in sa.value() {
                for k in p.get_referenced_context_params() {
                    set.insert(k);
                }
            }
        }
        if let Some(ref sa) = self.colors {
            for p in sa.value() {
                for k in p.get_referenced_context_params() {
                    set.insert(k);
                }
            }
        }
        set.into_iter().collect()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomModelDataFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType("set_custom_model_data")
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomModelDataFunction#apply
    fn apply_direct<T: Clone>(op: &Option<ListOperationStandAlone<T>>, base: &[T]) -> Vec<T> {
        match op {
            Some(sa) => sa.apply(base),
            None => base.to_vec(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomModelDataFunction#apply
    fn apply_mapped<E: Clone, F: Fn(&dyn NumberProvider) -> E>(
        op: &Option<ListOperationStandAlone<Box<dyn NumberProvider>>>,
        base: &[E],
        f: F,
    ) -> Vec<E> {
        match op {
            Some(sa) => {
                let mapped: Vec<E> = sa.value().iter().map(|v| f(v.as_ref())).collect();
                sa.operation().apply(base, &mapped)
            }
            None => base.to_vec(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetCustomModelDataFunction#run
    pub fn run<'a>(&self, stack: &'a mut ItemStack, ctx: &LootContext) -> &'a mut ItemStack {
        let current = stack.get_or_default_custom_model_data();
        let new_data = CustomModelData::new(
            Self::apply_mapped(&self.floats, &current.floats, |p: &dyn NumberProvider| {
                p.get_float(ctx)
            }),
            Self::apply_direct(&self.flags, &current.flags),
            Self::apply_direct(&self.strings, &current.strings),
            Self::apply_mapped(&self.colors, &current.colors, |p: &dyn NumberProvider| {
                p.get_int(ctx)
            }),
        );
        stack.set_custom_model_data(new_data);
        stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct KeyedNumberProvider {
        key: &'static str,
        value: f32,
    }

    impl NumberProvider for KeyedNumberProvider {
        fn get_float(&self, _ctx: &LootContext) -> f32 {
            self.value
        }

        fn get_int(&self, _ctx: &LootContext) -> i32 {
            self.value as i32
        }

        fn get_referenced_context_params(&self) -> Vec<ContextKey> {
            vec![ContextKey(self.key.to_string())]
        }
    }

    #[test]
    fn run_replaces_all_components() {
        let mut stack = ItemStack::new();
        stack.set_custom_model_data(CustomModelData::new(
            vec![1.0],
            vec![true],
            vec!["old".into()],
            vec![10],
        ));
        let func = SetCustomModelDataFunction::new(
            vec![],
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec![Box::new(ConstantValue(3.5)) as Box<dyn NumberProvider>],
            )),
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec![false, true],
            )),
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec!["new".into()],
            )),
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec![Box::new(ConstantValue(-7.9)) as Box<dyn NumberProvider>],
            )),
        );
        let ctx = LootContext::default();
        func.run(&mut stack, &ctx);
        let data = stack.get_or_default_custom_model_data();
        assert_eq!(data.floats, vec![3.5f32]);
        assert_eq!(data.flags, vec![false, true]);
        assert_eq!(data.strings, vec!["new".to_string()]);
        assert_eq!(data.colors, vec![-7i32]); // trunc toward zero
    }

    #[test]
    fn run_none_preserves_existing_data() {
        let mut stack = ItemStack::new();
        stack.set_custom_model_data(CustomModelData::new(
            vec![1.0],
            vec![false],
            vec!["keep".into()],
            vec![42],
        ));
        let func = SetCustomModelDataFunction::new(vec![], None, None, None, None);
        func.run(&mut stack, &LootContext::default());
        let data = stack.get_or_default_custom_model_data();
        assert_eq!(data.floats, vec![1.0f32]);
        assert_eq!(data.flags, vec![false]);
        assert_eq!(data.strings, vec!["keep".to_string()]);
        assert_eq!(data.colors, vec![42i32]);
    }

    #[test]
    fn referenced_context_params_dedup_floats_and_colors() {
        let func = SetCustomModelDataFunction::new(
            vec![],
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec![
                    Box::new(KeyedNumberProvider { key: "a", value: 1.0 }) as Box<dyn NumberProvider>,
                    Box::new(KeyedNumberProvider { key: "b", value: 2.0 }) as Box<dyn NumberProvider>,
                ],
            )),
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec![true],
            )),
            None,
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec![
                    Box::new(KeyedNumberProvider { key: "b", value: 3.0 }) as Box<dyn NumberProvider>,
                    Box::new(KeyedNumberProvider { key: "c", value: 4.0 }) as Box<dyn NumberProvider>,
                ],
            )),
        );
        let refs = func.get_referenced_context_params();
        assert_eq!(
            refs,
            vec![
                ContextKey("a".into()),
                ContextKey("b".into()),
                ContextKey("c".into()),
            ]
        );
        assert_eq!(func.get_type(), LootItemFunctionType("set_custom_model_data"));
    }

    #[test]
    fn float_bits_nan_neg_zero_and_int_cast_edge_cases() {
        let mut stack = ItemStack::new();
        let func = SetCustomModelDataFunction::new(
            vec![],
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec![
                    Box::new(ConstantValue(f32::NAN)) as Box<dyn NumberProvider>,
                    Box::new(ConstantValue(-0.0f32)) as Box<dyn NumberProvider>,
                    Box::new(ConstantValue(f32::INFINITY)) as Box<dyn NumberProvider>,
                    Box::new(ConstantValue(f32::NEG_INFINITY)) as Box<dyn NumberProvider>,
                ],
            )),
            None,
            None,
            Some(ListOperationStandAlone::new(
                ListOperation::replace_all(),
                vec![
                    Box::new(ConstantValue(f32::NAN)) as Box<dyn NumberProvider>,
                    Box::new(ConstantValue(1e30f32)) as Box<dyn NumberProvider>,
                    Box::new(ConstantValue(-1e30f32)) as Box<dyn NumberProvider>,
                    Box::new(ConstantValue(-1.9f32)) as Box<dyn NumberProvider>,
                    Box::new(ConstantValue(i32::MAX as f32)) as Box<dyn NumberProvider>,
                ],
            )),
        );
        func.run(&mut stack, &LootContext::default());
        let data = stack.get_or_default_custom_model_data();
        assert!(data.floats[0].is_nan());
        assert_eq!(data.floats[1].to_bits(), (-0.0f32).to_bits());
        assert_eq!(data.floats[2], f32::INFINITY);
        assert_eq!(data.floats[3], f32::NEG_INFINITY);

        assert_eq!(data.colors[0], 0); // NaN -> 0
        assert_eq!(data.colors[1], i32::MAX); // saturate
        assert_eq!(data.colors[2], i32::MIN); // saturate
        assert_eq!(data.colors[3], -1); // trunc toward zero
        assert_eq!(data.colors[4], i32::MAX); // large float saturates/casts as in Rust/Java semantics
    }

    #[test]
    fn append_and_insert_operations_handle_edges() {
        let mut stack = ItemStack::new();
        stack.set_custom_model_data(CustomModelData::new(
            vec![1.0],
            vec![true],
            vec!["a".into()],
            vec![1],
        ));
        let func = SetCustomModelDataFunction::new(
            vec![],
            Some(ListOperationStandAlone::new(
                ListOperation::append(),
                vec![Box::new(ConstantValue(2.0)) as Box<dyn NumberProvider>],
            )),
            Some(ListOperationStandAlone::new(
                ListOperation::insert(0),
                vec![false],
            )),
            Some(ListOperationStandAlone::new(
                ListOperation::append(),
                vec!["b".into()],
            )),
            Some(ListOperationStandAlone::new(
                ListOperation::insert(10), // clamp beyond len
                vec![Box::new(ConstantValue(2.0)) as Box<dyn NumberProvider>],
            )),
        );
        func.run(&mut stack, &LootContext::default());
        let data = stack.get_or_default_custom_model_data();
        assert_eq!(data.floats, vec![1.0f32, 2.0f32]);
        assert_eq!(data.flags, vec![false, true]);
        assert_eq!(data.strings, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(data.colors, vec![1i32, 2i32]);
    }
}
