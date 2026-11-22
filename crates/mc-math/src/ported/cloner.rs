use std::fmt;

/// Self-contained stand-in for a registry key used by `Cloner::Factory`.
///
/// Java `Cloner` is generic over codec/serialization infrastructure that is not
/// present in `mc-math`. This faithful structural port preserves the factory's
/// per-registry-key cloner storage and lookup behavior, while representing
/// cloning as an explicit fallible operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    identifier: String,
}

impl ResourceKey {
    // JAVA: net.minecraft.resources.ResourceKey
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
        }
    }

    // JAVA: net.minecraft.resources.ResourceKey#identifier
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloneError {
    message: String,
}

impl CloneError {
    // JAVA: net.minecraft.core.Cloner#clone (IllegalStateException message)
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CloneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CloneError {}

/// Minimal provider stand-in for Java `HolderLookup.Provider`.
///
/// The original Java implementation creates serialization contexts from these
/// providers. In this self-contained port, providers are opaque markers passed
/// through the clone operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HolderLookupProvider;

/// A cloner for values of type `T`.
///
/// Java uses Mojang Codec encode/decode with dynamic ops. This port keeps the
/// same class shape and fallible clone contract by delegating to an explicit
/// clone function.
pub struct Cloner<T> {
    clone_fn: fn(&T, HolderLookupProvider, HolderLookupProvider) -> Result<T, CloneError>,
}

impl<T> fmt::Debug for Cloner<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cloner").finish_non_exhaustive()
    }
}

impl<T> Cloner<T> {
    // JAVA: net.minecraft.core.Cloner#<init>
    pub fn new(
        clone_fn: fn(&T, HolderLookupProvider, HolderLookupProvider) -> Result<T, CloneError>,
    ) -> Self {
        Self { clone_fn }
    }

    // JAVA: net.minecraft.core.Cloner#clone
    pub fn clone(
        &self,
        value: &T,
        source: HolderLookupProvider,
        target: HolderLookupProvider,
    ) -> Result<T, CloneError> {
        (self.clone_fn)(value, source, target)
    }
}

/// Factory for registry-keyed cloners.
pub struct Factory<T> {
    codecs: std::collections::HashMap<ResourceKey, Cloner<T>>,
}

impl<T> fmt::Debug for Factory<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Factory")
            .field("len", &self.codecs.len())
            .finish_non_exhaustive()
    }
}

impl<T> Default for Factory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Factory<T> {
    // JAVA: net.minecraft.core.Cloner$Factory#<init> (implicit)
    pub fn new() -> Self {
        Self {
            codecs: std::collections::HashMap::new(),
        }
    }

    // JAVA: net.minecraft.core.Cloner$Factory#addCodec
    pub fn add_codec(
        &mut self,
        key: ResourceKey,
        clone_fn: fn(&T, HolderLookupProvider, HolderLookupProvider) -> Result<T, CloneError>,
    ) -> &mut Self {
        self.codecs.insert(key, Cloner::new(clone_fn));
        self
    }

    // JAVA: net.minecraft.core.Cloner$Factory#cloner
    pub fn cloner(&self, key: &ResourceKey) -> Option<&Cloner<T>> {
        self.codecs.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy_i32(
        value: &i32,
        _source: HolderLookupProvider,
        _target: HolderLookupProvider,
    ) -> Result<i32, CloneError> {
        Ok(*value)
    }

    fn fail_i32(
        _value: &i32,
        _source: HolderLookupProvider,
        _target: HolderLookupProvider,
    ) -> Result<i32, CloneError> {
        Err(CloneError::new("Failed to encode: boom"))
    }

    #[test]
    fn factory_add_and_lookup() {
        let mut factory = Factory::<i32>::new();
        let key = ResourceKey::new("minecraft:block");
        factory.add_codec(key.clone(), copy_i32);

        let cloner = factory.cloner(&key).expect("cloner should exist");
        let out = cloner
            .clone(&42, HolderLookupProvider, HolderLookupProvider)
            .expect("clone should succeed");
        assert_eq!(out, 42);
    }

    #[test]
    fn factory_missing_returns_none() {
        let factory = Factory::<i32>::new();
        let key = ResourceKey::new("minecraft:missing");
        assert!(factory.cloner(&key).is_none());
    }

    #[test]
    fn cloner_propagates_error() {
        let cloner = Cloner::new(fail_i32);
        let err = cloner
            .clone(&7, HolderLookupProvider, HolderLookupProvider)
            .expect_err("clone should fail");
        assert_eq!(err.message(), "Failed to encode: boom");
        assert_eq!(err.to_string(), "Failed to encode: boom");
    }

    #[test]
    fn factory_replaces_existing_codec() {
        let mut factory = Factory::<i32>::new();
        let key = ResourceKey::new("minecraft:item");
        factory.add_codec(key.clone(), fail_i32);
        factory.add_codec(key.clone(), copy_i32);

        let cloner = factory.cloner(&key).expect("cloner should exist");
        let out = cloner
            .clone(&-5, HolderLookupProvider, HolderLookupProvider)
            .expect("replacement codec should succeed");
        assert_eq!(out, -5);
    }
}
