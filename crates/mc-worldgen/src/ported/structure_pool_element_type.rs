//! Port of `net.minecraft.world.level.levelgen.structure.pools.StructurePoolElementType`.

use std::marker::PhantomData;

/// Marker trait for unported `StructurePoolElement` subtypes.
pub trait StructurePoolElement: 'static {}

/// Marker for unported `SinglePoolElement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SinglePoolElement;
impl StructurePoolElement for SinglePoolElement {}

/// Marker for unported `ListPoolElement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListPoolElement;
impl StructurePoolElement for ListPoolElement {}

/// Marker for unported `FeaturePoolElement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeaturePoolElement;
impl StructurePoolElement for FeaturePoolElement {}

/// Marker for unported `EmptyPoolElement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyPoolElement;
impl StructurePoolElement for EmptyPoolElement {}

/// Marker for unported `LegacySinglePoolElement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LegacySinglePoolElement;
impl StructurePoolElement for LegacySinglePoolElement {}

/// Placeholder for Mojang `MapCodec<P>` (codec infrastructure not yet ported).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<P> {
    _marker: PhantomData<fn() -> P>,
}

impl<P> MapCodec<P> {
    pub const fn placeholder() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Port of `StructurePoolElementType<P extends StructurePoolElement>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructurePoolElementType<P: StructurePoolElement> {
    name: &'static str,
    codec: MapCodec<P>,
}

impl<P: StructurePoolElement> StructurePoolElementType<P> {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElementType#codec
    pub const fn codec(&self) -> &MapCodec<P> {
        &self.codec
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElementType#register
    pub const fn register(name: &'static str, codec: MapCodec<P>) -> Self {
        // Registry.register(BuiltInRegistries.STRUCTURE_POOL_ELEMENT, name, () -> codec) omitted.
        Self { name, codec }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}

pub const SINGLE: StructurePoolElementType<SinglePoolElement> =
    StructurePoolElementType::register("single_pool_element", MapCodec::placeholder());
pub const LIST: StructurePoolElementType<ListPoolElement> =
    StructurePoolElementType::register("list_pool_element", MapCodec::placeholder());
pub const FEATURE: StructurePoolElementType<FeaturePoolElement> =
    StructurePoolElementType::register("feature_pool_element", MapCodec::placeholder());
pub const EMPTY: StructurePoolElementType<EmptyPoolElement> =
    StructurePoolElementType::register("empty_pool_element", MapCodec::placeholder());
pub const LEGACY: StructurePoolElementType<LegacySinglePoolElement> =
    StructurePoolElementType::register("legacy_single_pool_element", MapCodec::placeholder());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_names_match_java() {
        assert_eq!(SINGLE.name(), "single_pool_element");
        assert_eq!(LIST.name(), "list_pool_element");
        assert_eq!(FEATURE.name(), "feature_pool_element");
        assert_eq!(EMPTY.name(), "empty_pool_element");
        assert_eq!(LEGACY.name(), "legacy_single_pool_element");
    }

    #[test]
    fn register_creates_type_with_name() {
        let t = StructurePoolElementType::<SinglePoolElement>::register(
            "custom",
            MapCodec::placeholder(),
        );
        assert_eq!(t.name(), "custom");
    }

    #[test]
    fn codec_accessor_returns_codec() {
        let t = SINGLE;
        let codec: &MapCodec<SinglePoolElement> = t.codec();
        assert_eq!(*codec, MapCodec::placeholder());
    }

    #[test]
    fn same_name_and_codec_are_equal() {
        let a = StructurePoolElementType::<EmptyPoolElement>::register(
            "empty_pool_element",
            MapCodec::placeholder(),
        );
        assert_eq!(a, EMPTY);
    }

    #[test]
    fn different_registered_names_are_not_equal() {
        let a = StructurePoolElementType::<SinglePoolElement>::register(
            "a",
            MapCodec::placeholder(),
        );
        let b = StructurePoolElementType::<SinglePoolElement>::register(
            "b",
            MapCodec::placeholder(),
        );
        assert_ne!(a, b);
    }
}
