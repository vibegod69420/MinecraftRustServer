//! Faithful port of `net.minecraft.core.RegistryCodecs`.

use std::marker::PhantomData;

/// Minimal self-contained stub for `net.minecraft.resources.ResourceKey`.
#[derive(Debug, Clone)]
pub struct ResourceKey<T> {
    _marker: PhantomData<T>,
}

impl<T> ResourceKey<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for ResourceKey<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal self-contained stub for `net.minecraft.core.Registry`.
#[derive(Debug, Clone)]
pub struct Registry<T> {
    _marker: PhantomData<T>,
}

/// Minimal self-contained stub for `net.minecraft.core.Holder`.
#[derive(Debug, Clone)]
pub struct Holder<E> {
    _marker: PhantomData<E>,
}

/// Minimal self-contained stub for `net.minecraft.core.HolderSet`.
#[derive(Debug, Clone)]
pub struct HolderSet<E> {
    _marker: PhantomData<E>,
}

/// Minimal self-contained stub for `com.mojang.serialization.Codec`.
#[derive(Debug, Clone)]
pub struct Codec<T> {
    _marker: PhantomData<T>,
    pub kind: &'static str,
    pub allow_inline: bool,
}

impl<T> Codec<T> {
    fn new(kind: &'static str, allow_inline: bool) -> Self {
        Self {
            _marker: PhantomData,
            kind,
            allow_inline,
        }
    }
}

/// Minimal self-contained stub for `net.minecraft.resources.HolderSetCodec`.
struct HolderSetCodec;

impl HolderSetCodec {
    fn create<E>(
        _registry: &ResourceKey<Registry<E>>,
        _holder_codec: Codec<Holder<E>>,
        allow_inline: bool,
    ) -> Codec<HolderSet<E>> {
        Codec::new("holder_set", allow_inline)
    }
}

/// Minimal self-contained stub for `net.minecraft.resources.RegistryFileCodec`.
struct RegistryFileCodec;

impl RegistryFileCodec {
    fn create<E>(
        _registry: &ResourceKey<Registry<E>>,
        _element_codec: Codec<E>,
    ) -> Codec<Holder<E>> {
        Codec::new("registry_file", false)
    }
}

/// Minimal self-contained stub for `net.minecraft.resources.RegistryFixedCodec`.
struct RegistryFixedCodec;

impl RegistryFixedCodec {
    fn create<E>(_registry: &ResourceKey<Registry<E>>) -> Codec<Holder<E>> {
        Codec::new("registry_fixed", false)
    }
}

// JAVA: net.minecraft.core.RegistryCodecs#homogeneousList
pub fn homogeneous_list<E>(
    registry: &ResourceKey<Registry<E>>,
    element_codec: Codec<E>,
) -> Codec<HolderSet<E>> {
    homogeneous_list_with_inline(registry, element_codec, false)
}

// JAVA: net.minecraft.core.RegistryCodecs#homogeneousList
pub fn homogeneous_list_with_inline<E>(
    registry: &ResourceKey<Registry<E>>,
    element_codec: Codec<E>,
    allow_inline: bool,
) -> Codec<HolderSet<E>> {
    HolderSetCodec::create(registry, RegistryFileCodec::create(registry, element_codec), allow_inline)
}

// JAVA: net.minecraft.core.RegistryCodecs#homogeneousList
pub fn homogeneous_list_direct<E>(registry: &ResourceKey<Registry<E>>) -> Codec<HolderSet<E>> {
    homogeneous_list_direct_with_inline(registry, false)
}

// JAVA: net.minecraft.core.RegistryCodecs#homogeneousList
pub fn homogeneous_list_direct_with_inline<E>(
    registry: &ResourceKey<Registry<E>>,
    allow_inline: bool,
) -> Codec<HolderSet<E>> {
    HolderSetCodec::create(registry, RegistryFixedCodec::create(registry), allow_inline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn homogeneous_list_defaults_to_no_inline() {
        let reg: ResourceKey<Registry<i32>> = ResourceKey::new();
        let elem: Codec<i32> = Codec::new("element", false);
        let codec = homogeneous_list(&reg, elem);
        assert_eq!(codec.kind, "holder_set");
        assert!(!codec.allow_inline);
    }

    #[test]
    fn homogeneous_list_with_inline_propagates_flag() {
        let reg: ResourceKey<Registry<i32>> = ResourceKey::new();
        let elem: Codec<i32> = Codec::new("element", false);
        let codec = homogeneous_list_with_inline(&reg, elem, true);
        assert_eq!(codec.kind, "holder_set");
        assert!(codec.allow_inline);
    }

    #[test]
    fn homogeneous_list_direct_defaults_to_no_inline() {
        let reg: ResourceKey<Registry<i32>> = ResourceKey::new();
        let codec = homogeneous_list_direct::<i32>(&reg);
        assert_eq!(codec.kind, "holder_set");
        assert!(!codec.allow_inline);
    }

    #[test]
    fn homogeneous_list_direct_with_inline_propagates_flag() {
        let reg: ResourceKey<Registry<i32>> = ResourceKey::new();
        let codec = homogeneous_list_direct_with_inline::<i32>(&reg, true);
        assert_eq!(codec.kind, "holder_set");
        assert!(codec.allow_inline);
    }

    #[test]
    fn element_and_direct_variants_both_produce_holder_set_codec() {
        let reg: ResourceKey<Registry<i32>> = ResourceKey::new();
        let elem: Codec<i32> = Codec::new("element", false);
        let a = homogeneous_list_with_inline(&reg, elem, false);
        let b = homogeneous_list_direct_with_inline::<i32>(&reg, false);
        assert_eq!(a.kind, b.kind);
        assert_eq!(a.allow_inline, b.allow_inline);
    }
}
