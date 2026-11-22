//! Port of `net.minecraft.world.level.levelgen.structure.structures.IglooStructure`.
//! Self-contained with minimal stubs for unported Minecraft dependencies.

use std::fmt;
use std::marker::PhantomData;

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.StructureSettings`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureSettings;

/// Stub for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<T> {
    _phantom: PhantomData<T>,
}

impl<T> MapCodec<T> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.StructureType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Igloo,
}

/// Stub for `net.minecraft.core.BlockPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }
}

/// Stub for `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Stub for `net.minecraft.world.level.levelgen.WorldgenRandom`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldgenRandom {
    state: i64,
}

impl WorldgenRandom {
    pub fn new(seed: i64) -> Self {
        Self { state: seed }
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005i64)
            .wrapping_add(1_442_695_040_888_963_407i64);
        let val = ((self.state >> 33) as i32) & i32::MAX;
        val % bound
    }
}

/// Stub for `net.minecraft.world.level.block.Rotation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    pub fn get_random(random: &mut WorldgenRandom) -> Self {
        match random.next_int(4) & 3 {
            0 => Rotation::None,
            1 => Rotation::Clockwise90,
            2 => Rotation::Clockwise180,
            _ => Rotation::Counterclockwise90,
        }
    }
}

/// Stub for `net.minecraft.world.level.levelgen.Heightmap.Types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    WorldSurfaceWg,
}

/// Stub for `net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureTemplateManager;

/// Record of a piece placement for testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiecePlacement {
    pub pos: BlockPos,
    pub rotation: Rotation,
}

