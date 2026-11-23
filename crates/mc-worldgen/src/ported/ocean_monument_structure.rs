use std::sync::atomic::{AtomicI64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub const fn new(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.BoundingBox#minX
    pub fn min_x(&self) -> i32 {
        self.min_x
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.BoundingBox#minZ
    pub fn min_z(&self) -> i32 {
        self.min_z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getBlockX(int)
    pub fn get_block_x(&self, offset: i32) -> i32 {
        self.x.wrapping_mul(16).wrapping_add(offset)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getBlockZ(int)
    pub fn get_block_z(&self, offset: i32) -> i32 {
        self.z.wrapping_mul(16).wrapping_add(offset)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockX
    pub fn get_min_block_x(&self) -> i32 {
        self.x.wrapping_mul(16)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockZ
    pub fn get_min_block_z(&self) -> i32 {
        self.z.wrapping_mul(16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone)]
pub struct RandomSource {
    seed: i64,
}

impl RandomSource {
    const MULTIPLIER: i64 = 0x5DEECE66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1_i64 << 48) - 1;

    // JAVA: net.minecraft.util.RandomSource#create(long)
    pub fn create(seed: i64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: i32) -> i32 {
        self.seed = (self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND))
            & Self::MASK;
        ((self.seed as u64) >> (48_i32.wrapping_sub(bits)) as u32) as i32
    }

    // JAVA: net.minecraft.util.RandomSource#nextInt(int)
    pub fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            panic!("bound must be positive");
        }
        if (bound & bound.wrapping_neg()) == bound {
            return ((i64::from(bound).wrapping_mul(i64::from(self.next(31)))) >> 31) as i32;
        }
        loop {
            let bits = self.next(31);
            let val = bits % bound;
            if bits.wrapping_sub(val).wrapping_add(bound.wrapping_sub(1)) >= 0 {
                return val;
            }
        }
    }
}

// JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#<init>
pub fn legacy_random_source(seed: i64) -> RandomSource {
    RandomSource::create(seed)
}

static SEED_UNIQUIFIER: AtomicI64 = AtomicI64::new(8682522807148012_i64);

// JAVA: net.minecraft.util.RandomSupport#generateUniqueSeed
pub fn generate_unique_seed() -> i64 {
    let updated = SEED_UNIQUIFIER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |s| {
            Some(s.wrapping_mul(1181783497276652981_i64))
        })
        .unwrap_or(8682522807148012_i64)
        .wrapping_mul(1181783497276652981_i64);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);
    updated ^ nanos
}

#[derive(Debug, Clone)]
pub struct WorldgenRandom {
    random: RandomSource,
}

impl WorldgenRandom {
    // JAVA: net.minecraft.world.level.levelgen.WorldgenRandom#<init>(net.minecraft.util.RandomSource)
    pub fn new(random: RandomSource) -> Self {
        Self { random }
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldgenRandom#setLargeFeatureSeed
    pub fn set_large_feature_seed(&mut self, level_seed: i64, chunk_x: i32, chunk_z: i32) -> i64 {
        let mut seed = level_seed;
        seed = seed.wrapping_add(i64::from(chunk_x).wrapping_mul(i64::from(chunk_x).wrapping_mul(4987142)));
        seed = seed.wrapping_add(i64::from(chunk_z).wrapping_mul(i64::from(chunk_z).wrapping_mul(5947611)));
        seed = seed.wrapping_add(i64::from(chunk_x).wrapping_mul(4392871_i64));
        seed = seed.wrapping_add(i64::from(chunk_z).wrapping_mul(389711_i64));
        seed ^= 987234911_i64;
        self.random = RandomSource::create(seed);
        seed
    }

    // JAVA: net.minecraft.util.RandomSource#nextInt(int)
    pub fn next_int(&mut self, bound: i32) -> i32 {
        self.random.next_int(bound)
    }
}

// JAVA: net.minecraft.core.Direction$Plane#HORIZONTAL#getRandomDirection
pub fn horizontal_get_random_direction(random: &mut WorldgenRandom) -> Direction {
    match random.next_int(4) {
        0 => Direction::North,
        1 => Direction::South,
        2 => Direction::West,
        _ => Direction::East,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonumentBuilding {
    pub bounding_box: BoundingBox,
    pub orientation: Option<Direction>,
}

impl MonumentBuilding {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentPieces$MonumentBuilding#<init>
    pub fn new(_random: &mut WorldgenRandom, x: i32, z: i32, direction: Direction) -> Self {
        // Self-contained stub: preserve placement inputs in a representative bounding box.
        Self {
            bounding_box: BoundingBox::new(x, 0, z, x.wrapping_add(58), 23, z.wrapping_add(58)),
            orientation: Some(direction),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructurePiece {
    MonumentBuilding(MonumentBuilding),
    Other {
        bounding_box: BoundingBox,
        orientation: Option<Direction>,
    },
}

impl StructurePiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#getBoundingBox
    pub fn get_bounding_box(&self) -> BoundingBox {
        match self {
            StructurePiece::MonumentBuilding(p) => p.bounding_box,
            StructurePiece::Other { bounding_box, .. } => *bounding_box,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#getOrientation
    pub fn get_orientation(&self) -> Option<Direction> {
        match self {
            StructurePiece::MonumentBuilding(p) => p.orientation,
            StructurePiece::Other { orientation, .. } => *orientation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiecesContainer {
    pub pieces: Vec<StructurePiece>,
}

impl PiecesContainer {
    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer#isEmpty
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer#pieces
    pub fn pieces(&self) -> &[StructurePiece] {
        &self.pieces
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructurePiecesBuilder {
    pieces: Vec<StructurePiece>,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self { pieces: Vec::new() }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder#addPiece
    pub fn add_piece(&mut self, piece: StructurePiece) {
        self.pieces.push(piece);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder#build
    pub fn build(self) -> PiecesContainer {
        PiecesContainer {
            pieces: self.pieces,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    OceanFloorWg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureSettings;

pub struct GenerationContext {
    pub chunk_pos: ChunkPos,
    pub random: WorldgenRandom,
    pub sea_level: i32,
    pub biomes_required_ocean_monument_surrounding: bool,
}

impl GenerationContext {
    // JAVA: net.minecraft.world.level.levelgen.structure.Structure$GenerationContext#chunkPos
    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.Structure$GenerationContext#random
    pub fn random(&mut self) -> &mut WorldgenRandom {
        &mut self.random
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.Structure$GenerationContext#chunkGenerator
    pub fn chunk_generator_sea_level(&self) -> i32 {
        self.sea_level
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationStub {
    pub position: BlockPos,
    pub pieces: PiecesContainer,
}

pub struct OceanMonumentStructure {
    pub settings: StructureSettings,
}

impl OceanMonumentStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        Self { settings }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure#onTopOfChunkCenter (stubbed helper)
    fn on_top_of_chunk_center(
        context: &mut GenerationContext,
        _heightmap: HeightmapType,
        mut generator: impl FnMut(&mut StructurePiecesBuilder, &mut GenerationContext),
    ) -> Option<GenerationStub> {
        let mut builder = StructurePiecesBuilder::new();
        generator(&mut builder, context);
        let cp = context.chunk_pos();
        Some(GenerationStub {
            position: BlockPos::new(cp.get_block_x(9), context.chunk_generator_sea_level(), cp.get_block_z(9)),
            pieces: builder.build(),
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure#findGenerationPoint
    pub fn find_generation_point(&self, mut context: GenerationContext) -> Option<GenerationStub> {
        let _x = context.chunk_pos().get_block_x(9);
        let _z = context.chunk_pos().get_block_z(9);

        if !context.biomes_required_ocean_monument_surrounding {
            return None;
        }

        Self::on_top_of_chunk_center(&mut context, HeightmapType::OceanFloorWg, |builder, ctx| {
            generate_pieces(builder, ctx);
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure#type
    pub fn structure_type(&self) -> &'static str {
        "ocean_monument"
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure#createTopPiece
fn create_top_piece(chunk_pos: ChunkPos, random: &mut WorldgenRandom) -> StructurePiece {
    let x = chunk_pos.get_min_block_x().wrapping_sub(29);
    let z = chunk_pos.get_min_block_z().wrapping_sub(29);
    let direction = horizontal_get_random_direction(random);
    StructurePiece::MonumentBuilding(MonumentBuilding::new(random, x, z, direction))
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure#generatePieces
fn generate_pieces(builder: &mut StructurePiecesBuilder, context: &mut GenerationContext) {
    let chunk_pos = context.chunk_pos();
    let piece = create_top_piece(chunk_pos, context.random());
    builder.add_piece(piece);
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure#regeneratePiecesAfterLoad
pub fn regenerate_pieces_after_load(
    chunk_pos: ChunkPos,
    seed: i64,
    pieces: PiecesContainer,
) -> PiecesContainer {
    if pieces.is_empty() {
        return pieces;
    }

    let mut random = WorldgenRandom::new(legacy_random_source(generate_unique_seed()));
    random.set_large_feature_seed(seed, chunk_pos.x, chunk_pos.z);
    let first_piece = &pieces.pieces()[0];
    let bounding_box = first_piece.get_bounding_box();
    let x = bounding_box.min_x();
    let z = bounding_box.min_z();
    let direction = horizontal_get_random_direction(&mut random);
    let orientation = first_piece.get_orientation().unwrap_or(direction);
    let rebuilt = StructurePiece::MonumentBuilding(MonumentBuilding::new(&mut random, x, z, orientation));
    let mut builder = StructurePiecesBuilder::new();
    builder.add_piece(rebuilt);
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_pos_block_coords_and_negative() {
        let cp = ChunkPos::new(-1, 2);
        assert_eq!(cp.get_min_block_x(), -16);
        assert_eq!(cp.get_min_block_z(), 32);
        assert_eq!(cp.get_block_x(9), -7);
        assert_eq!(cp.get_block_z(9), 41);
    }

    #[test]
    fn test_horizontal_random_direction_range() {
        let mut rng = WorldgenRandom::new(legacy_random_source(1234_i64));
        for _ in 0..20 {
            let d = horizontal_get_random_direction(&mut rng);
            assert!(matches!(
                d,
                Direction::North | Direction::South | Direction::West | Direction::East
            ));
        }
    }

    #[test]
    fn test_regenerate_pieces_after_load_empty_returns_same() {
        let empty = PiecesContainer { pieces: Vec::new() };
        let out = regenerate_pieces_after_load(ChunkPos::new(0, 0), 1_i64, empty.clone());
        assert_eq!(out, empty);
    }

    #[test]
    fn test_regenerate_pieces_after_load_rebuilds_first_piece() {
        let original = PiecesContainer {
            pieces: vec![StructurePiece::MonumentBuilding(MonumentBuilding {
                bounding_box: BoundingBox::new(10, 0, 20, 68, 23, 78),
                orientation: Some(Direction::West),
            })],
        };
        let rebuilt = regenerate_pieces_after_load(ChunkPos::new(3, -4), 987654321_i64, original);
        assert_eq!(rebuilt.pieces().len(), 1);
        match &rebuilt.pieces()[0] {
            StructurePiece::MonumentBuilding(mb) => {
                assert_eq!(mb.bounding_box.min_x(), 10);
                assert_eq!(mb.bounding_box.min_z(), 20);
                assert_eq!(mb.orientation, Some(Direction::West));
            }
            _ => panic!("expected monument building"),
        }
    }

    #[test]
    fn test_find_generation_point_rejects_missing_biome_tag() {
        let structure = OceanMonumentStructure::new(StructureSettings);
        let ctx = GenerationContext {
            chunk_pos: ChunkPos::new(1, 2),
            random: WorldgenRandom::new(legacy_random_source(0)),
            sea_level: 63,
            biomes_required_ocean_monument_surrounding: false,
        };
        assert!(structure.find_generation_point(ctx).is_none());
    }

    #[test]
    fn test_find_generation_point_generates_piece_when_biomes_ok() {
        let structure = OceanMonumentStructure::new(StructureSettings);
        let ctx = GenerationContext {
            chunk_pos: ChunkPos::new(4, -3),
            random: WorldgenRandom::new(legacy_random_source(42)),
            sea_level: 63,
            biomes_required_ocean_monument_surrounding: true,
        };
        let stub = structure.find_generation_point(ctx).expect("stub");
        assert_eq!(stub.pieces.pieces().len(), 1);
        assert_eq!(stub.position.y, 63);
        assert_eq!(stub.position.x, ChunkPos::new(4, -3).get_block_x(9));
        assert_eq!(stub.position.z, ChunkPos::new(4, -3).get_block_z(9));
    }
}
