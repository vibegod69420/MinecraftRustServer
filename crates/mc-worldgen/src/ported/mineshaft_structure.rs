use std::fmt;

/// Minimal block state representation for self-contained porting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub block_name: &'static str,
}

impl BlockState {
    pub const fn new(block_name: &'static str) -> Self {
        Self { block_name }
    }
}

/// Minimal block representation for self-contained porting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub name: &'static str,
}

impl Block {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub const fn default_block_state(self) -> BlockState {
        BlockState::new(self.name)
    }
}

pub mod blocks {
    use super::Block;

    pub const OAK_LOG: Block = Block::new("oak_log");
    pub const OAK_PLANKS: Block = Block::new("oak_planks");
    pub const OAK_FENCE: Block = Block::new("oak_fence");
    pub const DARK_OAK_LOG: Block = Block::new("dark_oak_log");
    pub const DARK_OAK_PLANKS: Block = Block::new("dark_oak_planks");
    pub const DARK_OAK_FENCE: Block = Block::new("dark_oak_fence");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(
            self.x.wrapping_add(dx),
            self.y.wrapping_add(dy),
            self.z.wrapping_add(dz),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn get_block_x(&self, offset: i32) -> i32 {
        self.x.wrapping_shl(4).wrapping_add(offset)
    }

    pub fn get_block_z(&self, offset: i32) -> i32 {
        self.z.wrapping_shl(4).wrapping_add(offset)
    }

    pub fn get_middle_block_x(&self) -> i32 {
        self.get_block_x(8)
    }

    pub fn get_min_block_z(&self) -> i32 {
        self.get_block_z(0)
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
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    pub fn get_center(&self) -> BlockPos {
        BlockPos::new(
            self.min_x
                .wrapping_add(self.max_x.wrapping_sub(self.min_x).wrapping_add(1) / 2),
            self.min_y
                .wrapping_add(self.max_y.wrapping_sub(self.min_y).wrapping_add(1) / 2),
            self.min_z
                .wrapping_add(self.max_z.wrapping_sub(self.min_z).wrapping_add(1) / 2),
        )
    }

    pub fn get_y_span(&self) -> i32 {
        self.max_y.wrapping_sub(self.min_y).wrapping_add(1)
    }

    pub fn encompassing(boxes: impl IntoIterator<Item = BoundingBox>) -> Option<BoundingBox> {
        let mut iter = boxes.into_iter();
        let first = iter.next()?;
        let mut min_x = first.min_x;
        let mut min_y = first.min_y;
        let mut min_z = first.min_z;
        let mut max_x = first.max_x;
        let mut max_y = first.max_y;
        let mut max_z = first.max_z;
        for b in iter {
            min_x = min_x.min(b.min_x);
            min_y = min_y.min(b.min_y);
            min_z = min_z.min(b.min_z);
            max_x = max_x.max(b.max_x);
            max_y = max_y.max(b.max_y);
            max_z = max_z.max(b.max_z);
        }
        Some(BoundingBox::new(min_x, min_y, min_z, max_x, max_y, max_z))
    }
}

pub trait WorldgenRandom {
    fn next_double(&mut self) -> f64;
    fn next_int(&mut self, bound: i32) -> i32;
}

// JAVA: net.minecraft.util.Mth#randomBetweenInclusive
pub fn random_between_inclusive(random: &mut dyn WorldgenRandom, min: i32, max: i32) -> i32 {
    random
        .next_int(max.wrapping_sub(min).wrapping_add(1))
        .wrapping_add(min)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    WorldSurfaceWg,
}

pub trait HeightAccessor {}
pub trait RandomState {}

pub trait ChunkGenerator {
    fn get_sea_level(&self) -> i32;
    fn get_min_y(&self) -> i32;
    fn get_base_height(
        &self,
        x: i32,
        z: i32,
        heightmap_type: HeightmapType,
        height_accessor: &dyn HeightAccessor,
        random_state: &dyn RandomState,
    ) -> i32;
}

pub trait StructurePiece: fmt::Debug {
    fn bounding_box(&self) -> BoundingBox;
    fn offset_vertically(&mut self, dy: i32);
}

#[derive(Debug, Default)]
pub struct StructurePiecesBuilder {
    pieces: Vec<Box<dyn StructurePiece>>,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self { pieces: Vec::new() }
    }

    pub fn add_piece(&mut self, piece: Box<dyn StructurePiece>) {
        self.pieces.push(piece);
    }

    pub fn insert_piece_front(&mut self, piece: Box<dyn StructurePiece>) {
        self.pieces.insert(0, piece);
    }

    pub fn pieces(&self) -> &[Box<dyn StructurePiece>] {
        &self.pieces
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        BoundingBox::encompassing(self.pieces.iter().map(|p| p.bounding_box()))
            .unwrap_or_else(|| BoundingBox::new(0, 0, 0, 0, 0, 0))
    }

    pub fn offset_pieces_vertically(&mut self, dy: i32) {
        for piece in &mut self.pieces {
            piece.offset_vertically(dy);
        }
    }

    pub fn move_below_sea_level(
        &mut self,
        sea_level: i32,
        _min_y: i32,
        random: &mut dyn WorldgenRandom,
        offset: i32,
    ) -> i32 {
        let i = sea_level.wrapping_sub(offset);
        let mut j = self.get_bounding_box().get_y_span().wrapping_add(1);
        if j < i {
            j = j.wrapping_add(random.next_int(i.wrapping_sub(j)));
        }
        let k = j.wrapping_sub(self.get_bounding_box().max_y);
        self.offset_pieces_vertically(k);
        return k;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Mineshaft,
}

pub struct GenerationContext<'a> {
    pub chunk_pos: ChunkPos,
    pub random: &'a mut dyn WorldgenRandom,
    pub chunk_generator: &'a dyn ChunkGenerator,
    pub height_accessor: &'a dyn HeightAccessor,
    pub random_state: &'a dyn RandomState,
}

impl<'a> GenerationContext<'a> {
    pub fn new(
        chunk_pos: ChunkPos,
        random: &'a mut dyn WorldgenRandom,
        chunk_generator: &'a dyn ChunkGenerator,
        height_accessor: &'a dyn HeightAccessor,
        random_state: &'a dyn RandomState,
    ) -> Self {
        Self {
            chunk_pos,
            random,
            chunk_generator,
            height_accessor,
            random_state,
        }
    }

    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn random(&mut self) -> &mut dyn WorldgenRandom {
        self.random
    }

