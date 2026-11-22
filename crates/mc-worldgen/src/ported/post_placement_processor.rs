use std::fmt::Debug;

/// Minimal local stand-in for `net.minecraft.world.level.WorldGenLevel`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldGenLevel;

/// Minimal local stand-in for `net.minecraft.world.level.StructureManager`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructureManager;

/// Minimal local stand-in for `net.minecraft.world.level.chunk.ChunkGenerator`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkGenerator;

/// Minimal local stand-in for `net.minecraft.util.RandomSource`.
pub trait RandomSource: Debug {}

/// Minimal local stand-in for `net.minecraft.world.level.levelgen.structure.BoundingBox`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

/// Minimal local stand-in for `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Minimal local stand-in for
/// `net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PiecesContainer;

/// Port of `net.minecraft.world.level.levelgen.structure.PostPlacementProcessor`.
pub trait PostPlacementProcessor: Debug {
    // JAVA: net.minecraft.world.level.levelgen.structure.PostPlacementProcessor#afterPlace
    fn after_place(
        &self,
        level: &mut WorldGenLevel,
        structure_manager: &StructureManager,
        chunk_generator: &ChunkGenerator,
        random: &mut dyn RandomSource,
        bounding_box: &BoundingBox,
        chunk_pos: ChunkPos,
        pieces: &PiecesContainer,
    );
}

/// No-op implementation corresponding to `PostPlacementProcessor.NONE`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NonePostPlacementProcessor;

impl PostPlacementProcessor for NonePostPlacementProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.PostPlacementProcessor#afterPlace
    fn after_place(
        &self,
        _level: &mut WorldGenLevel,
        _structure_manager: &StructureManager,
        _chunk_generator: &ChunkGenerator,
        _random: &mut dyn RandomSource,
        _bounding_box: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pieces: &PiecesContainer,
    ) {
        // NONE: no-op, matching Java lambda ($$0, ..., $$6) -> {}
    }
}

/// Port of `PostPlacementProcessor.NONE`.
pub const NONE: NonePostPlacementProcessor = NonePostPlacementProcessor;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct SimpleRandomSource {
        seed: i64,
    }

    impl RandomSource for SimpleRandomSource {}

    #[test]
    fn none_after_place_is_noop() {
        let mut level = WorldGenLevel::default();
        let sm = StructureManager::default();
        let cg = ChunkGenerator::default();
        let mut rng = SimpleRandomSource { seed: 12345 };
        let rng_before = rng.clone();
        let bb = BoundingBox {
            min_x: -16,
            min_y: -64,
            min_z: 32,
            max_x: 16,
            max_y: 320,
            max_z: 64,
        };
        let pos = ChunkPos { x: -7, z: 13 };
        let pieces = PiecesContainer::default();

        NONE.after_place(&mut level, &sm, &cg, &mut rng, &bb, pos, &pieces);

        assert_eq!(level, WorldGenLevel::default());
        assert_eq!(rng, rng_before);
        assert_eq!(bb.min_x, -16);
        assert_eq!(pos.x, -7);
    }

    #[test]
    fn negative_and_extreme_integer_inputs() {
        let bb = BoundingBox {
            min_x: i32::MIN,
            min_y: -1,
            min_z: i32::MIN,
            max_x: i32::MAX,
            max_y: -320,
            max_z: i32::MAX,
        };
        let pos = ChunkPos {
            x: i32::MIN,
            z: -999_999,
        };

        assert_eq!(bb.min_x, i32::MIN);
        assert_eq!(bb.max_x, i32::MAX);
        assert_eq!(bb.max_y, -320);
        assert_eq!(pos.x, i32::MIN);
        assert_eq!(pos.z, -999_999);
    }

    #[test]
    fn integer_overflow_wrapping_semantics() {
        let pos = ChunkPos {
            x: i32::MAX,
            z: i32::MIN,
        };

        let wrapped_x = pos.x.wrapping_add(1);
        let wrapped_z = pos.z.wrapping_sub(1);
        let wrapped_mul = (-2_i32).wrapping_mul(i32::MIN);

        assert_eq!(wrapped_x, i32::MIN);
        assert_eq!(wrapped_z, i32::MAX);
        assert_eq!(wrapped_mul, 0);

        let shift_mask_31 = (1_i32 << (33_u32 & 31)) as i32;
        let shift_mask_63 = (1_i64 << (65_u32 & 63)) as i64;
        assert_eq!(shift_mask_31, 2);
        assert_eq!(shift_mask_63, 2);
    }

    #[test]
    fn float_bits_are_preserved() {
        let nan_f32_bits: u32 = 0x7FC0_0001;
        let nan_f32 = f32::from_bits(nan_f32_bits);
        assert_eq!(nan_f32.to_bits(), nan_f32_bits);

        let neg_zero_f32 = (-0.0_f32).to_bits();
        assert_eq!(neg_zero_f32, 0x8000_0000);
        assert_ne!(neg_zero_f32, 0.0_f32.to_bits());

        let nan_f64_bits: u64 = 0x7FF8_0000_0000_0001;
        let nan_f64 = f64::from_bits(nan_f64_bits);
        assert_eq!(nan_f64.to_bits(), nan_f64_bits);
    }

    #[test]
    fn none_can_be_used_through_trait_object() {
        let mut level = WorldGenLevel::default();
        let sm = StructureManager::default();
        let cg = ChunkGenerator::default();
        let mut rng = SimpleRandomSource { seed: i64::MIN };
        let rng_before = rng.clone();
        let bb = BoundingBox::default();
        let pos = ChunkPos {
            x: i32::MAX,
            z: i32::MIN,
        };
        let pieces = PiecesContainer::default();

        let processor: &dyn PostPlacementProcessor = &NONE;
        processor.after_place(&mut level, &sm, &cg, &mut rng, &bb, pos, &pieces);

        assert_eq!(level, WorldGenLevel::default());
        assert_eq!(rng, rng_before);
        assert_eq!(pos.x.wrapping_add(1), i32::MIN);
    }
}
