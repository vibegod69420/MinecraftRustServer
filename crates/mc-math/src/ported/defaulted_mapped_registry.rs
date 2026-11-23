// JAVA: net.minecraft.core.DefaultedMappedRegistry
//
// Faithful port of Minecraft Java 1.21.11 `DefaultedMappedRegistry<T>`.
//
// This module is self-contained: it defines the minimal registry/holder traits
// and types needed to mirror the Java behavior without depending on unported
// Minecraft classes.

use std::collections::HashMap;
use std::hash::Hash;

/// Minimal resource identifier equivalent for this ported module.
///
/// Java: `net.minecraft.resources.Identifier`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#parse
    pub fn parse(value: &str) -> Self {
        match value.split_once(':') {
            Some((namespace, path)) => Self {
                namespace: namespace.to_string(),
                path: path.to_string(),
            },
            None => Self {
                namespace: "minecraft".to_string(),
                path: value.to_string(),
            },
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal resource key equivalent.
///
/// Java: `net.minecraft.resources.ResourceKey`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    identifier: Identifier,
}

impl ResourceKey {
    pub fn new(identifier: Identifier) -> Self {
        Self { identifier }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

/// Minimal registration metadata placeholder.
///
/// Java: `net.minecraft.core.RegistrationInfo`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RegistrationInfo;

/// Minimal lifecycle placeholder.
///
/// Java: `com.mojang.serialization.Lifecycle`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Experimental,
}

/// Minimal random source abstraction used by `getRandom`.
///
/// Java: `net.minecraft.util.RandomSource`
pub trait RandomSource {
    fn next_int_bound(&mut self, bound: i32) -> i32;
}

/// Minimal holder reference equivalent.
///
/// Java: `net.minecraft.core.Holder.Reference<T>`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HolderReference<T: Clone> {
    key: ResourceKey,
    value: T,
}

impl<T: Clone> HolderReference<T> {
    pub fn new(key: ResourceKey, value: T) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &ResourceKey {
        &self.key
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Minimal base mapped registry behavior needed by `DefaultedMappedRegistry`.
///
/// Java: `net.minecraft.core.MappedRegistry<T>`
#[derive(Clone, Debug)]
pub struct MappedRegistry<T: Clone> {
    by_key: HashMap<ResourceKey, T>,
    by_id: Vec<T>,
    key_to_id: HashMap<ResourceKey, i32>,
    key_to_holder: HashMap<ResourceKey, HolderReference<T>>,
}

impl<T: Clone> MappedRegistry<T> {
    pub fn new(_registry_key: ResourceKey, _lifecycle: Lifecycle, _intrusive: bool) -> Self {
        Self {
            by_key: HashMap::new(),
            by_id: Vec::new(),
            key_to_id: HashMap::new(),
            key_to_holder: HashMap::new(),
        }
    }

    // JAVA: net.minecraft.core.MappedRegistry#register
    pub fn register(
        &mut self,
        key: ResourceKey,
        value: T,
        _info: RegistrationInfo,
    ) -> HolderReference<T> {
        let id = self.by_id.len() as i32;
        self.by_id.push(value.clone());
        self.by_key.insert(key.clone(), value.clone());
        self.key_to_id.insert(key.clone(), id);
        let holder = HolderReference::new(key.clone(), value);
        self.key_to_holder.insert(key, holder.clone());
        holder
    }

    // JAVA: net.minecraft.core.MappedRegistry#getId
    pub fn get_id(&self, value: &T) -> i32
    where
        T: PartialEq,
    {
        for (idx, v) in self.by_id.iter().enumerate() {
            if v == value {
                return idx as i32;
            }
        }
        -1
    }

    // JAVA: net.minecraft.core.MappedRegistry#getKey
    pub fn get_key(&self, value: &T) -> Option<Identifier>
    where
        T: PartialEq,
    {
        for (key, v) in &self.by_key {
            if v == value {
                return Some(key.identifier().clone());
            }
        }
        None
    }

    // JAVA: net.minecraft.core.MappedRegistry#getValue(net.minecraft.resources.Identifier)
    pub fn get_value(&self, id: Option<&Identifier>) -> Option<T> {
        let id = id?;
        for (key, value) in &self.by_key {
            if key.identifier() == id {
                return Some(value.clone());
            }
        }
        None
    }

    // JAVA: net.minecraft.core.MappedRegistry#byId
    pub fn by_id(&self, id: i32) -> Option<T> {
        if id < 0 {
            return None;
        }
        self.by_id.get(id as usize).cloned()
    }

    // JAVA: net.minecraft.core.MappedRegistry#getRandom
    pub fn get_random(&self, random: &mut dyn RandomSource) -> Option<HolderReference<T>> {
        if self.by_id.is_empty() {
            return None;
        }
        let bound = self.by_id.len() as i32;
        let idx = random.next_int_bound(bound);
        if idx < 0 || idx >= bound {
            return None;
        }
        let value = self.by_id[idx as usize].clone();
        // Find a matching holder/key for this value. Java random returns a holder reference;
        // this preserves behavior by returning any registered holder with the same value.
        for holder in self.key_to_holder.values() {
            if std::ptr::eq(holder.value() as *const T, &self.by_id[idx as usize] as *const T) {
                return Some(holder.clone());
            }
        }
        // Fallback: match by key order is not guaranteed in HashMap; search holders by value identity
        // through key_to_id/by_id index mapping is not stored per holder here, so match by equality.
        for holder in self.key_to_holder.values() {
            // Best-effort: return first holder whose stored value index matches.
            if let Some(&hid) = self.key_to_id.get(holder.key()) {
                if hid == idx {
                    return Some(holder.clone());
                }
            }
        }
        // Final fallback: construct a synthetic holder is not possible without key; return any holder.
        self.key_to_holder.values().next().cloned().map(|_| {
            // Rebuild holder from key found by id.
            for (key, &hid) in &self.key_to_id {
                if hid == idx {
                    return HolderReference::new(key.clone(), value.clone());
                }
            }
            // Should not happen for a consistent registry.
            HolderReference::new(
                ResourceKey::new(Identifier::parse("minecraft:unknown")),
                value,
            )
        })
    }

    pub fn contains_key(&self, key: &ResourceKey) -> bool {
        self.by_key.contains_key(key)
    }
}

/// Faithful port of `DefaultedMappedRegistry<T>`.
///
/// Java: `net.minecraft.core.DefaultedMappedRegistry`
#[derive(Clone, Debug)]
pub struct DefaultedMappedRegistry<T: Clone + PartialEq> {
    base: MappedRegistry<T>,
    default_key: Identifier,
    default_value: Option<HolderReference<T>>,
}

impl<T: Clone + PartialEq> DefaultedMappedRegistry<T> {
    // JAVA: net.minecraft.core.DefaultedMappedRegistry#<init>
    pub fn new(name: &str, registry_key: ResourceKey, lifecycle: Lifecycle, intrusive: bool) -> Self {
        Self {
            base: MappedRegistry::new(registry_key, lifecycle, intrusive),
            default_key: Identifier::parse(name),
            default_value: None,
        }
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#register
    pub fn register(
        &mut self,
        key: ResourceKey,
        value: T,
        info: RegistrationInfo,
    ) -> HolderReference<T> {
        let holder = self.base.register(key.clone(), value, info);
        if self.default_key == *key.identifier() {
            self.default_value = Some(holder.clone());
        }
        holder
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#getId
    pub fn get_id(&self, value: Option<&T>) -> i32 {
        let id = match value {
            Some(v) => self.base.get_id(v),
            None => -1,
        };
        if id == -1 {
            match &self.default_value {
                Some(default_holder) => self.base.get_id(default_holder.value()),
                None => -1,
            }
        } else {
            id
        }
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#getKey
    pub fn get_key(&self, value: &T) -> Identifier {
        match self.base.get_key(value) {
            Some(id) => id,
            None => self.default_key.clone(),
        }
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#getValue
    pub fn get_value(&self, id: Option<&Identifier>) -> Option<T> {
        let value = self.base.get_value(id);
        match value {
            Some(v) => Some(v),
            None => self.default_value.as_ref().map(|h| h.value().clone()),
        }
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#getOptional
    pub fn get_optional(&self, id: Option<&Identifier>) -> Option<T> {
        self.base.get_value(id)
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#getAny
    pub fn get_any(&self) -> Option<HolderReference<T>> {
        self.default_value.clone()
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#byId
    pub fn by_id(&self, id: i32) -> Option<T> {
        let value = self.base.by_id(id);
        match value {
            Some(v) => Some(v),
            None => self.default_value.as_ref().map(|h| h.value().clone()),
        }
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#getRandom
    pub fn get_random(&self, random: &mut dyn RandomSource) -> Option<HolderReference<T>> {
        match self.base.get_random(random) {
            Some(h) => Some(h),
            None => self.default_value.clone(),
        }
    }

    // JAVA: net.minecraft.core.DefaultedMappedRegistry#getDefaultKey
    pub fn get_default_key(&self) -> &Identifier {
        &self.default_key
    }

    /// Access the default holder reference, if registered.
    pub fn default_value(&self) -> Option<&HolderReference<T>> {
        self.default_value.as_ref()
    }

    /// Access the underlying base registry for testing/inspection.
    pub fn base(&self) -> &MappedRegistry<T> {
        &self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default)]
    struct FixedRandom {
        value: i32,
    }

    impl RandomSource for FixedRandom {
        fn next_int_bound(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                0
            } else {
                self.value % bound
            }
        }
    }

    fn reg_key(path: &str) -> ResourceKey {
        ResourceKey::new(Identifier::parse(&format!("minecraft:{}", path)))
    }

    fn make_registry() -> DefaultedMappedRegistry<String> {
        DefaultedMappedRegistry::new(
            "minecraft:default",
            reg_key("test_registry"),
            Lifecycle::Stable,
            false,
        )
    }

    #[test]
    fn register_sets_default_value_when_key_matches() {
        let mut reg = make_registry();
        let default = reg.register(reg_key("default"), "D".to_string(), RegistrationInfo);
        let other = reg.register(reg_key("other"), "O".to_string(), RegistrationInfo);

        assert_eq!(reg.get_default_key(), &Identifier::parse("minecraft:default"));
        assert_eq!(reg.default_value(), Some(&default));
        assert_ne!(reg.default_value(), Some(&other));
    }

    #[test]
    fn get_id_returns_default_id_for_missing_or_null_value() {
        let mut reg = make_registry();
        reg.register(reg_key("default"), "D".to_string(), RegistrationInfo);
        reg.register(reg_key("other"), "O".to_string(), RegistrationInfo);

        assert_eq!(reg.get_id(Some(&"O".to_string())), 1);
        assert_eq!(reg.get_id(Some(&"missing".to_string())), 0); // falls back to default value id
        assert_eq!(reg.get_id(None), 0);
    }

    #[test]
    fn get_key_and_get_value_fall_back_to_default() {
        let mut reg = make_registry();
        reg.register(reg_key("default"), "D".to_string(), RegistrationInfo);
        reg.register(reg_key("other"), "O".to_string(), RegistrationInfo);

        assert_eq!(reg.get_key(&"O".to_string()), Identifier::parse("minecraft:other"));
        assert_eq!(reg.get_key(&"missing".to_string()), Identifier::parse("minecraft:default"));

        assert_eq!(reg.get_value(Some(&Identifier::parse("minecraft:other"))), Some("O".to_string()));
        assert_eq!(
            reg.get_value(Some(&Identifier::parse("minecraft:missing"))),
            Some("D".to_string())
        );
        assert_eq!(reg.get_value(None), Some("D".to_string()));
    }

    #[test]
    fn get_optional_does_not_fall_back_but_by_id_does() {
        let mut reg = make_registry();
        reg.register(reg_key("default"), "D".to_string(), RegistrationInfo);
        reg.register(reg_key("other"), "O".to_string(), RegistrationInfo);

        assert_eq!(reg.get_optional(Some(&Identifier::parse("minecraft:other"))), Some("O".to_string()));
        assert_eq!(reg.get_optional(Some(&Identifier::parse("minecraft:missing"))), None);
        assert_eq!(reg.get_optional(None), None);

        assert_eq!(reg.by_id(1), Some("O".to_string()));
        assert_eq!(reg.by_id(99), Some("D".to_string()));
        assert_eq!(reg.by_id(-1), Some("D".to_string()));
    }

    #[test]
    fn get_any_and_get_random_fall_back_to_default() {
        let mut reg = make_registry();
        let default_holder = reg.register(reg_key("default"), "D".to_string(), RegistrationInfo);

        assert_eq!(reg.get_any(), Some(default_holder.clone()));

        let mut empty_random = FixedRandom { value: 0 };
        // Registry has entries, so base random should return one; with only default it returns default.
        let random_pick = reg.get_random(&mut empty_random);
        assert_eq!(random_pick, Some(default_holder.clone()));

        let mut empty_reg: DefaultedMappedRegistry<String> = DefaultedMappedRegistry::new(
            "minecraft:default",
            reg_key("empty"),
            Lifecycle::Stable,
            false,
        );
        // No default registered yet: get_any/get_random fallback should be None-like equivalent.
        assert_eq!(empty_reg.get_any(), None);
        assert_eq!(empty_reg.get_random(&mut FixedRandom { value: 0 }), None);

        let default_holder2 = empty_reg.register(reg_key("default"), "X".to_string(), RegistrationInfo);
        assert_eq!(empty_reg.get_random(&mut FixedRandom { value: 5 }), Some(default_holder2));
    }

    #[test]
    fn identifier_parse_handles_namespaced_and_plain() {
        let ns = Identifier::parse("custom:path");
        assert_eq!(ns.namespace(), "custom");
        assert_eq!(ns.path(), "path");
        assert_eq!(ns.to_string(), "custom:path");

        let plain = Identifier::parse("stone");
        assert_eq!(plain.namespace(), "minecraft");
        assert_eq!(plain.path(), "stone");
        assert_eq!(plain.to_string(), "minecraft:stone");
    }
}
