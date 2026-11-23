use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

/// JAVA: net.minecraft.core.component.DataComponentType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataComponentType<T: ?Sized> {
    pub id: u32,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: ?Sized> DataComponentType<T> {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Type-erased component type key, mirroring Java's `DataComponentType<?>`
/// identity/map-key usage in this record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DataComponentTypeKey {
    pub id: u32,
}

impl DataComponentTypeKey {
    pub const fn new(id: u32) -> Self {
        Self { id }
    }
}

impl<T: ?Sized> From<DataComponentType<T>> for DataComponentTypeKey {
    fn from(value: DataComponentType<T>) -> Self {
        Self { id: value.id }
    }
}

/// JAVA: net.minecraft.core.component.TypedDataComponent
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedDataComponent<V> {
    pub component_type: DataComponentTypeKey,
    pub value: V,
}

impl<V> TypedDataComponent<V> {
    pub fn new<T: ?Sized>(component_type: DataComponentType<T>, value: V) -> Self {
        Self {
            component_type: component_type.into(),
            value,
        }
    }

    pub fn from_key(component_type: DataComponentTypeKey, value: V) -> Self {
        Self {
            component_type,
            value,
        }
    }

    /// JAVA: net.minecraft.core.component.TypedDataComponent#type
    pub fn type_key(&self) -> DataComponentTypeKey {
        self.component_type
    }
}

/// Minimal patch representation sufficient for `HashedPatchMap` semantics.
/// JAVA: net.minecraft.core.component.DataComponentPatch (split view)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataComponentPatch<V> {
    added: Vec<TypedDataComponent<V>>,
    removed: HashSet<DataComponentTypeKey>,
}

/// JAVA: net.minecraft.core.component.DataComponentPatch.SplitResult
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitResult<V> {
    added: Vec<TypedDataComponent<V>>,
    removed: HashSet<DataComponentTypeKey>,
}

impl<V> SplitResult<V> {
    pub fn new(added: Vec<TypedDataComponent<V>>, removed: HashSet<DataComponentTypeKey>) -> Self {
        Self { added, removed }
    }

    /// JAVA: net.minecraft.core.component.DataComponentPatch.SplitResult#added
    pub fn added(&self) -> &[TypedDataComponent<V>] {
        &self.added
    }

    /// JAVA: net.minecraft.core.component.DataComponentPatch.SplitResult#removed
    pub fn removed(&self) -> &HashSet<DataComponentTypeKey> {
        &self.removed
    }
}

impl<V: Clone> DataComponentPatch<V> {
    pub fn new(added: Vec<TypedDataComponent<V>>, removed: HashSet<DataComponentTypeKey>) -> Self {
        Self { added, removed }
    }

    /// JAVA: net.minecraft.core.component.DataComponentPatch#split
    pub fn split(&self) -> SplitResult<V> {
        SplitResult {
            added: self.added.clone(),
            removed: self.removed.clone(),
        }
    }
}

/// JAVA: net.minecraft.network.HashedPatchMap.HashGenerator
pub trait HashGenerator<V> {
    fn apply(&self, component: &TypedDataComponent<V>) -> i32;
}

impl<V, F> HashGenerator<V> for F
where
    F: Fn(&TypedDataComponent<V>) -> i32,
{
    fn apply(&self, component: &TypedDataComponent<V>) -> i32 {
        (self)(component)
    }
}

/// JAVA: net.minecraft.network.HashedPatchMap
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedPatchMap {
    added_components: HashMap<DataComponentTypeKey, i32>,
    removed_components: HashSet<DataComponentTypeKey>,
}

impl HashedPatchMap {
    // JAVA: net.minecraft.network.HashedPatchMap#<init> (record canonical constructor)
    pub fn new(
        added_components: HashMap<DataComponentTypeKey, i32>,
        removed_components: HashSet<DataComponentTypeKey>,
    ) -> Self {
        Self {
            added_components,
            removed_components,
        }
    }

    /// JAVA: net.minecraft.network.HashedPatchMap#addedComponents
    pub fn added_components(&self) -> &HashMap<DataComponentTypeKey, i32> {
        &self.added_components
    }

    /// JAVA: net.minecraft.network.HashedPatchMap#removedComponents
    pub fn removed_components(&self) -> &HashSet<DataComponentTypeKey> {
        &self.removed_components
    }

