use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockX
    pub fn get_min_block_x(&self) -> i32 {
        self.x.wrapping_mul(16)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockZ
    pub fn get_min_block_z(&self) -> i32 {
        self.z.wrapping_mul(16)
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorldgenRandom {
    pub seed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePiece {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Default)]
pub struct StructurePiecesBuilder {
    pub pieces: Vec<StructurePiece>,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self { pieces: Vec::new() }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder#addPiece
    pub fn add_piece(&mut self, piece: StructurePiece) {
        self.pieces.push(piece);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    WorldSurfaceWg,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StructureSettings;

#[derive(Debug, Clone)]
pub struct ChunkGenerator {
    pub sea_level: i32,
}

impl ChunkGenerator {
    // JAVA: net.minecraft.world.level.chunk.ChunkGenerator#getSeaLevel
    pub fn get_sea_level(&self) -> i32 {
        self.sea_level
    }
}

#[derive(Debug, Clone)]
pub struct GenerationContext {
    pub chunk_pos: ChunkPos,
    pub chunk_generator: ChunkGenerator,
    pub random: WorldgenRandom,
    /// Stub value returned by `get_lowest_y` for self-contained testing.
    pub lowest_y: i32,
}

impl GenerationContext {
    // JAVA: net.minecraft.world.level.levelgen.structure.Structure$GenerationContext#chunkPos
    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.Structure$GenerationContext#chunkGenerator
    pub fn chunk_generator(&self) -> &ChunkGenerator {
        &self.chunk_generator
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.Structure$GenerationContext#random
    pub fn random(&mut self) -> &mut WorldgenRandom {
        &mut self.random
    }
}

#[derive(Debug, Clone)]
pub struct GenerationStub {
    pub position: ChunkPos,
    pub pieces: StructurePiecesBuilder,
}

// JAVA: net.minecraft.world.level.levelgen.structure.Structure#getLowestY
pub fn get_lowest_y(context: &GenerationContext, _width: i32, _depth: i32) -> i32 {
    context.lowest_y
}

// JAVA: net.minecraft.world.level.levelgen.structure.Structure#onTopOfChunkCenter
pub fn on_top_of_chunk_center<F>(
    context: &mut GenerationContext,
    _heightmap: HeightmapType,
    mut generator: F,
) -> Option<GenerationStub>
where
    F: FnMut(&mut StructurePiecesBuilder, &mut GenerationContext),
{
    let mut builder = StructurePiecesBuilder::new();
    generator(&mut builder, context);
    Some(GenerationStub {
        position: context.chunk_pos(),
        pieces: builder,
    })
}

pub trait PieceConstructor {
    // JAVA: net.minecraft.world.level.levelgen.structure.SinglePieceStructure$PieceConstructor#construct
    fn construct(&self, random: &mut WorldgenRandom, x: i32, z: i32) -> StructurePiece;
}

impl<F> PieceConstructor for F
where
    F: Fn(&mut WorldgenRandom, i32, i32) -> StructurePiece,
{
    fn construct(&self, random: &mut WorldgenRandom, x: i32, z: i32) -> StructurePiece {
        (self)(random, x, z)
    }
}

pub struct SinglePieceStructure<C> {
    pub settings: StructureSettings,
    pub constructor: C,
    pub width: i32,
    pub depth: i32,
}

impl<C: fmt::Debug> fmt::Debug for SinglePieceStructure<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SinglePieceStructure")
            .field("settings", &self.settings)
            .field("constructor", &self.constructor)
            .field("width", &self.width)
            .field("depth", &self.depth)
            .finish()
    }
}

impl<C: PieceConstructor> SinglePieceStructure<C> {
    // JAVA: net.minecraft.world.level.levelgen.structure.SinglePieceStructure#<init>
    pub fn new(constructor: C, width: i32, depth: i32, settings: StructureSettings) -> Self {
        Self {
            settings,
            constructor,
            width,
            depth,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.SinglePieceStructure#findGenerationPoint
    pub fn find_generation_point(&self, context: &mut GenerationContext) -> Option<GenerationStub> {
        let lowest_y = get_lowest_y(context, self.width, self.depth);
        let sea_level = context.chunk_generator().get_sea_level();
        if lowest_y < sea_level {
            return None;
        }
        on_top_of_chunk_center(context, HeightmapType::WorldSurfaceWg, |builder, ctx| {
            self.generate_pieces(builder, ctx);
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.SinglePieceStructure#generatePieces
    fn generate_pieces(&self, builder: &mut StructurePiecesBuilder, context: &mut GenerationContext) {
        let chunk_pos = context.chunk_pos();
        let x = chunk_pos.get_min_block_x();
        let z = chunk_pos.get_min_block_z();
        let piece = self.constructor.construct(context.random(), x, z);
        builder.add_piece(piece);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(chunk_x: i32, chunk_z: i32, sea_level: i32, lowest_y: i32) -> GenerationContext {
        GenerationContext {
            chunk_pos: ChunkPos {
                x: chunk_x,
                z: chunk_z,
            },
            chunk_generator: ChunkGenerator { sea_level },
            random: WorldgenRandom { seed: 0 },
            lowest_y,
        }
    }

    fn simple_structure() -> SinglePieceStructure<fn(&mut WorldgenRandom, i32, i32) -> StructurePiece> {
        fn ctor(_r: &mut WorldgenRandom, x: i32, z: i32) -> StructurePiece {
            StructurePiece { x, z }
        }
        SinglePieceStructure::new(ctor, 10, 10, StructureSettings)
    }

    #[test]
    fn returns_none_when_below_sea_level() {
        let s = simple_structure();
        let mut ctx = make_context(0, 0, 63, 62);
        assert!(s.find_generation_point(&mut ctx).is_none());
    }

    #[test]
    fn returns_some_at_sea_level_and_places_piece() {
        let s = simple_structure();
        let mut ctx = make_context(1, 2, 63, 63);
        let stub = s.find_generation_point(&mut ctx).expect("should generate");
        assert_eq!(stub.position, ChunkPos { x: 1, z: 2 });
        assert_eq!(stub.pieces.pieces.len(), 1);
        assert_eq!(stub.pieces.pieces[0], StructurePiece { x: 16, z: 32 });
    }

    #[test]
    fn negative_chunk_coords_produce_negative_block_coords() {
        let s = simple_structure();
        let mut ctx = make_context(-1, -2, 0, 10);
        let stub = s.find_generation_point(&mut ctx).expect("should generate");
        assert_eq!(
            stub.pieces.pieces[0],
            StructurePiece { x: -16, z: -32 }
        );
    }

    #[test]
    fn chunk_coord_wrapping_matches_java_int_mul() {
        let s = simple_structure();
        let mut ctx = make_context(i32::MAX, i32::MIN, 0, 0);
        let stub = s.find_generation_point(&mut ctx).expect("should generate");
        assert_eq!(stub.pieces.pieces[0].x, i32::MAX.wrapping_mul(16));
        assert_eq!(stub.pieces.pieces[0].z, i32::MIN.wrapping_mul(16));
    }

    #[test]
    fn constructor_stores_fields() {
        let s = SinglePieceStructure::new(
            |_r: &mut WorldgenRandom, x: i32, z: i32| StructurePiece { x, z },
            7,
            9,
            StructureSettings,
        );
        assert_eq!(s.width, 7);
        assert_eq!(s.depth, 9);
        assert_eq!(s.settings, StructureSettings);
    }
}
