//! Port of `net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement` (1.21.11).

use std::marker::PhantomData;

/// Minimal placeholder for `net.minecraft.core.Vec3i`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Minimal placeholder for
/// `net.minecraft.world.level.levelgen.structure.placement.StructurePlacement.FrequencyReductionMethod`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyReductionMethod {
    Default,
    LegacyType1,
    LegacyType2,
    LegacyType3,
}

/// Minimal placeholder for
/// `net.minecraft.world.level.levelgen.structure.placement.StructurePlacement.ExclusionZone`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExclusionZone;

/// Minimal placeholder for `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

/// Minimal placeholder for
/// `net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePlacementType {
    ConcentricRings,
}

/// Minimal placeholder for `net.minecraft.world.level.biome.Biome`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biome;

/// Minimal placeholder for `net.minecraft.core.HolderSet<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T> {
    _phantom: PhantomData<T>,
}

impl<T> HolderSet<T> {
    pub const fn empty() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Minimal placeholder for `net.minecraft.world.level.chunk.ChunkGeneratorStructureState`.
pub trait ChunkGeneratorStructureState<H> {
    fn get_ring_positions_for(
        &self,
        placement: &ConcentricRingsStructurePlacement<H>,
    ) -> Option<Vec<ChunkPos>>;
}

/// Port of `net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement`.
///
/// Generic over `H` to represent `HolderSet<Biome>` without requiring external registry types.
#[derive(Debug, Clone)]
pub struct ConcentricRingsStructurePlacement<H> {
    // Base StructurePlacement fields
    locate_offset: Vec3i,
    frequency_reduction_method: FrequencyReductionMethod,
    frequency: f32,
    salt: i32,
    exclusion_zone: Option<ExclusionZone>,
    // ConcentricRingsStructurePlacement fields
    distance: i32,
    spread: i32,
    count: i32,
    preferred_biomes: H,
}

impl<H> ConcentricRingsStructurePlacement<H> {
    // JAVA: net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement#<init>
    pub fn new(
        locate_offset: Vec3i,
        frequency_reduction_method: FrequencyReductionMethod,
        frequency: f32,
        salt: i32,
        exclusion_zone: Option<ExclusionZone>,
        distance: i32,
        spread: i32,
        count: i32,
        preferred_biomes: H,
    ) -> Self {
        Self {
            locate_offset,
            frequency_reduction_method,
            frequency,
            salt,
            exclusion_zone,
            distance,
            spread,
            count,
            preferred_biomes,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement#<init>
    pub fn new_simple(distance: i32, spread: i32, count: i32, preferred_biomes: H) -> Self {
        Self::new(
            Vec3i::ZERO,
            FrequencyReductionMethod::Default,
            1.0_f32,
            0,
            None,
            distance,
            spread,
            count,
            preferred_biomes,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement#distance
    pub fn distance(&self) -> i32 {
        self.distance
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement#spread
    pub fn spread(&self) -> i32 {
        self.spread
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement#count
    pub fn count(&self) -> i32 {
        self.count
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement#preferredBiomes
    pub fn preferred_biomes(&self) -> &H {
        &self.preferred_biomes
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement#isPlacementChunk
    pub fn is_placement_chunk<S: ChunkGeneratorStructureState<H>>(
        &self,
        state: &S,
        x: i32,
        z: i32,
    ) -> bool {
        match state.get_ring_positions_for(self) {
            None => false,
            Some(positions) => positions.contains(&ChunkPos::new(x, z)),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement#type
    pub fn placement_type(&self) -> StructurePlacementType {
        StructurePlacementType::ConcentricRings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyBiomes(u32);

    struct MockState {
        positions: Option<Vec<ChunkPos>>,
    }

    impl ChunkGeneratorStructureState<DummyBiomes> for MockState {
        fn get_ring_positions_for(
            &self,
            _placement: &ConcentricRingsStructurePlacement<DummyBiomes>,
        ) -> Option<Vec<ChunkPos>> {
            self.positions.clone()
        }
    }

    #[test]
    fn new_simple_sets_defaults_and_accessors() {
        let p = ConcentricRingsStructurePlacement::new_simple(32, 3, 128, DummyBiomes(7));
        assert_eq!(p.distance(), 32);
        assert_eq!(p.spread(), 3);
        assert_eq!(p.count(), 128);
        assert_eq!(p.preferred_biomes(), &DummyBiomes(7));
        assert_eq!(p.locate_offset, Vec3i::ZERO);
        assert_eq!(
            p.frequency_reduction_method,
            FrequencyReductionMethod::Default
        );
        assert_eq!(p.frequency.to_bits(), 1.0_f32.to_bits());
        assert_eq!(p.salt, 0);
        assert_eq!(p.exclusion_zone, None);
        assert_eq!(
            p.placement_type(),
            StructurePlacementType::ConcentricRings
        );
    }

    #[test]
    fn full_constructor_preserves_fields_negative_inputs_and_float_bits() {
        let freq = f32::from_bits(0x3F8C_CCCD); // ~1.1
        let p = ConcentricRingsStructurePlacement::new(
            Vec3i::new(-1, 2, -3),
            FrequencyReductionMethod::LegacyType1,
            freq,
            i32::MIN,
            Some(ExclusionZone),
            i32::MAX,
            -100,
            4095,
            DummyBiomes(99),
        );
        assert_eq!(p.distance(), i32::MAX);
        assert_eq!(p.spread(), -100);
        assert_eq!(p.count(), 4095);
        assert_eq!(p.preferred_biomes(), &DummyBiomes(99));
        assert_eq!(p.locate_offset, Vec3i::new(-1, 2, -3));
        assert_eq!(
            p.frequency_reduction_method,
            FrequencyReductionMethod::LegacyType1
        );
        assert_eq!(p.frequency.to_bits(), freq.to_bits());
        assert_eq!(p.salt, i32::MIN);
        assert_eq!(p.exclusion_zone, Some(ExclusionZone));
    }

    #[test]
    fn is_placement_chunk_returns_false_when_no_ring_positions() {
        let p = ConcentricRingsStructurePlacement::new_simple(1, 1, 1, DummyBiomes(0));
        let state = MockState { positions: None };
        assert!(!p.is_placement_chunk(&state, 0, 0));
        assert!(!p.is_placement_chunk(&state, -5, 10));
    }

    #[test]
    fn is_placement_chunk_checks_ring_positions_with_negative_coords() {
        let p = ConcentricRingsStructurePlacement::new_simple(8, 2, 4, DummyBiomes(1));
        let state = MockState {
            positions: Some(vec![
                ChunkPos::new(-3, 7),
                ChunkPos::new(0, 0),
                ChunkPos::new(12, -4),
            ]),
        };
        assert!(p.is_placement_chunk(&state, -3, 7));
        assert!(p.is_placement_chunk(&state, 0, 0));
        assert!(p.is_placement_chunk(&state, 12, -4));
        assert!(!p.is_placement_chunk(&state, -3, 8));
        assert!(!p.is_placement_chunk(&state, 1, 0));
    }

    #[test]
    fn is_placement_chunk_returns_false_for_empty_positions() {
        let p = ConcentricRingsStructurePlacement::new_simple(0, 0, 1, DummyBiomes(0));
        let state = MockState {
            positions: Some(vec![]),
        };
        assert!(!p.is_placement_chunk(&state, 0, 0));
        assert!(!p.is_placement_chunk(&state, i32::MIN, i32::MAX));
    }
}