    // JAVA: net.minecraft.network.HashedPatchMap#create
    pub fn create<V, G: HashGenerator<V>>(patch: &DataComponentPatch<V>, generator: &G) -> Self
    where
        V: Clone,
    {
        let split = patch.split();
        let mut added: HashMap<DataComponentTypeKey, i32> =
            HashMap::with_capacity(split.added().len());
        for component in split.added() {
            let hash = generator.apply(component);
            added.insert(component.type_key(), hash);
        }
        HashedPatchMap::new(added, split.removed().clone())
    }

    // JAVA: net.minecraft.network.HashedPatchMap#matches
    pub fn matches<V, G: HashGenerator<V>>(&self, patch: &DataComponentPatch<V>, generator: &G) -> bool
    where
        V: Clone,
    {
        let split = patch.split();
        if split.removed() != &self.removed_components {
            return false;
        }

        if self.added_components.len() != split.added().len() {
            return false;
        }

        for component in split.added() {
            let expected = self.added_components.get(&component.type_key());
            let Some(expected_hash) = expected else {
                return false;
            };
            let actual_hash = generator.apply(component);
            if actual_hash != *expected_hash {
                return false;
            }
        }

        true
    }
}

/// Placeholder for Java `STREAM_CODEC`; actual wire codec wiring depends on
/// registry/codec infrastructure not present in this self-contained module.
pub struct HashedPatchMapStreamCodecPlaceholder;

impl fmt::Debug for HashedPatchMapStreamCodecPlaceholder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HashedPatchMapStreamCodecPlaceholder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(id: u32) -> DataComponentTypeKey {
        DataComponentTypeKey::new(id)
    }

    fn typed(id: u32, val: i32) -> TypedDataComponent<i32> {
        TypedDataComponent::from_key(key(id), val)
    }

    fn hash_by_value(component: &TypedDataComponent<i32>) -> i32 {
        component.value.wrapping_mul(31).wrapping_add(component.component_type.id as i32)
    }

    #[test]
    fn create_builds_added_hashes_and_removed_set() {
        let patch = DataComponentPatch::new(
            vec![typed(1, 10), typed(2, 20)],
            HashSet::from([key(3), key(4)]),
        );
        let map = HashedPatchMap::create(&patch, &hash_by_value);

        assert_eq!(map.added_components().len(), 2);
        assert_eq!(
            map.added_components().get(&key(1)).copied(),
            Some(hash_by_value(&typed(1, 10)))
        );
        assert_eq!(
            map.added_components().get(&key(2)).copied(),
            Some(hash_by_value(&typed(2, 20)))
        );
        assert_eq!(map.removed_components(), &HashSet::from([key(3), key(4)]));
    }

    #[test]
    fn matches_returns_true_for_identical_patch() {
        let patch = DataComponentPatch::new(vec![typed(5, -7), typed(9, 0)], HashSet::from([key(2)]));
        let map = HashedPatchMap::create(&patch, &hash_by_value);
        assert!(map.matches(&patch, &hash_by_value));
    }

    #[test]
    fn matches_fails_when_removed_differs() {
        let patch = DataComponentPatch::new(vec![typed(1, 1)], HashSet::from([key(2)]));
        let map = HashedPatchMap::create(&patch, &hash_by_value);
        let other = DataComponentPatch::new(vec![typed(1, 1)], HashSet::from([key(3)]));
        assert!(!map.matches(&other, &hash_by_value));
    }

    #[test]
    fn matches_fails_when_added_size_differs() {
        let patch = DataComponentPatch::new(vec![typed(1, 1), typed(2, 2)], HashSet::new());
        let map = HashedPatchMap::create(&patch, &hash_by_value);
        let other = DataComponentPatch::new(vec![typed(1, 1)], HashSet::new());
        assert!(!map.matches(&other, &hash_by_value));
    }

    #[test]
    fn matches_fails_when_hash_differs() {
        let patch = DataComponentPatch::new(vec![typed(1, 100)], HashSet::new());
        let map = HashedPatchMap::create(&patch, &hash_by_value);
        let changed = DataComponentPatch::new(vec![typed(1, 101)], HashSet::new());
        assert!(!map.matches(&changed, &hash_by_value));
    }

    #[test]
    fn matches_fails_when_component_type_missing() {
        let patch = DataComponentPatch::new(vec![typed(1, 1)], HashSet::new());
        let map = HashedPatchMap::create(&patch, &hash_by_value);
        let different_type = DataComponentPatch::new(vec![typed(2, 1)], HashSet::new());
        assert!(!map.matches(&different_type, &hash_by_value));
    }
}
