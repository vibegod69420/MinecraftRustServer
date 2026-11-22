use std::any::Any;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock};

/// Typed key for a data component, identified by a stable numeric id.
#[derive(Debug)]
pub struct DataComponentType<T: 'static> {
    id: u64,
    _phantom: PhantomData<fn() -> T>,
}

impl<T: 'static> DataComponentType<T> {
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }
}

impl<T: 'static> Clone for DataComponentType<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for DataComponentType<T> {}

impl<T: 'static> PartialEq for DataComponentType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: 'static> Eq for DataComponentType<T> {}

impl<T: 'static> Hash for DataComponentType<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Type-erased component entry, equivalent to Java `TypedDataComponent<?>`.
#[derive(Clone)]
pub struct TypedDataComponent {
    type_id: u64,
    value: Arc<dyn Any + Send + Sync>,
}

impl TypedDataComponent {
    // JAVA: net.minecraft.core.component.TypedDataComponent#fromEntryUnchecked
    pub fn from_entry_unchecked(type_id: u64, value: Arc<dyn Any + Send + Sync>) -> Self {
        Self { type_id, value }
    }

    pub fn type_id(&self) -> u64 {
        self.type_id
    }

    pub fn value_arc(&self) -> Arc<dyn Any + Send + Sync> {
        self.value.clone()
    }

    pub fn downcast<T: 'static + Clone>(&self) -> Option<T> {
        self.value.downcast_ref::<T>().cloned()
    }
}

impl fmt::Debug for TypedDataComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedDataComponent")
            .field("type_id", &self.type_id)
            .finish_non_exhaustive()
    }
}

trait DataComponentMapImpl: Send + Sync {
    fn get_arc(&self, type_id: u64) -> Option<Arc<dyn Any + Send + Sync>>;
    fn key_set_ids(&self) -> HashSet<u64>;
    fn has_id(&self, type_id: u64) -> bool {
        self.get_arc(type_id).is_some()
    }
    fn iter_typed(&self) -> Box<dyn Iterator<Item = TypedDataComponent> + '_>;
    fn size_count(&self) -> usize {
        self.key_set_ids().len()
    }
}

#[derive(Clone)]
pub struct DataComponentMap {
    inner: Arc<dyn DataComponentMapImpl>,
}

impl fmt::Debug for DataComponentMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataComponentMap")
            .field("size", &self.size())
            .finish()
    }
}

struct EmptyMapImpl;

impl DataComponentMapImpl for EmptyMapImpl {
    // JAVA: net.minecraft.core.component.DataComponentMap#get
    fn get_arc(&self, _type_id: u64) -> Option<Arc<dyn Any + Send + Sync>> {
        None
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#keySet
    fn key_set_ids(&self) -> HashSet<u64> {
        HashSet::new()
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#iterator
    fn iter_typed(&self) -> Box<dyn Iterator<Item = TypedDataComponent> + '_> {
        Box::new(std::iter::empty())
    }

    fn size_count(&self) -> usize {
        0
    }
}

// JAVA: net.minecraft.core.component.DataComponentMap#EMPTY
pub fn empty() -> DataComponentMap {
    static EMPTY: LazyLock<DataComponentMap> = LazyLock::new(|| DataComponentMap {
        inner: Arc::new(EmptyMapImpl),
    });
    EMPTY.clone()
}

struct SimpleMap {
    entries: Vec<(u64, Arc<dyn Any + Send + Sync>)>,
    index: std::collections::HashMap<u64, usize>,
}

impl SimpleMap {
    fn from_entries(entries: Vec<(u64, Arc<dyn Any + Send + Sync>)>) -> Self {
        let mut index = std::collections::HashMap::with_capacity(entries.len());
        for (i, (k, _)) in entries.iter().enumerate() {
            index.insert(*k, i);
        }
        Self { entries, index }
    }
}

impl DataComponentMapImpl for SimpleMap {
    // JAVA: net.minecraft.core.component.DataComponentMap.Builder.SimpleMap#get
    fn get_arc(&self, type_id: u64) -> Option<Arc<dyn Any + Send + Sync>> {
        self.index.get(&type_id).map(|&i| self.entries[i].1.clone())
    }

    // JAVA: net.minecraft.core.component.DataComponentMap.Builder.SimpleMap#has
    fn has_id(&self, type_id: u64) -> bool {
        self.index.contains_key(&type_id)
    }

