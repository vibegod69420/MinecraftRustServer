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

    /// Returns the identity id used for identity-hash semantics.
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl<T: ?Sized> fmt::Display for ContextKey<T> {
    // JAVA: net.minecraft.util.context.ContextKey#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<parameter {}>", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_creates_minecraft_identifier() {
        let key: ContextKey<i32> = ContextKey::vanilla("test");
        assert_eq!(key.name().namespace, "minecraft");
        assert_eq!(key.name().path, "test");
        assert_eq!(key.to_string(), "<parameter minecraft:test>");
    }

    #[test]
    fn new_uses_given_identifier() {
        let id = Identifier {
            namespace: "custom".to_string(),
            path: "value".to_string(),
        };
        let key: ContextKey<String> = ContextKey::new(id.clone());
        assert_eq!(key.name(), &id);
        assert_eq!(key.to_string(), "<parameter custom:value>");
    }

    #[test]
    fn identity_semantics_different_instances_not_equal() {
        let a: ContextKey<i32> = ContextKey::vanilla("same");
        let b: ContextKey<i32> = ContextKey::vanilla("same");
        assert_ne!(a, b);
        assert_ne!(a.id(), b.id());
        assert_eq!(a.name(), b.name());
    }

    #[test]
    fn clone_preserves_identity() {
        let a: ContextKey<i32> = ContextKey::vanilla("clone");
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(a.id(), b.id());
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn hash_uses_identity() {
        use std::collections::HashSet;
        let a: ContextKey<i32> = ContextKey::vanilla("hash");
        let b = a.clone();
        let c: ContextKey<i32> = ContextKey::vanilla("hash");
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2);
    }
}
