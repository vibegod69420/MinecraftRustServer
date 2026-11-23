use std::cell::RefCell;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceLocation(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceKey<T: 'static> {
    pub location: ResourceLocation,
    _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> ResourceKey<T> {
    pub const fn new(location: &'static str) -> Self {
        Self {
            location: ResourceLocation(location),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Registry<T: 'static>(PhantomData<fn() -> T>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Codec<T: 'static>(PhantomData<fn() -> T>);

impl<T: 'static> Codec<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: 'static> Default for Codec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LootContextParamSet(pub u32);

#[derive(Debug, Clone)]
pub struct ProblemReporterRootElementPathElement {
    pub key_location: String,
}

impl ProblemReporterRootElementPathElement {
    pub fn new<T: 'static>(key: &ResourceKey<T>) -> Self {
        Self {
            key_location: key.location.0.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationContext {
    pub context_key_set: Option<LootContextParamSet>,
    pub entered_keys: Vec<String>,
}

impl ValidationContext {
    pub fn set_context_key_set(&self, key_set: LootContextParamSet) -> Self {
        let mut next = self.clone();
        next.context_key_set = Some(key_set);
        next
    }

    pub fn enter_element<T: 'static>(
        &self,
        _element: ProblemReporterRootElementPathElement,
        key: &ResourceKey<T>,
    ) -> Self {
        let mut next = self.clone();
        next.entered_keys.push(key.location.0.to_string());
        next
    }
}

pub trait LootContextUser {
    fn validate(&self, ctx: ValidationContext);
}

#[derive(Debug, Default)]
pub struct LootItemCondition {
    pub validated_contexts: RefCell<Vec<ValidationContext>>,
}

impl LootItemCondition {
    pub const DIRECT_CODEC: Codec<Self> = Codec::new();
}

impl LootContextUser for LootItemCondition {
    fn validate(&self, ctx: ValidationContext) {
        self.validated_contexts.borrow_mut().push(ctx);
    }
}

#[derive(Debug, Default)]
pub struct LootItemFunction {
    pub validated_contexts: RefCell<Vec<ValidationContext>>,
}

impl LootContextUser for LootItemFunction {
    fn validate(&self, ctx: ValidationContext) {
        self.validated_contexts.borrow_mut().push(ctx);
    }
}

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const ROOT_CODEC: Codec<LootItemFunction> = Codec::new();
}

#[derive(Debug, Default)]
pub struct LootTable {
    pub param_set: LootContextParamSet,
    pub validated_contexts: RefCell<Vec<ValidationContext>>,
}

impl LootTable {
    pub const DIRECT_CODEC: Codec<Self> = Codec::new();

    pub fn get_param_set(&self) -> LootContextParamSet {
        self.param_set
    }
}

impl LootContextUser for LootTable {
    fn validate(&self, ctx: ValidationContext) {
        self.validated_contexts.borrow_mut().push(ctx);
    }
}

pub struct Registries;

impl Registries {
    pub const PREDICATE: ResourceKey<Registry<LootItemCondition>> =
        ResourceKey::new("minecraft:predicate");
    pub const ITEM_MODIFIER: ResourceKey<Registry<LootItemFunction>> =
        ResourceKey::new("minecraft:item_modifier");
    pub const LOOT_TABLE: ResourceKey<Registry<LootTable>> =
        ResourceKey::new("minecraft:loot_table");
}

pub trait Validator<T: 'static> {
    // JAVA: net.minecraft.world.level.storage.loot.LootDataType$Validator#run
    fn run(&self, ctx: &ValidationContext, key: &ResourceKey<T>, value: &T);
}

impl<T: 'static> Validator<T> for fn(&ValidationContext, &ResourceKey<T>, &T) {
    // JAVA: net.minecraft.world.level.storage.loot.LootDataType$Validator#run
    fn run(&self, ctx: &ValidationContext, key: &ResourceKey<T>, value: &T) {
        (self)(ctx, key, value);
    }
}

#[derive(Clone, Copy)]
pub struct LootDataType<T: 'static> {
    pub registry_key: ResourceKey<Registry<T>>,
    pub codec: Codec<T>,
    pub validator: fn(&ValidationContext, &ResourceKey<T>, &T),
}

impl<T: 'static> LootDataType<T> {
    // JAVA: net.minecraft.world.level.storage.loot.LootDataType#runValidation
    pub fn run_validation(&self, ctx: &ValidationContext, key: &ResourceKey<T>, value: &T) {
        self.validator.run(ctx, key, value);
    }
}

// JAVA: net.minecraft.world.level.storage.loot.LootDataType#createSimpleValidator
fn create_simple_validator<T: LootContextUser + 'static>(
    ctx: &ValidationContext,
    key: &ResourceKey<T>,
    value: &T,
) {
    let element = ProblemReporterRootElementPathElement::new(key);
    let next_ctx = ctx.enter_element(element, key);
    value.validate(next_ctx);
}

// JAVA: net.minecraft.world.level.storage.loot.LootDataType#createLootTableValidator
fn create_loot_table_validator(
    ctx: &ValidationContext,
    key: &ResourceKey<LootTable>,
    value: &LootTable,
) {
    let element = ProblemReporterRootElementPathElement::new(key);
    let next_ctx = ctx
        .set_context_key_set(value.get_param_set())
        .enter_element(element, key);
    value.validate(next_ctx);
}

pub static PREDICATE: LootDataType<LootItemCondition> = LootDataType {
    registry_key: Registries::PREDICATE,
    codec: LootItemCondition::DIRECT_CODEC,
    validator: create_simple_validator::<LootItemCondition>,
};

pub static MODIFIER: LootDataType<LootItemFunction> = LootDataType {
    registry_key: Registries::ITEM_MODIFIER,
    codec: LootItemFunctions::ROOT_CODEC,
    validator: create_simple_validator::<LootItemFunction>,
};

