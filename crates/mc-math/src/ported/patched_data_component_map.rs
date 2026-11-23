use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Type-erased data component value with Java-like equality/cloning semantics.
pub trait DataComponentValue: fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn equals_value(&self, other: &dyn DataComponentValue) -> bool;
    fn clone_value(&self) -> Box<dyn DataComponentValue>;
    fn hash_value(&self) -> i32 {
        let mut h: i32 = 0;
        let s = format!("{:?}", self);
        for &b in s.as_bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as i8 as i32);
        }
        h
    }
}

impl<T> DataComponentValue for T
where
    T: Clone + PartialEq + fmt::Debug + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals_value(&self, other: &dyn DataComponentValue) -> bool {
        other.as_any().downcast_ref::<T>().is_some_and(|o| self == o)
    }

    fn clone_value(&self) -> Box<dyn DataComponentValue> {
        Box::new(self.clone())
    }
}

/// Owned type-erased component value wrapper.
pub struct Value(Box<dyn DataComponentValue>);

impl Value {
    pub fn new<T: Clone + PartialEq + fmt::Debug + Send + Sync + 'static>(v: T) -> Self {
        Value(Box::new(v))
    }

    pub fn from_box(v: Box<dyn DataComponentValue>) -> Self {
        Value(v)
    }

    pub fn as_ref_value(&self) -> &dyn DataComponentValue {
        self.0.as_ref()
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        Value(self.0.clone_value())
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.0.equals_value(other.0.as_ref())
    }
}

impl Eq for Value {}

/// Reference-identity component type key, matching Java Reference2ObjectMap semantics.
#[derive(Clone)]
pub struct DataComponentType {
    inner: Arc<DataComponentTypeInner>,
}

struct DataComponentTypeInner {
    name: String,
}

impl DataComponentType {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(DataComponentTypeInner { name: name.into() }),
        }
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }
}

impl PartialEq for DataComponentType {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for DataComponentType {}

impl Hash for DataComponentType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner).hash(state);
    }
}

impl fmt::Debug for DataComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.name)
    }
}

fn key_hash_code(k: &DataComponentType) -> i32 {
    Arc::as_ptr(&k.inner) as usize as i32
}

fn option_value_hash_code(v: &Option<Value>) -> i32 {
    match v {
        Some(val) => val.0.hash_value(),
        None => 0, // Optional.empty().hashCode() == 0
    }
}

fn patch_maps_equal(
    a: &HashMap<DataComponentType, Option<Value>>,
    b: &HashMap<DataComponentType, Option<Value>>,
) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (k, va) in a {
        match b.get(k) {
            Some(vb) if va == vb => {}
            _ => return false,
        }
    }
    true
}

fn patch_map_hash(m: &HashMap<DataComponentType, Option<Value>>) -> i32 {
    let mut h: i32 = 0;
    for (k, v) in m {
        h = h.wrapping_add(key_hash_code(k) ^ option_value_hash_code(v));
    }
    h
}

fn values_equal_opt(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => x == y,
        (None, None) => true,
        _ => false,
    }
}

/// Minimal typed component pair.
pub struct TypedDataComponent {
    pub type_: DataComponentType,
    pub value: Value,
}

impl TypedDataComponent {
    pub fn create_unchecked(type_: DataComponentType, value: Value) -> Self {
        Self { type_, value }
    }

    pub fn apply_to(&self, map: &mut PatchedDataComponentMap) {
        map.set(self.type_.clone(), Some(self.value.clone()));
    }

    pub fn to_string_repr(&self) -> String {
        format!("{:?}=>{:?}", self.type_, self.value)
    }
}

impl Clone for TypedDataComponent {
    fn clone(&self) -> Self {
        Self {
            type_: self.type_.clone(),
            value: self.value.clone(),
        }
    }
}

impl fmt::Debug for TypedDataComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_repr())
    }
}

impl fmt::Display for TypedDataComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_repr())
    }
}

/// Minimal self-contained DataComponentMap interface.
pub trait DataComponentMapLike: fmt::Debug + Send + Sync + Any {
    fn as_any_map(&self) -> &dyn Any;
    fn get(&self, type_: &DataComponentType) -> Option<&dyn DataComponentValue>;
    fn has(&self, type_: &DataComponentType) -> bool {
        self.get(type_).is_some()
    }
    fn key_set(&self) -> Vec<DataComponentType>;
    fn iter_typed(&self) -> Vec<TypedDataComponent>;
    fn size(&self) -> usize;
    fn eq_map(&self, other: &dyn DataComponentMapLike) -> bool;
    fn hash_map(&self) -> i32;
}

