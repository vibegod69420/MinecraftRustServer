use std::marker::PhantomData;

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.StructureSettings`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructureSettings;

/// Stub for `net.minecraft.world.level.levelgen.structure.StructureType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    SwampHut,
}

/// Stub for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<T>(PhantomData<fn() -> T>);

// JAVA: net.minecraft.world.level.levelgen.structure.Structure#simpleCodec
pub const fn simple_codec<T>(_ctor: fn(StructureSettings) -> T) -> MapCodec<T> {
    MapCodec(PhantomData)
}

/// Stub for `net.minecraft.world.level.levelgen.Heightmap.Types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    WorldSurfaceWg,
}

/// Stub for `net.minecraft.core.BlockPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Stub for `net.minecraft.world.level.ChunkPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockX
    pub fn get_min_block_x(&self) -> i32 {
        self.x.wrapping_shl(4)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockZ
    pub fn get_min_block_z(&self) -> i32 {
        self.z.wrapping_shl(4)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getBlockX
    pub fn get_block_x(&self, offset: i32) -> i32 {
        self.get_min_block_x().wrapping_add(offset)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getBlockZ
    pub fn get_block_z(&self, offset: i32) -> i32 {
        self.get_min_block_z().wrapping_add(offset)
    }
}

/// Stub for `net.minecraft.util.RandomSource`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RandomSource {
    pub seed: i64,
}

/// Stub for `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwampHutPiece {
    pub random: RandomSource,
    pub min_block_x: i32,
    pub min_block_z: i32,
}

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.GenerationContext`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenerationContext {
    pub chunk_pos: ChunkPos,
    pub random_source: RandomSource,
}

impl GenerationContext {
    // JAVA: net.minecraft.world.level.levelgen.structure.Structure.GenerationContext#random
    pub fn random(&self) -> RandomSource {
        self.random_source.clone()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.Structure.GenerationContext#chunkPos
    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder`.
#[derive(Debug, Clone, Default)]
pub struct StructurePiecesBuilder {
    pub pieces: Vec<SwampHutPiece>,
}

impl StructurePiecesBuilder {
    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder#addPiece
    pub fn add_piece(&mut self, piece: SwampHutPiece) {
        self.pieces.push(piece);
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.GenerationStub`.
#[derive(Debug, Clone)]
pub struct GenerationStub {
    pub position: BlockPos,
    pub heightmap: HeightmapType,
    pub pieces: Vec<SwampHutPiece>,
}

// JAVA: net.minecraft.world.level.levelgen.structure.Structure#onTopOfChunkCenter
pub fn on_top_of_chunk_center<F>(
    context: &GenerationContext,
    heightmap: HeightmapType,
    generator: F,
) -> Option<GenerationStub>
where
    F: FnOnce(&mut StructurePiecesBuilder),
{
    let mut builder = StructurePiecesBuilder::default();
    generator(&mut builder);
    let pos = BlockPos {
        x: context.chunk_pos().get_block_x(8),
        y: 0,
        z: context.chunk_pos().get_block_z(8),
    };
    Some(GenerationStub {
        position: pos,
        heightmap,
        pieces: builder.pieces,
    })
}

/// Port of `net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwampHutStructure {
    pub settings: StructureSettings,
}

impl SwampHutStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        Self { settings }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure#findGenerationPoint
    pub fn find_generation_point(&self, context: &GenerationContext) -> Option<GenerationStub> {
        on_top_of_chunk_center(context, HeightmapType::WorldSurfaceWg, |builder| {
            Self::generate_pieces(builder, context)
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure#generatePieces
    fn generate_pieces(builder: &mut StructurePiecesBuilder, context: &GenerationContext) {
        builder.add_piece(SwampHutPiece {
            random: context.random(),
            min_block_x: context.chunk_pos().get_min_block_x(),
            min_block_z: context.chunk_pos().get_min_block_z(),
        });
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::SwampHut
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure#CODEC
pub static CODEC: MapCodec<SwampHutStructure> = simple_codec(SwampHutStructure::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structure_type_is_swamp_hut() {
        let s = SwampHutStructure::new(StructureSettings);
        assert_eq!(s.structure_type(), StructureType::SwampHut);
    }

    #[test]
    fn generate_pieces_uses_chunk_min_block_and_random() {
        let ctx = GenerationContext {
            chunk_pos: ChunkPos { x: 3, z: -2 },
            random_source: RandomSource { seed: 42 },
        };
        let s = SwampHutStructure::new(StructureSettings);
        let stub = s.find_generation_point(&ctx).expect("generation stub");
        assert_eq!(stub.heightmap, HeightmapType::WorldSurfaceWg);
        assert_eq!(stub.pieces.len(), 1);
        let piece = &stub.pieces[0];
        assert_eq!(piece.min_block_x, 3_i32.wrapping_shl(4));
        assert_eq!(piece.min_block_z, (-2_i32).wrapping_shl(4));
        assert_eq!(piece.random, RandomSource { seed: 42 });
    }

    #[test]
    fn negative_and_zero_chunk_coords() {
        let ctx = GenerationContext {
            chunk_pos: ChunkPos { x: 0, z: -1 },
            random_source: RandomSource { seed: -7 },
        };
        let s = SwampHutStructure::new(StructureSettings);
        let stub = s.find_generation_point(&ctx).expect("generation stub");
        let piece = &stub.pieces[0];
        assert_eq!(piece.min_block_x, 0);
        assert_eq!(piece.min_block_z, -16);
        assert_eq!(piece.random.seed, -7);
    }

    #[test]
    fn chunk_min_block_wrapping_overflow() {
        let ctx = GenerationContext {
            chunk_pos: ChunkPos {
                x: i32::MAX,
                z: i32::MIN,
            },
            random_source: RandomSource { seed: 0 },
        };
        let s = SwampHutStructure::new(StructureSettings);
        let stub = s.find_generation_point(&ctx).expect("generation stub");
        let piece = &stub.pieces[0];
        assert_eq!(piece.min_block_x, i32::MAX.wrapping_shl(4));
        assert_eq!(piece.min_block_z, i32::MIN.wrapping_shl(4));
    }

    #[test]
    fn codec_static_is_accessible() {
        let _ = CODEC;
    }
}
