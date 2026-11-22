use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Minimal stubs for external Minecraft / DFU types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lifecycle(u8);

impl Lifecycle {
    pub fn stable() -> Self {
        Lifecycle(0)
    }

    pub fn experimental() -> Self {
        Lifecycle(1)
    }

    pub fn add(self, other: Lifecycle) -> Self {
        if self.0 >= other.0 {
            self
        } else {
            other
        }
    }
}

#[derive(Clone, Debug)]
pub struct FeatureFlagSet {
    mask: u64,
}

impl FeatureFlagSet {
    pub fn of(mask: u64) -> Self {
        Self { mask }
    }

    pub fn mask(&self) -> u64 {
        self.mask
    }
}

pub trait FeatureElement {
    fn is_enabled(&self, flags: &FeatureFlagSet) -> bool;
}

pub struct DynamicOps<V> {
    _marker: PhantomData<V>,
}

impl<V> DynamicOps<V> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<V> Default for DynamicOps<V> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RegistryOps<V> {
    _marker: PhantomData<V>,
}

impl<V> RegistryOps<V> {
    pub fn create<P: ?Sized>(_ops: DynamicOps<V>, _provider: &P) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Key / registry marker types
// ---------------------------------------------------------------------------

pub struct ResourceKey<T: ?Sized> {
    identifier: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> ResourceKey<T> {
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            _marker: PhantomData,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn cast<U: ?Sized>(&self) -> ResourceKey<U> {
        ResourceKey::new(self.identifier.clone())
    }
}

impl<T: ?Sized> Clone for ResourceKey<T> {
    fn clone(&self) -> Self {
        Self {
            identifier: self.identifier.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> PartialEq for ResourceKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl<T: ?Sized> Eq for ResourceKey<T> {}

impl<T: ?Sized> Hash for ResourceKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl<T: ?Sized> std::fmt::Debug for ResourceKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceKey")
            .field("identifier", &self.identifier)
            .finish()
    }
}

pub struct Registry<T: ?Sized>(PhantomData<fn() -> T>);

pub struct TagKey<T: ?Sized> {
    identifier: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> TagKey<T> {
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            _marker: PhantomData,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl<T: ?Sized> Clone for TagKey<T> {
    fn clone(&self) -> Self {
        Self {
            identifier: self.identifier.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> PartialEq for TagKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl<T: ?Sized> Eq for TagKey<T> {}

impl<T: ?Sized> Hash for TagKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl<T: ?Sized> std::fmt::Debug for TagKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TagKey")
            .field("identifier", &self.identifier)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Holder / HolderSet stubs
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct HolderReference<T: Clone> {
    key: ResourceKey<T>,
    value: T,
}

impl<T: Clone> HolderReference<T> {
    pub fn new(key: ResourceKey<T>, value: T) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &ResourceKey<T> {
        &self.key
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct HolderSetNamed<T: Clone> {
    key: TagKey<T>,
    contents: Vec<HolderReference<T>>,
}

impl<T: Clone> HolderSetNamed<T> {
    pub fn new(key: TagKey<T>, contents: Vec<HolderReference<T>>) -> Self {
        Self { key, contents }
    }

    pub fn key(&self) -> &TagKey<T> {
        &self.key
    }

    pub fn contents(&self) -> &[HolderReference<T>] {
        &self.contents
    }
}

// ---------------------------------------------------------------------------
// HolderGetter / HolderOwner / HolderLookup traits
// ---------------------------------------------------------------------------

pub trait HolderGetter<T: Clone + 'static> {
    // JAVA: net.minecraft.core.HolderGetter#get
    fn get(&self, key: &ResourceKey<T>) -> Option<HolderReference<T>>;

    // JAVA: net.minecraft.core.HolderGetter#get
    fn get_tag(&self, key: &TagKey<T>) -> Option<HolderSetNamed<T>>;
}

pub trait HolderOwner<T: Clone + 'static> {}

pub trait HolderLookup<T: Clone + 'static>: HolderGetter<T> {
    // JAVA: net.minecraft.core.HolderLookup#listElements
    fn list_elements(&self) -> Vec<HolderReference<T>>;

    // JAVA: net.minecraft.core.HolderLookup#listElementIds
    fn list_element_ids(&self) -> Vec<ResourceKey<T>> {
        self.list_elements()
            .into_iter()
            .map(|r| r.key().clone())
            .collect()
    }

    // JAVA: net.minecraft.core.HolderLookup#listTags
    fn list_tags(&self) -> Vec<HolderSetNamed<T>>;

    // JAVA: net.minecraft.core.HolderLookup#listTagIds
    fn list_tag_ids(&self) -> Vec<TagKey<T>> {
        self.list_tags().into_iter().map(|s| s.key().clone()).collect()
    }
}

// ---------------------------------------------------------------------------
// RegistryLookup
// ---------------------------------------------------------------------------

type FeatureChecker<T> = Arc<dyn Fn(&T, &FeatureFlagSet) -> bool>;
type ElementPredicate<T> = Arc<dyn Fn(&T) -> bool>;

struct RegistryLookupData<T: Clone + 'static> {
    key: ResourceKey<Registry<T>>,
    lifecycle: Lifecycle,
    elements: Vec<HolderReference<T>>,
    tags: Vec<HolderSetNamed<T>>,
    feature_checker: Option<FeatureChecker<T>>,
}

enum RegistryLookupKind<T: Clone + 'static> {
    Full(RegistryLookupData<T>),
    Filtered {
        parent: RegistryLookup<T>,
        predicate: ElementPredicate<T>,
    },
}

#[derive(Clone)]
pub struct RegistryLookup<T: Clone + 'static> {
    kind: Arc<RegistryLookupKind<T>>,
}

impl<T: Clone + 'static> RegistryLookup<T> {
    pub fn new(
        key: ResourceKey<Registry<T>>,
        lifecycle: Lifecycle,
        elements: Vec<HolderReference<T>>,
        tags: Vec<HolderSetNamed<T>>,
    ) -> Self {
        Self {
            kind: Arc::new(RegistryLookupKind::Full(RegistryLookupData {
                key,
                lifecycle,
                elements,
                tags,
                feature_checker: None,
            })),
        }
    }

    pub fn new_feature_filtered(
        key: ResourceKey<Registry<T>>,
        lifecycle: Lifecycle,
        elements: Vec<HolderReference<T>>,
        tags: Vec<HolderSetNamed<T>>,
        feature_checker: impl Fn(&T, &FeatureFlagSet) -> bool + 'static,
    ) -> Self {
        Self {
            kind: Arc::new(RegistryLookupKind::Full(RegistryLookupData {
                key,
                lifecycle,
                elements,
                tags,
                feature_checker: Some(Arc::new(feature_checker)),
            })),
        }
    }

    fn feature_checker(&self) -> Option<&FeatureChecker<T>> {
        match self.kind.as_ref() {
            RegistryLookupKind::Full(d) => d.feature_checker.as_ref(),
            RegistryLookupKind::Filtered { parent, .. } => parent.feature_checker(),
        }
    }

    // JAVA: net.minecraft.core.HolderLookup$RegistryLookup#key
    pub fn key(&self) -> &ResourceKey<Registry<T>> {
        match self.kind.as_ref() {
            RegistryLookupKind::Full(d) => &d.key,
            RegistryLookupKind::Filtered { parent, .. } => parent.key(),
        }
    }

    // JAVA: net.minecraft.core.HolderLookup$RegistryLookup#registryLifecycle
    pub fn registry_lifecycle(&self) -> Lifecycle {
        match self.kind.as_ref() {
            RegistryLookupKind::Full(d) => d.lifecycle,
            RegistryLookupKind::Filtered { parent, .. } => parent.registry_lifecycle(),
        }
    }

    // JAVA: net.minecraft.core.HolderGetter#get
    pub fn get(&self, key: &ResourceKey<T>) -> Option<HolderReference<T>> {
        match self.kind.as_ref() {
            RegistryLookupKind::Full(d) => d.elements.iter().find(|e| e.key() == key).cloned(),
            RegistryLookupKind::Filtered { parent, predicate } => {
                parent.get(key).filter(|r| predicate(r.value()))
            }
        }
    }

    // JAVA: net.minecraft.core.HolderGetter#get
    pub fn get_tag(&self, key: &TagKey<T>) -> Option<HolderSetNamed<T>> {
        match self.kind.as_ref() {
            RegistryLookupKind::Full(d) => d.tags.iter().find(|t| t.key() == key).cloned(),
            RegistryLookupKind::Filtered { parent, .. } => parent.get_tag(key),
        }
    }

    // JAVA: net.minecraft.core.HolderLookup#listElements
    pub fn list_elements(&self) -> Vec<HolderReference<T>> {
        match self.kind.as_ref() {
            RegistryLookupKind::Full(d) => d.elements.clone(),
            RegistryLookupKind::Filtered { parent, predicate } => parent
                .list_elements()
                .into_iter()
                .filter(|r| predicate(r.value()))
                .collect(),
        }
    }

    // JAVA: net.minecraft.core.HolderLookup#listElementIds
    pub fn list_element_ids(&self) -> Vec<ResourceKey<T>> {
        self.list_elements()
            .into_iter()
            .map(|r| r.key().clone())
            .collect()
    }

    // JAVA: net.minecraft.core.HolderLookup#listTags
    pub fn list_tags(&self) -> Vec<HolderSetNamed<T>> {
        match self.kind.as_ref() {
            RegistryLookupKind::Full(d) => d.tags.clone(),
            RegistryLookupKind::Filtered { parent, .. } => parent.list_tags(),
        }
    }

    // JAVA: net.minecraft.core.HolderLookup#listTagIds
    pub fn list_tag_ids(&self) -> Vec<TagKey<T>> {
        self.list_tags().into_iter().map(|s| s.key().clone()).collect()
    }

    // JAVA: net.minecraft.core.HolderLookup$RegistryLookup#filterElements
    pub fn filter_elements(&self, predicate: impl Fn(&T) -> bool + 'static) -> Self {
        Self {
            kind: Arc::new(RegistryLookupKind::Filtered {
                parent: self.clone(),
                predicate: Arc::new(predicate),
            }),
        }
    }

    // JAVA: net.minecraft.core.HolderLookup$RegistryLookup#filterFeatures
    pub fn filter_features(&self, flags: &FeatureFlagSet) -> Self {
        if let Some(checker) = self.feature_checker() {
            let flags = flags.clone();
            let checker = checker.clone();
            self.filter_elements(move |v: &T| checker(v, &flags))
        } else {
            self.clone()
        }
    }
}

impl<T: Clone + 'static> HolderGetter<T> for RegistryLookup<T> {
    // JAVA: net.minecraft.core.HolderGetter#get
    fn get(&self, key: &ResourceKey<T>) -> Option<HolderReference<T>> {
        RegistryLookup::get(self, key)
    }