    pub fn chunk_generator(&self) -> &dyn ChunkGenerator {
        self.chunk_generator
    }

    pub fn height_accessor(&self) -> &dyn HeightAccessor {
        self.height_accessor
    }

    pub fn random_state(&self) -> &dyn RandomState {
        self.random_state
    }
}

pub enum GenerationStubPieces {
    Pieces(StructurePiecesBuilder),
}

pub struct GenerationStub {
    pub position: BlockPos,
    pub pieces: GenerationStubPieces,
}

impl GenerationStub {
    pub fn new(position: BlockPos, pieces: GenerationStubPieces) -> Self {
        Self { position, pieces }
    }
}

pub trait MineshaftRoomPiece: StructurePiece {
    fn add_children(&mut self, builder: &mut StructurePiecesBuilder, random: &mut dyn WorldgenRandom);
}

/// Minimal stub for `MineshaftPieces.MineShaftRoom` to keep MineshaftStructure self-contained.
/// Real implementations can provide a factory that returns a faithful room piece.
#[derive(Debug, Clone)]
pub struct MineShaftRoom {
    bbox: BoundingBox,
    mineshaft_type: MineshaftType,
}

impl MineShaftRoom {
    pub fn new(
        _gen_depth: i32,
        _random: &mut dyn WorldgenRandom,
        x: i32,
        z: i32,
        mineshaft_type: MineshaftType,
    ) -> Self {
        Self {
            bbox: BoundingBox::new(x, 50, z, x.wrapping_add(7), 54, z.wrapping_add(7)),
            mineshaft_type,
        }
    }

