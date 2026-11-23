use std::fmt;

pub const MAX_TOTAL_STRUCTURE_RANGE: i32 = 128;
pub const MIN_DEPTH: i32 = 0;
pub const MAX_DEPTH: i32 = 20;
pub const DIMENSION_Y_SIZE: i32 = 4064;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn new(namespace: &str, path: &str) -> Self {
        Self(format!("{namespace}:{path}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn get_min_block_x(&self) -> i32 {
        self.x.wrapping_shl(4)
    }

    pub fn get_min_block_z(&self) -> i32 {
        self.z.wrapping_shl(4)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureTemplatePool;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolAliasBinding;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolAliasLookup;

impl PoolAliasLookup {
    pub fn create(_aliases: &[PoolAliasBinding], _pos: BlockPos, _seed: i64) -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionPadding {
    pub top: i32,
    pub bottom: i32,
}

impl DimensionPadding {
    pub const ZERO: Self = Self { top: 0, bottom: 0 };
}

pub const DEFAULT_DIMENSION_PADDING: DimensionPadding = DimensionPadding::ZERO;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidSettings {
    ApplyWaterlogging,
    IgnoreWaterlogging,
}

pub const DEFAULT_LIQUID_SETTINGS: LiquidSettings = LiquidSettings::ApplyWaterlogging;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAdaptation {
    None,
    Bury,
    BeardThin,
    BeardBox,
    Encapsulate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureSettings {
    pub terrain_adaptation: TerrainAdaptation,
}

impl StructureSettings {
    pub fn new(terrain_adaptation: TerrainAdaptation) -> Self {
        Self {
            terrain_adaptation,
        }
    }
}

pub trait RandomSource {
    fn next_i32(&mut self) -> i32;
}

#[derive(Debug, Clone, Default)]
pub struct ChunkGenerator;

#[derive(Debug, Clone, Default)]
pub struct LevelHeightAccessor;

#[derive(Debug)]
pub struct WorldGenerationContext<'a> {
    pub chunk_generator: &'a ChunkGenerator,
    pub height_accessor: &'a LevelHeightAccessor,
}

impl<'a> WorldGenerationContext<'a> {
    pub fn new(chunk_generator: &'a ChunkGenerator, height_accessor: &'a LevelHeightAccessor) -> Self {
        Self {
            chunk_generator,
            height_accessor,
        }
    }
}

pub struct StructureGenerationContext<'a> {
    chunk_pos: ChunkPos,
    random: &'a mut dyn RandomSource,
    chunk_generator: &'a ChunkGenerator,
    height_accessor: &'a LevelHeightAccessor,
    seed: i64,
}

impl<'a> StructureGenerationContext<'a> {
    pub fn new(
        chunk_pos: ChunkPos,
        random: &'a mut dyn RandomSource,
        chunk_generator: &'a ChunkGenerator,
        height_accessor: &'a LevelHeightAccessor,
        seed: i64,
    ) -> Self {
        Self {
            chunk_pos,
            random,
            chunk_generator,
            height_accessor,
            seed,
        }
    }

    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn random(&mut self) -> &mut dyn RandomSource {
        self.random
    }

    pub fn chunk_generator(&self) -> &'a ChunkGenerator {
        self.chunk_generator
    }

    pub fn height_accessor(&self) -> &'a LevelHeightAccessor {
        self.height_accessor
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationStub {
    pub position: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Jigsaw,
}

pub trait HeightProvider: fmt::Debug {
    fn sample(&self, random: &mut dyn RandomSource, context: &WorldGenerationContext<'_>) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxDistance {
    horizontal: i32,
    vertical: i32,
}

impl MaxDistance {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure$MaxDistance#<init>
    pub fn new(horizontal: i32, vertical: i32) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure$MaxDistance#<init>
    pub fn symmetric(value: i32) -> Self {
        Self {
            horizontal: value,
            vertical: value,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure$MaxDistance#horizontal
    pub fn horizontal(&self) -> i32 {
        self.horizontal
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure$MaxDistance#vertical
    pub fn vertical(&self) -> i32 {
        self.vertical
    }
}

pub trait JigsawPlacer {
    fn add_pieces(
        &self,
        ctx: &mut StructureGenerationContext<'_>,
        start_pool: &Holder<StructureTemplatePool>,
        start_jigsaw_name: Option<&Identifier>,
        max_depth: i32,
        pos: BlockPos,
        use_expansion_hack: bool,
        project_start_to_heightmap: Option<HeightmapType>,
        max_distance_from_center: MaxDistance,
        pool_alias_lookup: PoolAliasLookup,
        dimension_padding: DimensionPadding,
        liquid_settings: LiquidSettings,
    ) -> Option<GenerationStub>;
}

pub struct JigsawStructure {
    settings: StructureSettings,
    start_pool: Holder<StructureTemplatePool>,
    start_jigsaw_name: Option<Identifier>,
    max_depth: i32,
    start_height: Box<dyn HeightProvider>,
    use_expansion_hack: bool,
    project_start_to_heightmap: Option<HeightmapType>,
    max_distance_from_center: MaxDistance,
    pool_aliases: Vec<PoolAliasBinding>,
    dimension_padding: DimensionPadding,
    liquid_settings: LiquidSettings,
}

impl fmt::Debug for JigsawStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JigsawStructure")
            .field("settings", &self.settings)
            .field("start_pool", &self.start_pool)
            .field("start_jigsaw_name", &self.start_jigsaw_name)
            .field("max_depth", &self.max_depth)
            .field("start_height", &self.start_height)
            .field("use_expansion_hack", &self.use_expansion_hack)
            .field("project_start_to_heightmap", &self.project_start_to_heightmap)
            .field("max_distance_from_center", &self.max_distance_from_center)
            .field("pool_aliases", &self.pool_aliases)
            .field("dimension_padding", &self.dimension_padding)
            .field("liquid_settings", &self.liquid_settings)
            .finish()
    }
}

impl JigsawStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure#verifyRange
    pub fn verify_range(&self) -> Result<(), &'static str> {
        let terrain_offset: i32 = match self.settings.terrain_adaptation {
            TerrainAdaptation::None => 0,
            TerrainAdaptation::Bury
            | TerrainAdaptation::BeardThin
            | TerrainAdaptation::BeardBox
            | TerrainAdaptation::Encapsulate => 12,
        };
        let total = self
            .max_distance_from_center
            .horizontal()
            .wrapping_add(terrain_offset);
        if total > MAX_TOTAL_STRUCTURE_RANGE {
            Err("Horizontal structure size including terrain adaptation must not exceed 128")
        } else {
            Ok(())
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure#<init>
    pub fn new(
        settings: StructureSettings,
        start_pool: Holder<StructureTemplatePool>,
        start_jigsaw_name: Option<Identifier>,
        max_depth: i32,
        start_height: Box<dyn HeightProvider>,
        use_expansion_hack: bool,
        project_start_to_heightmap: Option<HeightmapType>,
        max_distance_from_center: MaxDistance,
        pool_aliases: Vec<PoolAliasBinding>,
        dimension_padding: DimensionPadding,
        liquid_settings: LiquidSettings,
    ) -> Self {
        Self {
            settings,
            start_pool,
            start_jigsaw_name,
            max_depth,
            start_height,
            use_expansion_hack,
            project_start_to_heightmap,
            max_distance_from_center,
            pool_aliases,
            dimension_padding,
            liquid_settings,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure#<init>
    pub fn with_heightmap(
        settings: StructureSettings,
        start_pool: Holder<StructureTemplatePool>,
        max_depth: i32,
        start_height: Box<dyn HeightProvider>,
        use_expansion_hack: bool,
        project_start_to_heightmap: HeightmapType,
    ) -> Self {
        Self::new(
            settings,
            start_pool,
            None,
            max_depth,
            start_height,
            use_expansion_hack,
            Some(project_start_to_heightmap),
            MaxDistance::symmetric(80),
            Vec::new(),
            DEFAULT_DIMENSION_PADDING,
            DEFAULT_LIQUID_SETTINGS,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure#<init>
    pub fn without_heightmap(
        settings: StructureSettings,
        start_pool: Holder<StructureTemplatePool>,
        max_depth: i32,
        start_height: Box<dyn HeightProvider>,
        use_expansion_hack: bool,
    ) -> Self {
        Self::new(
            settings,
            start_pool,
            None,
            max_depth,
            start_height,
            use_expansion_hack,
            None,
            MaxDistance::symmetric(80),
            Vec::new(),
            DEFAULT_DIMENSION_PADDING,
            DEFAULT_LIQUID_SETTINGS,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure#findGenerationPoint
    pub fn find_generation_point<P: JigsawPlacer>(
        &self,
        ctx: &mut StructureGenerationContext<'_>,
        placer: &P,
    ) -> Option<GenerationStub> {
        let chunk_pos = ctx.chunk_pos();
        let world_gen_ctx =
            WorldGenerationContext::new(ctx.chunk_generator(), ctx.height_accessor());
        let y = self.start_height.sample(ctx.random(), &world_gen_ctx);
        let pos = BlockPos::new(chunk_pos.get_min_block_x(), y, chunk_pos.get_min_block_z());
        placer.add_pieces(
            ctx,
            &self.start_pool,
            self.start_jigsaw_name.as_ref(),
            self.max_depth,
            pos,
            self.use_expansion_hack,
            self.project_start_to_heightmap,
            self.max_distance_from_center,
            PoolAliasLookup::create(&self.pool_aliases, pos, ctx.seed()),
            self.dimension_padding,
            self.liquid_settings,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::Jigsaw
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure#getStartPool
    pub fn get_start_pool(&self) -> &Holder<StructureTemplatePool> {
        &self.start_pool
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JigsawStructure#getPoolAliases
    pub fn get_pool_aliases(&self) -> &[PoolAliasBinding] {
        &self.pool_aliases
    }

    pub fn settings(&self) -> &StructureSettings {
        &self.settings
    }

    pub fn terrain_adaptation(&self) -> TerrainAdaptation {
        self.settings.terrain_adaptation
    }

    pub fn max_depth(&self) -> i32 {
        self.max_depth
    }

    pub fn use_expansion_hack(&self) -> bool {
        self.use_expansion_hack
    }

    pub fn project_start_to_heightmap(&self) -> Option<HeightmapType> {
        self.project_start_to_heightmap
    }

    pub fn max_distance_from_center(&self) -> MaxDistance {
        self.max_distance_from_center
    }

    pub fn dimension_padding(&self) -> DimensionPadding {
        self.dimension_padding
    }

    pub fn liquid_settings(&self) -> LiquidSettings {
        self.liquid_settings
    }

    pub fn start_jigsaw_name(&self) -> Option<&Identifier> {
        self.start_jigsaw_name.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct ConstHeight(i32);

    impl HeightProvider for ConstHeight {
        fn sample(
            &self,
            _random: &mut dyn RandomSource,
            _context: &WorldGenerationContext<'_>,
        ) -> i32 {
            self.0
        }
    }

    #[derive(Debug)]
    struct FloatBitsHeight(f32);

    impl HeightProvider for FloatBitsHeight {
        fn sample(
            &self,
            _random: &mut dyn RandomSource,
            _context: &WorldGenerationContext<'_>,
        ) -> i32 {
            f32::from_bits(self.0.to_bits()).to_bits() as i32
        }
    }

    struct SeqRandom(i32);

    impl RandomSource for SeqRandom {
        fn next_i32(&mut self) -> i32 {
            let v = self.0;
            self.0 = self.0.wrapping_add(1);
            v
        }
    }

    struct CapturePlacer {
        expected_pos: BlockPos,
        expected_max_depth: i32,
        expected_use_expansion_hack: bool,
    }

    impl JigsawPlacer for CapturePlacer {
        fn add_pieces(
            &self,
            _ctx: &mut StructureGenerationContext<'_>,
            _start_pool: &Holder<StructureTemplatePool>,
            _start_jigsaw_name: Option<&Identifier>,
            max_depth: i32,
            pos: BlockPos,
            use_expansion_hack: bool,
            _project_start_to_heightmap: Option<HeightmapType>,
            _max_distance_from_center: MaxDistance,
            _pool_alias_lookup: PoolAliasLookup,
            _dimension_padding: DimensionPadding,
            _liquid_settings: LiquidSettings,
        ) -> Option<GenerationStub> {
            assert_eq!(pos, self.expected_pos);
            assert_eq!(max_depth, self.expected_max_depth);
            assert_eq!(use_expansion_hack, self.expected_use_expansion_hack);
            Some(GenerationStub { position: pos })
        }
    }

    fn base_settings(adaptation: TerrainAdaptation) -> StructureSettings {
        StructureSettings::new(adaptation)
    }

    #[test]
    fn verify_range_boundary_and_terrain_offset() {
        let ok = JigsawStructure::new(
            base_settings(TerrainAdaptation::None),
            Holder(StructureTemplatePool),
            None,
            7,
            Box::new(ConstHeight(0)),
            false,
            None,
            MaxDistance::symmetric(MAX_TOTAL_STRUCTURE_RANGE),
            vec![],
            DEFAULT_DIMENSION_PADDING,
            DEFAULT_LIQUID_SETTINGS,
        );
        assert!(ok.verify_range().is_ok());

        let err = JigsawStructure::new(
            base_settings(TerrainAdaptation::Bury),
            Holder(StructureTemplatePool),
            None,
            7,
            Box::new(ConstHeight(0)),
            false,
            None,
            MaxDistance::symmetric(117),
            vec![],
            DEFAULT_DIMENSION_PADDING,
            DEFAULT_LIQUID_SETTINGS,
        );
        assert_eq!(
            err.verify_range(),
            Err("Horizontal structure size including terrain adaptation must not exceed 128")
        );
    }

    #[test]
    fn verify_range_wrapping_overflow_matches_java_int() {
        let s = JigsawStructure::new(
            base_settings(TerrainAdaptation::BeardBox),
            Holder(StructureTemplatePool),
            None,
            1,
            Box::new(ConstHeight(0)),
            false,
            None,
            MaxDistance::new(i32::MAX, 1),
            vec![],
            DEFAULT_DIMENSION_PADDING,
            DEFAULT_LIQUID_SETTINGS,
        );
        // i32::MAX.wrapping_add(12) overflows to i32::MIN + 11, which is <= 128.
        assert!(s.verify_range().is_ok());
    }

    #[test]
    fn find_generation_point_negative_chunk_and_float_bits_height() {
        let structure = JigsawStructure::without_heightmap(
            base_settings(TerrainAdaptation::None),
            Holder(StructureTemplatePool),
            5,
            Box::new(FloatBitsHeight(f32::NEG_INFINITY)),
            true,
        );
        let mut rng = SeqRandom(0);
        let cg = ChunkGenerator;
        let ha = LevelHeightAccessor;
        let mut ctx = StructureGenerationContext::new(
            ChunkPos::new(-1, -2),
            &mut rng,
            &cg,
            &ha,
            12_345_i64,
        );
        let expected_y = f32::from_bits(f32::NEG_INFINITY.to_bits()).to_bits() as i32;
        let placer = CapturePlacer {
            expected_pos: BlockPos::new(-16, expected_y, -32),
            expected_max_depth: 5,
            expected_use_expansion_hack: true,
        };
        let stub = structure
            .find_generation_point(&mut ctx, &placer)
            .expect("placer should return stub");
        assert_eq!(stub.position, BlockPos::new(-16, expected_y, -32));
    }

    #[test]
    fn convenience_constructors_set_defaults() {
        let with_hm = JigsawStructure::with_heightmap(
            base_settings(TerrainAdaptation::None),
            Holder(StructureTemplatePool),
            3,
            Box::new(ConstHeight(10)),
            false,
            HeightmapType::WorldSurfaceWg,
        );
        assert_eq!(with_hm.max_depth(), 3);
        assert_eq!(
            with_hm.project_start_to_heightmap(),
            Some(HeightmapType::WorldSurfaceWg)
        );
        assert_eq!(with_hm.max_distance_from_center(), MaxDistance::symmetric(80));
        assert_eq!(with_hm.dimension_padding(), DEFAULT_DIMENSION_PADDING);
        assert_eq!(with_hm.liquid_settings(), DEFAULT_LIQUID_SETTINGS);
        assert!(with_hm.start_jigsaw_name().is_none());
        assert!(with_hm.get_pool_aliases().is_empty());

        let without_hm = JigsawStructure::without_heightmap(
            base_settings(TerrainAdaptation::None),
            Holder(StructureTemplatePool),
            4,
            Box::new(ConstHeight(-64)),
            true,
        );
        assert_eq!(without_hm.project_start_to_heightmap(), None);
        assert_eq!(without_hm.max_distance_from_center(), MaxDistance::symmetric(80));
        assert!(without_hm.use_expansion_hack());
        assert_eq!(without_hm.structure_type(), StructureType::Jigsaw);
    }

    #[test]
    fn max_distance_accessors_and_symmetric() {
        let md = MaxDistance::new(10, 20);
        assert_eq!(md.horizontal(), 10);
        assert_eq!(md.vertical(), 20);

        let sym = MaxDistance::symmetric(80);
        assert_eq!(sym.horizontal(), 80);
        assert_eq!(sym.vertical(), 80);

        let neg = MaxDistance::new(-1, i32::MIN);
        assert_eq!(neg.horizontal(), -1);
        assert_eq!(neg.vertical(), i32::MIN);
    }
}