    // JAVA: net.minecraft.core.HolderGetter#get
    fn get_tag(&self, key: &TagKey<T>) -> Option<HolderSetNamed<T>> {
        RegistryLookup::get_tag(self, key)
    }
}

impl<T: Clone + 'static> HolderLookup<T> for RegistryLookup<T> {
    // JAVA: net.minecraft.core.HolderLookup#listElements
    fn list_elements(&self) -> Vec<HolderReference<T>> {
        RegistryLookup::list_elements(self)
    }

    // JAVA: net.minecraft.core.HolderLookup#listTags
    fn list_tags(&self) -> Vec<HolderSetNamed<T>> {
        RegistryLookup::list_tags(self)
    }
}

impl<T: Clone + 'static> HolderOwner<T> for RegistryLookup<T> {}

// ---------------------------------------------------------------------------
// Type-erased registry lookup for Provider
// ---------------------------------------------------------------------------

pub trait ErasedRegistryLookup: Any {
    fn key_identifier(&self) -> &str;
    fn registry_lifecycle_erased(&self) -> Lifecycle;
    fn as_any(&self) -> &dyn Any;
}

impl<T: Clone + 'static> ErasedRegistryLookup for RegistryLookup<T> {
    fn key_identifier(&self) -> &str {
        self.key().identifier()
    }

