use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CONTEXT_KEY_ID: AtomicU64 = AtomicU64::new(1);

/// Minimal self-contained Identifier stub for standalone compilation.
/// JAVA: net.minecraft.resources.Identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Type-erased context key handle preserving identity semantics.
#[derive(Clone, Debug)]
pub struct ErasedContextKey {
    id: u64,
    name: Identifier,
}

impl PartialEq for ErasedContextKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ErasedContextKey {}

impl Hash for ErasedContextKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl ErasedContextKey {
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl fmt::Display for ErasedContextKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<parameter {}>", self.name)
    }
}

/// JAVA: net.minecraft.util.context.ContextKey
#[derive(Debug)]
pub struct ContextKey<T: ?Sized> {
    id: u64,
    name: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Clone for ContextKey<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> PartialEq for ContextKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: ?Sized> Eq for ContextKey<T> {}

impl<T: ?Sized> Hash for ContextKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: ?Sized> ContextKey<T> {
    // JAVA: net.minecraft.util.context.ContextKey#<init>
    pub fn new(name: Identifier) -> Self {
        Self {
            id: NEXT_CONTEXT_KEY_ID.fetch_add(1, Ordering::Relaxed),
            name,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.util.context.ContextKey#vanilla
    pub fn vanilla(name: &str) -> Self
    where
        T: Sized,
    {
        Self::new(Identifier::with_default_namespace(name))
    }

    // JAVA: net.minecraft.util.context.ContextKey#name
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn erased(&self) -> ErasedContextKey {
        ErasedContextKey {
            id: self.id,
            name: self.name.clone(),
        }
    }
}

impl<T: ?Sized> fmt::Display for ContextKey<T> {
    // JAVA: net.minecraft.util.context.ContextKey#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<parameter {}>", self.name)
    }
}

/// Minimal self-contained ContextKeySet for ContextMap validation.
/// JAVA: net.minecraft.util.context.ContextKeySet
#[derive(Clone, Debug)]
pub struct ContextKeySet {
    required: HashSet<ErasedContextKey>,
    allowed: HashSet<ErasedContextKey>,
}

impl ContextKeySet {
    // JAVA: net.minecraft.util.context.ContextKeySet#<init>
    pub fn new(required: HashSet<ErasedContextKey>, optional: HashSet<ErasedContextKey>) -> Self {
        let mut allowed = required.clone();
        allowed.extend(optional);
        Self { required, allowed }
    }

    // JAVA: net.minecraft.util.context.ContextKeySet#required
    pub fn required(&self) -> &HashSet<ErasedContextKey> {
        &self.required
    }

    // JAVA: net.minecraft.util.context.ContextKeySet#allowed
    pub fn allowed(&self) -> &HashSet<ErasedContextKey> {
        &self.allowed
    }
}

impl fmt::Display for ContextKeySet {
    // JAVA: net.minecraft.util.context.ContextKeySet#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts: Vec<String> = self
            .allowed
            .iter()
            .map(|k| {
                let prefix = if self.required.contains(k) { "!" } else { "" };
                format!("{}{}", prefix, k.name())
            })
            .collect();
        parts.sort();
        write!(f, "[{}]", parts.join(", "))
    }
}

/// JAVA: net.minecraft.util.context.ContextKeySet$Builder
pub struct ContextKeySetBuilder {
    required: HashSet<ErasedContextKey>,
    optional: HashSet<ErasedContextKey>,
}

impl Default for ContextKeySetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextKeySetBuilder {
    // JAVA: net.minecraft.util.context.ContextKeySet$Builder#<init>
    pub fn new() -> Self {
        Self {
            required: HashSet::new(),
            optional: HashSet::new(),
        }
    }

    // JAVA: net.minecraft.util.context.ContextKeySet$Builder#required
    pub fn required<T: ?Sized>(&mut self, key: &ContextKey<T>) -> &mut Self {
        let erased = key.erased();
        if self.optional.contains(&erased) {
            panic!("Parameter {} is already optional", key.name());
        }
        self.required.insert(erased);
        self
    }

