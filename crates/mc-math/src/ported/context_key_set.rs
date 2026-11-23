use std::collections::HashSet;
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

/// Type-erased context key handle used by ContextKeySet.
/// Preserves identity semantics via unique id.
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

/// JAVA: net.minecraft.util.context.ContextKeySet
#[derive(Clone, Debug)]
pub struct ContextKeySet {
    required: HashSet<ErasedContextKey>,
    allowed: HashSet<ErasedContextKey>,
}

impl ContextKeySet {
    // JAVA: net.minecraft.util.context.ContextKeySet#<init>
    pub fn new(required: HashSet<ErasedContextKey>, optional: HashSet<ErasedContextKey>) -> Self {
        let required = required;
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
pub struct Builder {
    required: HashSet<ErasedContextKey>,
    optional: HashSet<ErasedContextKey>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_builds_required_and_allowed() {
        let a: ContextKey<i32> = ContextKey::vanilla("a");
        let b: ContextKey<i32> = ContextKey::vanilla("b");
        let set = Builder::new().required(&a).optional(&b).build();
        assert!(set.required().contains(&a.erased()));
        assert!(!set.required().contains(&b.erased()));
        assert!(set.allowed().contains(&a.erased()));
        assert!(set.allowed().contains(&b.erased()));
        assert_eq!(set.required().len(), 1);
        assert_eq!(set.allowed().len(), 2);
    }

    #[test]
    fn to_string_marks_required_with_exclamation() {
        let a: ContextKey<i32> = ContextKey::vanilla("alpha");
        let b: ContextKey<i32> = ContextKey::vanilla("beta");
        let set = Builder::new().required(&a).optional(&b).build();
        assert_eq!(set.to_string(), "[!minecraft:alpha, minecraft:beta]");
    }

    #[test]
    #[should_panic(expected = "is already optional")]
    fn required_rejects_optional_key() {
        let a: ContextKey<i32> = ContextKey::vanilla("x");
        let mut b = Builder::new();
        b.optional(&a);
        b.required(&a);
    }

    #[test]
    #[should_panic(expected = "is already required")]
    fn optional_rejects_required_key() {
        let a: ContextKey<i32> = ContextKey::vanilla("x");
        let mut b = Builder::new();
        b.required(&a);
        b.optional(&a);
    }

    #[test]
    fn empty_builder_produces_empty_set() {
        let set = Builder::new().build();
        assert!(set.required().is_empty());
        assert!(set.allowed().is_empty());
        assert_eq!(set.to_string(), "[]");
    }
}