/// Stub for `net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder`.
#[derive(Debug, Default)]
pub struct StructurePiecesBuilder {
    pub placements: Vec<PiecePlacement>,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_piece(&mut self, placement: PiecePlacement) {
        self.placements.push(placement);
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.GenerationStub`.
pub struct GenerationStub {
    pub position: BlockPos,
    generator: Box<dyn FnOnce(&mut StructurePiecesBuilder)>,
}

impl fmt::Debug for GenerationStub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenerationStub")
            .field("position", &self.position)
            .finish_non_exhaustive()
    }
}

impl GenerationStub {
    pub fn new<F>(position: BlockPos, generator: F) -> Self
    where
        F: FnOnce(&mut StructurePiecesBuilder) + 'static,
    {
        Self {
            position,
            generator: Box::new(generator),
        }
    }

    pub fn generate(self, builder: &mut StructurePiecesBuilder) {
        (self.generator)(builder);
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.GenerationContext`.
#[derive(Debug, Clone)]
pub struct GenerationContext {
    chunk_pos: ChunkPos,
    random_state: WorldgenRandom,
    template_manager: StructureTemplateManager,
}

impl GenerationContext {
    pub fn new(
        chunk_pos: ChunkPos,
        random_state: WorldgenRandom,
        template_manager: StructureTemplateManager,
    ) -> Self {
        Self {
            chunk_pos,
            random_state,
            template_manager,
        }
    }

    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn random(&mut self) -> &mut WorldgenRandom {
        &mut self.random_state
    }

    pub fn structure_template_manager(&self) -> &StructureTemplateManager {
        &self.template_manager
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.structures.IglooPieces`.
pub struct IglooPieces;

impl IglooPieces {
    pub fn add_pieces(
        _template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
        builder: &mut StructurePiecesBuilder,
        _random: &mut WorldgenRandom,
    ) {
        builder.add_piece(PiecePlacement { pos, rotation });
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.Structure#onTopOfChunkCenter
pub fn on_top_of_chunk_center<F>(
    context: &GenerationContext,
    _heightmap_type: HeightmapType,
    generator: F,
) -> Option<GenerationStub>
where
    F: FnOnce(&mut StructurePiecesBuilder) + 'static,
{
    let cp = context.chunk_pos();
    let x = cp.get_min_block_x().wrapping_add(7);
    let z = cp.get_min_block_z().wrapping_add(7);
    let pos = BlockPos::new(x, 0, z);
    Some(GenerationStub::new(pos, generator))
}

/// `public static final MapCodec<IglooStructure> CODEC = simpleCodec(IglooStructure::new);`
pub const IGLOO_STRUCTURE_CODEC: MapCodec<IglooStructure> = MapCodec::new();

/// Port of `IglooStructure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IglooStructure {
    pub settings: StructureSettings,
}

impl IglooStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.IglooStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        Self { settings }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.IglooStructure#findGenerationPoint
    pub fn find_generation_point(&self, context: &mut GenerationContext) -> Option<GenerationStub> {
        let ctx_snapshot = context.clone();
        let structure = self.clone();
        on_top_of_chunk_center(
            context,
            HeightmapType::WorldSurfaceWg,
            move |builder| {
                let mut ctx = ctx_snapshot;
                structure.generate_pieces(builder, &mut ctx);
            },
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.IglooStructure#generatePieces
    fn generate_pieces(&self, builder: &mut StructurePiecesBuilder, context: &mut GenerationContext) {
        let chunk_pos = context.chunk_pos();
        let block_pos = BlockPos::new(chunk_pos.get_min_block_x(), 90, chunk_pos.get_min_block_z());
        let rotation = Rotation::get_random(context.random());
        let template_manager = context.structure_template_manager().clone();
        IglooPieces::add_pieces(
            &template_manager,
            block_pos,
            rotation,
            builder,
            context.random(),
        );
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.IglooStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::Igloo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_type() {
        let s = IglooStructure::new(StructureSettings::default());
        assert_eq!(s.structure_type(), StructureType::Igloo);
        assert_eq!(s.settings, StructureSettings::default());
    }

    #[test]
    fn generate_pieces_uses_min_block_xz_and_y_90() {
        let s = IglooStructure::new(StructureSettings::default());
        let mut ctx = GenerationContext::new(
            ChunkPos::new(3, -2),
            WorldgenRandom::new(12_345),
            StructureTemplateManager::default(),
        );
        let mut builder = StructurePiecesBuilder::new();
        s.generate_pieces(&mut builder, &mut ctx);
        assert_eq!(builder.placements.len(), 1);
        let p = &builder.placements[0];
        assert_eq!(
            p.pos,
            BlockPos::new(3i32.wrapping_shl(4), 90, (-2i32).wrapping_shl(4))
        );
    }

    #[test]
    fn generate_pieces_negative_chunk_coords() {
        let s = IglooStructure::new(StructureSettings::default());
        let mut ctx = GenerationContext::new(
            ChunkPos::new(-1, -1),
            WorldgenRandom::new(0),
            StructureTemplateManager::default(),
        );
        let mut builder = StructurePiecesBuilder::new();
        s.generate_pieces(&mut builder, &mut ctx);
        assert_eq!(builder.placements[0].pos.x, -16);
        assert_eq!(builder.placements[0].pos.y, 90);
        assert_eq!(builder.placements[0].pos.z, -16);
    }

    #[test]
    fn chunk_pos_min_block_wrapping_shift_edge() {
        let cp = ChunkPos::new(i32::MAX, i32::MIN);
        assert_eq!(cp.get_min_block_x(), i32::MAX.wrapping_shl(4));
        assert_eq!(cp.get_min_block_z(), i32::MIN.wrapping_shl(4));
    }

    #[test]
    fn find_generation_point_returns_stub_and_generates() {
        let s = IglooStructure::new(StructureSettings::default());
        let mut ctx = GenerationContext::new(
            ChunkPos::new(0, 0),
            WorldgenRandom::new(1),
            StructureTemplateManager::default(),
        );
        let stub = s.find_generation_point(&mut ctx).expect("stub");
        let mut builder = StructurePiecesBuilder::new();
        stub.generate(&mut builder);
        assert_eq!(builder.placements.len(), 1);
        assert_eq!(builder.placements[0].pos.y, 90);
        // Float bits sanity (no float ops in this structure).
        assert_eq!(90.0f32.to_bits(), 0x42B4_0000);
    }
}
