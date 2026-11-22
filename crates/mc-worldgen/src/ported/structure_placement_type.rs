//! Port of `net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType`.
//!
//! This module is self-contained: registry and codec infrastructure from Minecraft/DFU
//! are represented with minimal local placeholders.

use std::marker::PhantomData;

/// Minimal marker for `net.minecraft.world.level.levelgen.structure.placement.StructurePlacement`.
pub trait StructurePlacement {}

/// Minimal placeholder for
/// `net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RandomSpreadStructurePlacement;

impl StructurePlacement for RandomSpreadStructurePlacement {}

/// Minimal placeholder for
/// `net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConcentricRingsStructurePlacement;

impl StructurePlacement for ConcentricRingsStructurePlacement {}

/// Minimal placeholder for `com.mojang.serialization.MapCodec<SP>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<SP>(PhantomData<fn() -> SP>);

impl<SP> MapCodec<SP> {
    /// Creates a new placeholder codec.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<SP> Default for MapCodec<SP> {
    fn default() -> Self {
        Self::new()
    }
}

/// Port of `net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType`.
///
/// In Java this is a functional interface whose single abstract method returns a `MapCodec<SP>`.
/// Registry registration is represented here as a self-contained static constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructurePlacementType<SP: StructurePlacement> {
    name: &'static str,
    codec: MapCodec<SP>,
}

impl<SP: StructurePlacement> StructurePlacementType<SP> {
    // JAVA: net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType#codec
    pub const fn codec(&self) -> &MapCodec<SP> {
        &self.codec
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType#register
    const fn register(name: &'static str, codec: MapCodec<SP>) -> Self {
        Self { name, codec }
    }

    /// Returns the registry name used when this type was registered.
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType#RANDOM_SPREAD
pub const RANDOM_SPREAD: StructurePlacementType<RandomSpreadStructurePlacement> =
    StructurePlacementType::register("random_spread", MapCodec::new());

// JAVA: net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType#CONCENTRIC_RINGS
pub const CONCENTRIC_RINGS: StructurePlacementType<ConcentricRingsStructurePlacement> =
    StructurePlacementType::register("concentric_rings", MapCodec::new());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_spread_has_expected_name_and_codec() {
        assert_eq!(RANDOM_SPREAD.name(), "random_spread");
        assert_eq!(
            *RANDOM_SPREAD.codec(),
            MapCodec::<RandomSpreadStructurePlacement>::new()
        );
    }

    #[test]
    fn concentric_rings_has_expected_name_and_codec() {
        assert_eq!(CONCENTRIC_RINGS.name(), "concentric_rings");
        assert_eq!(
            *CONCENTRIC_RINGS.codec(),
            MapCodec::<ConcentricRingsStructurePlacement>::new()
        );
    }

    #[test]
    fn register_preserves_name_and_codec() {
        let codec = MapCodec::<RandomSpreadStructurePlacement>::new();
        let placement_type = StructurePlacementType::register("custom", codec);
        assert_eq!(placement_type.name(), "custom");
        assert_eq!(*placement_type.codec(), codec);
    }

    #[test]
    fn known_constants_have_distinct_names() {
        assert_ne!(RANDOM_SPREAD.name(), CONCENTRIC_RINGS.name());
    }

    #[test]
    fn map_codec_default_matches_new() {
        let from_new = MapCodec::<ConcentricRingsStructurePlacement>::new();
        let from_default = MapCodec::<ConcentricRingsStructurePlacement>::default();
        assert_eq!(from_new, from_default);
    }
}
