use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, LazyLock};

/// Minimal self-contained stand-in for `net.minecraft.resources.ResourceKey`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub location: String,
}

impl ResourceKey {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
        }
    }
}

impl fmt::Display for ResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.location)
    }
}

/// Minimal type-erased stand-in for `net.minecraft.core.Registry`
/// as used by `RegistryAccess`.
pub trait ErasedRegistry: Send + Sync {
    fn registry_key(&self) -> &ResourceKey;
    fn get_optional_registry(&self, key: &ResourceKey) -> Option<Arc<dyn ErasedRegistry>>;
    fn registry_entries(&self) -> Vec<(ResourceKey, Arc<dyn ErasedRegistry>)>;
    fn freeze_erased(&self) -> Arc<dyn ErasedRegistry>;
}

/// Port of `net.minecraft.core.RegistryAccess.RegistryEntry`.
#[derive(Clone)]
pub struct RegistryEntry {
    pub key: ResourceKey,
    pub value: Arc<dyn ErasedRegistry>,
}

impl RegistryEntry {
    // JAVA: net.minecraft.core.RegistryAccess$RegistryEntry#fromMapEntry
    pub fn from_map_entry(key: ResourceKey, value: Arc<dyn ErasedRegistry>) -> Self {
        Self::from_untyped(key, value)
    }

    // JAVA: net.minecraft.core.RegistryAccess$RegistryEntry#fromUntyped
    pub fn from_untyped(key: ResourceKey, value: Arc<dyn ErasedRegistry>) -> Self {
        Self { key, value }
    }

    // JAVA: net.minecraft.core.RegistryAccess$RegistryEntry#freeze
    pub fn freeze(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: self.value.freeze_erased(),
        }
    }
}

/// Port of `net.minecraft.core.RegistryAccess.Frozen`.
pub trait FrozenRegistryAccess: RegistryAccess {}

/// Port of `net.minecraft.core.RegistryAccess`.
pub trait RegistryAccess: Send + Sync {
    // JAVA: net.minecraft.core.RegistryAccess#lookup
    fn lookup(&self, key: &ResourceKey) -> Option<Arc<dyn ErasedRegistry>>;

    // JAVA: net.minecraft.core.RegistryAccess#lookupOrThrow
    fn lookup_or_throw(&self, key: &ResourceKey) -> Arc<dyn ErasedRegistry> {
        self.lookup(key)
            .unwrap_or_else(|| panic!("Missing registry: {}", key))
    }

    // JAVA: net.minecraft.core.RegistryAccess#registries
    fn registries(&self) -> Vec<RegistryEntry>;

    // JAVA: net.minecraft.core.RegistryAccess#listRegistryKeys
    fn list_registry_keys(&self) -> Vec<ResourceKey> {
        self.registries().into_iter().map(|e| e.key).collect()
    }

    // JAVA: net.minecraft.core.RegistryAccess#freeze
    fn freeze(&self) -> Arc<dyn FrozenRegistryAccess + Send + Sync> {
        let entries = self.registries().into_iter().map(|e| e.freeze()).collect();
        Arc::new(FrozenImmutableRegistryAccess::new(entries))
    }
}

/// Port of `net.minecraft.core.RegistryAccess.ImmutableRegistryAccess`.
pub struct ImmutableRegistryAccess {
    registries: HashMap<ResourceKey, Arc<dyn ErasedRegistry>>,
}

impl ImmutableRegistryAccess {
    // JAVA: net.minecraft.core.RegistryAccess$ImmutableRegistryAccess#ImmutableRegistryAccess
    pub fn from_registries(registries: Vec<Arc<dyn ErasedRegistry>>) -> Self {
        let mut map = HashMap::with_capacity(registries.len());
        for r in registries {
            let k = r.registry_key().clone();
            if map.insert(k.clone(), r).is_some() {
                panic!("Duplicate registry key: {}", k);
            }
        }
        Self { registries: map }
    }

    // JAVA: net.minecraft.core.RegistryAccess$ImmutableRegistryAccess#ImmutableRegistryAccess
    pub fn from_map(registries: HashMap<ResourceKey, Arc<dyn ErasedRegistry>>) -> Self {
        Self { registries }
    }

    // JAVA: net.minecraft.core.RegistryAccess$ImmutableRegistryAccess#ImmutableRegistryAccess
    pub fn from_entries(entries: Vec<RegistryEntry>) -> Self {
        let mut registries = HashMap::with_capacity(entries.len());
        for e in entries {
            if registries.insert(e.key.clone(), e.value).is_some() {
                panic!("Duplicate registry key: {}", e.key);
            }
        }
        Self { registries }
    }
}

