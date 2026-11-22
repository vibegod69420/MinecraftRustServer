/// Placeholder for unported `net.minecraft.world.level.storage.loot.functions.LootItemFunction`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LootItemFunction;

/// Placeholder for unported
/// `net.minecraft.world.level.storage.loot.functions.LootItemFunction.Builder`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LootItemFunctionBuilder;

impl LootItemFunctionBuilder {
    // JAVA: net.minecraft.world.level.storage.loot.functions.LootItemFunction$Builder#build
    pub fn build(self) -> LootItemFunction {
        LootItemFunction
    }
}

/// Port of `net.minecraft.world.level.storage.loot.functions.FunctionUserBuilder<T>`.
pub trait FunctionUserBuilder: Sized {
    // JAVA: net.minecraft.world.level.storage.loot.functions.FunctionUserBuilder#apply
    fn apply(self, builder: LootItemFunctionBuilder) -> Self;

    // JAVA: net.minecraft.world.level.storage.loot.functions.FunctionUserBuilder#apply
    fn apply_iterable<E, I, F>(self, items: I, f: F) -> Self
    where
        I: IntoIterator<Item = E>,
        F: Fn(E) -> LootItemFunctionBuilder,
    {
        let mut result = self.unwrap();
        for item in items {
            result = result.apply(f(item));
        }
        result
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FunctionUserBuilder#apply
    fn apply_array<E, F>(self, items: &[E], f: F) -> Self
    where
        F: Fn(&E) -> LootItemFunctionBuilder,
    {
        let mut result = self.unwrap();
        for item in items {
            result = result.apply(f(item));
        }
        result
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.FunctionUserBuilder#unwrap
    fn unwrap(self) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, PartialEq, Eq)]
    struct MockUser {
        count: i32,
    }

    impl FunctionUserBuilder for MockUser {
        fn apply(mut self, _builder: LootItemFunctionBuilder) -> Self {
            self.count = self.count.wrapping_add(1);
            self
        }

        fn unwrap(self) -> Self {
            self
        }
    }

    #[test]
    fn apply_single_increments() {
        let u = MockUser::default().apply(LootItemFunctionBuilder);
        assert_eq!(u.count, 1);
    }

    #[test]
    fn apply_iterable_applies_all_in_order_count() {
        let items = vec![10_i32, -20, 0, i32::MAX];
        let u = MockUser::default().apply_iterable(items, |_e| LootItemFunctionBuilder);
        assert_eq!(u.count, 4);
    }

    #[test]
    fn apply_array_applies_all() {
        let items: [i32; 3] = [1, 2, 3];
        let u = MockUser::default().apply_array(&items, |_e: &i32| LootItemFunctionBuilder);
        assert_eq!(u.count, 3);
    }

    #[test]
    fn apply_iterable_empty_no_change() {
        let items: Vec<i32> = Vec::new();
        let u = MockUser { count: 7 }.apply_iterable(items, |_e| LootItemFunctionBuilder);
        assert_eq!(u.count, 7);
    }

    #[test]
    fn unwrap_returns_self() {
        let u = MockUser { count: -5 };
        let u2 = u.unwrap();
        assert_eq!(u2.count, -5);
    }

    #[test]
    fn builder_build_returns_placeholder_function() {
        let f = LootItemFunctionBuilder.build();
        assert_eq!(f, LootItemFunction);
    }
}
