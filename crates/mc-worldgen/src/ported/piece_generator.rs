//! Port of `net.minecraft.world.level.levelgen.structure.pieces.PieceGenerator`.

/// Minimal stub for `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration`.
pub trait FeatureConfiguration {}

/// Minimal stub for `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Minimal stub for `net.minecraft.world.level.chunk.ChunkGenerator`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkGenerator;

/// Minimal stub for `net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StructureTemplateManager;

/// Minimal stub for `net.minecraft.world.level.LevelHeightAccessor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LevelHeightAccessor {
    pub min_y: i32,
    pub height: i32,
}

/// Minimal stub for `net.minecraft.world.level.levelgen.WorldgenRandom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldgenRandom {
    pub seed: i64,
}

/// Minimal stub for `net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructurePiecesBuilder {
    pub piece_count: u32,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self { piece_count: 0 }
    }
}

/// Port of `net.minecraft.world.level.levelgen.structure.pieces.PieceGenerator.Context`.
///
/// Java record: `Context<C extends FeatureConfiguration>(C config, ChunkGenerator chunkGenerator,
/// StructureTemplateManager structureTemplateManager, ChunkPos chunkPos,
/// LevelHeightAccessor heightAccessor, WorldgenRandom random, long seed)`
#[derive(Debug, Clone)]
pub struct Context<C: FeatureConfiguration> {
    pub config: C,
    pub chunk_generator: ChunkGenerator,
    pub structure_template_manager: StructureTemplateManager,
    pub chunk_pos: ChunkPos,
    pub height_accessor: LevelHeightAccessor,
    pub random: WorldgenRandom,
    pub seed: i64,
}

/// Port of `net.minecraft.world.level.levelgen.structure.pieces.PieceGenerator`.
///
/// Java `@FunctionalInterface`: `generatePieces(StructurePiecesBuilder, Context<C>)`.
pub trait PieceGenerator<C: FeatureConfiguration> {
    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PieceGenerator#generatePieces
    fn generate_pieces(&self, builder: &mut StructurePiecesBuilder, context: &Context<C>);
}

impl<C: FeatureConfiguration, F> PieceGenerator<C> for F
where
    F: Fn(&mut StructurePiecesBuilder, &Context<C>),
{
    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PieceGenerator#generatePieces
    fn generate_pieces(&self, builder: &mut StructurePiecesBuilder, context: &Context<C>) {
        (self)(builder, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestConfig {
        value: i32,
    }

    impl FeatureConfiguration for TestConfig {}

    #[derive(Debug, Clone, Copy)]
    struct FloatConfig {
        f: f32,
        d: f64,
    }

    impl FeatureConfiguration for FloatConfig {}

    fn make_context(config: TestConfig, seed: i64, chunk_x: i32, chunk_z: i32) -> Context<TestConfig> {
        Context {
            config,
            chunk_generator: ChunkGenerator,
            structure_template_manager: StructureTemplateManager,
            chunk_pos: ChunkPos {
                x: chunk_x,
                z: chunk_z,
            },
            height_accessor: LevelHeightAccessor {
                min_y: -64,
                height: 384,
            },
            random: WorldgenRandom { seed },
            seed,
        }
    }

    #[test]
    fn context_fields_round_trip() {
        let ctx = make_context(TestConfig { value: 7 }, 12345_i64, 4, -9);
        assert_eq!(ctx.config, TestConfig { value: 7 });
        assert_eq!(ctx.chunk_pos, ChunkPos { x: 4, z: -9 });
        assert_eq!(ctx.seed, 12345_i64);
        assert_eq!(ctx.random.seed, 12345_i64);
        assert_eq!(ctx.height_accessor.min_y, -64);
        assert_eq!(ctx.height_accessor.height, 384);
    }

    #[test]
    fn context_negative_and_edge_case_inputs() {
        let ctx = Context {
            config: TestConfig { value: i32::MIN },
            chunk_generator: ChunkGenerator,
            structure_template_manager: StructureTemplateManager,
            chunk_pos: ChunkPos {
                x: i32::MIN,
                z: i32::MAX,
            },
            height_accessor: LevelHeightAccessor {
                min_y: i32::MIN,
                height: 0,
            },
            random: WorldgenRandom { seed: i64::MIN },
            seed: i64::MAX,
        };

        assert_eq!(ctx.config.value, i32::MIN);
        assert_eq!(ctx.chunk_pos.x, i32::MIN);
        assert_eq!(ctx.chunk_pos.z, i32::MAX);
        assert_eq!(ctx.height_accessor.min_y, i32::MIN);
        assert_eq!(ctx.random.seed, i64::MIN);
        assert_eq!(ctx.seed, i64::MAX);
    }

    #[test]
    fn trait_impl_generate_pieces_is_invoked() {
        struct MockGenerator;

        impl PieceGenerator<TestConfig> for MockGenerator {
            fn generate_pieces(
                &self,
                builder: &mut StructurePiecesBuilder,
                context: &Context<TestConfig>,
            ) {
                if context.config.value < 0 {
                    builder.piece_count = builder.piece_count.wrapping_add(1);
                }
                if context.seed == i64::MIN {
                    builder.piece_count = builder.piece_count.wrapping_add(2);
                }
            }
        }

        let generator = MockGenerator;
        let mut builder = StructurePiecesBuilder::new();
        let ctx = make_context(TestConfig { value: -1 }, i64::MIN, -3, 5);
        generator.generate_pieces(&mut builder, &ctx);
        assert_eq!(builder.piece_count, 3);
    }

    #[test]
    fn closure_impl_and_wrapping_overflow_semantics() {
        let generator = |builder: &mut StructurePiecesBuilder, ctx: &Context<TestConfig>| {
            let delta: u32 = if ctx.seed.wrapping_add(1) == i64::MIN {
                1
            } else {
                0
            };
            builder.piece_count = builder.piece_count.wrapping_add(delta);
        };

        let mut builder = StructurePiecesBuilder {
            piece_count: u32::MAX,
        };
        let ctx = make_context(TestConfig { value: 0 }, i64::MAX, 0, 0);
        PieceGenerator::generate_pieces(&generator, &mut builder, &ctx);
        assert_eq!(builder.piece_count, 0);

        let neg: i32 = i32::MIN;
        assert_eq!(neg.wrapping_sub(1), i32::MAX);
        let shifted_mask_31: u32 = 1u32.wrapping_shl(32 & 31);
        let shifted_mask_63: u64 = 1u64.wrapping_shl(64 & 63);
        assert_eq!(shifted_mask_31, 1u32);
        assert_eq!(shifted_mask_63, 1u64);
    }

    #[test]
    fn float_bits_preserved_through_context() {
        let nan32_bits: u32 = 0x7FC0_0001;
        let neg_zero64_bits: u64 = 0x8000_0000_0000_0000;
        let ctx = Context {
            config: FloatConfig {
                f: f32::from_bits(nan32_bits),
                d: f64::from_bits(neg_zero64_bits),
            },
            chunk_generator: ChunkGenerator,
            structure_template_manager: StructureTemplateManager,
            chunk_pos: ChunkPos { x: -7, z: 3 },
            height_accessor: LevelHeightAccessor {
                min_y: -64,
                height: 384,
            },
            random: WorldgenRandom { seed: -999 },
            seed: -999,
        };

        assert_eq!(ctx.config.f.to_bits(), nan32_bits);
        assert!(ctx.config.f.is_nan());
        assert_eq!(ctx.config.d.to_bits(), neg_zero64_bits);
        assert!(ctx.config.d.is_sign_negative());
        assert_eq!(ctx.config.d, -0.0_f64);
    }
}
