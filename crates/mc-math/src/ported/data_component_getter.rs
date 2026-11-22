use std::marker::PhantomData;

/// Minimal local stand-in for `net.minecraft.core.component.DataComponentType<T>`.
#[derive(Debug)]
pub struct DataComponentType<T> {
    id: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Clone for DataComponentType<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for DataComponentType<T> {}

impl<T> PartialEq for DataComponentType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for DataComponentType<T> {}

impl<T> DataComponentType<T> {
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }
}

/// Minimal local stand-in for `net.minecraft.core.component.TypedDataComponent<T>`.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedDataComponent<T> {
    type_: DataComponentType<T>,
    value: T,
}

impl<T> TypedDataComponent<T> {
    pub fn new(type_: DataComponentType<T>, value: T) -> Self {
        Self { type_, value }
    }

    pub fn type_(&self) -> &DataComponentType<T> {
        &self.type_
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

/// Port of `net.minecraft.core.component.DataComponentGetter`.
pub trait DataComponentGetter {
    // JAVA: net.minecraft.core.component.DataComponentGetter#get
    fn get<T: 'static>(&self, component_type: &DataComponentType<T>) -> Option<&T>;

    // JAVA: net.minecraft.core.component.DataComponentGetter#getOrDefault
    fn get_or_default<'a, T: 'static>(
        &'a self,
        component_type: &DataComponentType<T>,
        default_value: &'a T,
    ) -> &'a T {
        match self.get(component_type) {
            Some(value) => value,
            None => default_value,
        }
    }

    // JAVA: net.minecraft.core.component.DataComponentGetter#getTyped
    fn get_typed<T: Clone + 'static>(
        &self,
        component_type: &DataComponentType<T>,
    ) -> Option<TypedDataComponent<T>> {
        match self.get(component_type) {
            Some(value) => Some(TypedDataComponent::new(*component_type, value.clone())),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::collections::HashMap;

    struct MockGetter {
        values: HashMap<u64, Box<dyn Any>>,
    }

    impl MockGetter {
        fn new() -> Self {
            Self {
                values: HashMap::new(),
            }
        }

        fn insert<T: 'static>(&mut self, type_: DataComponentType<T>, value: T) {
            self.values.insert(type_.id(), Box::new(value));
        }
    }

    impl DataComponentGetter for MockGetter {
        fn get<T: 'static>(&self, component_type: &DataComponentType<T>) -> Option<&T> {
            self.values
                .get(&component_type.id())
                .and_then(|v| v.downcast_ref::<T>())
        }
    }

    #[test]
    fn get_returns_present_value_and_none_when_missing() {
        let type_i32: DataComponentType<i32> = DataComponentType::new(1);
        let missing: DataComponentType<i32> = DataComponentType::new(2);
        let mut getter = MockGetter::new();
        getter.insert(type_i32, 42);

        assert_eq!(getter.get(&type_i32), Some(&42));
        assert_eq!(getter.get(&missing), None);
    }

    #[test]
    fn get_or_default_returns_stored_value_when_present() {
        let type_i32: DataComponentType<i32> = DataComponentType::new(10);
        let mut getter = MockGetter::new();
        getter.insert(type_i32, i32::MIN);
        let default = 99;

        assert_eq!(getter.get_or_default(&type_i32, &default), &i32::MIN);
    }

    #[test]
    fn get_or_default_returns_default_when_missing() {
        let type_i32: DataComponentType<i32> = DataComponentType::new(10);
        let getter = MockGetter::new();
        let default = -7;

        assert_eq!(getter.get_or_default(&type_i32, &default), &-7);
    }

    #[test]
    fn get_typed_returns_typed_component_when_present() {
        let type_str: DataComponentType<String> = DataComponentType::new(20);
        let mut getter = MockGetter::new();
        getter.insert(type_str, "hello".to_string());

        let typed = getter.get_typed(&type_str).expect("should be present");
        assert_eq!(typed.type_(), &type_str);
        assert_eq!(typed.value(), "hello");
        assert_eq!(typed.into_value(), "hello".to_string());
    }

    #[test]
    fn get_typed_returns_none_when_missing() {
        let type_i32: DataComponentType<i32> = DataComponentType::new(30);
        let getter = MockGetter::new();

        assert!(getter.get_typed(&type_i32).is_none());
    }

    #[test]
    fn supports_multiple_component_types_and_negative_values() {
        let type_i32: DataComponentType<i32> = DataComponentType::new(1);
        let type_i64: DataComponentType<i64> = DataComponentType::new(2);
        let mut getter = MockGetter::new();
        getter.insert(type_i32, -123);
        getter.insert(type_i64, i64::MIN);

        assert_eq!(getter.get(&type_i32), Some(&-123));
        assert_eq!(getter.get(&type_i64), Some(&i64::MIN));

        let default_i64 = 0_i64;
        assert_eq!(getter.get_or_default(&type_i64, &default_i64), &i64::MIN);
    }
}
