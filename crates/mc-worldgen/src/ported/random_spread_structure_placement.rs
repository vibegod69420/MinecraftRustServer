/// Minimal 3D integer vector used by structure placement.
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

/// Minimal chunk position.
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

/// Minimal interface for structure placement chunk queries.
pub trait ChunkGeneratorStructureState {
    fn get_level_seed(&self) -> i64;
}

/// Minimal random source interface used by spread evaluation.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Java LegacyRandomSource (LCG) compatible random source.
#[derive(Debug, Clone)]
pub struct LegacyRandomSource {
    seed: i64,
}

impl LegacyRandomSource {
    const MULTIPLIER: i64 = 2_521_490_3917_i64;
    const INCREMENT: i64 = 11_i64;
    const MODULUS_MASK: i64 = 281_474_976_710_655_i64; // (1 << 48) - 1

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#LegacyRandomSource
    pub fn new(seed: i64) -> Self {
        let mut s = Self { seed: 0 };
        s.set_seed(seed);
        s
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#setSeed
    pub fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ Self::MULTIPLIER) & Self::MODULUS_MASK;
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#next
    pub fn next(&mut self, bits: i32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT)
            & Self::MODULUS_MASK;
        let shift = (48_i32.wrapping_sub(bits)) & 63;
        ((self.seed as u64) >> shift) as i32
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#nextInt
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if (bound & bound.wrapping_neg()) == bound {
            let n = (bound as i64).wrapping_mul(self.next(31) as i64);
            return (n >> 31) as i32;
        }
        loop {
            let bits = self.next(31);
            let val = bits % bound;
            if bits.wrapping_sub(val).wrapping_add(bound.wrapping_sub(1)) >= 0 {
                return val;
            }
        }
    }
}

impl RandomSource for LegacyRandomSource {
    fn next_int(&mut self, bound: i32) -> i32 {
        self.next_int_bound(bound)
    }
}

/// Java WorldgenRandom compatible wrapper used by structure placement.
#[derive(Debug, Clone)]
pub struct WorldgenRandom {
    random_source: LegacyRandomSource,
}

