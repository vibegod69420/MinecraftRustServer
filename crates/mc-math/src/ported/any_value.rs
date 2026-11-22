use std::any::Any;

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentType`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataComponentType(pub &'static str);

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentGetter`.
pub trait DataComponentGetter {
    fn get(&self, component_type: &DataComponentType) -> Option<&dyn Any>;
}

/// Minimal self-contained stand-in for
/// `net.minecraft.core.component.predicates.DataComponentPredicate`.
pub trait DataComponentPredicate {
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool;
}

/// Port of `net.minecraft.core.component.predicates.AnyValue`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnyValue {
    pub type_: DataComponentType,
}

impl AnyValue {
    pub fn new(type_: DataComponentType) -> Self {
        Self { type_ }
    }
}

impl DataComponentPredicate for AnyValue {
    // JAVA: net.minecraft.core.component.predicates.AnyValue#matches
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        getter.get(&self.type_).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockGetter {
        present: HashMap<&'static str, i32>,
    }

    impl MockGetter {
        fn with(keys: &[&'static str]) -> Self {
            let mut present = HashMap::new();
            for (i, k) in keys.iter().enumerate() {
                present.insert(*k, i as i32);
            }
            Self { present }
        }
    }

    impl DataComponentGetter for MockGetter {
        fn get(&self, component_type: &DataComponentType) -> Option<&dyn Any> {
            self.present.get(component_type.0).map(|v| v as &dyn Any)
        }
    }

    #[test]
    fn matches_when_component_present() {
        let pred = AnyValue::new(DataComponentType("minecraft:damage"));
        let getter = MockGetter::with(&["minecraft:damage"]);
        assert!(pred.matches(&getter));
    }

    #[test]
    fn does_not_match_when_component_missing() {
        let pred = AnyValue::new(DataComponentType("minecraft:damage"));
        let getter = MockGetter::with(&["minecraft:food"]);
        assert!(!pred.matches(&getter));
    }

    #[test]
    fn does_not_match_empty_getter() {
        let pred = AnyValue::new(DataComponentType("minecraft:any"));
        let getter = MockGetter::with(&[]);
        assert!(!pred.matches(&getter));
    }

    #[test]
    fn distinguishes_different_component_types() {
        let getter = MockGetter::with(&["a", "b"]);
        assert!(AnyValue::new(DataComponentType("a")).matches(&getter));
        assert!(AnyValue::new(DataComponentType("b")).matches(&getter));
        assert!(!AnyValue::new(DataComponentType("c")).matches(&getter));
    }
}