impl RegistryAccess for ImmutableRegistryAccess {
    // JAVA: net.minecraft.core.RegistryAccess$ImmutableRegistryAccess#lookup
    fn lookup(&self, key: &ResourceKey) -> Option<Arc<dyn ErasedRegistry>> {
        self.registries.get(key).cloned()
    }

    // JAVA: net.minecraft.core.RegistryAccess$ImmutableRegistryAccess#registries
    fn registries(&self) -> Vec<RegistryEntry> {
        self.registries
            .iter()
            .map(|(k, v)| RegistryEntry::from_map_entry(k.clone(), v.clone()))
            .collect()
    }
}

/// Frozen concrete access corresponding to Java's `1FrozenAccess` / frozen immutable access.
pub struct FrozenImmutableRegistryAccess {
    inner: ImmutableRegistryAccess,
}

impl FrozenImmutableRegistryAccess {
    // JAVA: net.minecraft.core.RegistryAccess$ImmutableRegistryAccess#ImmutableRegistryAccess
    pub fn new(entries: Vec<RegistryEntry>) -> Self {
        Self {
            inner: ImmutableRegistryAccess::from_entries(entries),
        }
    }
}

impl RegistryAccess for FrozenImmutableRegistryAccess {
    // JAVA: net.minecraft.core.RegistryAccess$ImmutableRegistryAccess#lookup
    fn lookup(&self, key: &ResourceKey) -> Option<Arc<dyn ErasedRegistry>> {
        self.inner.lookup(key)
    }

    // JAVA: net.minecraft.core.RegistryAccess$ImmutableRegistryAccess#registries
    fn registries(&self) -> Vec<RegistryEntry> {
        self.inner.registries()
    }
}

impl FrozenRegistryAccess for FrozenImmutableRegistryAccess {}

struct RegistryOfRegistriesFrozen {
    registry: Arc<dyn ErasedRegistry>,
}

impl RegistryAccess for RegistryOfRegistriesFrozen {
    // JAVA: net.minecraft.core.RegistryAccess#lookup
    fn lookup(&self, key: &ResourceKey) -> Option<Arc<dyn ErasedRegistry>> {
        self.registry.get_optional_registry(key)
    }

    // JAVA: net.minecraft.core.RegistryAccess#registries
    fn registries(&self) -> Vec<RegistryEntry> {
        self.registry
            .registry_entries()
            .into_iter()
            .map(|(k, v)| RegistryEntry::from_map_entry(k, v))
            .collect()
    }

    // JAVA: net.minecraft.core.RegistryAccess#freeze
    fn freeze(&self) -> Arc<dyn FrozenRegistryAccess + Send + Sync> {
        Arc::new(Self {
            registry: self.registry.clone(),
        })
    }
}

impl FrozenRegistryAccess for RegistryOfRegistriesFrozen {}

// JAVA: net.minecraft.core.RegistryAccess#fromRegistryOfRegistries
pub fn from_registry_of_registries(
    registry: Arc<dyn ErasedRegistry>,
) -> Arc<dyn FrozenRegistryAccess + Send + Sync> {
    Arc::new(RegistryOfRegistriesFrozen { registry })
}

