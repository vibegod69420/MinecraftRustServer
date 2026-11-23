use std::marker::PhantomData;

// ── Minimal stubs for external types (self-contained) ──────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn get_middle_block_x(&self) -> i32 {
        self.x.wrapping_mul(16).wrapping_add(8)
    }

    pub fn get_middle_block_z(&self) -> i32 {
        self.z.wrapping_mul(16).wrapping_add(8)
    }
}

pub struct QuartPos;

impl QuartPos {
    pub fn from_block(value: i32) -> i32 {
        value >> 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeHolder(pub u32);

pub trait NoiseSampler {}

pub trait RandomState {
    type Sampler: NoiseSampler;
    fn sampler(&self) -> Self::Sampler;
}

pub trait LevelHeightAccessor {}

pub trait BiomeSource<S: NoiseSampler> {
    fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32, sampler: &S) -> BiomeHolder;
}

pub trait ChunkGenerator<HA: LevelHeightAccessor, RS: RandomState> {
    type BS: BiomeSource<RS::Sampler>;
    fn get_first_occupied_height(
        &self,
        x: i32,
        z: i32,
        heightmap: HeightmapType,
        height_accessor: &HA,
        random_state: &RS,
    ) -> i32;
    fn get_biome_source(&self) -> &Self::BS;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StructureTemplateManager;

#[derive(Debug, Clone, Copy, Default)]
pub struct RegistryAccess;

/// Marker trait for feature configuration types.
pub trait FeatureConfiguration {}

// ── Ported types ───────────────────────────────────────────────────────────

/// JAVA: net.minecraft.world.level.levelgen.structure.pieces.PieceGeneratorSupplier$Context
pub struct Context<C, CG, BS, RS, HA, STM = StructureTemplateManager, RA = RegistryAccess> {
    pub chunk_generator: CG,
    pub biome_source: BS,
    pub random_state: RS,
    pub seed: i64,
    pub chunk_pos: ChunkPos,
    pub config: C,
    pub height_accessor: HA,
    pub valid_biome: Box<dyn Fn(&BiomeHolder) -> bool>,
    pub structure_template_manager: STM,
    pub registry_access: RA,
}

impl<C, CG, BS, RS, HA, STM, RA> Context<C, CG, BS, RS, HA, STM, RA>
where
    CG: ChunkGenerator<HA, RS>,
    RS: RandomState,
    HA: LevelHeightAccessor,
{
    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PieceGeneratorSupplier$Context#validBiomeOnTop
    pub fn valid_biome_on_top(&self, heightmap: HeightmapType) -> bool {
        let x = self.chunk_pos.get_middle_block_x();
        let z = self.chunk_pos.get_middle_block_z();
        let y = self.chunk_generator.get_first_occupied_height(
            x,
            z,
            heightmap,
            &self.height_accessor,
            &self.random_state,
        );
        let biome = self.chunk_generator.get_biome_source().get_noise_biome(
            QuartPos::from_block(x),
            QuartPos::from_block(y),
            QuartPos::from_block(z),
            &self.random_state.sampler(),
        );
        (self.valid_biome)(&biome)
    }
}

/// Functional supplier for structure piece generation.
///
/// `G` represents the piece generator type (`PieceGenerator<C>` in Java).
pub struct PieceGeneratorSupplier<Ctx, G> {
    f: Box<dyn Fn(&Ctx) -> Option<G>>,
}

impl<Ctx, G: Clone + 'static> PieceGeneratorSupplier<Ctx, G> {
    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PieceGeneratorSupplier#createGenerator
    pub fn create_generator(&self, context: &Ctx) -> Option<G> {
        (self.f)(context)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PieceGeneratorSupplier#simple
    pub fn simple(predicate: impl Fn(&Ctx) -> bool + 'static, generator: G) -> Self {
        let result = Some(generator);
        Self {
            f: Box::new(move |ctx| {
                if predicate(ctx) {
                    result.clone()
                } else {
                    None
                }
            }),
        }
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pieces.PieceGeneratorSupplier#checkForBiomeOnTop
pub fn check_for_biome_on_top<C, CG, BS, RS, HA, STM, RA>(
    heightmap: HeightmapType,
) -> impl Fn(&Context<C, CG, BS, RS, HA, STM, RA>) -> bool
where
    CG: ChunkGenerator<HA, RS>,
    RS: RandomState,
    HA: LevelHeightAccessor,
{
    move |ctx| ctx.valid_biome_on_top(heightmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestConfig;
    impl FeatureConfiguration for TestConfig {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestGen(i32);

    #[derive(Clone, Copy)]
    struct TestSampler;
    impl NoiseSampler for TestSampler {}

    struct TestRandomState;
    impl RandomState for TestRandomState {
        type Sampler = TestSampler;
        fn sampler(&self) -> Self::Sampler {
            TestSampler
        }
    }

    struct TestHeightAccessor;
    impl LevelHeightAccessor for TestHeightAccessor {}

    struct TestBiomeSource {
        biome_at: BiomeHolder,
    }
    impl BiomeSource<TestSampler> for TestBiomeSource {
        fn get_noise_biome(&self, _qx: i32, _qy: i32, _qz: i32, _sampler: &TestSampler) -> BiomeHolder {
            self.biome_at
        }
    }

    struct TestChunkGenerator {
        height: i32,
        biome_source: TestBiomeSource,
    }
    impl ChunkGenerator<TestHeightAccessor, TestRandomState> for TestChunkGenerator {
        type BS = TestBiomeSource;
        fn get_first_occupied_height(
            &self,
            _x: i32,
            _z: i32,
            _heightmap: HeightmapType,
            _height_accessor: &TestHeightAccessor,
            _random_state: &TestRandomState,
        ) -> i32 {
            self.height
        }
        fn get_biome_source(&self) -> &Self::BS {
            &self.biome_source
        }
    }

    fn make_context(valid_id: u32, height: i32, biome_id: u32) -> Context<TestConfig, TestChunkGenerator, (), TestRandomState, TestHeightAccessor> {
        Context {
            chunk_generator: TestChunkGenerator {
                height,
                biome_source: TestBiomeSource {
                    biome_at: BiomeHolder(biome_id),
                },
            },
            biome_source: (),
            random_state: TestRandomState,
            seed: 12345,
            chunk_pos: ChunkPos { x: 0, z: 0 },
            config: TestConfig,
            height_accessor: TestHeightAccessor,
            valid_biome: Box::new(move |b| b.0 == valid_id),
            structure_template_manager: StructureTemplateManager,
            registry_access: RegistryAccess,
        }
    }

    #[test]
    fn simple_returns_generator_when_predicate_true() {
        let supplier = PieceGeneratorSupplier::simple(|_: &i32| true, TestGen(7));
        assert_eq!(supplier.create_generator(&42), Some(TestGen(7)));
    }

    #[test]
    fn simple_returns_none_when_predicate_false() {
        let supplier = PieceGeneratorSupplier::simple(|_: &i32| false, TestGen(7));
        assert_eq!(supplier.create_generator(&42), None);
    }

    #[test]
    fn valid_biome_on_top_checks_biome() {
        let ctx = make_context(5, 64, 5);
        assert!(ctx.valid_biome_on_top(HeightmapType::WorldSurfaceWg));
        let ctx2 = make_context(5, 64, 6);
        assert!(!ctx2.valid_biome_on_top(HeightmapType::WorldSurfaceWg));
    }

    #[test]
    fn check_for_biome_on_top_returns_predicate() {
        let pred = check_for_biome_on_top(HeightmapType::OceanFloor);
        let ctx_ok = make_context(1, -10, 1);
        let ctx_bad = make_context(1, -10, 2);
        assert!(pred(&ctx_ok));
        assert!(!pred(&ctx_bad));
    }

    #[test]
    fn chunk_pos_middle_and_quart_pos_negative() {
        let pos = ChunkPos { x: -1, z: -2 };
        assert_eq!(pos.get_middle_block_x(), -8); // -16 + 8
        assert_eq!(pos.get_middle_block_z(), -24); // -32 + 8
        assert_eq!(QuartPos::from_block(-1), -1); // arithmetic shift
        assert_eq!(QuartPos::from_block(-4), -1);
        assert_eq!(QuartPos::from_block(-5), -2);
    }

    #[test]
    fn chunk_pos_wrapping_overflow() {
        let pos = ChunkPos { x: i32::MAX, z: i32::MIN };
        assert_eq!(pos.get_middle_block_x(), i32::MAX.wrapping_mul(16).wrapping_add(8));
        assert_eq!(pos.get_middle_block_z(), i32::MIN.wrapping_mul(16).wrapping_add(8));
    }
}
