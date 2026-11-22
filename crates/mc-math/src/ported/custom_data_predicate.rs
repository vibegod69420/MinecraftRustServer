//! Port of `net.minecraft.core.component.predicates.CustomDataPredicate`.

/// Minimal stand-in for `net.minecraft.core.component.DataComponentGetter`.
pub trait DataComponentGetter {}

/// Minimal stand-in for `net.minecraft.advancements.criterion.NbtPredicate`.
pub trait NbtPredicate {
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool;
}

/// Minimal stand-in for `net.minecraft.core.component.predicates.DataComponentPredicate`.
pub trait DataComponentPredicate {
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomDataPredicate<V> {
    pub value: V,
}

impl<V: NbtPredicate> CustomDataPredicate<V> {
    // JAVA: net.minecraft.core.component.predicates.CustomDataPredicate#customData
    pub fn custom_data(value: V) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.core.component.predicates.CustomDataPredicate#value
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<V: NbtPredicate> DataComponentPredicate for CustomDataPredicate<V> {
    // JAVA: net.minecraft.core.component.predicates.CustomDataPredicate#matches
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        self.value.matches(getter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockGetter(i32);
    impl DataComponentGetter for MockGetter {}

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockNbt(bool);
    impl NbtPredicate for MockNbt {
        fn matches(&self, _getter: &dyn DataComponentGetter) -> bool {
            self.0
        }
    }

    #[test]
    fn matches_delegates_true() {
        let pred = CustomDataPredicate::custom_data(MockNbt(true));
        let getter = MockGetter(0);
        assert!(DataComponentPredicate::matches(&pred, &getter));
    }

    #[test]
    fn matches_delegates_false() {
        let pred = CustomDataPredicate::custom_data(MockNbt(false));
        let getter = MockGetter(-1);
        assert!(!DataComponentPredicate::matches(&pred, &getter));
    }

    #[test]
    fn value_accessor_returns_inner_predicate() {
        let pred = CustomDataPredicate::custom_data(MockNbt(true));
        assert_eq!(pred.value(), &MockNbt(true));
    }

    #[test]
    fn custom_data_factory_constructs_predicate() {
        let pred = CustomDataPredicate::custom_data(MockNbt(false));
        assert_eq!(pred.value, MockNbt(false));
    }
}