    // JAVA: net.minecraft.util.context.ContextKeySet$Builder#optional
    pub fn optional<T: ?Sized>(&mut self, key: &ContextKey<T>) -> &mut Self {
        let erased = key.erased();
        if self.required.contains(&erased) {
            panic!("Parameter {} is already required", key.name());
        }
        self.optional.insert(erased);
        self
    }

    // JAVA: net.minecraft.util.context.ContextKeySet$Builder#build
    pub fn build(&self) -> ContextKeySet {
        ContextKeySet::new(self.required.clone(), self.optional.clone())
    }
}

/// JAVA: net.minecraft.util.context.ContextMap
pub struct ContextMap {
    params: HashMap<ErasedContextKey, Box<dyn Any>>,
}

impl fmt::Debug for ContextMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut keys: Vec<String> = self.params.keys().map(|k| k.name().to_string()).collect();
        keys.sort();
        f.debug_struct("ContextMap").field("params", &keys).finish()
    }
}

impl ContextMap {
    // JAVA: net.minecraft.util.context.ContextMap#<init>
    pub fn new(params: HashMap<ErasedContextKey, Box<dyn Any>>) -> Self {
        Self { params }
    }

    // JAVA: net.minecraft.util.context.ContextMap#has
    pub fn has<T: ?Sized>(&self, key: &ContextKey<T>) -> bool {
        self.params.contains_key(&key.erased())
    }

    // JAVA: net.minecraft.util.context.ContextMap#getOrThrow
    pub fn get_or_throw<T: 'static>(&self, key: &ContextKey<T>) -> &T {
        self.get_optional(key)
            .unwrap_or_else(|| panic!("{}", key.name()))
    }

    // JAVA: net.minecraft.util.context.ContextMap#getOptional
    pub fn get_optional<T: 'static>(&self, key: &ContextKey<T>) -> Option<&T> {
        self.params
            .get(&key.erased())
            .and_then(|v| v.downcast_ref::<T>())
    }

    // JAVA: net.minecraft.util.context.ContextMap#getOrDefault
    pub fn get_or_default<'a, T: 'static>(
        &'a self,
        key: &ContextKey<T>,
        default: Option<&'a T>,
    ) -> Option<&'a T> {
        match self.get_optional(key) {
            Some(v) => Some(v),
            None => default,
        }
    }
}

