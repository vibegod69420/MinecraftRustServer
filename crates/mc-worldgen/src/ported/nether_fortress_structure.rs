use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft dependencies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>
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

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Blaze,
    ZombifiedPiglin,
    WitherSkeleton,
    Skeleton,
    MagmaCube,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnerData {
    pub entity_type: EntityType,
    pub min_count: i32,
    pub max_count: i32,
}

impl SpawnerData {
    pub fn new(entity_type: EntityType, min_count: i32, max_count: i32) -> Self {
        Self {
            entity_type,
            min_count,
            max_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedEntry<T> {
    pub data: T,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedList<T> {
    pub entries: Vec<WeightedEntry<T>>,
}

impl<T> WeightedList<T> {
    pub fn builder() -> WeightedListBuilder<T> {
        WeightedListBuilder {
            entries: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub struct WeightedListBuilder<T> {
    entries: Vec<WeightedEntry<T>>,
}

impl<T> WeightedListBuilder<T> {
    pub fn add(mut self, data: T, weight: i32) -> Self {
        self.entries.push(WeightedEntry { data, weight });
        self
    }

    pub fn build(self) -> WeightedList<T> {
        WeightedList {
            entries: self.entries,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Fortress,
}

#[derive(Debug, Clone, Default)]
pub struct StructureSettings;

pub struct GenerationContext<'a> {
    pub chunk_pos: ChunkPos,
    pub random: &'a mut dyn RandomSource,
}

impl<'a> GenerationContext<'a> {
    pub fn new(chunk_pos: ChunkPos, random: &'a mut dyn RandomSource) -> Self {
        Self { chunk_pos, random }
    }

    pub fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn random(&mut self) -> &mut dyn RandomSource {
        self.random
    }
}

pub struct StructurePiecesBuilder {
    pieces_count: usize,
    moved_inside_heights: Option<(i32, i32)>,
}

impl StructurePiecesBuilder {
    pub fn new() -> Self {
        Self {
            pieces_count: 0,
            moved_inside_heights: None,
        }
    }

    pub fn add_piece(&mut self, _piece: &StartPiece) {
        self.pieces_count = self.pieces_count.wrapping_add(1);
    }

    pub fn move_inside_heights(&mut self, _random: &mut dyn RandomSource, min: i32, max: i32) {
        self.moved_inside_heights = Some((min, max));
    }

    pub fn pieces_count(&self) -> usize {
        self.pieces_count
    }

    pub fn moved_inside_heights(&self) -> Option<(i32, i32)> {
        self.moved_inside_heights
    }
}

impl Default for StructurePiecesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub trait StructurePiece {
    fn add_children(
        &mut self,
        start: &SharedStartPiece,
        builder: &mut StructurePiecesBuilder,
        random: &mut dyn RandomSource,
    );
}

pub type SharedStartPiece = Rc<RefCell<StartPiece>>;

// Stub for net.minecraft.world.level.levelgen.structure.structures.NetherFortressPieces.StartPiece
pub struct StartPiece {
    pub pending_children: Vec<Box<dyn StructurePiece>>,
    pub block_x: i32,
    pub block_z: i32,
}

impl StartPiece {
    pub fn new(_random: &mut dyn RandomSource, block_x: i32, block_z: i32) -> Self {
        Self {
            pending_children: Vec::new(),
            block_x,
            block_z,
        }
    }

    pub fn add_children(
        &mut self,
        _start: &SharedStartPiece,
        _builder: &mut StructurePiecesBuilder,
        _random: &mut dyn RandomSource,
    ) {
        // Stub: actual piece graph generation lives in NetherFortressPieces.
    }
}

pub struct GenerationStub {
    pub position: BlockPos,
    pub generator: Box<dyn FnOnce(&mut StructurePiecesBuilder, &mut dyn RandomSource)>,
}

// ---------------------------------------------------------------------------
// Ported class: NetherFortressStructure
// ---------------------------------------------------------------------------

pub struct NetherFortressStructure {
    pub settings: StructureSettings,
}

impl NetherFortressStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure#FORTRESS_ENEMIES
    pub fn fortress_enemies() -> &'static WeightedList<SpawnerData> {
        static ENEMIES: OnceLock<WeightedList<SpawnerData>> = OnceLock::new();
        ENEMIES.get_or_init(|| {
            WeightedList::builder()
                .add(SpawnerData::new(EntityType::Blaze, 2, 3), 10)
                .add(SpawnerData::new(EntityType::ZombifiedPiglin, 4, 4), 5)
                .add(SpawnerData::new(EntityType::WitherSkeleton, 5, 5), 8)
                .add(SpawnerData::new(EntityType::Skeleton, 5, 5), 2)
                .add(SpawnerData::new(EntityType::MagmaCube, 4, 4), 3)
                .build()
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        Self { settings }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure#findGenerationPoint
    pub fn find_generation_point(&self, ctx: &GenerationContext<'_>) -> Option<GenerationStub> {
        let chunk_pos = ctx.chunk_pos();
        let pos = BlockPos::new(chunk_pos.get_min_block_x(), 64, chunk_pos.get_min_block_z());
        let cp = chunk_pos;
        Some(GenerationStub {
            position: pos,
            generator: Box::new(move |builder, random| {
                Self::generate_pieces(builder, random, cp);
            }),
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure#generatePieces
    fn generate_pieces(
        builder: &mut StructurePiecesBuilder,
        random: &mut dyn RandomSource,
        chunk_pos: ChunkPos,
    ) {
        let start = Rc::new(RefCell::new(StartPiece::new(
            random,
            chunk_pos.get_block_x(2),
            chunk_pos.get_block_z(2),
        )));
        builder.add_piece(&start.borrow());
        start
            .borrow_mut()
            .add_children(&start, builder, random);

        while !start.borrow().pending_children.is_empty() {
            let len = start.borrow().pending_children.len() as i32;
            let idx = random.next_int(len) as usize;
            let mut piece = start.borrow_mut().pending_children.remove(idx);
            piece.add_children(&start, builder, random);
        }

        builder.move_inside_heights(random, 48, 70);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::Fortress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        values: Vec<i32>,
        index: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.index < self.values.len() {
                let val = self.values[self.index];
                self.index += 1;
                val
            } else {
                0
            };
            v.rem_euclid(bound)
        }
    }

    struct TestPiece {
        pushes: usize,
    }

    impl StructurePiece for TestPiece {
        fn add_children(
            &mut self,
            start: &SharedStartPiece,
            _builder: &mut StructurePiecesBuilder,
            _random: &mut dyn RandomSource,
        ) {
            for _ in 0..self.pushes {
                start
                    .borrow_mut()
                    .pending_children
                    .push(Box::new(TestPiece { pushes: 0 }));
            }
            self.pushes = 0;
        }
    }

    #[test]
    fn fortress_enemies_match_java_weights_and_counts() {
        let enemies = NetherFortressStructure::fortress_enemies();
        assert_eq!(enemies.len(), 5);
        assert_eq!(enemies.entries[0].weight, 10);
        assert_eq!(enemies.entries[0].data.entity_type, EntityType::Blaze);
        assert_eq!(enemies.entries[0].data.min_count, 2);
        assert_eq!(enemies.entries[0].data.max_count, 3);
        assert_eq!(enemies.entries[4].data.entity_type, EntityType::MagmaCube);
        assert_eq!(enemies.entries[4].weight, 3);
    }

    #[test]
    fn find_generation_point_uses_chunk_min_and_y64() {
        let mut rng = SeqRandom::new(vec![]);
        let ctx = GenerationContext::new(ChunkPos::new(-2, 3), &mut rng);
        let structure = NetherFortressStructure::new(StructureSettings);
        let stub = structure.find_generation_point(&ctx).expect("stub");
        assert_eq!(stub.position, BlockPos::new(-32, 64, 48));
    }

    #[test]
    fn generate_pieces_adds_start_moves_heights_and_drains_pending() {
        let mut rng = SeqRandom::new(vec![0, 0, 0]);
        let chunk_pos = ChunkPos::new(1, -1);
        let mut builder = StructurePiecesBuilder::new();

        // Manually mirror generate_pieces but seed pending children through a custom start.
        let start = Rc::new(RefCell::new(StartPiece::new(
            &mut rng,
            chunk_pos.get_block_x(2),
            chunk_pos.get_block_z(2),
        )));
        start
            .borrow_mut()
            .pending_children
            .push(Box::new(TestPiece { pushes: 1 }));
        start
            .borrow_mut()
            .pending_children
            .push(Box::new(TestPiece { pushes: 0 }));

        builder.add_piece(&start.borrow());
        start.borrow_mut().add_children(&start, &mut builder, &mut rng);
        while !start.borrow().pending_children.is_empty() {
            let len = start.borrow().pending_children.len() as i32;
            let idx = rng.next_int(len) as usize;
            let mut piece = start.borrow_mut().pending_children.remove(idx);
            piece.add_children(&start, &mut builder, &mut rng);
        }
        builder.move_inside_heights(&mut rng, 48, 70);

        assert_eq!(builder.pieces_count(), 1);
        assert_eq!(builder.moved_inside_heights(), Some((48, 70)));
        assert!(start.borrow().pending_children.is_empty());
    }

    #[test]
    fn generate_pieces_via_stub_uses_chunk_block_offset_2() {
        let structure = NetherFortressStructure::new(StructureSettings);
        let mut rng = SeqRandom::new(vec![]);
        let ctx = GenerationContext::new(ChunkPos::new(4, 5), &mut rng);
        let stub = structure.find_generation_point(&ctx).expect("stub");
        let mut builder = StructurePiecesBuilder::new();
        let mut gen_rng = SeqRandom::new(vec![]);
        (stub.generator)(&mut builder, &mut gen_rng);
        assert_eq!(builder.pieces_count(), 1);
        assert_eq!(builder.moved_inside_heights(), Some((48, 70)));
    }

    #[test]
    fn chunk_pos_wrapping_and_negative_coordinates() {
        let cp = ChunkPos::new(-1, i32::MIN >> 4);
        assert_eq!(cp.get_min_block_x(), -16);
        assert_eq!(cp.get_block_x(2), -14);
        // wrapping shift/mul behavior for extreme values
        let extreme = ChunkPos::new(i32::MAX / 16, -3);
        assert_eq!(extreme.get_min_block_z(), -48);
    }

    #[test]
    fn structure_type_is_fortress() {
        let structure = NetherFortressStructure::new(StructureSettings);
        assert_eq!(structure.structure_type(), StructureType::Fortress);
    }
}
