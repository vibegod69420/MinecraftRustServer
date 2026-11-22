//! Port of `net.minecraft.world.level.storage.loot.functions.LootItemFunction`.

/// Minimal self-contained ItemStack stub for loot function port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
}

impl ItemStack {
    pub fn new(id: impl Into<String>, count: i32) -> Self {
        Self {
            id: id.into(),
            count,
        }
    }
}

/// Minimal self-contained LootContext stub for loot function port.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LootContext;

/// Minimal self-contained LootContextUser marker trait.
pub trait LootContextUser {}

/// Minimal self-contained loot function type token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LootItemFunctionType {
    pub id: u32,
}

/// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction
pub trait LootItemFunction: LootContextUser {
    /// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction#getType
    fn get_type(&self) -> LootItemFunctionType;

    /// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction#apply
    fn apply(&self, stack: ItemStack, context: &LootContext) -> ItemStack;
}

/// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction#decorate
pub fn decorate<'a, F, C>(
    function: F,
    consumer: C,
    context: &'a LootContext,
) -> impl Fn(ItemStack) + 'a
where
    F: Fn(ItemStack, &LootContext) -> ItemStack + 'a,
    C: Fn(ItemStack) + 'a,
{
    move |stack: ItemStack| {
        let result = function(stack, context);
        consumer(result);
    }
}

/// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction$Builder
pub trait LootItemFunctionBuilder {
    type Function: LootItemFunction;

    /// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction$Builder#build
    fn build(self) -> Self::Function;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct AddCountFunction {
        type_token: LootItemFunctionType,
        delta: i32,
    }

    impl AddCountFunction {
        fn new(delta: i32) -> Self {
            Self {
                type_token: LootItemFunctionType { id: 1 },
                delta,
            }
        }
    }

    impl LootContextUser for AddCountFunction {}

    impl LootItemFunction for AddCountFunction {
        fn get_type(&self) -> LootItemFunctionType {
            self.type_token
        }

        fn apply(&self, mut stack: ItemStack, _context: &LootContext) -> ItemStack {
            stack.count = stack.count.wrapping_add(self.delta);
            stack
        }
    }

    struct SimpleBuilder {
        delta: i32,
    }

    impl LootItemFunctionBuilder for SimpleBuilder {
        type Function = AddCountFunction;

        fn build(self) -> Self::Function {
            AddCountFunction::new(self.delta)
        }
    }

    #[test]
    fn decorate_applies_function_then_consumer() {
        let ctx = LootContext;
        let seen = Rc::new(RefCell::new(Vec::new()));
        let seen_clone = Rc::clone(&seen);
        let consumer = decorate(
            |mut stack: ItemStack, _ctx: &LootContext| {
                stack.count = stack.count.wrapping_add(1);
                stack
            },
            move |stack: ItemStack| seen_clone.borrow_mut().push(stack),
            &ctx,
        );
        consumer(ItemStack::new("minecraft:stone", 5));
        let result = seen.borrow();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ItemStack::new("minecraft:stone", 6));
    }

    #[test]
    fn decorate_handles_negative_and_overflow_counts() {
        let ctx = LootContext;
        let seen = Rc::new(RefCell::new(None::<ItemStack>));
        let seen_clone = Rc::clone(&seen);
        let consumer = decorate(
            |mut stack: ItemStack, _ctx: &LootContext| {
                stack.count = stack.count.wrapping_add(1);
                stack
            },
            move |stack: ItemStack| *seen_clone.borrow_mut() = Some(stack),
            &ctx,
        );
        consumer(ItemStack::new("minecraft:stone", i32::MAX));
        assert_eq!(seen.borrow().as_ref().unwrap().count, i32::MIN);

        let seen2 = Rc::new(RefCell::new(None::<ItemStack>));
        let seen2_clone = Rc::clone(&seen2);
        let consumer2 = decorate(
            |mut stack: ItemStack, _ctx: &LootContext| {
                stack.count = stack.count.wrapping_sub(1);
                stack
            },
            move |stack: ItemStack| *seen2_clone.borrow_mut() = Some(stack),
            &ctx,
        );
        consumer2(ItemStack::new("minecraft:stone", i32::MIN));
        assert_eq!(seen2.borrow().as_ref().unwrap().count, i32::MAX);
    }

    #[test]
    fn builder_and_apply_with_negative_inputs() {
        let func = SimpleBuilder { delta: -3 }.build();
        let ctx = LootContext;
        assert_eq!(func.get_type(), LootItemFunctionType { id: 1 });
        let out = func.apply(ItemStack::new("minecraft:dirt", 10), &ctx);
        assert_eq!(out.count, 7);
        let out_neg = func.apply(ItemStack::new("minecraft:dirt", -5), &ctx);
        assert_eq!(out_neg.count, -8_i32);
    }

    #[test]
    fn decorate_with_trait_function_and_float_bits() {
        let ctx = LootContext;
        let func = AddCountFunction::new(10);
        let seen = Rc::new(RefCell::new(Vec::new()));
        let seen_clone = Rc::clone(&seen);
        let consumer = decorate(
            move |stack: ItemStack, c: &LootContext| func.apply(stack, c),
            move |stack: ItemStack| seen_clone.borrow_mut().push(stack.count),
            &ctx,
        );
        consumer(ItemStack::new("minecraft:apple", 0));
        consumer(ItemStack::new("minecraft:apple", -1));
        assert_eq!(&*seen.borrow(), &[10_i32, 9_i32]);

        // Float bits edge cases: exact bit patterns must be preserved.
        let nan_bits: u32 = 0xFFC0_0001;
        assert_eq!(f32::from_bits(nan_bits).to_bits(), nan_bits);
        let neg_zero_bits: u32 = 0x8000_0000;
        let neg_zero = f32::from_bits(neg_zero_bits);
        assert_eq!(neg_zero.to_bits(), neg_zero_bits);
        let nan64_bits: u64 = 0xFFF8_0000_0000_0001;
        assert_eq!(f64::from_bits(nan64_bits).to_bits(), nan64_bits);
    }
}