pub static EMPTY: LazyLock<Arc<dyn FrozenRegistryAccess + Send + Sync>> =
    LazyLock::new(|| ImmutableRegistryAccess::from_map(HashMap::new()).freeze());

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRegistry {
        key: ResourceKey,
        children: HashMap<ResourceKey, Arc<dyn ErasedRegistry>>,
    }

    impl MockRegistry {
        fn new(key: impl Into<String>) -> Self {
            Self {
                key: ResourceKey::new(key),
                children: HashMap::new(),
            }
        }

        fn with_children(key: impl Into<String>, children: Vec<Arc<dyn ErasedRegistry>>) -> Self {
            let mut map = HashMap::new();
            for c in children {
                map.insert(c.registry_key().clone(), c);
            }
            Self {
                key: ResourceKey::new(key),
                children: map,
            }
        }
    }

    impl ErasedRegistry for MockRegistry {
        fn registry_key(&self) -> &ResourceKey {
            &self.key
        }

        fn get_optional_registry(&self, key: &ResourceKey) -> Option<Arc<dyn ErasedRegistry>> {
            self.children.get(key).cloned()
        }

        fn registry_entries(&self) -> Vec<(ResourceKey, Arc<dyn ErasedRegistry>)> {
            self.children
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }

        fn freeze_erased(&self) -> Arc<dyn ErasedRegistry> {
            Arc::new(Self {
                key: self.key.clone(),
                children: self.children.clone(),
            })
        }
    }

    #[test]
    fn empty_access_has_no_registries() {
        let empty = ImmutableRegistryAccess::from_map(HashMap::new());
        assert!(empty.registries().is_empty());
        assert!(empty.list_registry_keys().is_empty());
        assert!(empty.lookup(&ResourceKey::new("missing")).is_none());

        let global_empty = EMPTY.clone();
        assert!(global_empty.registries().is_empty());
    }

    #[test]
    fn lookup_and_lookup_or_throw() {
        let reg = Arc::new(MockRegistry::new("minecraft:block")) as Arc<dyn ErasedRegistry>;
        let access = ImmutableRegistryAccess::from_registries(vec![reg]);
        let found = access.lookup(&ResourceKey::new("minecraft:block")).unwrap();
        assert_eq!(found.registry_key().location, "minecraft:block");
        assert!(access.lookup(&ResourceKey::new("minecraft:item")).is_none());

        let thrown = access.lookup_or_throw(&ResourceKey::new("minecraft:block"));
        assert_eq!(thrown.registry_key().location, "minecraft:block");
    }

    #[test]
    #[should_panic(expected = "Missing registry: minecraft:item")]
    fn lookup_or_throw_panics_on_missing() {
        let access = ImmutableRegistryAccess::from_map(HashMap::new());
        let _ = access.lookup_or_throw(&ResourceKey::new("minecraft:item"));
    }

    #[test]
    fn freeze_and_from_registry_of_registries() {
        let child = Arc::new(MockRegistry::new("minecraft:block")) as Arc<dyn ErasedRegistry>;
        let root =
            Arc::new(MockRegistry::with_children("minecraft:root", vec![child])) as Arc<dyn ErasedRegistry>;

        let access = from_registry_of_registries(root);
        let found = access.lookup(&ResourceKey::new("minecraft:block")).unwrap();
        assert_eq!(found.registry_key().location, "minecraft:block");
        assert_eq!(access.list_registry_keys().len(), 1);

        let frozen = access.freeze();
        assert!(frozen.lookup(&ResourceKey::new("minecraft:block")).is_some());
        let frozen_again = frozen.freeze();
        assert!(frozen_again.lookup(&ResourceKey::new("minecraft:block")).is_some());
    }

    #[test]
    fn edge_case_negative_overflow_and_float_bits() {
        let neg: i32 = -1;
        let overflowed: i32 = i32::MIN.wrapping_sub(1);
        assert_eq!(overflowed, i32::MAX);

        let neg_key = format!("neg_{}_{}", neg, overflowed);
        let reg_neg = Arc::new(MockRegistry::new(neg_key.clone())) as Arc<dyn ErasedRegistry>;
        let access = ImmutableRegistryAccess::from_registries(vec![reg_neg]);
        assert!(access.lookup(&ResourceKey::new(&neg_key)).is_some());

        let nan_bits = f64::NAN.to_bits();
        let neg_inf_bits = f64::NEG_INFINITY.to_bits();
        let neg_zero_bits = (-0.0f64).to_bits();
        assert_ne!(nan_bits, neg_inf_bits);
        assert_ne!(neg_zero_bits, 0.0f64.to_bits());

        let float_key = format!("float_{}_{}_{}", nan_bits, neg_inf_bits, neg_zero_bits);
        let reg_float = Arc::new(MockRegistry::new(float_key.clone())) as Arc<dyn ErasedRegistry>;
        let access2 = ImmutableRegistryAccess::from_registries(vec![reg_float]);
        let found = access2.lookup_or_throw(&ResourceKey::new(&float_key));
        assert_eq!(found.registry_key().location, float_key);
    }

    #[test]
    fn registry_entry_freeze_clones_key_and_freezes_value() {
        let reg = Arc::new(MockRegistry::new("minecraft:item")) as Arc<dyn ErasedRegistry>;
        let entry = RegistryEntry::from_map_entry(ResourceKey::new("minecraft:item"), reg);
        let frozen_entry = entry.freeze();
        assert_eq!(frozen_entry.key.location, "minecraft:item");
        assert_eq!(frozen_entry.value.registry_key().location, "minecraft:item");
    }
}
