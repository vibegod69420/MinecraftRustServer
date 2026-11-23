use std::fmt::Debug;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rotation {
    #[default]
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

pub trait RandomSource: Debug {}

#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleRandomSource {
    pub seed: i64,
}

impl RandomSource for SimpleRandomSource {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LiquidSettings;

#[derive(Debug, Clone, Default)]
pub struct StructureTemplateManager;

#[derive(Debug, Clone, Default)]
pub struct StructureManager;

#[derive(Debug, Clone, Default)]
pub struct WorldGenLevel;

#[derive(Debug, Clone, Default)]
pub struct ChunkGenerator;

#[derive(Debug, Clone, Default)]
pub struct LevelAccessor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StructurePoolElementType(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn parse(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Holder<T: Debug + Clone>(pub T);

impl<T: Debug + Clone> Holder<T> {
    pub fn direct(value: T) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StructureProcessorList(pub Vec<()>);

#[derive(Debug, Clone, Default)]
pub struct PlacedFeature;

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub mod structure_template {
    #[derive(Debug, Clone, Default)]
    pub struct JigsawBlockInfo;

    #[derive(Debug, Clone, Default)]
    pub struct StructureBlockInfo;
}

pub mod structure_template_pool {
    pub use super::Projection;
}

// ---------------------------------------------------------------------------
// Projection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    TerrainMatching,
    Rigid,
}

// ---------------------------------------------------------------------------
// Projection state (base field from StructurePoolElement)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ProjectionState {
    projection: Option<Projection>,
}

impl ProjectionState {
    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#<init>
    pub fn new(projection: Projection) -> Self {
        Self {
            projection: Some(projection),
        }
    }

    fn unset() -> Self {
        Self { projection: None }
    }
}

// ---------------------------------------------------------------------------
// StructurePoolElement trait (abstract base class)
// ---------------------------------------------------------------------------

pub trait StructurePoolElement: Debug {
    fn projection_state(&self) -> &ProjectionState;
    fn projection_state_mut(&mut self) -> &mut ProjectionState;

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#getSize
    fn get_size(
        &self,
        structure_template_manager: &StructureTemplateManager,
        rotation: Rotation,
    ) -> Vec3i;

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#getShuffledJigsawBlocks
    fn get_shuffled_jigsaw_blocks(
        &self,
        structure_template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
        random: &mut dyn RandomSource,
    ) -> Vec<structure_template::JigsawBlockInfo>;

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#getBoundingBox
    fn get_bounding_box(
        &self,
        structure_template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
    ) -> BoundingBox;

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#place
    fn place(
        &self,
        structure_template_manager: &StructureTemplateManager,
        world_gen_level: &mut WorldGenLevel,
        structure_manager: &StructureManager,
        chunk_generator: &ChunkGenerator,
        pos: BlockPos,
        pos2: BlockPos,
        rotation: Rotation,
        bounding_box: BoundingBox,
        random: &mut dyn RandomSource,
        liquid_settings: LiquidSettings,
        keep_jigsaws: bool,
    ) -> bool;

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#getType
    fn get_type(&self) -> StructurePoolElementType;

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#handleDataMarker
    fn handle_data_marker(
        &mut self,
        _level_accessor: &mut LevelAccessor,
        _structure_block_info: &structure_template::StructureBlockInfo,
        _pos: BlockPos,
        _rotation: Rotation,
        _random: &mut dyn RandomSource,
        _bounding_box: BoundingBox,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#setProjection
    fn set_projection(&mut self, projection: Projection) -> &mut Self
    where
        Self: Sized,
    {
        self.projection_state_mut().projection = Some(projection);
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#getProjection
    fn get_projection(&self) -> Projection {
        self.projection_state()
            .projection
            .expect("java.lang.IllegalStateException")
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#getGroundLevelDelta
    fn get_ground_level_delta(&self) -> i32 {
        1
    }
}

fn empty_processor_list() -> Holder<StructureProcessorList> {
    Holder::direct(StructureProcessorList(Vec::new()))
}

// ---------------------------------------------------------------------------
// Minimal concrete element stubs (for factory self-containment)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EmptyPoolElement {
    projection: ProjectionState,
}

impl EmptyPoolElement {
    fn instance() -> Self {
        Self {
            projection: ProjectionState::new(Projection::TerrainMatching),
        }
    }
}

impl StructurePoolElement for EmptyPoolElement {
    fn projection_state(&self) -> &ProjectionState {
        &self.projection
    }
    fn projection_state_mut(&mut self) -> &mut ProjectionState {
        &mut self.projection
    }
    fn get_size(&self, _: &StructureTemplateManager, _: Rotation) -> Vec3i {
        Vec3i::default()
    }
    fn get_shuffled_jigsaw_blocks(
        &self,
        _: &StructureTemplateManager,
        _: BlockPos,
        _: Rotation,
        _: &mut dyn RandomSource,
    ) -> Vec<structure_template::JigsawBlockInfo> {
        Vec::new()
    }
    fn get_bounding_box(&self, _: &StructureTemplateManager, _: BlockPos, _: Rotation) -> BoundingBox {
        BoundingBox::default()
    }
    fn place(
        &self,
        _: &StructureTemplateManager,
        _: &mut WorldGenLevel,
        _: &StructureManager,
        _: &ChunkGenerator,
        _: BlockPos,
        _: BlockPos,
        _: Rotation,
        _: BoundingBox,
        _: &mut dyn RandomSource,
        _: LiquidSettings,
        _: bool,
    ) -> bool {
        false
    }
    fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType(0)
    }
}

#[derive(Debug, Clone)]
pub struct LegacySinglePoolElement {
    projection: ProjectionState,
    pub template: Either<Identifier, String>,
    pub processors: Holder<StructureProcessorList>,
    pub override_liquid_settings: Option<LiquidSettings>,
}

impl StructurePoolElement for LegacySinglePoolElement {
    fn projection_state(&self) -> &ProjectionState {
        &self.projection
    }
    fn projection_state_mut(&mut self) -> &mut ProjectionState {
        &mut self.projection
    }
    fn get_size(&self, _: &StructureTemplateManager, _: Rotation) -> Vec3i {
        Vec3i::default()
    }
    fn get_shuffled_jigsaw_blocks(
        &self,
        _: &StructureTemplateManager,
        _: BlockPos,
        _: Rotation,
        _: &mut dyn RandomSource,
    ) -> Vec<structure_template::JigsawBlockInfo> {
        Vec::new()
    }
    fn get_bounding_box(&self, _: &StructureTemplateManager, _: BlockPos, _: Rotation) -> BoundingBox {
        BoundingBox::default()
    }
    fn place(
        &self,
        _: &StructureTemplateManager,
        _: &mut WorldGenLevel,
        _: &StructureManager,
        _: &ChunkGenerator,
        _: BlockPos,
        _: BlockPos,
        _: Rotation,
        _: BoundingBox,
        _: &mut dyn RandomSource,
        _: LiquidSettings,
        _: bool,
    ) -> bool {
        false
    }
    fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType(1)
    }
}

#[derive(Debug, Clone)]
pub struct SinglePoolElement {
    projection: ProjectionState,
    pub template: Either<Identifier, String>,
    pub processors: Holder<StructureProcessorList>,
    pub override_liquid_settings: Option<LiquidSettings>,
}

impl StructurePoolElement for SinglePoolElement {
    fn projection_state(&self) -> &ProjectionState {
        &self.projection
    }
    fn projection_state_mut(&mut self) -> &mut ProjectionState {
        &mut self.projection
    }
    fn get_size(&self, _: &StructureTemplateManager, _: Rotation) -> Vec3i {
        Vec3i::default()
    }
    fn get_shuffled_jigsaw_blocks(
        &self,
        _: &StructureTemplateManager,
        _: BlockPos,
        _: Rotation,
        _: &mut dyn RandomSource,
    ) -> Vec<structure_template::JigsawBlockInfo> {
        Vec::new()
    }
    fn get_bounding_box(&self, _: &StructureTemplateManager, _: BlockPos, _: Rotation) -> BoundingBox {
        BoundingBox::default()
    }
    fn place(
        &self,
        _: &StructureTemplateManager,
        _: &mut WorldGenLevel,
        _: &StructureManager,
        _: &ChunkGenerator,
        _: BlockPos,
        _: BlockPos,
        _: Rotation,
        _: BoundingBox,
        _: &mut dyn RandomSource,
        _: LiquidSettings,
        _: bool,
    ) -> bool {
        false
    }
    fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType(2)
    }
}

#[derive(Debug, Clone)]
pub struct FeaturePoolElement {
    projection: ProjectionState,
    pub feature: Holder<PlacedFeature>,
}

impl StructurePoolElement for FeaturePoolElement {
    fn projection_state(&self) -> &ProjectionState {
        &self.projection
    }
    fn projection_state_mut(&mut self) -> &mut ProjectionState {
        &mut self.projection
    }
    fn get_size(&self, _: &StructureTemplateManager, _: Rotation) -> Vec3i {
        Vec3i::default()
    }
    fn get_shuffled_jigsaw_blocks(
        &self,
        _: &StructureTemplateManager,
        _: BlockPos,
        _: Rotation,
        _: &mut dyn RandomSource,
    ) -> Vec<structure_template::JigsawBlockInfo> {
        Vec::new()
    }
    fn get_bounding_box(&self, _: &StructureTemplateManager, _: BlockPos, _: Rotation) -> BoundingBox {
        BoundingBox::default()
    }
    fn place(
        &self,
        _: &StructureTemplateManager,
        _: &mut WorldGenLevel,
        _: &StructureManager,
        _: &ChunkGenerator,
        _: BlockPos,
        _: BlockPos,
        _: Rotation,
        _: BoundingBox,
        _: &mut dyn RandomSource,
        _: LiquidSettings,
        _: bool,
    ) -> bool {
        false
    }
    fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType(3)
    }
}