    fn registry_lifecycle_erased(&self) -> Lifecycle {
        self.registry_lifecycle()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

pub struct Provider {
    lookups: HashMap<String, Box<dyn ErasedRegistryLookup>>,
}

impl Provider {
    // JAVA: net.minecraft.core.HolderLookup$Provider#create
    pub fn create(lookups: Vec<Box<dyn ErasedRegistryLookup>>) -> Self {
        let lookups = lookups
            .into_iter()
            .map(|l| (l.key_identifier().to_string(), l))
            .collect();
        Self { lookups }
    }

    // JAVA: net.minecraft.core.HolderLookup$Provider#listRegistryKeys
    pub fn list_registry_keys(&self) -> Vec<ResourceKey<Registry<()>>> {
        self.lookups
            .keys()
            .map(|k| ResourceKey::new(k.clone()))
            .collect()
    }

    // JAVA: net.minecraft.core.HolderLookup$Provider#listRegistries
    pub fn list_registries(&self) -> Vec<&dyn ErasedRegistryLookup> {
        self.list_registry_keys()
            .into_iter()
            .map(|k| {
                self.lookups
                    .get(k.identifier())
                    .expect("registry present")
                    .as_ref()
            })
            .collect()
    }

    // JAVA: net.minecraft.core.HolderLookup$Provider#lookup
    pub fn lookup<T: Clone + 'static>(
        &self,
        key: &ResourceKey<Registry<T>>,
    ) -> Option<RegistryLookup<T>> {
        self.lookups
            .get(key.identifier())
            .and_then(|e| e.as_any().downcast_ref::<RegistryLookup<T>>())
            .cloned()
    }

    // JAVA: net.minecraft.core.HolderLookup$Provider#lookupOrThrow
    pub fn lookup_or_throw<T: Clone + 'static>(
        &self,
        key: &ResourceKey<Registry<T>>,
    ) -> RegistryLookup<T> {
        self.lookup(key)
            .unwrap_or_else(|| panic!("Registry {} not found", key.identifier()))
    }

