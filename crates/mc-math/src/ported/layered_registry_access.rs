use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

pub type ResourceKey = String;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Registry {
    pub key: ResourceKey,
}

impl Registry {
    pub fn new(key: impl Into<ResourceKey>) -> Self {
        Self { key: key.into() }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RegistryAccessFrozen {
    registries: HashMap<ResourceKey, Registry>,
}

impl RegistryAccessFrozen {
    pub fn empty() -> Self {
        Self {
            registries: HashMap::new(),
        }
    }

    pub fn from_registries(registries: HashMap<ResourceKey, Registry>) -> Self {
        Self { registries }
    }

    pub fn registries(&self) -> &HashMap<ResourceKey, Registry> {
        &self.registries
    }
}

pub trait RegistryAccessLayer: Clone + Debug + PartialEq + Eq + Hash + Any {}
impl<T: Clone + Debug + PartialEq + Eq + Hash + Any> RegistryAccessLayer for T {}

#[derive(Clone, Debug)]
pub struct LayeredRegistryAccess<T: RegistryAccessLayer> {
    keys: Vec<T>,
    values: Vec<RegistryAccessFrozen>,
    composite: Arc<RegistryAccessFrozen>,
}

impl<T: RegistryAccessLayer> LayeredRegistryAccess<T> {
    // JAVA: net.minecraft.core.LayeredRegistryAccess#<init>(List)
    pub fn new(keys: Vec<T>) -> Self {
        let values = vec![RegistryAccessFrozen::empty(); keys.len()];
        Self::from_parts(keys, values)
    }

    // JAVA: net.minecraft.core.LayeredRegistryAccess#<init>(List,List)
    fn from_parts(keys: Vec<T>, values: Vec<RegistryAccessFrozen>) -> Self {
        let composite = Arc::new(RegistryAccessFrozen::from_registries(
            collect_registries(values.iter()),
        ));
        Self {
            keys,
            values,
            composite,
        }
    }

    // JAVA: net.minecraft.core.LayeredRegistryAccess#getLayerIndexOrThrow
    fn get_layer_index_or_throw(&self, key: &T) -> usize {
        self.keys.iter().position(|k| k == key).unwrap_or_else(|| {
            panic!(
                "Can't find {:?} inside {:?}",
                key, self.keys
            )
        })
    }

    // JAVA: net.minecraft.core.LayeredRegistryAccess#getLayer
    pub fn get_layer(&self, key: &T) -> &RegistryAccessFrozen {
        let idx = self.get_layer_index_or_throw(key);
        &self.values[idx]
    }

    // JAVA: net.minecraft.core.LayeredRegistryAccess#getAccessForLoading
    pub fn get_access_for_loading(&self, key: &T) -> RegistryAccessFrozen {
        let idx = self.get_layer_index_or_throw(key);
        self.get_composite_access_for_layers(0, idx)
    }

    // JAVA: net.minecraft.core.LayeredRegistryAccess#getAccessFrom
    pub fn get_access_from(&self, key: &T) -> RegistryAccessFrozen {
        let idx = self.get_layer_index_or_throw(key);
        self.get_composite_access_for_layers(idx, self.values.len())
    }

    // JAVA: net.minecraft.core.LayeredRegistryAccess#getCompositeAccessForLayers
    fn get_composite_access_for_layers(&self, start: usize, end: usize) -> RegistryAccessFrozen {
        RegistryAccessFrozen::from_registries(collect_registries(self.values[start..end].iter()))
    }

    // JAVA: net.minecraft.core.LayeredRegistryAccess#replaceFrom
    pub fn replace_from(&self, key: &T, replacements: Vec<RegistryAccessFrozen>) -> Self {
        let idx = self.get_layer_index_or_throw(key);
        if replacements.len() > self.values.len().saturating_sub(idx) {
            panic!("Too many values to replace");
        }
        let mut new_values: Vec<RegistryAccessFrozen> = Vec::with_capacity(self.values.len());
        for i in 0..idx {
            new_values.push(self.values[i].clone());
        }
        new_values.extend(replacements);
        while new_values.len() < self.values.len() {
            new_values.push(RegistryAccessFrozen::empty());
        }
        Self::from_parts(self.keys.clone(), new_values)
    }

    // JAVA: net.minecraft.core.LayeredRegistryAccess#compositeAccess
    pub fn composite_access(&self) -> &RegistryAccessFrozen {
        &self.composite
    }
}

// JAVA: net.minecraft.core.LayeredRegistryAccess#collectRegistries
fn collect_registries<'a, I>(accesses: I) -> HashMap<ResourceKey, Registry>
where
    I: Iterator<Item = &'a RegistryAccessFrozen>,
{
    let mut out: HashMap<ResourceKey, Registry> = HashMap::new();
    for access in accesses {
        for (k, v) in access.registries() {
            if out.insert(k.clone(), v.clone()).is_some() {
                panic!("Duplicated registry {}", k);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frozen_with(keys: &[&str]) -> RegistryAccessFrozen {
        let mut m = HashMap::new();
        for k in keys {
            m.insert((*k).to_string(), Registry::new(*k));
        }
        RegistryAccessFrozen::from_registries(m)
    }

    #[test]
    fn new_creates_empty_layers() {
        let l: LayeredRegistryAccess<String> = LayeredRegistryAccess::new(vec!["a".into(), "b".into()]);
        assert!(l.get_layer(&"a".to_string()).registries().is_empty());
        assert!(l.composite_access().registries().is_empty());
    }

    #[test]
    fn replace_from_and_access() {
        let l: LayeredRegistryAccess<String> = LayeredRegistryAccess::new(vec!["a".into(), "b".into(), "c".into()]);
        let l2 = l.replace_from(&"b".to_string(), vec![frozen_with(&["r1"])]);
        assert!(l2.get_layer(&"a".to_string()).registries().is_empty());
        assert!(l2.get_layer(&"b".to_string()).registries().contains_key("r1"));
        assert!(l2.get_layer(&"c".to_string()).registries().is_empty());
        assert!(l2.composite_access().registries().contains_key("r1"));
    }

    #[test]
    fn get_access_for_loading_and_from() {
        let l: LayeredRegistryAccess<i32> = LayeredRegistryAccess::new(vec![1, 2, 3]);
        let l2 = l.replace_from(&1, vec![frozen_with(&["r1"]), frozen_with(&["r2"]), frozen_with(&["r3"])]);
        let loading = l2.get_access_for_loading(&2);
        assert!(loading.registries().contains_key("r1"));
        assert!(!loading.registries().contains_key("r2"));
        let from = l2.get_access_from(&2);
        assert!(from.registries().contains_key("r2"));
        assert!(from.registries().contains_key("r3"));
    }

    #[test]
    #[should_panic(expected = "Duplicated registry")]
    fn collect_registries_panics_on_duplicate() {
        let a = frozen_with(&["r1"]);
        let b = frozen_with(&["r1"]);
        let _ = collect_registries([a, b].iter());
    }

    #[test]
    #[should_panic(expected = "Too many values to replace")]
    fn replace_too_many_panics() {
        let l: LayeredRegistryAccess<i32> = LayeredRegistryAccess::new(vec![1, 2]);
        let _ = l.replace_from(&2, vec![frozen_with(&["a"]), frozen_with(&["b"])]);
    }

    #[test]
    #[should_panic(expected = "Can't find")]
    fn missing_key_panics() {
        let l: LayeredRegistryAccess<i32> = LayeredRegistryAccess::new(vec![1, 2]);
        let _ = l.get_layer(&3);
    }
}