#[derive(Debug)]
pub struct ListPoolElement {
    projection: ProjectionState,
    pub elements: Vec<Box<dyn StructurePoolElement>>,
}

impl StructurePoolElement for ListPoolElement {
    fn projection_state(&self) -> &ProjectionState {
        &self.projection
    }
    fn projection_state_mut(&mut self) -> &mut ProjectionState {
        &mut self.projection
    }
    fn get_size(&self, _: &StructureTemplateManager, _: Rotation) -> Vec3i {
        Vec3i::default()
    }
    fn get_shuffled_jigsaw_blocks(
        &self,
        _: &StructureTemplateManager,
        _: BlockPos,
        _: Rotation,
        _: &mut dyn RandomSource,
    ) -> Vec<structure_template::JigsawBlockInfo> {
        Vec::new()
    }
    fn get_bounding_box(&self, _: &StructureTemplateManager, _: BlockPos, _: Rotation) -> BoundingBox {
        BoundingBox::default()
    }
    fn place(
        &self,
        _: &StructureTemplateManager,
        _: &mut WorldGenLevel,
        _: &StructureManager,
        _: &ChunkGenerator,
        _: BlockPos,
        _: BlockPos,
        _: Rotation,
        _: BoundingBox,
        _: &mut dyn RandomSource,
        _: LiquidSettings,
        _: bool,
    ) -> bool {
        false
    }
    fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType(4)
    }
}

