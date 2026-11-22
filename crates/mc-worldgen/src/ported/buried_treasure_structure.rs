use std::marker::PhantomData;

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.StructureSettings`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructureSettings;

/// Stub for `net.minecraft.world.level.levelgen.structure.StructureType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    BuriedTreasure,
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
    OceanFloorWg,
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

/// Stub for `net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces`.
pub mod buried_treasure_pieces {
    use super::BlockPos;

    /// Stub for `net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces.BuriedTreasurePiece`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BuriedTreasurePiece {
        pub pos: BlockPos,
    }

    impl BuriedTreasurePiece {
        // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces.BuriedTreasurePiece#<init>
        pub fn new(pos: BlockPos) -> Self {
            Self { pos }
        }
    }
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
    pub pieces: Vec<buried_treasure_pieces::BuriedTreasurePiece>,
}

impl StructurePiecesBuilder {
    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder#addPiece
    pub fn add_piece(&mut self, piece: buried_treasure_pieces::BuriedTreasurePiece) {
        self.pieces.push(piece);
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.Structure.GenerationStub`.
#[derive(Debug, Clone)]
pub struct GenerationStub {
    pub position: BlockPos,
    pub heightmap: HeightmapType,
    pub pieces: Vec<buried_treasure_pieces::BuriedTreasurePiece>,
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

/// Port of `net.minecraft.world.level.levelgen.structure.structures.BuriedTreasureStructure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuriedTreasureStructure {
    pub settings: StructureSettings,
}

impl BuriedTreasureStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasureStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        Self { settings }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasureStructure#findGenerationPoint
    pub fn find_generation_point(&self, context: &GenerationContext) -> Option<GenerationStub> {
        on_top_of_chunk_center(context, HeightmapType::OceanFloorWg, |builder| {
            Self::generate_pieces(builder, context)
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasureStructure#generatePieces
    fn generate_pieces(builder: &mut StructurePiecesBuilder, context: &GenerationContext) {
        let pos = BlockPos {
            x: context.chunk_pos().get_block_x(9),
            y: 90,
            z: context.chunk_pos().get_block_z(9),
        };
        builder.add_piece(buried_treasure_pieces::BuriedTreasurePiece::new(pos));
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasureStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::BuriedTreasure
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasureStructure#CODEC
pub static CODEC: MapCodec<BuriedTreasureStructure> =
    simple_codec(BuriedTreasureStructure::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structure_type_is_buried_treasure() {
        let s = BuriedTreasureStructure::new(StructureSettings);
        assert_eq!(s.structure_type(), StructureType::BuriedTreasure);
    }

    #[test]
    fn generate_pieces_places_piece_at_block_xz_9_y_90() {
        let ctx = GenerationContext {
            chunk_pos: ChunkPos { x: 4, z: -3 },
            random_source: RandomSource::default(),
        };
        let s = BuriedTreasureStructure::new(StructureSettings);
        let stub = s.find_generation_point(&ctx).expect("generation stub");
        assert_eq!(stub.heightmap, HeightmapType::OceanFloorWg);
        assert_eq!(stub.pieces.len(), 1);
        let pos = stub.pieces[0].pos;
        assert_eq!(pos.x, 4_i32.wrapping_shl(4).wrapping_add(9));
        assert_eq!(pos.y, 90);
        assert_eq!(pos.z, (-3_i32).wrapping_shl(4).wrapping_add(9));
    }

    #[test]
    fn zero_chunk_origin() {
        let ctx = GenerationContext {
            chunk_pos: ChunkPos { x: 0, z: 0 },
            random_source: RandomSource { seed: 1 },
        };
        let s = BuriedTreasureStructure::new(StructureSettings);
        let stub = s.find_generation_point(&ctx).expect("generation stub");
        let pos = stub.pieces[0].pos;
        assert_eq!(pos.x, 9);
        assert_eq!(pos.y, 90);
        assert_eq!(pos.z, 9);
    }

    #[test]
    fn chunk_block_coords_wrapping_overflow() {
        let ctx = GenerationContext {
            chunk_pos: ChunkPos {
                x: i32::MAX,
                z: i32::MIN,
            },
            random_source: RandomSource::default(),
        };
        let s = BuriedTreasureStructure::new(StructureSettings);
        let stub = s.find_generation_point(&ctx).expect("generation stub");
        let pos = stub.pieces[0].pos;
        assert_eq!(pos.x, i32::MAX.wrapping_shl(4).wrapping_add(9));
        assert_eq!(pos.y, 90);
        assert_eq!(pos.z, i32::MIN.wrapping_shl(4).wrapping_add(9));
    }

    #[test]
    fn codec_static_is_accessible() {
        let _ = CODEC;
    }
}
