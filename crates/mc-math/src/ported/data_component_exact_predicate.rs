use std::any::Any;
use std::fmt;
use std::sync::LazyLock;

/// Minimal self-contained stand-in for Minecraft's typed component value.
/// Implementations must provide Java-like equality and hash semantics for the
/// concrete value they represent.
pub trait ComponentVal: Any + fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn equals_val(&self, other: &dyn ComponentVal) -> bool;
    fn hash_val(&self) -> i32;
    fn clone_val(&self) -> Box<dyn ComponentVal>;
}

impl Clone for Box<dyn ComponentVal> {
    fn clone(&self) -> Self {
        self.clone_val()
    }
}

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentType`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DataComponentType {
    pub id: u32,
    pub transient: bool,
}

impl DataComponentType {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            transient: false,
        }
    }

    pub fn transient(id: u32) -> Self {
        Self { id, transient: true }
    }

    pub fn is_transient(&self) -> bool {
        self.transient
    }

    pub fn hash_code(&self) -> i32 {
        self.id as i32
    }
}

/// Minimal self-contained stand-in for `net.minecraft.core.component.TypedDataComponent`.
pub struct TypedDataComponent {
    component_type: DataComponentType,
    value: Box<dyn ComponentVal>,
}

impl TypedDataComponent {
    pub fn new(component_type: DataComponentType, value: Box<dyn ComponentVal>) -> Self {
        Self {
            component_type,
            value,
        }
    }

    pub fn component_type(&self) -> &DataComponentType {
        &self.component_type
    }

    pub fn value(&self) -> &dyn ComponentVal {
        &*self.value
    }

    pub fn hash_code(&self) -> i32 {
        // Java: Objects.hash(type, value)
        let mut result: i32 = 1;
        result = result
            .wrapping_mul(31)
            .wrapping_add(self.component_type.hash_code());
        result = result
            .wrapping_mul(31)
            .wrapping_add(self.value.hash_val());
        result
    }
}

impl Clone for TypedDataComponent {
    fn clone(&self) -> Self {
        Self {
            component_type: self.component_type.clone(),
            value: self.value.clone_val(),
        }
    }
}

impl fmt::Debug for TypedDataComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedDataComponent")
            .field("type", &self.component_type)
            .field("value", &self.value)
            .finish()
    }
}

impl fmt::Display for TypedDataComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}={:?}", self.component_type, self.value)
    }
}

impl PartialEq for TypedDataComponent {
    fn eq(&self, other: &Self) -> bool {
        self.component_type == other.component_type && self.value.equals_val(&*other.value)
    }
}

impl Eq for TypedDataComponent {}

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentGetter`.
pub trait DataComponentGetter {
    fn get(&self, component_type: &DataComponentType) -> Option<&dyn ComponentVal>;
}

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentMap`.
#[derive(Clone, Debug, Default)]
pub struct DataComponentMap {
    components: Vec<TypedDataComponent>,
}

impl DataComponentMap {
    pub fn new(components: Vec<TypedDataComponent>) -> Self {
        Self { components }
    }

    pub fn iter_typed(&self) -> impl Iterator<Item = &TypedDataComponent> {
        self.components.iter()
    }

    pub fn get_typed(&self, component_type: &DataComponentType) -> Option<&TypedDataComponent> {
        self.components
            .iter()
            .find(|c| c.component_type() == component_type)
    }
}

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentPatch`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataComponentPatch {
    pub components: Vec<TypedDataComponent>,
}

impl DataComponentPatch {
    pub fn builder() -> DataComponentPatchBuilder {
        DataComponentPatchBuilder {
            components: Vec::new(),
        }
    }
}

pub struct DataComponentPatchBuilder {
    components: Vec<TypedDataComponent>,
}

impl DataComponentPatchBuilder {
    pub fn set(&mut self, component: TypedDataComponent) -> &mut Self {
        self.components.push(component);
        self
    }

    pub fn build(&self) -> DataComponentPatch {
        DataComponentPatch {
            components: self.components.clone(),
        }
    }
}

/// Port of `net.minecraft.core.component.DataComponentExactPredicate`.
pub struct DataComponentExactPredicate {
    expected_components: Vec<TypedDataComponent>,
}

pub static EMPTY: LazyLock<DataComponentExactPredicate> =
    LazyLock::new(|| DataComponentExactPredicate {
        expected_components: Vec::new(),
    });