    // JAVA: net.minecraft.core.component.DataComponentMap.Builder.SimpleMap#keySet
    fn key_set_ids(&self) -> HashSet<u64> {
        self.entries.iter().map(|(k, _)| *k).collect()
    }

    // JAVA: net.minecraft.core.component.DataComponentMap.Builder.SimpleMap#iterator
    fn iter_typed(&self) -> Box<dyn Iterator<Item = TypedDataComponent> + '_> {
        Box::new(
            self.entries
                .iter()
                .map(|(k, v)| TypedDataComponent::from_entry_unchecked(*k, v.clone())),
        )
    }

    // JAVA: net.minecraft.core.component.DataComponentMap.Builder.SimpleMap#size
    fn size_count(&self) -> usize {
        self.entries.len()
    }
}

impl fmt::Display for SimpleMap {
    // JAVA: net.minecraft.core.component.DataComponentMap.Builder.SimpleMap#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        write!(f, "{{")?;
        for (k, _) in &self.entries {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}=<value>", k)?;
            first = false;
        }
        write!(f, "}}")
    }
}

struct CompositeMap {
    first: Arc<dyn DataComponentMapImpl>,
    second: Arc<dyn DataComponentMapImpl>,
}

impl DataComponentMapImpl for CompositeMap {
    // JAVA: net.minecraft.core.component.DataComponentMap#composite
    fn get_arc(&self, type_id: u64) -> Option<Arc<dyn Any + Send + Sync>> {
        if let Some(v) = self.second.get_arc(type_id) {
            return Some(v);
        }
        self.first.get_arc(type_id)
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#composite
    fn key_set_ids(&self) -> HashSet<u64> {
        let mut set = self.first.key_set_ids();
        set.extend(self.second.key_set_ids());
        set
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#iterator
    fn iter_typed(&self) -> Box<dyn Iterator<Item = TypedDataComponent> + '_> {
        let keys: Vec<u64> = self.key_set_ids().into_iter().collect();
        let first = self.first.clone();
        let second = self.second.clone();
        Box::new(keys.into_iter().filter_map(move |k| {
            let v = second.get_arc(k).or_else(|| first.get_arc(k));
            v.map(|arc| TypedDataComponent::from_entry_unchecked(k, arc))
        }))
    }

    fn size_count(&self) -> usize {
        self.key_set_ids().len()
    }
}

struct FilteredMap {
    source: Arc<dyn DataComponentMapImpl>,
    predicate: Arc<dyn Fn(u64) -> bool + Send + Sync>,
}

impl DataComponentMapImpl for FilteredMap {
    // JAVA: net.minecraft.core.component.DataComponentMap#filter
    fn get_arc(&self, type_id: u64) -> Option<Arc<dyn Any + Send + Sync>> {
        if (self.predicate)(type_id) {
            self.source.get_arc(type_id)
        } else {
            None
        }
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#filter
    fn key_set_ids(&self) -> HashSet<u64> {
        self.source
            .key_set_ids()
            .into_iter()
            .filter(|k| (self.predicate)(*k))
            .collect()
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#iterator
    fn iter_typed(&self) -> Box<dyn Iterator<Item = TypedDataComponent> + '_> {
        let keys: Vec<u64> = self.key_set_ids().into_iter().collect();
        let source = self.source.clone();
        Box::new(keys.into_iter().filter_map(move |k| {
            source
                .get_arc(k)
                .map(|arc| TypedDataComponent::from_entry_unchecked(k, arc))
        }))
    }

    fn size_count(&self) -> usize {
        self.key_set_ids().len()
    }
}

impl DataComponentMap {
    // JAVA: net.minecraft.core.component.DataComponentMap#get
    pub fn get<T: 'static + Clone + Send + Sync>(
        &self,
        component: DataComponentType<T>,
    ) -> Option<T> {
        self.inner
            .get_arc(component.id())
            .and_then(|v| v.downcast_ref::<T>().cloned())
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#keySet
    pub fn key_set(&self) -> HashSet<u64> {
        self.inner.key_set_ids()
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#has
    pub fn has<T: 'static + Send + Sync>(&self, component: DataComponentType<T>) -> bool {
        self.inner.has_id(component.id())
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#iterator
    pub fn iter(&self) -> Box<dyn Iterator<Item = TypedDataComponent> + '_> {
        self.inner.iter_typed()
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#stream
    pub fn stream(&self) -> Vec<TypedDataComponent> {
        self.iter().collect()
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#size
    pub fn size(&self) -> usize {
        self.inner.size_count()
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#isEmpty
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    // JAVA: net.minecraft.core.component.DataComponentMap#filter
    pub fn filter(
        &self,
        predicate: impl Fn(u64) -> bool + Send + Sync + 'static,
    ) -> DataComponentMap {
        let pred = Arc::new(predicate);
        DataComponentMap {
            inner: Arc::new(FilteredMap {
                source: self.inner.clone(),
                predicate: pred,
            }),
        }
    }
}

// JAVA: net.minecraft.core.component.DataComponentMap#composite
pub fn composite(first: DataComponentMap, second: DataComponentMap) -> DataComponentMap {
    DataComponentMap {
        inner: Arc::new(CompositeMap {
            first: first.inner,
            second: second.inner,
        }),
    }
}

// JAVA: net.minecraft.core.component.DataComponentMap#builder
pub fn builder() -> Builder {
    Builder::new()
}

pub struct Builder {
    entries: Vec<(u64, Arc<dyn Any + Send + Sync>)>,
    index: std::collections::HashMap<u64, usize>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    // JAVA: net.minecraft.core.component.DataComponentMap.Builder#Builder
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: std::collections::HashMap::new(),
        }
    }

    // JAVA: net.minecraft.core.component.DataComponentMap.Builder#set
    pub fn set<T: 'static + Send + Sync>(
        &mut self,
        component: DataComponentType<T>,
        value: Option<T>,
    ) -> &mut Self {
        self.set_unchecked(
            component.id(),
            value.map(|v| Arc::new(v) as Arc<dyn Any + Send + Sync>),
        );
        self
    }

    // JAVA: net.minecraft.core.component.DataComponentMap.Builder#setUnchecked
    fn set_unchecked(&mut self, type_id: u64, value: Option<Arc<dyn Any + Send + Sync>>) {
        if let Some(v) = value {
            if let Some(&i) = self.index.get(&type_id) {
                self.entries[i].1 = v;
            } else {
                let i = self.entries.len();
                self.entries.push((type_id, v));
                self.index.insert(type_id, i);
            }
        } else if let Some(&i) = self.index.get(&type_id) {
            self.entries.remove(i);
            self.index.remove(&type_id);
            for j in i..self.entries.len() {
                let k = self.entries[j].0;
                self.index.insert(k, j);
            }
        }
    }

    // JAVA: net.minecraft.core.component.DataComponentMap.Builder#addAll
    pub fn add_all(&mut self, other: &DataComponentMap) -> &mut Self {
        for comp in other.iter() {
            let type_id = comp.type_id();
            let value = comp.value_arc();
            if let Some(&i) = self.index.get(&type_id) {
                self.entries[i].1 = value;
            } else {
                let i = self.entries.len();
                self.entries.push((type_id, value));
                self.index.insert(type_id, i);
            }
        }
        self
    }

    // JAVA: net.minecraft.core.component.DataComponentMap.Builder#build
    pub fn build(&self) -> DataComponentMap {
        build_from_map_trusted(&self.entries)
    }
}

// JAVA: net.minecraft.core.component.DataComponentMap.Builder#buildFromMapTrusted
fn build_from_map_trusted(entries: &[(u64, Arc<dyn Any + Send + Sync>)]) -> DataComponentMap {
    if entries.is_empty() {
        return empty();
    }
    // Java chooses ArrayMap for < 8, OpenHashMap otherwise; we preserve insertion order in both.
    if entries.len() < 8 {
        DataComponentMap {
            inner: Arc::new(SimpleMap::from_entries(entries.to_vec())),
        }
    } else {
        DataComponentMap {
            inner: Arc::new(SimpleMap::from_entries(entries.to_vec())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T_I32: DataComponentType<i32> = DataComponentType::new(1);
    const T_I64: DataComponentType<i64> = DataComponentType::new(2);
    const T_F64: DataComponentType<f64> = DataComponentType::new(3);
    const T_F32: DataComponentType<f32> = DataComponentType::new(4);

    #[test]
    fn empty_map_basics() {
        let m = empty();
        assert!(m.is_empty());
        assert_eq!(m.size(), 0);
        assert!(!m.has(T_I32));
        assert_eq!(m.get(T_I32), None);
        assert!(m.key_set().is_empty());
        assert_eq!(m.iter().count(), 0);
        assert!(m.stream().is_empty());
    }

    #[test]
    fn builder_set_get_overwrite_remove() {
        let mut b = builder();
        b.set(T_I32, Some(i32::MIN));
        b.set(T_I64, Some(-123_456_789_i64));
        let m = b.build();
        assert_eq!(m.get(T_I32), Some(i32::MIN));
        assert_eq!(m.get(T_I64), Some(-123_456_789_i64));
        assert_eq!(m.size(), 2);
        assert!(m.has(T_I32));

        b.set(T_I32, Some(42));
        b.set(T_I64, None);
        let m2 = b.build();
        assert_eq!(m2.get(T_I32), Some(42));
        assert!(!m2.has(T_I64));
        assert_eq!(m2.size(), 1);
    }

    #[test]
    fn builder_add_all_and_size_threshold() {
        let mut b1 = builder();
        for i in 0..8u64 {
            let ty = DataComponentType::<i32>::new(100 + i);
            b1.set(ty, Some(i as i32));
        }
        let m1 = b1.build();
        assert_eq!(m1.size(), 8);

        let mut b2 = builder();
        b2.set(T_I32, Some(1));
        b2.add_all(&m1);
        let m2 = b2.build();
        assert_eq!(m2.size(), 9);
        assert_eq!(m2.get(T_I32), Some(1));
        assert_eq!(m2.get(DataComponentType::<i32>::new(107)), Some(7));
    }

    #[test]
    fn composite_prefers_second_and_unions_keys() {
        let mut b1 = builder();
        b1.set(T_I32, Some(10));
        b1.set(T_I64, Some(100_i64));
        let m1 = b1.build();

        let mut b2 = builder();
        b2.set(T_I32, Some(20));
        b2.set(T_F64, Some(1.5_f64));
        let m2 = b2.build();

        let c = composite(m1, m2);
        assert_eq!(c.get(T_I32), Some(20));
        assert_eq!(c.get(T_I64), Some(100_i64));
        assert_eq!(c.get(T_F64), Some(1.5_f64));
        assert_eq!(c.size(), 3);
        let keys = c.key_set();
        assert!(keys.contains(&T_I32.id()));
        assert!(keys.contains(&T_I64.id()));
        assert!(keys.contains(&T_F64.id()));
    }

    #[test]
    fn filter_and_float_bits_negative_overflow_values() {
        let nan_bits: u64 = 0x7FF8_0000_0000_0001;
        let neg_zero_bits: u32 = 0x8000_0000;
        let mut b = builder();
        b.set(T_I32, Some(i32::MAX));
        b.set(T_I64, Some(i64::MIN));
        b.set(T_F64, Some(f64::from_bits(nan_bits)));
        b.set(T_F32, Some(f32::from_bits(neg_zero_bits)));
        let m = b.build();

        let filtered = m.filter(|id| id == T_F64.id() || id == T_I32.id());
        assert_eq!(filtered.size(), 2);
        assert!(filtered.has(T_F64));
        assert!(filtered.has(T_I32));
        assert!(!filtered.has(T_I64));

        let got_nan = filtered.get(T_F64).unwrap();
        assert_eq!(got_nan.to_bits(), nan_bits);

        let got_nz = m.get(T_F32).unwrap();
        assert_eq!(got_nz.to_bits(), neg_zero_bits);

        let max = filtered.get(T_I32).unwrap();
        assert_eq!(max.wrapping_add(1), i32::MIN);
        assert_eq!(m.get(T_I64).unwrap(), i64::MIN);
    }

    #[test]
    fn typed_iteration_returns_entries() {
        let mut b = builder();
        b.set(T_I32, Some(-7));
        b.set(T_F64, Some(-0.0_f64));
        let m = b.build();

        let mut seen = HashSet::new();
        for comp in m.iter() {
            seen.insert(comp.type_id());
            if comp.type_id() == T_I32.id() {
                assert_eq!(comp.downcast::<i32>(), Some(-7));
            }
            if comp.type_id() == T_F64.id() {
                let v = comp.downcast::<f64>().unwrap();
                assert_eq!(v.to_bits(), (-0.0_f64).to_bits());
            }
        }
        assert_eq!(seen.len(), 2);
        assert!(seen.contains(&T_I32.id()));
        assert!(seen.contains(&T_F64.id()));
    }
}