    // JAVA: net.minecraft.core.HolderLookup$Provider#createSerializationContext
    pub fn create_serialization_context<V>(&self, ops: DynamicOps<V>) -> RegistryOps<V> {
        RegistryOps::create(ops, self)
    }

    // JAVA: net.minecraft.core.HolderLookup$Provider#allRegistriesLifecycle
    pub fn all_registries_lifecycle(&self) -> Lifecycle {
        self.list_registries()
            .into_iter()
            .map(|r| r.registry_lifecycle_erased())
            .fold(Lifecycle::stable(), |a, b| a.add(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn i32_lookup(name: &str, elements: Vec<(String, i32)>, tags: Vec<(String, Vec<String>)>) -> RegistryLookup<i32> {
        let refs: Vec<HolderReference<i32>> = elements
            .iter()
            .map(|(k, v)| HolderReference::new(ResourceKey::new(k.clone()), *v))
            .collect();
        let tag_sets: Vec<HolderSetNamed<i32>> = tags
            .into_iter()
            .map(|(tk, members)| {
                let contents = members
                    .into_iter()
                    .filter_map(|mk| refs.iter().find(|r| r.key().identifier() == mk).cloned())
                    .collect();
                HolderSetNamed::new(TagKey::new(tk), contents)
            })
            .collect();
        RegistryLookup::new(
            ResourceKey::<Registry<i32>>::new(name),
            Lifecycle::stable(),
            refs,
            tag_sets,
        )
    }

    #[test]
    fn list_ids_and_defaults() {
        let lookup = i32_lookup(
            "test:reg",
            vec![("test:a".into(), 1), ("test:b".into(), -1)],
            vec![("test:tag".into(), vec!["test:a".into()])],
        );
        let ids: Vec<String> = lookup
            .list_element_ids()
            .into_iter()
            .map(|k| k.identifier().to_string())
            .collect();
        assert_eq!(ids, vec!["test:a", "test:b"]);
        let tag_ids: Vec<String> = lookup
            .list_tag_ids()
            .into_iter()
            .map(|k| k.identifier().to_string())
            .collect();
        assert_eq!(tag_ids, vec!["test:tag"]);
    }

    #[test]
    fn provider_lookup_and_create() {
        let reg_key = ResourceKey::<Registry<i32>>::new("test:ints");
        let lookup = RegistryLookup::new(
            reg_key.clone(),
            Lifecycle::stable(),
            vec![HolderReference::new(ResourceKey::new("test:one"), 1i32)],
            vec![],
        );
        let provider = Provider::create(vec![Box::new(lookup)]);
        assert!(provider.lookup(&reg_key).is_some());
        assert_eq!(provider.list_registry_keys().len(), 1);
        assert_eq!(provider.list_registries().len(), 1);
        let missing = ResourceKey::<Registry<i32>>::new("test:missing");
        assert!(provider.lookup(&missing).is_none());
    }

    #[test]
    #[should_panic(expected = "Registry test:missing not found")]
    fn provider_lookup_or_throw_panics() {
        let provider = Provider::create(vec![]);
        let missing = ResourceKey::<Registry<i32>>::new("test:missing");
        let _: RegistryLookup<i32> = provider.lookup_or_throw(&missing);
    }

    #[test]
    fn filter_elements_filters_elements_not_tags() {
        let lookup = i32_lookup(
            "test:reg",
            vec![
                ("test:a".into(), 10),
                ("test:b".into(), -20),
                ("test:c".into(), i32::MIN),
            ],
            vec![("test:tag".into(), vec!["test:a".into(), "test:b".into()])],
        );
        let filtered = lookup.filter_elements(|v: &i32| *v > 0);
        let vals: Vec<i32> = filtered.list_elements().into_iter().map(|r| *r.value()).collect();
        assert_eq!(vals, vec![10]);
        assert!(filtered.get(&ResourceKey::new("test:b")).is_none());
        assert!(filtered.get(&ResourceKey::new("test:a")).is_some());
        assert_eq!(filtered.list_tags().len(), 1);
        assert!(filtered.get_tag(&TagKey::new("test:tag")).is_some());
        assert_eq!(filtered.key().identifier(), "test:reg");
        assert_eq!(filtered.registry_lifecycle(), Lifecycle::stable());
    }

    #[test]
    fn filter_features_and_lifecycle_edge_cases() {
        let key = ResourceKey::<Registry<i32>>::new("test:feat");
        let lookup = RegistryLookup::new_feature_filtered(
            key,
            Lifecycle::stable(),
            vec![
                HolderReference::new(ResourceKey::new("test:a"), 1i32),
                HolderReference::new(ResourceKey::new("test:b"), 2i32),
            ],
            vec![],
            |v: &i32, flags: &FeatureFlagSet| ((*v as u64) & flags.mask()) != 0,
        );
        let filtered = lookup.filter_features(&FeatureFlagSet::of(1));
        assert_eq!(filtered.list_elements().len(), 1);
        assert_eq!(filtered.list_elements()[0].value(), &1);

        let plain = RegistryLookup::new(
            ResourceKey::<Registry<i32>>::new("test:plain"),
            Lifecycle::stable(),
            vec![HolderReference::new(ResourceKey::new("test:a"), 1i32)],
            vec![],
        );
        assert_eq!(plain.filter_features(&FeatureFlagSet::of(0)).list_elements().len(), 1);

        let l1 = RegistryLookup::new(ResourceKey::<Registry<i32>>::new("test:l1"), Lifecycle::stable(), vec![], vec![]);
        let l2 = RegistryLookup::new(ResourceKey::<Registry<i32>>::new("test:l2"), Lifecycle::experimental(), vec![], vec![]);
        let provider = Provider::create(vec![Box::new(l1), Box::new(l2)]);
        assert_eq!(provider.all_registries_lifecycle(), Lifecycle::experimental());
    }

    #[test]
    fn overflow_negative_and_float_bits() {
        let edge_key = ResourceKey::<Registry<i64>>::new("test:edge");
        let vals = [i64::MIN, i64::MAX, -1i64, 0i64, 1i64];
        let elements: Vec<_> = vals
            .iter()
            .enumerate()
            .map(|(i, v)| HolderReference::new(ResourceKey::new(format!("test:{i}")), *v))
            .collect();
        let lookup = RegistryLookup::new(edge_key, Lifecycle::stable(), elements, vec![]);
        let filtered = lookup.filter_elements(|v: &i64| *v < 0);
        let got: Vec<i64> = filtered.list_elements().into_iter().map(|r| *r.value()).collect();
        assert_eq!(got, vec![i64::MIN, -1]);

        let nan_bits = f32::NAN.to_bits();
        let neg_zero_bits = (-0.0f32).to_bits();
        let inf_bits = f32::INFINITY.to_bits();
        let fkey = ResourceKey::<Registry<u32>>::new("test:float");
        let felems = vec![
            HolderReference::new(ResourceKey::new("test:nan"), nan_bits),
            HolderReference::new(ResourceKey::new("test:nz"), neg_zero_bits),
            HolderReference::new(ResourceKey::new("test:inf"), inf_bits),
        ];
        let flookup = RegistryLookup::new(fkey, Lifecycle::stable(), felems, vec![]);
        assert_eq!(flookup.list_elements()[0].value(), &nan_bits);
        assert_eq!(flookup.list_elements()[1].value(), &neg_zero_bits);
        assert_ne!(neg_zero_bits, 0u32);
        assert_eq!(flookup.list_element_ids().len(), 3);
    }
}