// ---------------------------------------------------------------------------
// Static factory methods from StructurePoolElement
// ---------------------------------------------------------------------------

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#empty
pub fn empty() -> impl Fn(Projection) -> EmptyPoolElement {
    |_projection| EmptyPoolElement::instance()
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#legacy
pub fn legacy(template: &str) -> impl Fn(Projection) -> LegacySinglePoolElement + '_ {
    move |projection| LegacySinglePoolElement {
        projection: ProjectionState::new(projection),
        template: Either::Left(Identifier::parse(template)),
        processors: empty_processor_list(),
        override_liquid_settings: None,
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#legacy
pub fn legacy_with_processors(
    template: &str,
    processors: Holder<StructureProcessorList>,
) -> impl Fn(Projection) -> LegacySinglePoolElement + '_ {
    move |projection| LegacySinglePoolElement {
        projection: ProjectionState::new(projection),
        template: Either::Left(Identifier::parse(template)),
        processors: processors.clone(),
        override_liquid_settings: None,
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#single
pub fn single(template: &str) -> impl Fn(Projection) -> SinglePoolElement + '_ {
    move |projection| SinglePoolElement {
        projection: ProjectionState::new(projection),
        template: Either::Left(Identifier::parse(template)),
        processors: empty_processor_list(),
        override_liquid_settings: None,
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#single
pub fn single_with_processors(
    template: &str,
    processors: Holder<StructureProcessorList>,
) -> impl Fn(Projection) -> SinglePoolElement + '_ {
    move |projection| SinglePoolElement {
        projection: ProjectionState::new(projection),
        template: Either::Left(Identifier::parse(template)),
        processors: processors.clone(),
        override_liquid_settings: None,
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#single
pub fn single_with_liquid_settings(
    template: &str,
    liquid_settings: LiquidSettings,
) -> impl Fn(Projection) -> SinglePoolElement + '_ {
    move |projection| SinglePoolElement {
        projection: ProjectionState::new(projection),
        template: Either::Left(Identifier::parse(template)),
        processors: empty_processor_list(),
        override_liquid_settings: Some(liquid_settings),
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#single
pub fn single_with_processors_and_liquid_settings(
    template: &str,
    processors: Holder<StructureProcessorList>,
    liquid_settings: LiquidSettings,
) -> impl Fn(Projection) -> SinglePoolElement + '_ {
    move |projection| SinglePoolElement {
        projection: ProjectionState::new(projection),
        template: Either::Left(Identifier::parse(template)),
        processors: processors.clone(),
        override_liquid_settings: Some(liquid_settings),
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#feature
pub fn feature(
    placed_feature: Holder<PlacedFeature>,
) -> impl Fn(Projection) -> FeaturePoolElement {
    move |projection| FeaturePoolElement {
        projection: ProjectionState::new(projection),
        feature: placed_feature.clone(),
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement#list
pub fn list(
    elements: Vec<Box<dyn Fn(Projection) -> Box<dyn StructurePoolElement>>>,
) -> impl Fn(Projection) -> ListPoolElement {
    move |projection| ListPoolElement {
        projection: ProjectionState::new(projection),
        elements: elements.iter().map(|f| f(projection)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestElement {
        projection: ProjectionState,
    }

    impl TestElement {
        fn new(projection: Projection) -> Self {
            Self {
                projection: ProjectionState::new(projection),
            }
        }
    }

    impl StructurePoolElement for TestElement {
        fn projection_state(&self) -> &ProjectionState {
            &self.projection
        }
        fn projection_state_mut(&mut self) -> &mut ProjectionState {
            &mut self.projection
        }
        fn get_size(&self, _: &StructureTemplateManager, _: Rotation) -> Vec3i {
            Vec3i { x: -1, y: i32::MIN, z: i32::MAX }
        }
        fn get_shuffled_jigsaw_blocks(
            &self,
            _: &StructureTemplateManager,
            _: BlockPos,
            _: Rotation,
            _: &mut dyn RandomSource,
        ) -> Vec<structure_template::JigsawBlockInfo> {
            Vec::new()
        }
        fn get_bounding_box(&self, _: &StructureTemplateManager, _: BlockPos, _: Rotation) -> BoundingBox {
            BoundingBox { min_x: -10, min_y: -64, min_z: -10, max_x: 10, max_y: 320, max_z: 10 }
        }
        fn place(
            &self,
            _: &StructureTemplateManager,
            _: &mut WorldGenLevel,
            _: &StructureManager,
            _: &ChunkGenerator,
            _: BlockPos,
            _: BlockPos,
            _: Rotation,
            _: BoundingBox,
            _: &mut dyn RandomSource,
            _: LiquidSettings,
            _: bool,
        ) -> bool {
            true
        }
        fn get_type(&self) -> StructurePoolElementType {
            StructurePoolElementType(u32::MAX)
        }
    }

    #[test]
    fn projection_set_get_and_fluent() {
        let mut e = TestElement::new(Projection::Rigid);
        assert_eq!(e.get_projection(), Projection::Rigid);
        let ret = e.set_projection(Projection::TerrainMatching);
        assert_eq!(ret.get_projection(), Projection::TerrainMatching);
        assert_eq!(e.get_projection(), Projection::TerrainMatching);
    }

    #[test]
    #[should_panic(expected = "IllegalStateException")]
    fn get_projection_panics_when_unset() {
        let e = TestElement {
            projection: ProjectionState::unset(),
        };
        let _ = e.get_projection();
    }

    #[test]
    fn ground_level_delta_default() {
        let e = TestElement::new(Projection::Rigid);
        assert_eq!(e.get_ground_level_delta(), 1);
    }

    #[test]
    fn handle_data_marker_default_noop_and_edge_inputs() {
        let mut e = TestElement::new(Projection::TerrainMatching);
        let mut level = LevelAccessor;
        let info = structure_template::StructureBlockInfo;
        let mut rng = SimpleRandomSource { seed: i64::MIN };
        let bb = BoundingBox { min_x: i32::MIN, min_y: -1, min_z: i32::MAX, max_x: 0, max_y: 0, max_z: 0 };
        e.handle_data_marker(&mut level, &info, BlockPos { x: -7, y: 0, z: 7 }, Rotation::Clockwise180, &mut rng, bb);
        assert_eq!(e.get_projection(), Projection::TerrainMatching);
    }

    #[test]
    fn factories_build_expected_elements() {
        let p = Projection::Rigid;
        let empty_el = empty()(p);
        assert_eq!(empty_el.get_projection(), Projection::TerrainMatching); // Java ignores projection arg

        let leg = legacy("minecraft:village/test")(p);
        assert_eq!(leg.get_projection(), Projection::Rigid);
        assert!(matches!(leg.template, Either::Left(Identifier(ref s)) if s == "minecraft:village/test"));

        let s = single_with_processors_and_liquid_settings(
            "minecraft:test",
            Holder::direct(StructureProcessorList(vec![()])),
            LiquidSettings,
        )(Projection::TerrainMatching);
        assert_eq!(s.get_projection(), Projection::TerrainMatching);
        assert!(s.override_liquid_settings.is_some());
        assert_eq!(s.processors.0 .0.len(), 1);

        let f = feature(Holder::direct(PlacedFeature))(p);
        assert_eq!(f.get_projection(), Projection::Rigid);

        let l = list(vec![
            Box::new(|proj| Box::new(TestElement::new(proj)) as Box<dyn StructurePoolElement>),
        ])(Projection::TerrainMatching);
        assert_eq!(l.get_projection(), Projection::TerrainMatching);
        assert_eq!(l.elements.len(), 1);
        assert_eq!(l.elements[0].get_projection(), Projection::TerrainMatching);
    }

    #[test]
    fn negative_and_extreme_integer_fields_preserved() {
        let e = TestElement::new(Projection::Rigid);
        let size = e.get_size(&StructureTemplateManager, Rotation::None);
        assert_eq!(size.x, -1);
        assert_eq!(size.y, i32::MIN);
        assert_eq!(size.z, i32::MAX);
        let bb = e.get_bounding_box(&StructureTemplateManager, BlockPos { x: i32::MIN, y: -64, z: i32::MAX }, Rotation::Clockwise90);
        assert_eq!(bb.min_x, -10);
        assert_eq!(bb.min_y, -64);
        assert_eq!(bb.max_y, 320);
        let ty = e.get_type();
        assert_eq!(ty.0, u32::MAX);
    }
}
