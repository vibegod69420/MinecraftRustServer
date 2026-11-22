//! Faithful port of `net.minecraft.world.level.storage.loot.entries.LootPoolEntry`.

use std::fmt;

/// Minimal local stand-in for `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item_id: String,
    pub count: i32,
}

impl ItemStack {
    pub fn new(item_id: impl Into<String>, count: i32) -> Self {
        Self {
            item_id: item_id.into(),
            count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item_id.is_empty()
    }
}

/// Minimal local stand-in for `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, PartialEq)]
pub struct LootContext {
    pub luck: f32,
}

impl LootContext {
    pub fn new(luck: f32) -> Self {
        Self { luck }
    }
}

/// Faithful port of `net.minecraft.world.level.storage.loot.entries.LootPoolEntry`.
///
/// Java interface:
/// - `int getWeight(float var1)`
/// - `void createItemStack(Consumer<ItemStack> var1, LootContext var2)`
pub trait LootPoolEntry: fmt::Debug {
    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntry#getWeight
    fn get_weight(&self, luck: f32) -> i32;

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntry#createItemStack
    fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), context: &LootContext);
}

#[cfg(test)]
mod tests {
    use super::{ItemStack, LootContext, LootPoolEntry};

    #[derive(Debug, Clone)]
    struct FixedWeightEntry {
        weight: i32,
        stack: ItemStack,
    }

    impl LootPoolEntry for FixedWeightEntry {
        fn get_weight(&self, _luck: f32) -> i32 {
            self.weight
        }

        fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), _context: &LootContext) {
            output(self.stack.clone());
        }
    }

    #[derive(Debug)]
    struct LuckCastingEntry;

    impl LootPoolEntry for LuckCastingEntry {
        fn get_weight(&self, luck: f32) -> i32 {
            // Java float-to-int cast semantics: NaN -> 0, saturate to int min/max.
            luck as i32
        }

        fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), context: &LootContext) {
            let count = self.get_weight(context.luck);
            if count > 0 {
                output(ItemStack::new("minecraft:stone", count));
            }
        }
    }

    #[test]
    fn fixed_weight_and_item_creation() {
        let entry = FixedWeightEntry {
            weight: 5,
            stack: ItemStack::new("minecraft:apple", 2),
        };
        let ctx = LootContext::new(0.0);
        assert_eq!(entry.get_weight(0.0), 5);
        assert_eq!(entry.get_weight(-10.0), 5);

        let mut out = Vec::new();
        entry.create_item_stack(&mut |s| out.push(s), &ctx);
        assert_eq!(out, vec![ItemStack::new("minecraft:apple", 2)]);
    }

    #[test]
    fn create_item_stack_uses_context_and_consumer() {
        let entry = LuckCastingEntry;
        let mut out = Vec::new();
        entry.create_item_stack(&mut |s| out.push(s), &LootContext::new(3.75));
        assert_eq!(out, vec![ItemStack::new("minecraft:stone", 3)]);

        let mut out_neg = Vec::new();
        entry.create_item_stack(&mut |s| out_neg.push(s), &LootContext::new(-2.5));
        assert!(out_neg.is_empty());
    }

    #[test]
    fn get_weight_float_bits_edge_cases() {
        let entry = LuckCastingEntry;

        // NaN payloads -> Java cast yields 0.
        assert_eq!(entry.get_weight(f32::NAN), 0);
        assert_eq!(entry.get_weight(f32::from_bits(0x7FC0_0001)), 0);
        assert_eq!(entry.get_weight(f32::from_bits(0xFFC0_0001)), 0);

        // Infinities saturate to int bounds.
        assert_eq!(entry.get_weight(f32::INFINITY), i32::MAX);
        assert_eq!(entry.get_weight(f32::NEG_INFINITY), i32::MIN);

        // Signed zero and subnormals.
        assert_eq!(entry.get_weight(0.0_f32), 0);
        assert_eq!(entry.get_weight(-0.0_f32), 0);
        assert_eq!(entry.get_weight(f32::from_bits(0x0000_0001)), 0);
        assert_eq!(entry.get_weight(f32::from_bits(0x8000_0001)), 0);

        // Exact i32 boundary-relevant float values.
        assert_eq!(entry.get_weight(-2147483648.0_f32), i32::MIN);
        assert_eq!(entry.get_weight(2147483648.0_f32), i32::MAX);
    }

    #[test]
    fn get_weight_negative_and_overflow_inputs() {
        let entry = LuckCastingEntry;
        assert_eq!(entry.get_weight(-1.0), -1);
        assert_eq!(entry.get_weight(-1.9), -1);
        assert_eq!(entry.get_weight(i32::MIN as f32), i32::MIN);
        assert_eq!(entry.get_weight(i32::MAX as f32), i32::MAX);

        // Values outside i32 range saturate like Java cast.
        let huge_neg = -3.4028235e38_f32;
        let huge_pos = 3.4028235e38_f32;
        assert_eq!(entry.get_weight(huge_neg), i32::MIN);
        assert_eq!(entry.get_weight(huge_pos), i32::MAX);
    }

    #[test]
    fn item_stack_empty_semantics() {
        assert!(ItemStack::new("minecraft:stone", 0).is_empty());
        assert!(ItemStack::new("minecraft:stone", -1).is_empty());
        assert!(ItemStack::new("", 1).is_empty());
        assert!(!ItemStack::new("minecraft:stone", 1).is_empty());
    }
}