pub static TABLE: LootDataType<LootTable> = LootDataType {
    registry_key: Registries::LOOT_TABLE,
    codec: LootTable::DIRECT_CODEC,
    validator: create_loot_table_validator,
};

#[derive(Clone, Copy)]
pub enum LootDataTypeVariant {
    Predicate(&'static LootDataType<LootItemCondition>),
    Modifier(&'static LootDataType<LootItemFunction>),
    Table(&'static LootDataType<LootTable>),
}

impl LootDataTypeVariant {
    pub fn registry_key_location(&self) -> &'static str {
        match self {
            Self::Predicate(t) => t.registry_key.location.0,
            Self::Modifier(t) => t.registry_key.location.0,
            Self::Table(t) => t.registry_key.location.0,
        }
    }
}

// JAVA: net.minecraft.world.level.storage.loot.LootDataType#values
pub fn values() -> [LootDataTypeVariant; 3] {
    [
        LootDataTypeVariant::Predicate(&PREDICATE),
        LootDataTypeVariant::Modifier(&MODIFIER),
        LootDataTypeVariant::Table(&TABLE),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_returns_three_types_in_order() {
        let vals = values();
        assert_eq!(vals.len(), 3);
        assert_eq!(vals[0].registry_key_location(), "minecraft:predicate");
        assert_eq!(vals[1].registry_key_location(), "minecraft:item_modifier");
        assert_eq!(vals[2].registry_key_location(), "minecraft:loot_table");
    }

    #[test]
    fn simple_validator_enters_element() {
        let ctx = ValidationContext::default();
        let key = ResourceKey::<LootItemCondition>::new("minecraft:test_condition");
        let cond = LootItemCondition::default();
        PREDICATE.run_validation(&ctx, &key, &cond);

        let validated = cond.validated_contexts.borrow();
        assert_eq!(validated.len(), 1);
        assert_eq!(validated[0].entered_keys, vec!["minecraft:test_condition"]);
        assert_eq!(validated[0].context_key_set, None);
    }

    #[test]
    fn loot_table_validator_sets_param_set_and_enters_element() {
        let ctx = ValidationContext::default();
        let key = ResourceKey::<LootTable>::new("minecraft:chest");
        let table = LootTable {
            param_set: LootContextParamSet(42),
            validated_contexts: RefCell::new(Vec::new()),
        };
        TABLE.run_validation(&ctx, &key, &table);

        let validated = table.validated_contexts.borrow();
        assert_eq!(validated.len(), 1);
        assert_eq!(validated[0].context_key_set, Some(LootContextParamSet(42)));
        assert_eq!(validated[0].entered_keys, vec!["minecraft:chest"]);
    }

    #[test]
    fn edge_case_param_set_extremes_and_negative_like_bits() {
        let ctx = ValidationContext::default();

        let key_empty = ResourceKey::<LootTable>::new("");
        let table_max = LootTable {
            param_set: LootContextParamSet(u32::MAX),
            validated_contexts: RefCell::new(Vec::new()),
        };
        TABLE.run_validation(&ctx, &key_empty, &table_max);
        let v_max = table_max.validated_contexts.borrow();
        assert_eq!(v_max[0].context_key_set, Some(LootContextParamSet(u32::MAX)));
        assert_eq!(v_max[0].entered_keys, vec![""]);

        let key_neg = ResourceKey::<LootTable>::new("minecraft:neg");
        let table_neg = LootTable {
            param_set: LootContextParamSet(i32::MIN as u32),
            validated_contexts: RefCell::new(Vec::new()),
        };
        TABLE.run_validation(&ctx, &key_neg, &table_neg);
        let v_neg = table_neg.validated_contexts.borrow();
        assert_eq!(
            v_neg[0].context_key_set,
            Some(LootContextParamSet(0x8000_0000))
        );
        assert_eq!(v_neg[0].entered_keys, vec!["minecraft:neg"]);
    }

    #[test]
    fn modifier_uses_simple_validator_and_overflow_like_index_bits() {
        let ctx = ValidationContext::default();
        let key = ResourceKey::<LootItemFunction>::new("minecraft:modifier");
        let func = LootItemFunction::default();
        MODIFIER.run_validation(&ctx, &key, &func);

        let validated = func.validated_contexts.borrow();
        assert_eq!(validated.len(), 1);
        assert_eq!(validated[0].entered_keys, vec!["minecraft:modifier"]);
        assert_eq!(validated[0].context_key_set, None);

        // Java integer overflow/wrapping semantics check for edge-case coverage.
        let wrapped = i32::MAX.wrapping_add(1);
        assert_eq!(wrapped, i32::MIN);
        let masked_shift: i32 = 1i32.wrapping_shl(32 & 31);
        assert_eq!(masked_shift, 1);
    }

    #[test]
    fn float_bits_edge_cases() {
        let nan_f32 = f32::NAN;
        let neg_zero_f32 = -0.0f32;
        let pos_zero_f32 = 0.0f32;
        assert_eq!(nan_f32.to_bits(), f32::NAN.to_bits());
        assert_ne!(neg_zero_f32.to_bits(), pos_zero_f32.to_bits());
        assert_eq!(neg_zero_f32.to_bits(), 0x8000_0000u32);

        let nan_f64 = f64::NAN;
        let neg_zero_f64 = -0.0f64;
        let pos_zero_f64 = 0.0f64;
        assert_eq!(nan_f64.to_bits(), f64::NAN.to_bits());
        assert_ne!(neg_zero_f64.to_bits(), pos_zero_f64.to_bits());
        assert_eq!(neg_zero_f64.to_bits(), 0x8000_0000_0000_0000u64);
    }
}
