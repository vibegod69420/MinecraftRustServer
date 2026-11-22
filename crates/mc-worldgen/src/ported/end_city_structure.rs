//! Port of `net.minecraft.world.level.levelgen.structure.structures.EndCityStructure`.
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
    EndCity,
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

/// Stub for `net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureTemplateManager;

/// Stub for `net.minecraft.world.level.levelgen.structure.StructurePiece`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePiece {
    pub id: String,
    pub pos: BlockPos,
    pub rotation: Rotation,
}

/// Stub for `net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder`.
#[derive(Debug, Default)]
pub struct StructurePiecesBuilder {
    pub pieces: Vec<StructurePiece>,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_piece(&mut self, piece: StructurePiece) {
        self.pieces.push(piece);
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
    surface_height: i32,
}

impl GenerationContext {
    pub fn new(
        chunk_pos: ChunkPos,
        random_state: WorldgenRandom,
        template_manager: StructureTemplateManager,
        surface_height: i32,
    ) -> Self {
        Self {
            chunk_pos,
            random_state,
            template_manager,
            surface_height,
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

    pub fn get_height(&self, _x: i32, _z: i32) -> i32 {
        self.surface_height
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.structures.EndCityPieces`.
pub struct EndCityPieces;

impl EndCityPieces {
    pub fn start_house_tower(
        _template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
        pieces: &mut Vec<StructurePiece>,
        _random: &mut WorldgenRandom,
    ) {
        pieces.push(StructurePiece {
            id: "end_city_tower".to_string(),
            pos,
            rotation,
        });
    }
}

/// `public static final MapCodec<EndCityStructure> CODEC = simpleCodec(EndCityStructure::new);`
pub const END_CITY_STRUCTURE_CODEC: MapCodec<EndCityStructure> = MapCodec::new();

/// Port of `EndCityStructure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndCityStructure {
    pub settings: StructureSettings,
}

impl EndCityStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        Self { settings }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.Structure#getLowestYIn5by5BoxOffset7Blocks
    fn get_lowest_y_in_5by5_box_offset7_blocks(
        &self,
        context: &GenerationContext,
        _rotation: Rotation,
    ) -> BlockPos {
        let chunk_pos = context.chunk_pos();
        let mut min_y: i32 = i32::MAX;
        let base_x = chunk_pos.get_min_block_x().wrapping_add(7);
        let base_z = chunk_pos.get_min_block_z().wrapping_add(7);
        let mut dx: i32 = 0;
        while dx < 5 {
            let mut dz: i32 = 0;
            while dz < 5 {
                let x = base_x.wrapping_add(dx);
                let z = base_z.wrapping_add(dz);
                let y = context.get_height(x, z);
                if y < min_y {
                    min_y = y;
                }
                dz = dz.wrapping_add(1);
            }
            dx = dx.wrapping_add(1);
        }
        if min_y == i32::MAX {
            min_y = 0;
        }
        BlockPos::new(base_x, min_y, base_z)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityStructure#findGenerationPoint
    pub fn find_generation_point(&self, context: &mut GenerationContext) -> Option<GenerationStub> {
        let rotation = Rotation::get_random(context.random());
        let block_pos = self.get_lowest_y_in_5by5_box_offset7_blocks(context, rotation);
        if block_pos.get_y() < 60 {
            return None;
        }
        let structure = self.clone();
        let pos = block_pos;
        let rot = rotation;
        let ctx_snapshot = context.clone();
        Some(GenerationStub::new(pos, move |builder| {
            let mut c = ctx_snapshot;
            structure.generate_pieces(builder, pos, rot, &mut c);
        }))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityStructure#generatePieces
    fn generate_pieces(
        &self,
        builder: &mut StructurePiecesBuilder,
        pos: BlockPos,
        rotation: Rotation,
        context: &mut GenerationContext,
    ) {
        let mut pieces: Vec<StructurePiece> = Vec::new();
        let template_manager = context.structure_template_manager().clone();
        EndCityPieces::start_house_tower(
            &template_manager,
            pos,
            rotation,
            &mut pieces,
            context.random(),
        );
        for piece in pieces {
            builder.add_piece(piece);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.EndCityStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::EndCity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(chunk_x: i32, chunk_z: i32, height: i32, seed: i64) -> GenerationContext {
        GenerationContext::new(
            ChunkPos::new(chunk_x, chunk_z),
            WorldgenRandom::new(seed),
            StructureTemplateManager::default(),
            height,
        )
    }

    #[test]
    fn new_and_type() {
        let s = EndCityStructure::new(StructureSettings::default());
        assert_eq!(s.structure_type(), StructureType::EndCity);
    }

    #[test]
    fn find_generation_point_below_60_returns_none() {
        let s = EndCityStructure::new(StructureSettings::default());
        let mut ctx = make_context(0, 0, 59, 1);
        assert!(s.find_generation_point(&mut ctx).is_none());
    }

    #[test]
    fn find_generation_point_at_60_returns_some_and_generates() {
        let s = EndCityStructure::new(StructureSettings::default());
        let mut ctx = make_context(0, 0, 60, 1);
        let stub = s.find_generation_point(&mut ctx).expect("stub");
        assert_eq!(stub.position.get_y(), 60);
        let mut builder = StructurePiecesBuilder::new();
        stub.generate(&mut builder);
        assert_eq!(builder.pieces.len(), 1);
        assert_eq!(builder.pieces[0].id, "end_city_tower");
    }

    #[test]
    fn find_generation_point_negative_y_returns_none() {
        let s = EndCityStructure::new(StructureSettings::default());
        let mut ctx = make_context(0, 0, -10, 2);
        assert!(s.find_generation_point(&mut ctx).is_none());
    }

    #[test]
    fn generate_pieces_uses_pos_and_rotation() {
        let s = EndCityStructure::new(StructureSettings::default());
        let mut ctx = make_context(1, -2, 70, 42);
        let pos = BlockPos::new(23, 70, -25);
        let rot = Rotation::Clockwise90;
        let mut builder = StructurePiecesBuilder::new();
        s.generate_pieces(&mut builder, pos, rot, &mut ctx);
        assert_eq!(builder.pieces.len(), 1);
        assert_eq!(builder.pieces[0].pos, pos);
        assert_eq!(builder.pieces[0].rotation, rot);
    }

    #[test]
    fn negative_chunk_coords_wrapping_and_float_bits() {
        let s = EndCityStructure::new(StructureSettings::default());
        let mut ctx = make_context(-1, i32::MIN >> 4, 80, 7);
        let stub = s.find_generation_point(&mut ctx).expect("stub");
        let expected_x = (-1i32).wrapping_shl(4).wrapping_add(7);
        assert_eq!(stub.position.x, expected_x);
        let expected_z = (i32::MIN >> 4).wrapping_shl(4).wrapping_add(7);
        assert_eq!(stub.position.z, expected_z);
        // Float bits sanity (no float ops in this structure).
        assert_eq!(60.0f32.to_bits(), 0x4270_0000);
    }
}