/// JAVA: net.minecraft.util.context.ContextMap$Builder
pub struct Builder {
    params: HashMap<ErasedContextKey, Box<dyn Any>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    // JAVA: net.minecraft.util.context.ContextMap$Builder#<init>
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    // JAVA: net.minecraft.util.context.ContextMap$Builder#withParameter
    pub fn with_parameter<T: 'static>(&mut self, key: &ContextKey<T>, value: T) -> &mut Self {
        self.params.insert(key.erased(), Box::new(value));
        self
    }

    // JAVA: net.minecraft.util.context.ContextMap$Builder#withOptionalParameter
    pub fn with_optional_parameter<T: 'static>(
        &mut self,
        key: &ContextKey<T>,
        value: Option<T>,
    ) -> &mut Self {
        let erased = key.erased();
        match value {
            Some(v) => {
                self.params.insert(erased, Box::new(v));
            }
            None => {
                self.params.remove(&erased);
            }
        }
        self
    }

    // JAVA: net.minecraft.util.context.ContextMap$Builder#getParameter
    pub fn get_parameter<T: 'static>(&self, key: &ContextKey<T>) -> &T {
        self.params
            .get(&key.erased())
            .and_then(|v| v.downcast_ref::<T>())
            .unwrap_or_else(|| panic!("{}", key.name()))
    }

    // JAVA: net.minecraft.util.context.ContextMap$Builder#getOptionalParameter
    pub fn get_optional_parameter<T: 'static>(&self, key: &ContextKey<T>) -> Option<&T> {
        self.params
            .get(&key.erased())
            .and_then(|v| v.downcast_ref::<T>())
    }

    // JAVA: net.minecraft.util.context.ContextMap$Builder#create
    pub fn create(&mut self, key_set: &ContextKeySet) -> ContextMap {
        let mut disallowed: Vec<String> = self
            .params
            .keys()
            .filter(|k| !key_set.allowed().contains(*k))
            .map(|k| k.to_string())
            .collect();
        disallowed.sort();
        if !disallowed.is_empty() {
            panic!(
                "Parameters not allowed in this parameter set: {:?}",
                disallowed
            );
        }

        let mut missing: Vec<String> = key_set
            .required()
            .iter()
            .filter(|k| !self.params.contains_key(*k))
            .map(|k| k.to_string())
            .collect();
        missing.sort();
        if !missing.is_empty() {
            panic!("Missing required parameters: {:?}", missing);
        }

        ContextMap {
            params: std::mem::take(&mut self.params),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_and_get_or_throw() {
        let key: ContextKey<i32> = ContextKey::vanilla("value");
        let set = ContextKeySetBuilder::new().required(&key).build();
        let mut builder = Builder::new();
        builder.with_parameter(&key, 42_i32);
        let map = builder.create(&set);
        assert!(map.has(&key));
        assert_eq!(*map.get_or_throw(&key), 42_i32);
    }

    #[test]
    fn get_optional_and_default() {
        let present: ContextKey<i32> = ContextKey::vanilla("present");
        let missing: ContextKey<i32> = ContextKey::vanilla("missing");
        let set = ContextKeySetBuilder::new()
            .required(&present)
            .optional(&missing)
            .build();
        let mut builder = Builder::new();
        builder.with_parameter(&present, 7_i32);
        let map = builder.create(&set);

        assert_eq!(map.get_optional(&present), Some(&7_i32));
        assert_eq!(map.get_optional(&missing), None);
        let default = 99_i32;
        assert_eq!(map.get_or_default(&missing, Some(&default)), Some(&99_i32));
        assert_eq!(map.get_or_default(&present, Some(&default)), Some(&7_i32));
    }

    #[test]
    #[should_panic(expected = "minecraft:missing")]
    fn get_or_throw_panics_when_missing() {
        let key: ContextKey<i32> = ContextKey::vanilla("missing");
        let set = ContextKeySetBuilder::new().optional(&key).build();
        let mut builder = Builder::new();
        let map = builder.create(&set);
        let _ = map.get_or_throw(&key);
    }

    #[test]
    fn with_optional_parameter_removes_on_none() {
        let key: ContextKey<i32> = ContextKey::vanilla("opt");
        let mut builder = Builder::new();
        builder.with_parameter(&key, 1_i32);
        assert_eq!(builder.get_optional_parameter(&key), Some(&1_i32));
        builder.with_optional_parameter::<i32>(&key, None);
        assert_eq!(builder.get_optional_parameter(&key), None);
    }

    #[test]
    #[should_panic(expected = "Missing required parameters")]
    fn create_panics_on_missing_required() {
        let req: ContextKey<i32> = ContextKey::vanilla("req");
        let set = ContextKeySetBuilder::new().required(&req).build();
        let mut builder = Builder::new();
        let _ = builder.create(&set);
    }

    #[test]
    #[should_panic(expected = "Parameters not allowed in this parameter set")]
    fn create_panics_on_disallowed_parameter() {
        let allowed: ContextKey<i32> = ContextKey::vanilla("allowed");
        let extra: ContextKey<i32> = ContextKey::vanilla("extra");
        let set = ContextKeySetBuilder::new().optional(&allowed).build();
        let mut builder = Builder::new();
        builder.with_parameter(&extra, 5_i32);
        let _ = builder.create(&set);
    }
}