impl WorldgenRandom {
    // JAVA: net.minecraft.world.level.levelgen.WorldgenRandom#WorldgenRandom
    pub fn new(random_source: LegacyRandomSource) -> Self {
        Self { random_source }
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldgenRandom#setSeed
    pub fn set_seed(&mut self, seed: i64) {
        self.random_source.set_seed(seed);
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldgenRandom#setLargeFeatureWithSalt
    pub fn set_large_feature_with_salt(
        &mut self,
        world_seed: i64,
        region_x: i32,
        region_z: i32,
        salt: i32,
    ) {
        let l = (region_x as i64)
            .wrapping_mul(341_873_128_712_i64)
            .wrapping_add((region_z as i64).wrapping_mul(132_897_987_541_i64))
            .wrapping_add(world_seed)
            .wrapping_add(salt as i64);
        self.set_seed(l);
    }
}

impl RandomSource for WorldgenRandom {
    fn next_int(&mut self, bound: i32) -> i32 {
        self.random_source.next_int_bound(bound)
    }
}

/// Port of `RandomSpreadType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomSpreadType {
    Linear,
    Triangular,
}

impl RandomSpreadType {
    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadType#evaluate
    pub fn evaluate(self, random: &mut impl RandomSource, bound: i32) -> i32 {
        match self {
            Self::Linear => random.next_int(bound),
            Self::Triangular => random
                .next_int(bound)
                .wrapping_add(random.next_int(bound))
                / 2,
        }
    }
}

/// Minimal base placement enums/types for self-contained porting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyReductionMethod {
    Default,
    LegacyType1,
    LegacyType2,
    LegacyType3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExclusionZone;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePlacementType {
    RandomSpread,
}

/// Port of `net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement`.
#[derive(Debug, Clone)]
pub struct RandomSpreadStructurePlacement {
    locate_offset: Vec3i,
    frequency_reduction_method: FrequencyReductionMethod,
    frequency: f32,
    salt: i32,
    exclusion_zone: Option<ExclusionZone>,
    spacing: i32,
    separation: i32,
    spread_type: RandomSpreadType,
}

impl RandomSpreadStructurePlacement {
    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#validate
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.spacing <= self.separation {
            Err("Spacing has to be larger than separation")
        } else {
            Ok(())
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#RandomSpreadStructurePlacement
    pub fn new(
        locate_offset: Vec3i,
        frequency_reduction_method: FrequencyReductionMethod,
        frequency: f32,
        salt: i32,
        exclusion_zone: Option<ExclusionZone>,
        spacing: i32,
        separation: i32,
        spread_type: RandomSpreadType,
    ) -> Self {
        Self {
            locate_offset,
            frequency_reduction_method,
            frequency,
            salt,
            exclusion_zone,
            spacing,
            separation,
            spread_type,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#RandomSpreadStructurePlacement
    pub fn with_defaults(spacing: i32, separation: i32, spread_type: RandomSpreadType, salt: i32) -> Self {
        Self::new(
            Vec3i::ZERO,
            FrequencyReductionMethod::Default,
            1.0_f32,
            salt,
            None,
            spacing,
            separation,
            spread_type,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#spacing
    pub fn spacing(&self) -> i32 {
        self.spacing
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#separation
    pub fn separation(&self) -> i32 {
        self.separation
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#spreadType
    pub fn spread_type(&self) -> RandomSpreadType {
        self.spread_type
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.StructurePlacement#salt
    pub fn salt(&self) -> i32 {
        self.salt
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#getPotentialStructureChunk
    pub fn get_potential_structure_chunk(&self, seed: i64, chunk_x: i32, chunk_z: i32) -> ChunkPos {
        let region_x = chunk_x.div_euclid(self.spacing);
        let region_z = chunk_z.div_euclid(self.spacing);
        let mut random = WorldgenRandom::new(LegacyRandomSource::new(0));
        random.set_large_feature_with_salt(seed, region_x, region_z, self.salt);
        let spread = self.spacing.wrapping_sub(self.separation);
        let offset_x = self.spread_type.evaluate(&mut random, spread);
        let offset_z = self.spread_type.evaluate(&mut random, spread);
        ChunkPos::new(
            region_x.wrapping_mul(self.spacing).wrapping_add(offset_x),
            region_z.wrapping_mul(self.spacing).wrapping_add(offset_z),
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#isPlacementChunk
    pub fn is_placement_chunk(
        &self,
        state: &impl ChunkGeneratorStructureState,
        chunk_x: i32,
        chunk_z: i32,
    ) -> bool {
        let pos = self.get_potential_structure_chunk(state.get_level_seed(), chunk_x, chunk_z);
        pos.x == chunk_x && pos.z == chunk_z
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement#type
    pub fn placement_type(&self) -> StructurePlacementType {
        StructurePlacementType::RandomSpread
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestState(i64);

    impl ChunkGeneratorStructureState for TestState {
        fn get_level_seed(&self) -> i64 {
            self.0
        }
    }

    #[test]
    fn validate_spacing_must_be_greater_than_separation() {
        let invalid_equal =
            RandomSpreadStructurePlacement::with_defaults(10, 10, RandomSpreadType::Linear, 0);
        assert!(invalid_equal.validate().is_err());

        let invalid_smaller =
            RandomSpreadStructurePlacement::with_defaults(10, 11, RandomSpreadType::Linear, 0);
        assert!(invalid_smaller.validate().is_err());

        let valid =
            RandomSpreadStructurePlacement::with_defaults(11, 10, RandomSpreadType::Linear, 0);
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn potential_chunk_is_deterministic_and_within_region() {
        let p = RandomSpreadStructurePlacement::with_defaults(32, 8, RandomSpreadType::Linear, 12345);
        let a = p.get_potential_structure_chunk(987_654_321_i64, 5, -3);
        let b = p.get_potential_structure_chunk(987_654_321_i64, 5, -3);
        assert_eq!(a, b);

        let region_x = 5_i32.div_euclid(32);
        let region_z = (-3_i32).div_euclid(32);
        let dx = a.x.wrapping_sub(region_x.wrapping_mul(32));
        let dz = a.z.wrapping_sub(region_z.wrapping_mul(32));
        assert!(dx >= 0 && dx < 24);
        assert!(dz >= 0 && dz < 24);
    }

    #[test]
    fn negative_coordinates_use_floor_div_semantics() {
        let p = RandomSpreadStructurePlacement::with_defaults(20, 5, RandomSpreadType::Linear, 0);
        let pos = p.get_potential_structure_chunk(1, -1, -20);

        let region_x = (-1_i32).div_euclid(20);
        let region_z = (-20_i32).div_euclid(20);
        assert_eq!(region_x, -1);
        assert_eq!(region_z, -1);

        let dx = pos.x.wrapping_sub(region_x.wrapping_mul(20));
        let dz = pos.z.wrapping_sub(region_z.wrapping_mul(20));
        assert!(dx >= 0 && dx < 15);
        assert!(dz >= 0 && dz < 15);
    }

    #[test]
    fn is_placement_chunk_matches_potential_chunk() {
        let p =
            RandomSpreadStructurePlacement::with_defaults(24, 4, RandomSpreadType::Triangular, 7);
        let state = TestState(42);
        let candidate = p.get_potential_structure_chunk(42, 10, 20);
        assert!(p.is_placement_chunk(&state, candidate.x, candidate.z));

        if candidate.x != 10 || candidate.z != 20 {
            assert!(!p.is_placement_chunk(&state, 10, 20));
        }
    }

    #[test]
    fn float_bits_and_defaults_preserved() {
        let p = RandomSpreadStructurePlacement::with_defaults(10, 5, RandomSpreadType::Linear, 99);
        assert_eq!(p.frequency.to_bits(), 1.0_f32.to_bits());
        assert_eq!(p.locate_offset, Vec3i::ZERO);
        assert_eq!(
            p.frequency_reduction_method,
            FrequencyReductionMethod::Default
        );
        assert_eq!(p.salt(), 99);
        assert!(p.exclusion_zone.is_none());
        assert_eq!(p.placement_type(), StructurePlacementType::RandomSpread);
    }

    #[test]
    fn overflow_and_large_inputs_do_not_panic() {
        let p = RandomSpreadStructurePlacement::with_defaults(
            4096,
            0,
            RandomSpreadType::Triangular,
            i32::MAX,
        );
        let pos = p.get_potential_structure_chunk(i64::MIN, i32::MAX, i32::MIN);
        let _ = (pos.x, pos.z);

        let state = TestState(i64::MAX);
        let _ = p.is_placement_chunk(&state, i32::MIN, i32::MAX);

        let linear = RandomSpreadStructurePlacement::with_defaults(
            4096,
            1,
            RandomSpreadType::Linear,
            i32::MIN,
        );
        let pos2 = linear.get_potential_structure_chunk(i64::MAX, -1_000_000, 1_000_000);
        let spread = linear.spacing().wrapping_sub(linear.separation());
        let region_x = (-1_000_000_i32).div_euclid(linear.spacing());
        let dx = pos2.x.wrapping_sub(region_x.wrapping_mul(linear.spacing()));
        assert!(dx >= 0 && dx < spread);
    }
}
