use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey(pub String);

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub struct LootContext {
    random: Box<dyn RandomSource>,
}

impl LootContext {
    pub fn new(random: Box<dyn RandomSource>) -> Self {
        Self { random }
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContext#getRandom
    pub fn get_random(&mut self) -> &mut dyn RandomSource {
        &mut *self.random
    }
}

pub trait NumberProvider {
    fn get_float(&self, ctx: &mut LootContext) -> f32;
    fn get_referenced_context_params(&self) -> Vec<ContextKey>;
    fn clone_box(&self) -> Box<dyn NumberProvider>;
}

impl Clone for Box<dyn NumberProvider> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait LootItemCondition {
    fn test(&self, ctx: &mut LootContext) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute(pub Identifier);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlotGroup {
    Any,
    Mainhand,
    Offhand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
    Saddle,
}

impl EquipmentSlotGroup {
    fn ordinal(self) -> usize {
        match self {
            Self::Any => 0,
            Self::Mainhand => 1,
            Self::Offhand => 2,
            Self::Hand => 3,
            Self::Feet => 4,
            Self::Legs => 5,
            Self::Chest => 6,
            Self::Head => 7,
            Self::Armor => 8,
            Self::Body => 9,
            Self::Saddle => 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifier {
    pub id: Identifier,
    pub amount: f64,
    pub operation: Operation,
}

impl AttributeModifier {
    pub fn new(id: Identifier, amount: f64, operation: Operation) -> Self {
        Self {
            id,
            amount,
            operation,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifierEntry {
    pub attribute: Holder<Attribute>,
    pub modifier: AttributeModifier,
    pub slot: EquipmentSlotGroup,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemAttributeModifiers {
    pub modifiers: Vec<AttributeModifierEntry>,
}

impl ItemAttributeModifiers {
    pub fn empty() -> Self {
        Self {
            modifiers: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.item.component.ItemAttributeModifiers#withModifierAdded
    pub fn with_modifier_added(
        mut self,
        attribute: Holder<Attribute>,
        modifier: AttributeModifier,
        slot: EquipmentSlotGroup,
    ) -> Self {
        self.modifiers.push(AttributeModifierEntry {
            attribute,
            modifier,
            slot,
        });
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub attribute_modifiers: Option<ItemAttributeModifiers>,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#set
    pub fn set_attribute_modifiers(&mut self, value: ItemAttributeModifiers) {
        self.attribute_modifiers = Some(value);
    }

    // JAVA: net.minecraft.world.item.ItemStack#update
    pub fn update_attribute_modifiers<F>(&mut self, default: ItemAttributeModifiers, f: F)
    where
        F: FnOnce(ItemAttributeModifiers) -> ItemAttributeModifiers,
    {
        let current = self.attribute_modifiers.take().unwrap_or(default);
        self.attribute_modifiers = Some(f(current));
    }
}

pub struct Util;

impl Util {
    // JAVA: net.minecraft.util.Util#getRandom
    pub fn get_random<T: Clone>(list: &[T], random: &mut dyn RandomSource) -> T {
        let idx = random.next_int(list.len() as i32) as usize;
        list[idx].clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemFunctionType(pub u32);

pub struct LootItemFunctions;

impl LootItemFunctions {
    pub const SET_ATTRIBUTES: LootItemFunctionType = LootItemFunctionType(1);
}

// JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$Modifier
#[derive(Clone)]
pub struct Modifier {
    pub id: Identifier,
    pub attribute: Holder<Attribute>,
    pub operation: Operation,
    pub amount: Box<dyn NumberProvider>,
    pub slots: Vec<EquipmentSlotGroup>,
}

// JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction
pub struct SetAttributesFunction {
    pub predicates: Vec<Box<dyn LootItemCondition>>,
    pub modifiers: Vec<Modifier>,
    pub replace: bool,
}

impl SetAttributesFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction#<init>
    pub fn new(
        predicates: Vec<Box<dyn LootItemCondition>>,
        modifiers: Vec<Modifier>,
        replace: bool,
    ) -> Self {
        Self {
            predicates,
            modifiers,
            replace,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctions::SET_ATTRIBUTES
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        self.modifiers
            .iter()
            .flat_map(|m| m.amount.get_referenced_context_params())
            .collect()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction#run
    pub fn run(&self, stack: &mut ItemStack, ctx: &mut LootContext) {
        if self.replace {
            let updated = self.update_modifiers(ctx, ItemAttributeModifiers::empty());
            stack.set_attribute_modifiers(updated);
        } else {
            let current = stack
                .attribute_modifiers
                .take()
                .unwrap_or_else(ItemAttributeModifiers::empty);
            let updated = self.update_modifiers(ctx, current);
            stack.set_attribute_modifiers(updated);
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction#updateModifiers
    fn update_modifiers(
        &self,
        ctx: &mut LootContext,
        mut modifiers: ItemAttributeModifiers,
    ) -> ItemAttributeModifiers {
        for m in &self.modifiers {
            let slot = Util::get_random(&m.slots, ctx.get_random());
            let amount = m.amount.get_float(ctx);
            modifiers = modifiers.with_modifier_added(
                m.attribute.clone(),
                AttributeModifier::new(m.id.clone(), amount as f64, m.operation),
                slot,
            );
        }
        modifiers
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction#modifier
    pub fn modifier(
        id: Identifier,
        attribute: Holder<Attribute>,
        operation: Operation,
        amount: Box<dyn NumberProvider>,
    ) -> ModifierBuilder {
        ModifierBuilder::new(id, attribute, operation, amount)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction#setAttributes
    pub fn set_attributes() -> Builder {
        Builder::new_default()
    }
}

// JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$Builder
pub struct Builder {
    pub replace: bool,
    pub modifiers: Vec<Modifier>,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$Builder#<init>
    pub fn new(replace: bool) -> Self {
        Self {
            replace,
            modifiers: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$Builder#<init>
    pub fn new_default() -> Self {
        Self::new(false)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$Builder#getThis
    pub fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$Builder#withModifier
    pub fn with_modifier(mut self, modifier_builder: ModifierBuilder) -> Self {
        self.modifiers.push(modifier_builder.build());
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$Builder#build
    pub fn build(self) -> SetAttributesFunction {
        SetAttributesFunction::new(Vec::new(), self.modifiers, self.replace)
    }
}

// JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$ModifierBuilder
pub struct ModifierBuilder {
    pub id: Identifier,
    pub attribute: Holder<Attribute>,
    pub operation: Operation,
    pub amount: Box<dyn NumberProvider>,
    pub slots: Vec<EquipmentSlotGroup>,
}

impl ModifierBuilder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$ModifierBuilder#<init>
    pub fn new(
        id: Identifier,
        attribute: Holder<Attribute>,
        operation: Operation,
        amount: Box<dyn NumberProvider>,
    ) -> Self {
        Self {
            id,
            attribute,
            operation,
            amount,
            slots: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$ModifierBuilder#forSlot
    pub fn for_slot(mut self, slot: EquipmentSlotGroup) -> Self {
        if !self.slots.contains(&slot) {
            self.slots.push(slot);
        }
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetAttributesFunction$ModifierBuilder#build
    pub fn build(self) -> Modifier {
        let mut slots = self.slots;
        slots.sort_by_key(|s| s.ordinal());
        slots.dedup();
        Modifier {
            id: self.id,
            attribute: self.attribute,
            operation: self.operation,
            amount: self.amount,
            slots,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ConstNumber(f32);
    impl NumberProvider for ConstNumber {
        fn get_float(&self, _ctx: &mut LootContext) -> f32 {
            self.0
        }
        fn get_referenced_context_params(&self) -> Vec<ContextKey> {
            vec![]
        }
        fn clone_box(&self) -> Box<dyn NumberProvider> {
            Box::new(ConstNumber(self.0))
        }
    }

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }
    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }
    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    fn id(ns: &str, path: &str) -> Identifier {
        Identifier::new(ns, path)
    }

    #[test]
    fn run_replace_sets_modifiers() {
        let mut stack = ItemStack {
            attribute_modifiers: Some(ItemAttributeModifiers {
                modifiers: vec![AttributeModifierEntry {
                    attribute: Holder(Attribute(id("m", "old_attr"))),
                    modifier: AttributeModifier::new(id("m", "old"), 1.0, Operation::AddValue),
                    slot: EquipmentSlotGroup::Mainhand,
                }],
            }),
        };
        let func = SetAttributesFunction::new(
            vec![],
            vec![Modifier {
                id: id("m", "new"),
                attribute: Holder(Attribute(id("m", "attr"))),
                operation: Operation::AddValue,
                amount: Box::new(ConstNumber(5.0)),
                slots: vec![EquipmentSlotGroup::Head],
            }],
            true,
        );
        let mut ctx = LootContext::new(Box::new(SeqRandom::new(vec![0])));
        func.run(&mut stack, &mut ctx);
        let mods = &stack.attribute_modifiers.unwrap().modifiers;
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].modifier.id, id("m", "new"));
        assert_eq!(mods[0].modifier.amount.to_bits(), 5.0f64.to_bits());
        assert_eq!(mods[0].slot, EquipmentSlotGroup::Head);
    }

    #[test]
    fn run_append_preserves_existing() {
        let mut stack = ItemStack {
            attribute_modifiers: Some(ItemAttributeModifiers {
                modifiers: vec![AttributeModifierEntry {
                    attribute: Holder(Attribute(id("m", "old_attr"))),
                    modifier: AttributeModifier::new(id("m", "old"), 1.0, Operation::AddValue),
                    slot: EquipmentSlotGroup::Mainhand,
                }],
            }),
        };
        let func = SetAttributesFunction::new(
            vec![],
            vec![Modifier {
                id: id("m", "new"),
                attribute: Holder(Attribute(id("m", "attr"))),
                operation: Operation::AddMultipliedBase,
                amount: Box::new(ConstNumber(2.0)),
                slots: vec![EquipmentSlotGroup::Chest],
            }],
            false,
        );
        let mut ctx = LootContext::new(Box::new(SeqRandom::new(vec![0])));
        func.run(&mut stack, &mut ctx);
        let mods = &stack.attribute_modifiers.unwrap().modifiers;
        assert_eq!(mods.len(), 2);
        assert_eq!(mods[0].modifier.id, id("m", "old"));
        assert_eq!(mods[1].modifier.id, id("m", "new"));
        assert_eq!(mods[1].modifier.operation, Operation::AddMultipliedBase);
    }

    #[test]
    fn update_modifiers_selects_slot_and_handles_negative_nan() {
        let func = SetAttributesFunction::new(
            vec![],
            vec![
                Modifier {
                    id: id("m", "a"),
                    attribute: Holder(Attribute(id("m", "atk"))),
                    operation: Operation::AddValue,
                    amount: Box::new(ConstNumber(-3.5)),
                    slots: vec![EquipmentSlotGroup::Mainhand, EquipmentSlotGroup::Offhand],
                },
                Modifier {
                    id: id("m", "b"),
                    attribute: Holder(Attribute(id("m", "def"))),
                    operation: Operation::AddMultipliedTotal,
                    amount: Box::new(ConstNumber(f32::NAN)),
                    slots: vec![EquipmentSlotGroup::Head],
                },
            ],
            true,
        );
        let mut stack = ItemStack {
            attribute_modifiers: None,
        };
        let mut ctx = LootContext::new(Box::new(SeqRandom::new(vec![1, 0])));
        func.run(&mut stack, &mut ctx);
        let mods = &stack.attribute_modifiers.unwrap().modifiers;
        assert_eq!(mods.len(), 2);
        assert_eq!(mods[0].slot, EquipmentSlotGroup::Offhand);
        assert_eq!(mods[0].modifier.amount.to_bits(), (-3.5f32 as f64).to_bits());
        assert!(mods[1].modifier.amount.is_nan());
        assert_eq!(mods[1].slot, EquipmentSlotGroup::Head);
    }

    #[test]
    fn float_bits_preserved_through_widening() {
        let neg_zero = f32::from_bits(0x8000_0000);
        let func = SetAttributesFunction::new(
            vec![],
            vec![Modifier {
                id: id("m", "z"),
                attribute: Holder(Attribute(id("m", "a"))),
                operation: Operation::AddValue,
                amount: Box::new(ConstNumber(neg_zero)),
                slots: vec![EquipmentSlotGroup::Any],
            }],
            true,
        );
        let mut stack = ItemStack {
            attribute_modifiers: None,
        };
        let mut ctx = LootContext::new(Box::new(SeqRandom::new(vec![0])));
        func.run(&mut stack, &mut ctx);
        let amount = stack.attribute_modifiers.unwrap().modifiers[0].modifier.amount;
        assert_eq!(amount.to_bits(), (-0.0f64).to_bits());
        assert!(amount.is_sign_negative());
    }

    #[test]
    fn referenced_context_params_and_builder() {
        struct RefNumber(Vec<ContextKey>);
        impl NumberProvider for RefNumber {
            fn get_float(&self, _: &mut LootContext) -> f32 {
                0.0
            }
            fn get_referenced_context_params(&self) -> Vec<ContextKey> {
                self.0.clone()
            }
            fn clone_box(&self) -> Box<dyn NumberProvider> {
                Box::new(RefNumber(self.0.clone()))
            }
        }

        let func = SetAttributesFunction::set_attributes()
            .with_modifier(
                SetAttributesFunction::modifier(
                    id("m", "id"),
                    Holder(Attribute(id("m", "a"))),
                    Operation::AddValue,
                    Box::new(RefNumber(vec![
                        ContextKey("foo".into()),
                        ContextKey("bar".into()),
                    ])),
                )
                .for_slot(EquipmentSlotGroup::Mainhand),
            )
            .build();

        let params = func.get_referenced_context_params();
        assert_eq!(params.len(), 2);
        assert!(params.contains(&ContextKey("foo".into())));
        assert!(params.contains(&ContextKey("bar".into())));
        assert_eq!(func.get_type(), LootItemFunctions::SET_ATTRIBUTES);
        assert!(!func.replace);
    }

    #[test]
    fn modifier_builder_deduplicates_and_orders_slots() {
        let m = ModifierBuilder::new(
            id("m", "i"),
            Holder(Attribute(id("m", "a"))),
            Operation::AddValue,
            Box::new(ConstNumber(1.0)),
        )
        .for_slot(EquipmentSlotGroup::Head)
        .for_slot(EquipmentSlotGroup::Mainhand)
        .for_slot(EquipmentSlotGroup::Head)
        .build();
        assert_eq!(
            m.slots,
            vec![EquipmentSlotGroup::Mainhand, EquipmentSlotGroup::Head]
        );
    }
}
