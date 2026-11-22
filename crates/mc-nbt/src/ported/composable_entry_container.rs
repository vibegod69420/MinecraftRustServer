use std::fmt;

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.LootContext`.
#[derive(Debug, Clone, Default)]
pub struct LootContext {
    pub luck: f32,
}

/// Minimal stub for unported `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemStack {
    pub count: i32,
}

/// Minimal stub for unported `net.minecraft.world.level.storage.loot.entries.LootPoolEntry`.
pub trait LootPoolEntry {
    fn get_weight(&self, luck: f32) -> i32;
    fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), context: &LootContext);
}

type ExpandFn = dyn Fn(&LootContext, &mut dyn FnMut(&dyn LootPoolEntry)) -> bool;

/// Port of `net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer`.
pub struct ComposableEntryContainer {
    expand_fn: Box<ExpandFn>,
}

impl fmt::Debug for ComposableEntryContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComposableEntryContainer(..)")
    }
}

impl ComposableEntryContainer {
    /// Creates a container from a Rust closure. Not a Java method; helper for self-contained porting.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&LootContext, &mut dyn FnMut(&dyn LootPoolEntry)) -> bool + 'static,
    {
        Self {
            expand_fn: Box::new(f),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#ALWAYS_FALSE
    pub fn always_false() -> Self {
        Self::new(|_context, _consumer| false)
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#ALWAYS_TRUE
    pub fn always_true() -> Self {
        Self::new(|_context, _consumer| true)
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#expand
    pub fn expand(
        &self,
        context: &LootContext,
        consumer: &mut dyn FnMut(&dyn LootPoolEntry),
    ) -> bool {
        (self.expand_fn)(context, consumer)
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#and
    pub fn and(self, other: ComposableEntryContainer) -> ComposableEntryContainer {
        // Java: Objects.requireNonNull(other) - non-null by Rust type.
        ComposableEntryContainer::new(move |context, consumer| {
            self.expand(context, consumer) && other.expand(context, consumer)
        })
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#or
    pub fn or(self, other: ComposableEntryContainer) -> ComposableEntryContainer {
        // Java: Objects.requireNonNull(other) - non-null by Rust type.
        ComposableEntryContainer::new(move |context, consumer| {
            self.expand(context, consumer) || other.expand(context, consumer)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    struct DummyEntry(i32);

    impl LootPoolEntry for DummyEntry {
        fn get_weight(&self, _luck: f32) -> i32 {
            self.0
        }

        fn create_item_stack(&self, output: &mut dyn FnMut(ItemStack), _context: &LootContext) {
            output(ItemStack { count: self.0 });
        }
    }

    #[test]
    fn always_false_and_true() {
        let ctx = LootContext::default();
        let mut called = 0i32;
        let mut consumer = |_entry: &dyn LootPoolEntry| {
            called = called.wrapping_add(1);
        };

        assert!(!ComposableEntryContainer::always_false().expand(&ctx, &mut consumer));
        assert!(ComposableEntryContainer::always_true().expand(&ctx, &mut consumer));
        assert_eq!(called, 0);
    }

    #[test]
    fn expand_can_emit_entries() {
        let entry = DummyEntry(7);
        let container = ComposableEntryContainer::new(move |_ctx, consumer| {
            consumer(&entry);
            true
        });
        let ctx = LootContext::default();
        let mut weights = Vec::new();
        let mut consumer = |e: &dyn LootPoolEntry| weights.push(e.get_weight(0.0));
        assert!(container.expand(&ctx, &mut consumer));
        assert_eq!(weights, vec![7]);
    }

    #[test]
    fn and_short_circuits_on_false() {
        let right_called = Rc::new(Cell::new(false));
        let right_called_clone = Rc::clone(&right_called);
        let right = ComposableEntryContainer::new(move |_ctx, _consumer| {
            right_called_clone.set(true);
            true
        });

        let combined = ComposableEntryContainer::always_false().and(right);
        let ctx = LootContext::default();
        let mut consumer = |_e: &dyn LootPoolEntry| {};
        assert!(!combined.expand(&ctx, &mut consumer));
        assert!(!right_called.get());
    }

    #[test]
    fn or_short_circuits_on_true() {
        let right_called = Rc::new(Cell::new(false));
        let right_called_clone = Rc::clone(&right_called);
        let right = ComposableEntryContainer::new(move |_ctx, _consumer| {
            right_called_clone.set(true);
            false
        });

        let combined = ComposableEntryContainer::always_true().or(right);
        let ctx = LootContext::default();
        let mut consumer = |_e: &dyn LootPoolEntry| {};
        assert!(combined.expand(&ctx, &mut consumer));
        assert!(!right_called.get());
    }

    #[test]
    fn and_or_with_negative_and_float_edge_context() {
        let ctx_neg = LootContext { luck: -0.0f32 };
        assert_eq!(ctx_neg.luck.to_bits(), 0x8000_0000);
        let ctx_nan = LootContext { luck: f32::NAN };
        assert!(ctx_nan.luck.is_nan());

        let left = ComposableEntryContainer::new(|ctx, _c| ctx.luck.to_bits() == 0x8000_0000);
        let right = ComposableEntryContainer::new(|ctx, _c| ctx.luck.is_nan());
        let combined = left.or(right);

        let mut consumer = |_e: &dyn LootPoolEntry| {};
        assert!(combined.expand(&ctx_neg, &mut consumer));
        assert!(combined.expand(&ctx_nan, &mut consumer));
    }
}