impl DataComponentExactPredicate {
    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#<init>
    fn new(expected_components: Vec<TypedDataComponent>) -> Self {
        Self {
            expected_components,
        }
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#builder
    pub fn builder() -> Builder {
        Builder::new()
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#expect
    pub fn expect(component_type: DataComponentType, value: Box<dyn ComponentVal>) -> Self {
        Self::new(vec![TypedDataComponent::new(component_type, value)])
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#allOf
    pub fn all_of(map: &DataComponentMap) -> Self {
        Self::new(map.iter_typed().cloned().collect())
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#someOf
    pub fn some_of(map: &DataComponentMap, types: &[DataComponentType]) -> Self {
        let mut builder = Self::builder();
        for component_type in types {
            if let Some(component) = map.get_typed(component_type) {
                builder.expect_typed(component.clone());
            }
        }
        builder.build()
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#isEmpty
    pub fn is_empty(&self) -> bool {
        self.expected_components.is_empty()
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#hashCode
    pub fn hash_code(&self) -> i32 {
        let mut result: i32 = 1;
        for component in &self.expected_components {
            result = result
                .wrapping_mul(31)
                .wrapping_add(component.hash_code());
        }
        result
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#test
    pub fn test(&self, getter: &dyn DataComponentGetter) -> bool {
        for expected in &self.expected_components {
            match getter.get(expected.component_type()) {
                Some(actual) if expected.value().equals_val(actual) => {}
                _ => return false,
            }
        }
        true
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#alwaysMatches
    pub fn always_matches(&self) -> bool {
        self.expected_components.is_empty()
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#asPatch
    pub fn as_patch(&self) -> DataComponentPatch {
        let mut builder = DataComponentPatch::builder();
        for component in &self.expected_components {
            builder.set(component.clone());
        }
        builder.build()
    }

    pub fn expected_components(&self) -> &[TypedDataComponent] {
        &self.expected_components
    }
}

impl Clone for DataComponentExactPredicate {
    fn clone(&self) -> Self {
        Self {
            expected_components: self.expected_components.clone(),
        }
    }
}

impl fmt::Debug for DataComponentExactPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataComponentExactPredicate")
            .field("expected_components", &self.expected_components)
            .finish()
    }
}

impl PartialEq for DataComponentExactPredicate {
    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#equals
    fn eq(&self, other: &Self) -> bool {
        self.expected_components == other.expected_components
    }
}

impl Eq for DataComponentExactPredicate {}

impl fmt::Display for DataComponentExactPredicate {
    // JAVA: net.minecraft.core.component.DataComponentExactPredicate#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, component) in self.expected_components.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", component)?;
        }
        write!(f, "]")
    }
}

/// Port of `net.minecraft.core.component.DataComponentExactPredicate$Builder`.
pub struct Builder {
    expected_components: Vec<TypedDataComponent>,
}

impl Builder {
    // JAVA: net.minecraft.core.component.DataComponentExactPredicate$Builder#<init>
    fn new() -> Self {
        Self {
            expected_components: Vec::new(),
        }
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate$Builder#expect
    pub fn expect_typed(&mut self, component: TypedDataComponent) -> &mut Self {
        let component_type = component.component_type().clone();
        let value = component.value().clone_val();
        self.expect(component_type, value)
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate$Builder#expect
    pub fn expect(
        &mut self,
        component_type: DataComponentType,
        value: Box<dyn ComponentVal>,
    ) -> &mut Self {
        for existing in &self.expected_components {
            if existing.component_type() == &component_type {
                panic!(
                    "Predicate already has component of type: '{:?}'",
                    component_type
                );
            }
        }
        self.expected_components
            .push(TypedDataComponent::new(component_type, value));
        self
    }

    // JAVA: net.minecraft.core.component.DataComponentExactPredicate$Builder#build
    pub fn build(&self) -> DataComponentExactPredicate {
        DataComponentExactPredicate::new(self.expected_components.clone())
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("expected_components", &self.expected_components)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[derive(Clone, Debug)]
    struct I32Val(i32);

    impl ComponentVal for I32Val {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn equals_val(&self, other: &dyn ComponentVal) -> bool {
            other
                .as_any()
                .downcast_ref::<I32Val>()
                .is_some_and(|o| self.0 == o.0)
        }
        fn hash_val(&self) -> i32 {
            self.0
        }
        fn clone_val(&self) -> Box<dyn ComponentVal> {
            Box::new(self.clone())
        }
    }

    #[derive(Clone, Debug)]
    struct F64Val(f64);

    impl ComponentVal for F64Val {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn equals_val(&self, other: &dyn ComponentVal) -> bool {
            other
                .as_any()
                .downcast_ref::<F64Val>()
                .is_some_and(|o| self.0.to_bits() == o.0.to_bits())
        }
        fn hash_val(&self) -> i32 {
            let bits = self.0.to_bits();
            ((bits ^ (bits >> 32)) as i32)
        }
        fn clone_val(&self) -> Box<dyn ComponentVal> {
            Box::new(self.clone())
        }
    }

    struct TestGetter(Vec<TypedDataComponent>);

    impl DataComponentGetter for TestGetter {
        fn get(&self, component_type: &DataComponentType) -> Option<&dyn ComponentVal> {
            self.0
                .iter()
                .find(|c| c.component_type() == component_type)
                .map(|c| c.value())
        }
    }

    #[test]
    fn empty_predicate_matches_and_basics() {
        let pred = DataComponentExactPredicate::new(Vec::new());
        assert!(pred.is_empty());
        assert!(pred.always_matches());
        assert!(pred.test(&TestGetter(Vec::new())));
        assert_eq!(pred.hash_code(), 1); // Java empty List.hashCode()
        assert_eq!(pred.to_string(), "[]");
        assert_eq!(pred, *EMPTY);
    }

    #[test]
    fn test_match_mismatch_and_negative() {
        let t1 = DataComponentType::new(1);
        let t2 = DataComponentType::new(2);
        let pred = DataComponentExactPredicate::expect(t1.clone(), Box::new(I32Val(-42)));

        let ok = TestGetter(vec![TypedDataComponent::new(
            t1.clone(),
            Box::new(I32Val(-42)),
        )]);
        let wrong = TestGetter(vec![TypedDataComponent::new(t1, Box::new(I32Val(42)))]);
        let missing = TestGetter(vec![TypedDataComponent::new(t2, Box::new(I32Val(-42)))]);

        assert!(pred.test(&ok));
        assert!(!pred.test(&wrong));
        assert!(!pred.test(&missing));
        assert!(!pred.is_empty());
    }

    #[test]
    #[should_panic(expected = "Predicate already has component of type")]
    fn builder_rejects_duplicate_type() {
        let t = DataComponentType::new(7);
        let mut b = DataComponentExactPredicate::builder();
        b.expect(t.clone(), Box::new(I32Val(1)));
        b.expect(t, Box::new(I32Val(i32::MIN)));
    }

    #[test]
    fn hash_code_wrapping_overflow() {
        let t = DataComponentType::new(u32::MAX);
        let pred = DataComponentExactPredicate::expect(t.clone(), Box::new(I32Val(i32::MIN)));

        let type_h = u32::MAX as i32; // -1
        let val_h = i32::MIN;
        let comp_h = 1i32
            .wrapping_mul(31)
            .wrapping_add(type_h)
            .wrapping_mul(31)
            .wrapping_add(val_h);
        let list_h = 1i32.wrapping_mul(31).wrapping_add(comp_h);

        assert_eq!(pred.expected_components()[0].hash_code(), comp_h);
        assert_eq!(pred.hash_code(), list_h);
    }

    #[test]
    fn float_bits_equality_and_hash() {
        let t = DataComponentType::new(5);
        let neg_zero = DataComponentExactPredicate::expect(t.clone(), Box::new(F64Val(-0.0_f64)));
        let pos_zero = DataComponentExactPredicate::expect(t.clone(), Box::new(F64Val(0.0_f64)));
        assert_ne!(neg_zero, pos_zero);

        let nan_bits_a = 0x7FF8_0000_0000_0001u64;
        let nan_bits_b = 0x7FF8_0000_0000_0002u64;
        let nan_a = f64::from_bits(nan_bits_a);
        let nan_a2 = f64::from_bits(nan_bits_a);
        let nan_b = f64::from_bits(nan_bits_b);

        let p1 = DataComponentExactPredicate::expect(t.clone(), Box::new(F64Val(nan_a)));
        let p2 = DataComponentExactPredicate::expect(t.clone(), Box::new(F64Val(nan_a2)));
        let p3 = DataComponentExactPredicate::expect(t, Box::new(F64Val(nan_b)));
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);

        let expected_h = ((nan_bits_a ^ (nan_bits_a >> 32)) as i32);
        assert_eq!(F64Val(nan_a).hash_val(), expected_h);
    }

    #[test]
    fn some_of_all_of_and_as_patch() {
        let t1 = DataComponentType::new(1);
        let t2 = DataComponentType::new(2);
        let t3 = DataComponentType::new(3);
        let map = DataComponentMap::new(vec![
            TypedDataComponent::new(t1.clone(), Box::new(I32Val(10))),
            TypedDataComponent::new(t2.clone(), Box::new(I32Val(-20))),
        ]);

        let all = DataComponentExactPredicate::all_of(&map);
        assert_eq!(all.expected_components().len(), 2);
        assert!(!all.is_empty());

        let some = DataComponentExactPredicate::some_of(&map, &[t2.clone(), t3]);
        assert_eq!(some.expected_components().len(), 1);
        assert_eq!(some.expected_components()[0].component_type(), &t2);

        let patch = all.as_patch();
        assert_eq!(patch.components.len(), 2);
        assert_eq!(patch.components[0], all.expected_components()[0]);
    }
}