/// Minimal self-contained DataComponentPatch.
#[derive(Clone)]
pub struct DataComponentPatch {
    pub map: Arc<HashMap<DataComponentType, Option<Value>>>,
}

impl DataComponentPatch {
    pub fn empty() -> Self {
        static EMPTY: std::sync::OnceLock<DataComponentPatch> = std::sync::OnceLock::new();
        EMPTY
            .get_or_init(|| DataComponentPatch {
                map: Arc::new(HashMap::new()),
            })
            .clone()
    }

    pub fn new(map: HashMap<DataComponentType, Option<Value>>) -> Self {
        Self { map: Arc::new(map) }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl fmt::Debug for DataComponentPatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataComponentPatch({:?})", self.map)
    }
}

impl PartialEq for DataComponentPatch {
    fn eq(&self, other: &Self) -> bool {
        patch_maps_equal(&self.map, &other.map)
    }
}

impl Eq for DataComponentPatch {}

/// Port of net.minecraft.core.component.PatchedDataComponentMap.
pub struct PatchedDataComponentMap {
    prototype: Arc<dyn DataComponentMapLike>,
    patch: Arc<HashMap<DataComponentType, Option<Value>>>,
    copy_on_write: bool,
}

impl PatchedDataComponentMap {
    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#<init>
    pub fn new(prototype: Arc<dyn DataComponentMapLike>) -> Self {
        Self {
            prototype,
            patch: Arc::new(HashMap::new()),
            copy_on_write: true,
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#<init>
    fn new_with_patch(
        prototype: Arc<dyn DataComponentMapLike>,
        patch: Arc<HashMap<DataComponentType, Option<Value>>>,
        copy_on_write: bool,
    ) -> Self {
        Self {
            prototype,
            patch,
            copy_on_write,
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#fromPatch
    pub fn from_patch(
        prototype: Arc<dyn DataComponentMapLike>,
        patch: DataComponentPatch,
    ) -> Self {
        if Self::is_patch_sanitized(prototype.as_ref(), &patch.map) {
            return Self::new_with_patch(prototype, patch.map, true);
        }
        let mut result = Self::new(prototype);
        result.apply_patch(&patch);
        result
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#isPatchSanitized
    fn is_patch_sanitized(
        prototype: &dyn DataComponentMapLike,
        patch: &HashMap<DataComponentType, Option<Value>>,
    ) -> bool {
        for (key, value) in patch.iter() {
            let proto_val = prototype.get(key);
            match value {
                Some(v) => {
                    if let Some(pv) = proto_val {
                        if v.0.equals_value(pv) {
                            return false;
                        }
                    }
                }
                None => {
                    if proto_val.is_none() {
                        return false;
                    }
                }
            }
        }
        true
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#get
    pub fn get(&self, type_: &DataComponentType) -> Option<&dyn DataComponentValue> {
        if let Some(opt) = self.patch.get(type_) {
            return opt.as_ref().map(|v| v.0.as_ref());
        }
        self.prototype.get(type_)
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#hasNonDefault
    pub fn has_non_default(&self, type_: &DataComponentType) -> bool {
        self.patch.contains_key(type_)
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#set
    pub fn set(&mut self, type_: DataComponentType, value: Option<Value>) -> Option<Value> {
        self.ensure_map_ownership();
        let proto_val = self.prototype.get(&type_).map(|v| Value(v.clone_value()));
        let patch_map = Arc::make_mut(&mut self.patch);
        let old_patch = if values_equal_opt(value.as_ref(), proto_val.as_ref()) {
            patch_map.remove(&type_)
        } else {
            patch_map.insert(type_, value)
        };
        match old_patch {
            Some(opt) => opt.or(proto_val),
            None => proto_val,
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#set
    pub fn set_typed(&mut self, component: TypedDataComponent) -> Option<Value> {
        self.set(component.type_, Some(component.value))
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#remove
    pub fn remove(&mut self, type_: &DataComponentType) -> Option<Value> {
        self.ensure_map_ownership();
        let proto_val = self.prototype.get(type_).map(|v| Value(v.clone_value()));
        let patch_map = Arc::make_mut(&mut self.patch);
        let old_patch = if proto_val.is_some() {
            patch_map.insert(type_.clone(), None)
        } else {
            patch_map.remove(type_)
        };
        match old_patch {
            Some(opt) => opt,
            None => proto_val,
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#applyPatch
    pub fn apply_patch(&mut self, patch: &DataComponentPatch) {
        self.ensure_map_ownership();
        let entries: Vec<(DataComponentType, Option<Value>)> =
            patch.map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        for (k, v) in entries {
            self.apply_patch_entry(k, v);
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#applyPatch
    fn apply_patch_entry(&mut self, type_: DataComponentType, value: Option<Value>) {
        let proto_val = self.prototype.get(&type_);
        let patch_map = Arc::make_mut(&mut self.patch);
        match &value {
            Some(v) => {
                let eq_proto = proto_val.is_some_and(|pv| v.0.equals_value(pv));
                if eq_proto {
                    patch_map.remove(&type_);
                } else {
                    patch_map.insert(type_, value);
                }
            }
            None => {
                if proto_val.is_some() {
                    patch_map.insert(type_, None);
                } else {
                    patch_map.remove(&type_);
                }
            }
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#restorePatch
    pub fn restore_patch(&mut self, patch: &DataComponentPatch) {
        self.ensure_map_ownership();
        let patch_map = Arc::make_mut(&mut self.patch);
        patch_map.clear();
        for (k, v) in patch.map.iter() {
            patch_map.insert(k.clone(), v.clone());
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#clearPatch
    pub fn clear_patch(&mut self) {
        self.ensure_map_ownership();
        Arc::make_mut(&mut self.patch).clear();
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#setAll
    pub fn set_all(&mut self, map: &dyn DataComponentMapLike) {
        for comp in map.iter_typed() {
            comp.apply_to(self);
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#ensureMapOwnership
    fn ensure_map_ownership(&mut self) {
        if self.copy_on_write {
            self.patch = Arc::new((*self.patch).clone());
            self.copy_on_write = false;
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#keySet
    pub fn key_set(&self) -> Vec<DataComponentType> {
        if self.patch.is_empty() {
            return self.prototype.key_set();
        }
        let mut result: Vec<DataComponentType> = Vec::new();
        let mut in_result: std::collections::HashSet<DataComponentType> =
            std::collections::HashSet::new();
        for k in self.prototype.key_set() {
            let removed_by_patch = matches!(self.patch.get(&k), Some(None));
            if !removed_by_patch && in_result.insert(k.clone()) {
                result.push(k);
            }
        }
        for (k, v) in self.patch.iter() {
            if v.is_some() && in_result.insert(k.clone()) {
                result.push(k.clone());
            }
        }
        result
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#iterator
    pub fn iter(&self) -> Vec<TypedDataComponent> {
        if self.patch.is_empty() {
            return self.prototype.iter_typed();
        }
        let mut result: Vec<TypedDataComponent> =
            Vec::with_capacity(self.patch.len().wrapping_add(self.prototype.size()));
        for (k, v) in self.patch.iter() {
            if let Some(val) = v {
                result.push(TypedDataComponent::create_unchecked(k.clone(), val.clone()));
            }
        }
        for comp in self.prototype.iter_typed() {
            if !self.patch.contains_key(&comp.type_) {
                result.push(comp);
            }
        }
        result
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#size
    pub fn size(&self) -> usize {
        let mut count: i32 = self.prototype.size() as i32;
        for (k, v) in self.patch.iter() {
            let has_patch_value = v.is_some();
            let has_proto = self.prototype.has(k);
            if has_patch_value != has_proto {
                count = if has_patch_value {
                    count.wrapping_add(1)
                } else {
                    count.wrapping_sub(1)
                };
            }
        }
        count as usize
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#asPatch
    pub fn as_patch(&mut self) -> DataComponentPatch {
        if self.patch.is_empty() {
            return DataComponentPatch::empty();
        }
        self.copy_on_write = true;
        DataComponentPatch {
            map: Arc::clone(&self.patch),
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#copy
    pub fn copy(&mut self) -> PatchedDataComponentMap {
        self.copy_on_write = true;
        PatchedDataComponentMap {
            prototype: Arc::clone(&self.prototype),
            patch: Arc::clone(&self.patch),
            copy_on_write: true,
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#toImmutableMap
    pub fn to_immutable_map(&mut self) -> Arc<dyn DataComponentMapLike> {
        if self.patch.is_empty() {
            Arc::clone(&self.prototype)
        } else {
            Arc::new(self.copy())
        }
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#equals
    pub fn equals_map(&self, other: &PatchedDataComponentMap) -> bool {
        self.prototype.eq_map(other.prototype.as_ref()) && patch_maps_equal(&self.patch, &other.patch)
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#hashCode
    pub fn hash_code(&self) -> i32 {
        let proto_hash = self.prototype.hash_map();
        let patch_hash = patch_map_hash(&self.patch);
        proto_hash.wrapping_add(patch_hash.wrapping_mul(31))
    }

    // JAVA: net.minecraft.core.component.PatchedDataComponentMap#toString
    pub fn to_string_repr(&self) -> String {
        let parts: Vec<String> = self.iter().iter().map(|c| c.to_string_repr()).collect();
        format!("{{{}}}", parts.join(", "))
    }
}

impl fmt::Debug for PatchedDataComponentMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_repr())
    }
}

impl PartialEq for PatchedDataComponentMap {
    fn eq(&self, other: &Self) -> bool {
        self.equals_map(other)
    }
}

impl Eq for PatchedDataComponentMap {}

impl DataComponentMapLike for PatchedDataComponentMap {
    fn as_any_map(&self) -> &dyn Any {
        self
    }

    fn get(&self, type_: &DataComponentType) -> Option<&dyn DataComponentValue> {
        PatchedDataComponentMap::get(self, type_)
    }

    fn key_set(&self) -> Vec<DataComponentType> {
        PatchedDataComponentMap::key_set(self)
    }

    fn iter_typed(&self) -> Vec<TypedDataComponent> {
        self.iter()
    }

    fn size(&self) -> usize {
        PatchedDataComponentMap::size(self)
    }

    fn eq_map(&self, other: &dyn DataComponentMapLike) -> bool {
        if let Some(other_patched) = other.as_any_map().downcast_ref::<PatchedDataComponentMap>() {
            return self.equals_map(other_patched);
        }
        if self.size() != other.size() {
            return false;
        }
        for comp in self.iter() {
            match other.get(&comp.type_) {
                Some(v) if comp.value.0.equals_value(v) => {}
                _ => return false,
            }
        }
        true
    }

    fn hash_map(&self) -> i32 {
        self.hash_code()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default)]
    struct TestPrototype {
        entries: HashMap<DataComponentType, Value>,
    }

    impl TestPrototype {
        fn new() -> Self {
            Self {
                entries: HashMap::new(),
            }
        }

        fn with(
            mut self,
            k: DataComponentType,
            v: impl Clone + PartialEq + fmt::Debug + Send + Sync + 'static,
        ) -> Self {
            self.entries.insert(k, Value::new(v));
            self
        }
    }

    impl DataComponentMapLike for TestPrototype {
        fn as_any_map(&self) -> &dyn Any {
            self
        }

        fn get(&self, type_: &DataComponentType) -> Option<&dyn DataComponentValue> {
            self.entries.get(type_).map(|v| v.0.as_ref())
        }

        fn key_set(&self) -> Vec<DataComponentType> {
            let mut keys: Vec<_> = self.entries.keys().cloned().collect();
            keys.sort_by_key(|k| k.name().to_string());
            keys
        }

        fn iter_typed(&self) -> Vec<TypedDataComponent> {
            let mut entries: Vec<_> = self.entries.iter().collect();
            entries.sort_by_key(|(k, _)| k.name().to_string());
            entries
                .into_iter()
                .map(|(k, v)| TypedDataComponent::create_unchecked(k.clone(), v.clone()))
                .collect()
        }

        fn size(&self) -> usize {
            self.entries.len()
        }

        fn eq_map(&self, other: &dyn DataComponentMapLike) -> bool {
            if self.size() != other.size() {
                return false;
            }
            for (k, v) in &self.entries {
                match other.get(k) {
                    Some(ov) if v.0.equals_value(ov) => {}
                    _ => return false,
                }
            }
            true
        }

        fn hash_map(&self) -> i32 {
            let mut h: i32 = 0;
            for (k, v) in &self.entries {
                h = h.wrapping_add(key_hash_code(k) ^ v.0.hash_value());
            }
            h
        }
    }

    #[test]
    fn test_set_get_remove_and_has_non_default() {
        let t_a = DataComponentType::new("a");
        let t_b = DataComponentType::new("b");
        let proto: Arc<dyn DataComponentMapLike> =
            Arc::new(TestPrototype::new().with(t_a.clone(), 10i32));
        let mut map = PatchedDataComponentMap::new(proto);

        assert_eq!(
            map.get(&t_a).unwrap().as_any().downcast_ref::<i32>(),
            Some(&10)
        );
        assert!(!map.has_non_default(&t_a));

        let old = map.set(t_a.clone(), Some(Value::new(20i32)));
        assert_eq!(
            old.unwrap().as_ref_value().as_any().downcast_ref::<i32>(),
            Some(&10)
        );
        assert!(map.has_non_default(&t_a));

        let old2 = map.set(t_a.clone(), Some(Value::new(10i32)));
        assert_eq!(
            old2.unwrap().as_ref_value().as_any().downcast_ref::<i32>(),
            Some(&20)
        );
        assert!(!map.has_non_default(&t_a));

        let old3 = map.remove(&t_a);
        assert_eq!(
            old3.unwrap().as_ref_value().as_any().downcast_ref::<i32>(),
            Some(&10)
        );
        assert!(map.get(&t_a).is_none());
        assert!(map.has_non_default(&t_a));

        // Java quirk: set() returns orElse(prototype) when old patch was explicit removal.
        let old4 = map.set(t_a.clone(), Some(Value::new(10i32)));
        assert_eq!(
            old4.unwrap().as_ref_value().as_any().downcast_ref::<i32>(),
            Some(&10)
        );
        assert!(!map.has_non_default(&t_a));

        let old5 = map.remove(&t_b);
        assert!(old5.is_none());
        assert!(!map.has_non_default(&t_b));
    }

    #[test]
    fn test_patch_sanitization_and_from_patch() {
        let t_a = DataComponentType::new("a");
        let t_b = DataComponentType::new("b");
        let t_c = DataComponentType::new("c");
        let proto: Arc<dyn DataComponentMapLike> =
            Arc::new(TestPrototype::new().with(t_a.clone(), 1i32));

        let mut patch_map = HashMap::new();
        patch_map.insert(t_a.clone(), Some(Value::new(2i32)));
        let patch = DataComponentPatch::new(patch_map);
        let map = PatchedDataComponentMap::from_patch(Arc::clone(&proto), patch);
        assert_eq!(
            map.get(&t_a).unwrap().as_any().downcast_ref::<i32>(),
            Some(&2)
        );
        assert!(map.has_non_default(&t_a));

        let mut patch_map2 = HashMap::new();
        patch_map2.insert(t_a.clone(), Some(Value::new(1i32))); // redundant
        patch_map2.insert(t_b.clone(), None); // redundant removal
        patch_map2.insert(t_c.clone(), Some(Value::new(3i32)));
        let patch2 = DataComponentPatch::new(patch_map2);
        let map2 = PatchedDataComponentMap::from_patch(Arc::clone(&proto), patch2);
        assert!(!map2.has_non_default(&t_a));
        assert!(!map2.has_non_default(&t_b));
        assert_eq!(
            map2.get(&t_c).unwrap().as_any().downcast_ref::<i32>(),
            Some(&3)
        );
    }

    #[test]
    fn test_copy_on_write_and_as_patch() {
        let t_a = DataComponentType::new("a");
        let proto: Arc<dyn DataComponentMapLike> =
            Arc::new(TestPrototype::new().with(t_a.clone(), 1i32));
        let mut map = PatchedDataComponentMap::new(proto);
        map.set(t_a.clone(), Some(Value::new(2i32)));

        let mut copy = map.copy();
        assert_eq!(
            copy.get(&t_a).unwrap().as_any().downcast_ref::<i32>(),
            Some(&2)
        );
        copy.set(t_a.clone(), Some(Value::new(3i32)));
        assert_eq!(
            copy.get(&t_a).unwrap().as_any().downcast_ref::<i32>(),
            Some(&3)
        );
        assert_eq!(
            map.get(&t_a).unwrap().as_any().downcast_ref::<i32>(),
            Some(&2)
        );

        let mut map2 = PatchedDataComponentMap::new(Arc::new(TestPrototype::new()));
        let t_b = DataComponentType::new("b");
        map2.set(t_b.clone(), Some(Value::new(7i32)));
        let patch = map2.as_patch();
        assert_eq!(
            patch
                .map
                .get(&t_b)
                .unwrap()
                .as_ref()
                .unwrap()
                .as_ref_value()
                .as_any()
                .downcast_ref::<i32>(),
            Some(&7)
        );
        map2.set(t_b.clone(), Some(Value::new(8i32)));
        assert_eq!(
            patch
                .map
                .get(&t_b)
                .unwrap()
                .as_ref()
                .unwrap()
                .as_ref_value()
                .as_any()
                .downcast_ref::<i32>(),
            Some(&7)
        );
        assert_eq!(
            map2.get(&t_b).unwrap().as_any().downcast_ref::<i32>(),
            Some(&8)
        );

        let mut empty_map = PatchedDataComponentMap::new(Arc::new(TestPrototype::new()));
        assert!(empty_map.as_patch().is_empty());
    }

    #[test]
    fn test_apply_restore_clear_and_size() {
        let t_a = DataComponentType::new("a");
        let t_b = DataComponentType::new("b");
        let proto: Arc<dyn DataComponentMapLike> = Arc::new(
            TestPrototype::new()
                .with(t_a.clone(), 1i32)
                .with(t_b.clone(), 2i32),
        );
        let mut map = PatchedDataComponentMap::new(proto);

        let mut pm = HashMap::new();
        pm.insert(t_a.clone(), Some(Value::new(10i32)));
        pm.insert(t_b.clone(), None);
        map.apply_patch(&DataComponentPatch::new(pm));
        assert_eq!(
            map.get(&t_a).unwrap().as_any().downcast_ref::<i32>(),
            Some(&10)
        );
        assert!(map.get(&t_b).is_none());
        assert_eq!(map.size(), 1);

        let mut restore_pm = HashMap::new();
        restore_pm.insert(t_a.clone(), Some(Value::new(99i32)));
        map.restore_patch(&DataComponentPatch::new(restore_pm));
        assert_eq!(
            map.get(&t_a).unwrap().as_any().downcast_ref::<i32>(),
            Some(&99)
        );
        assert_eq!(
            map.get(&t_b).unwrap().as_any().downcast_ref::<i32>(),
            Some(&2)
        );
        assert_eq!(map.size(), 2);

        map.clear_patch();
        assert!(!map.has_non_default(&t_a));
        assert_eq!(
            map.get(&t_a).unwrap().as_any().downcast_ref::<i32>(),
            Some(&1)
        );
    }

    #[test]
    fn test_negative_float_bits_equals_hash_and_size() {
        let t_neg = DataComponentType::new("neg");
        let t_f32 = DataComponentType::new("f32");
        let t_f64 = DataComponentType::new("f64");
        let t_proto_only = DataComponentType::new("proto_only");
        let proto: Arc<dyn DataComponentMapLike> =
            Arc::new(TestPrototype::new().with(t_proto_only.clone(), -999i32));
        let mut m1 = PatchedDataComponentMap::new(Arc::clone(&proto));
        let mut m2 = PatchedDataComponentMap::new(Arc::clone(&proto));

        m1.set(t_neg.clone(), Some(Value::new(-1i32)));
        m1.set(t_f32.clone(), Some(Value::new(0xFFC0_0001u32)));
        m1.set(t_f64.clone(), Some(Value::new(0xFFF0_0000_0000_0001u64)));

        m2.set(t_neg.clone(), Some(Value::new(-1i32)));
        m2.set(t_f32.clone(), Some(Value::new(0xFFC0_0001u32)));
        m2.set(t_f64.clone(), Some(Value::new(0xFFF0_0000_0000_0001u64)));

        assert_eq!(
            m1.get(&t_neg).unwrap().as_any().downcast_ref::<i32>(),
            Some(&-1)
        );
        assert_eq!(
            m1.get(&t_f32).unwrap().as_any().downcast_ref::<u32>(),
            Some(&0xFFC0_0001u32)
        );
        assert_eq!(
            m1.get(&t_f64).unwrap().as_any().downcast_ref::<u64>(),
            Some(&0xFFF0_0000_0000_0001u64)
        );
        assert!(m1.equals_map(&m2));
        assert_eq!(m1.hash_code(), m2.hash_code());

        assert_eq!(m1.size(), 4);
        m1.remove(&t_proto_only);
        assert!(m1.has_non_default(&t_proto_only));
        assert!(m1.get(&t_proto_only).is_none());
        assert_eq!(m1.size(), 3);
    }
}