    pub fn mineshaft_type(&self) -> MineshaftType {
        self.mineshaft_type
    }
}

impl StructurePiece for MineShaftRoom {
    fn bounding_box(&self) -> BoundingBox {
        self.bbox
    }

    fn offset_vertically(&mut self, dy: i32) {
        self.bbox.min_y = self.bbox.min_y.wrapping_add(dy);
        self.bbox.max_y = self.bbox.max_y.wrapping_add(dy);
    }
}

impl MineshaftRoomPiece for MineShaftRoom {
    fn add_children(&mut self, _builder: &mut StructurePiecesBuilder, _random: &mut dyn WorldgenRandom) {
        // Stub: real MineshaftPieces.MineShaftRoom adds corridor pieces.
    }
}

pub trait MineshaftRoomFactory {
    fn create_room(
        &self,
        gen_depth: i32,
        random: &mut dyn WorldgenRandom,
        x: i32,
        z: i32,
        mineshaft_type: MineshaftType,
    ) -> Box<dyn MineshaftRoomPiece>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultMineshaftRoomFactory;

impl MineshaftRoomFactory for DefaultMineshaftRoomFactory {
    fn create_room(
        &self,
        gen_depth: i32,
        random: &mut dyn WorldgenRandom,
        x: i32,
        z: i32,
        mineshaft_type: MineshaftType,
    ) -> Box<dyn MineshaftRoomPiece> {
        Box::new(MineShaftRoom::new(gen_depth, random, x, z, mineshaft_type))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineshaftType {
    Normal,
    Mesa,
}

impl MineshaftType {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure.Type#getName
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Mesa => "mesa",
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure.Type#byId
    pub fn by_id(id: i32) -> Self {
        match id {
            1 => Self::Mesa,
            _ => Self::Normal,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure.Type#getWoodState
    pub fn get_wood_state(&self) -> BlockState {
        match self {
            Self::Normal => blocks::OAK_LOG.default_block_state(),
            Self::Mesa => blocks::DARK_OAK_LOG.default_block_state(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure.Type#getPlanksState
    pub fn get_planks_state(&self) -> BlockState {
        match self {
            Self::Normal => blocks::OAK_PLANKS.default_block_state(),
            Self::Mesa => blocks::DARK_OAK_PLANKS.default_block_state(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure.Type#getFenceState
    pub fn get_fence_state(&self) -> BlockState {
        match self {
            Self::Normal => blocks::OAK_FENCE.default_block_state(),
            Self::Mesa => blocks::DARK_OAK_FENCE.default_block_state(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure.Type#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }
}

#[derive(Debug, Clone)]
pub struct MineshaftStructure {
    pub settings: StructureSettings,
    pub type_: MineshaftType,
}

impl MineshaftStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure#<init>
    pub fn new(settings: StructureSettings, type_: MineshaftType) -> Self {
        Self { settings, type_ }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure#findGenerationPoint
    pub fn find_generation_point(&self, ctx: &mut GenerationContext<'_>) -> Option<GenerationStub> {
        self.find_generation_point_with_factory(ctx, &DefaultMineshaftRoomFactory)
    }

    pub fn find_generation_point_with_factory(
        &self,
        ctx: &mut GenerationContext<'_>,
        room_factory: &dyn MineshaftRoomFactory,
    ) -> Option<GenerationStub> {
        ctx.random().next_double();
        let chunk_pos = ctx.chunk_pos();
        let block_pos = BlockPos::new(chunk_pos.get_middle_block_x(), 50, chunk_pos.get_min_block_z());
        let mut pieces_builder = StructurePiecesBuilder::new();
        let y_offset = self.generate_pieces_and_adjust_with_factory(&mut pieces_builder, ctx, room_factory);
        Some(GenerationStub::new(
            block_pos.offset(0, y_offset, 0),
            GenerationStubPieces::Pieces(pieces_builder),
        ))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure#generatePiecesAndAdjust
    fn generate_pieces_and_adjust(
        &self,
        pieces: &mut StructurePiecesBuilder,
        ctx: &mut GenerationContext<'_>,
    ) -> i32 {
        self.generate_pieces_and_adjust_with_factory(pieces, ctx, &DefaultMineshaftRoomFactory)
    }

    fn generate_pieces_and_adjust_with_factory(
        &self,
        pieces: &mut StructurePiecesBuilder,
        ctx: &mut GenerationContext<'_>,
        room_factory: &dyn MineshaftRoomFactory,
    ) -> i32 {
        let chunk_pos = ctx.chunk_pos();
        let x = chunk_pos.get_block_x(2);
        let z = chunk_pos.get_block_z(2);
        let mut room = room_factory.create_room(0, ctx.random(), x, z, self.type_);
        // Java order: addPiece(room); room.addChildren(room, builder, random);
        // We preserve final order [room, children...] by adding children first, then inserting room at front.
        room.add_children(pieces, ctx.random());
        pieces.insert_piece_front(room);

        let sea_level = ctx.chunk_generator().get_sea_level();
        if self.type_ == MineshaftType::Mesa {
            let center = pieces.get_bounding_box().get_center();
            let base_height = ctx.chunk_generator().get_base_height(
                center.get_x(),
                center.get_z(),
                HeightmapType::WorldSurfaceWg,
                ctx.height_accessor(),
                ctx.random_state(),
            );
            let target_y = if base_height <= sea_level {
                sea_level
            } else {
                random_between_inclusive(ctx.random(), sea_level, base_height)
            };
            let y_offset = target_y.wrapping_sub(center.get_y());
            pieces.offset_pieces_vertically(y_offset);
            return y_offset;
        } else {
            return pieces.move_below_sea_level(sea_level, ctx.chunk_generator().get_min_y(), ctx.random(), 10);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::Mineshaft
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct MockRandom {
        doubles: Vec<f64>,
        ints: Vec<i32>,
        double_idx: usize,
        int_idx: usize,
    }

    impl MockRandom {
        fn with_doubles(doubles: Vec<f64>) -> Self {
            Self { doubles, ..Default::default() }
        }

        fn with_ints(ints: Vec<i32>) -> Self {
            Self { ints, ..Default::default() }
        }
    }

    impl WorldgenRandom for MockRandom {
        fn next_double(&mut self) -> f64 {
            let v = self.doubles.get(self.double_idx).copied().unwrap_or(0.0);
            self.double_idx += 1;
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.int_idx).copied().unwrap_or(0);
            self.int_idx += 1;
            if bound <= 0 { 0 } else { v.rem_euclid(bound) }
        }
    }

    struct MockHeightAccessor;
    impl HeightAccessor for MockHeightAccessor {}
    struct MockRandomState;
    impl RandomState for MockRandomState {}

    struct MockChunkGenerator {
        sea_level: i32,
        min_y: i32,
        base_height: i32,
    }

    impl ChunkGenerator for MockChunkGenerator {
        fn get_sea_level(&self) -> i32 { self.sea_level }
        fn get_min_y(&self) -> i32 { self.min_y }
        fn get_base_height(
            &self,
            _x: i32,
            _z: i32,
            _heightmap_type: HeightmapType,
            _height_accessor: &dyn HeightAccessor,
            _random_state: &dyn RandomState,
        ) -> i32 {
            self.base_height
        }
    }

    #[derive(Debug, Clone)]
    struct FixedRoom {
        bbox: BoundingBox,
    }

    impl StructurePiece for FixedRoom {
        fn bounding_box(&self) -> BoundingBox { self.bbox }
        fn offset_vertically(&mut self, dy: i32) {
            self.bbox.min_y = self.bbox.min_y.wrapping_add(dy);
            self.bbox.max_y = self.bbox.max_y.wrapping_add(dy);
        }
    }

    impl MineshaftRoomPiece for FixedRoom {
        fn add_children(&mut self, _builder: &mut StructurePiecesBuilder, _random: &mut dyn WorldgenRandom) {}
    }

    struct FixedRoomFactory {
        bbox: BoundingBox,
    }

    impl MineshaftRoomFactory for FixedRoomFactory {
        fn create_room(
            &self,
            _gen_depth: i32,
            _random: &mut dyn WorldgenRandom,
            _x: i32,
            _z: i32,
            _mineshaft_type: MineshaftType,
        ) -> Box<dyn MineshaftRoomPiece> {
            Box::new(FixedRoom { bbox: self.bbox })
        }
    }

    #[test]
    fn mineshaft_type_properties_and_by_id() {
        assert_eq!(MineshaftType::Normal.get_name(), "normal");
        assert_eq!(MineshaftType::Mesa.get_name(), "mesa");
        assert_eq!(MineshaftType::Normal.get_serialized_name(), "normal");
        assert_eq!(MineshaftType::Mesa.get_serialized_name(), "mesa");
        assert_eq!(MineshaftType::by_id(0), MineshaftType::Normal);
        assert_eq!(MineshaftType::by_id(1), MineshaftType::Mesa);
        assert_eq!(MineshaftType::by_id(2), MineshaftType::Normal); // out of bounds -> ZERO
        assert_eq!(MineshaftType::by_id(-1), MineshaftType::Normal);

        assert_eq!(MineshaftType::Normal.get_wood_state(), BlockState::new("oak_log"));
        assert_eq!(MineshaftType::Mesa.get_wood_state(), BlockState::new("dark_oak_log"));
        assert_eq!(MineshaftType::Normal.get_planks_state(), BlockState::new("oak_planks"));
        assert_eq!(MineshaftType::Mesa.get_fence_state(), BlockState::new("dark_oak_fence"));
    }

    #[test]
    fn find_generation_point_consumes_next_double_and_offsets() {
        let structure = MineshaftStructure::new(StructureSettings, MineshaftType::Normal);
        let mut random = MockRandom::with_doubles(vec![0.123456789]);
        let generator = MockChunkGenerator { sea_level: 63, min_y: -64, base_height: 80 };
        let ha = MockHeightAccessor;
        let rs = MockRandomState;
        let mut ctx = GenerationContext::new(ChunkPos::new(1, -2), &mut random, &generator, &ha, &rs);
        let factory = FixedRoomFactory {
            bbox: BoundingBox::new(0, 50, 0, 10, 60, 10),
        };
        let stub = structure.find_generation_point_with_factory(&mut ctx, &factory).unwrap();
        assert_eq!(random.double_idx, 1);
        // chunk (1,-2): middleBlockX = 1*16+8=24, minBlockZ = -2*16+0=-32
        // Normal type uses move_below_sea_level; y is adjusted from base 50 by returned offset.
        assert_eq!(stub.position.get_x(), 24);
        assert_eq!(stub.position.get_z(), -32);
        let GenerationStubPieces::Pieces(pieces) = stub.pieces;
        assert_eq!(pieces.pieces().len(), 1);
    }

    #[test]
    fn mesa_type_uses_base_height_and_random_between_inclusive() {
        let structure = MineshaftStructure::new(StructureSettings, MineshaftType::Mesa);
        // next_double consumed by findGenerationPoint, then randomBetweenInclusive needs nextInt(bound)
        let mut random = MockRandom { doubles: vec![0.5], ints: vec![5], double_idx: 0, int_idx: 0 };
        let generator = MockChunkGenerator { sea_level: 63, min_y: -64, base_height: 100 };
        let ha = MockHeightAccessor;
        let rs = MockRandomState;
        let mut ctx = GenerationContext::new(ChunkPos::new(0, 0), &mut random, &generator, &ha, &rs);
        // center of bbox (0,50,0,10,60,10) is (5,55,5)
        let factory = FixedRoomFactory {
            bbox: BoundingBox::new(0, 50, 0, 10, 60, 10),
        };
        let stub = structure.find_generation_point_with_factory(&mut ctx, &factory).unwrap();
        // bound = 100 - 63 + 1 = 38, nextInt returns 5%38=5, targetY = 63+5=68, offset = 68-55 = 13
        assert_eq!(stub.position.get_y(), 50 + 13);
        let GenerationStubPieces::Pieces(pieces) = stub.pieces;
        let bb = pieces.get_bounding_box();
        assert_eq!(bb.min_y, 63);
        assert_eq!(bb.max_y, 73);
    }

    #[test]
    fn mesa_type_uses_sea_level_when_base_height_not_above() {
        let structure = MineshaftStructure::new(StructureSettings, MineshaftType::Mesa);
        let mut random = MockRandom::with_doubles(vec![0.0]);
        let generator = MockChunkGenerator { sea_level: 63, min_y: -64, base_height: 50 };
        let ha = MockHeightAccessor;
        let rs = MockRandomState;
        let mut ctx = GenerationContext::new(ChunkPos::new(0, 0), &mut random, &generator, &ha, &rs);
        let factory = FixedRoomFactory {
            bbox: BoundingBox::new(0, 50, 0, 10, 60, 10),
        };
        let stub = structure.find_generation_point_with_factory(&mut ctx, &factory).unwrap();
        // baseHeight 50 <= seaLevel 63 => targetY = 63, center.y = 55, offset = 8
        assert_eq!(stub.position.get_y(), 58);
    }

    #[test]
    fn wrapping_and_negative_chunk_coordinates() {
        let cp = ChunkPos::new(-1, -1);
        assert_eq!(cp.get_block_x(2), (-1i32).wrapping_shl(4).wrapping_add(2));
        assert_eq!(cp.get_block_z(0), (-1i32).wrapping_shl(4));
        assert_eq!(cp.get_middle_block_x(), (-1i32).wrapping_shl(4).wrapping_add(8));

        let pos = BlockPos::new(i32::MAX, 0, i32::MIN);
        let off = pos.offset(1, -1, -1);
        assert_eq!(off.get_x(), i32::MIN);
        assert_eq!(off.get_y(), -1);
        assert_eq!(off.get_z(), i32::MAX);

        assert_eq!(MineshaftType::by_id(i32::MIN), MineshaftType::Normal);
        assert_eq!(MineshaftType::by_id(i32::MAX), MineshaftType::Normal);
    }

    #[test]
    fn structure_type_and_random_between_inclusive_float_bits_irrelevant() {
        let structure = MineshaftStructure::new(StructureSettings, MineshaftType::Normal);
        assert_eq!(structure.structure_type(), StructureType::Mineshaft);

        // Verify randomBetweenInclusive uses Java int semantics: (max-min+1) bound
        let mut random = MockRandom::with_ints(vec![0]);
        let v = random_between_inclusive(&mut random, -5, -5);
        assert_eq!(v, -5);

        let mut random2 = MockRandom::with_ints(vec![3]);
        let v2 = random_between_inclusive(&mut random2, -2, 2); // bound 5, 3%5=3, -2+3=1
        assert_eq!(v2, 1);

        // next_double is consumed and its float bits don't affect structure position logic
        let mut random3 = MockRandom { doubles: vec![f64::from_bits(0xFFF8_0000_0000_0001u64)], ..Default::default() };
        let _ = random3.next_double();
        assert_eq!(random3.double_idx, 1);
    }
}
