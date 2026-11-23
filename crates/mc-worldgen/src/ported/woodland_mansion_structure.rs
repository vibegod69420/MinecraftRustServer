#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn set_y(&mut self, y: i32) -> &mut Self {
        self.y = y;
        self
    }

    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

impl Default for MutableBlockPos {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    pub fn new(
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

    pub fn is_inside_mut(&self, pos: &MutableBlockPos) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.y >= self.min_y
            && pos.y <= self.max_y
            && pos.z >= self.min_z
            && pos.z <= self.max_z
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

pub trait WorldgenRandom {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    // JAVA: net.minecraft.world.level.block.Rotation#getRandom
    pub fn get_random(random: &mut dyn WorldgenRandom) -> Self {
        match random.next_int(4).rem_euclid(4) {
            0 => Rotation::None,
            1 => Rotation::Clockwise90,
            2 => Rotation::Clockwise180,
            _ => Rotation::Counterclockwise90,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlockState {
    pub is_air: bool,
    pub is_liquid: bool,
    pub is_cobblestone: bool,
}

impl BlockState {
    pub fn air() -> Self {
        Self {
            is_air: true,
            is_liquid: false,
            is_cobblestone: false,
        }
    }

    pub fn cobblestone() -> Self {
        Self {
            is_air: false,
            is_liquid: false,
            is_cobblestone: true,
        }
    }

    pub fn liquid(&self) -> bool {
        self.is_liquid
    }
}

pub trait WorldGenLevel {
    fn get_min_y(&self) -> i32;
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32);
}

pub trait PiecesContainer {
    fn calculate_bounding_box(&self) -> BoundingBox;
    fn is_inside_piece(&self, pos: BlockPos) -> bool;
}

pub trait HeightSampler {
    fn get_lowest_y_in_5by5_box_offset_7_blocks(&self, rotation: Rotation) -> BlockPos;
}

#[derive(Clone, Debug, Default)]
pub struct StructureSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureType {
    WoodlandMansion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerationStub {
    pub position: BlockPos,
    pub rotation: Rotation,
}

pub struct GenerationContext<'a> {
    pub random: &'a mut dyn WorldgenRandom,
    pub height_sampler: &'a dyn HeightSampler,
}

/// Stub for unused afterPlace parameter.
pub struct StructureManager;
/// Stub for unused afterPlace parameter.
pub struct ChunkGenerator;
/// Stub for unused afterPlace parameter.
pub struct RandomSource;

#[derive(Clone, Debug)]
pub struct WoodlandMansionStructure {
    pub settings: StructureSettings,
}

impl WoodlandMansionStructure {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionStructure#<init>
    pub fn new(settings: StructureSettings) -> Self {
        Self { settings }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionStructure#findGenerationPoint
    pub fn find_generation_point(&self, ctx: &mut GenerationContext<'_>) -> Option<GenerationStub> {
        let rotation = Rotation::get_random(ctx.random);
        let pos = ctx
            .height_sampler
            .get_lowest_y_in_5by5_box_offset_7_blocks(rotation);
        if pos.get_y() < 60 {
            return None;
        }
        Some(GenerationStub {
            position: pos,
            rotation,
        })
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionStructure#generatePieces
    pub fn generate_pieces(&self, _pos: BlockPos, _rotation: Rotation) -> usize {
        // Actual piece generation delegates to WoodlandMansionPieces.generateMansion.
        // Self-contained stub: returns 0 pieces.
        0
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionStructure#afterPlace
    pub fn after_place(
        &self,
        level: &mut dyn WorldGenLevel,
        _structure_manager: &StructureManager,
        _chunk_generator: &ChunkGenerator,
        _random: &RandomSource,
        box_: BoundingBox,
        _chunk_pos: ChunkPos,
        pieces: &dyn PiecesContainer,
    ) {
        let mut mutable_pos = MutableBlockPos::new();
        let min_level_y = level.get_min_y();
        let pieces_box = pieces.calculate_bounding_box();
        let pieces_min_y = pieces_box.min_y;

        let mut x = box_.min_x;
        while x <= box_.max_x {
            let mut z = box_.min_z;
            while z <= box_.max_z {
                mutable_pos.set(x, pieces_min_y, z);
                let pos_at_top = mutable_pos.to_block_pos();
                if !level.is_empty_block(pos_at_top)
                    && pieces_box.is_inside_mut(&mutable_pos)
                    && pieces.is_inside_piece(pos_at_top)
                {
                    let mut y = pieces_min_y.wrapping_sub(1);
                    while y > min_level_y {
                        mutable_pos.set_y(y);
                        let pos_below = mutable_pos.to_block_pos();
                        if !level.is_empty_block(pos_below)
                            && !level.get_block_state(pos_below).liquid()
                        {
                            break;
                        }
                        level.set_block(pos_below, BlockState::cobblestone(), 2);
                        y = y.wrapping_sub(1);
                    }
                }
                z = z.wrapping_add(1);
            }
            x = x.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionStructure#type
    pub fn structure_type(&self) -> StructureType {
        StructureType::WoodlandMansion
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct FixedRandom(i32);
    impl WorldgenRandom for FixedRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                0
            } else {
                self.0.rem_euclid(bound)
            }
        }
    }

    struct FixedHeight(BlockPos);
    impl HeightSampler for FixedHeight {
        fn get_lowest_y_in_5by5_box_offset_7_blocks(&self, _rotation: Rotation) -> BlockPos {
            self.0
        }
    }

    struct TestLevel {
        min_y: i32,
        blocks: BTreeMap<(i32, i32, i32), BlockState>,
    }

    impl TestLevel {
        fn new(min_y: i32) -> Self {
            Self {
                min_y,
                blocks: BTreeMap::new(),
            }
        }
        fn set(&mut self, x: i32, y: i32, z: i32, state: BlockState) {
            self.blocks.insert((x, y, z), state);
        }
        fn get(&self, x: i32, y: i32, z: i32) -> BlockState {
            self.blocks
                .get(&(x, y, z))
                .copied()
                .unwrap_or(BlockState::air())
        }
    }

    impl WorldGenLevel for TestLevel {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }
        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.get(pos.x, pos.y, pos.z).is_air
        }
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.get(pos.x, pos.y, pos.z)
        }
        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) {
            self.set(pos.x, pos.y, pos.z, state);
        }
    }

    struct TestPieces {
        bbox: BoundingBox,
        inside: Vec<(i32, i32, i32)>,
    }

    impl TestPieces {
        fn new(bbox: BoundingBox, inside: Vec<(i32, i32, i32)>) -> Self {
            Self { bbox, inside }
        }
    }

    impl PiecesContainer for TestPieces {
        fn calculate_bounding_box(&self) -> BoundingBox {
            self.bbox
        }
        fn is_inside_piece(&self, pos: BlockPos) -> bool {
            self.inside.contains(&(pos.x, pos.y, pos.z))
        }
    }

    #[test]
    fn find_generation_point_rejects_below_60() {
        let mut rng = FixedRandom(0);
        let hs = FixedHeight(BlockPos::new(10, 59, 20));
        let mut ctx = GenerationContext {
            random: &mut rng,
            height_sampler: &hs,
        };
        let s = WoodlandMansionStructure::new(StructureSettings);
        assert!(s.find_generation_point(&mut ctx).is_none());
    }

    #[test]
    fn find_generation_point_accepts_60_or_above() {
        let mut rng = FixedRandom(2);
        let hs = FixedHeight(BlockPos::new(-5, 60, 7));
        let mut ctx = GenerationContext {
            random: &mut rng,
            height_sampler: &hs,
        };
        let s = WoodlandMansionStructure::new(StructureSettings);
        let stub = s.find_generation_point(&mut ctx).unwrap();
        assert_eq!(stub.position, BlockPos::new(-5, 60, 7));
        assert_eq!(stub.rotation, Rotation::Clockwise180);
    }

    #[test]
    fn after_place_fills_cobblestone_below_piece() {
        let mut level = TestLevel::new(0);
        let pieces_box = BoundingBox::new(0, 5, 0, 2, 10, 2);
        level.set(
            1,
            5,
            1,
            BlockState {
                is_air: false,
                is_liquid: false,
                is_cobblestone: false,
            },
        );
        let pieces = TestPieces::new(pieces_box, vec![(1, 5, 1)]);
        let chunk_box = BoundingBox::new(0, 0, 0, 15, 15, 15);
        let s = WoodlandMansionStructure::new(StructureSettings);
        s.after_place(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &RandomSource,
            chunk_box,
            ChunkPos::new(0, 0),
            &pieces,
        );

        for y in 1..=4 {
            let st = level.get(1, y, 1);
            assert!(st.is_cobblestone, "expected cobblestone at y={y}");
        }
        assert!(level.get(1, 0, 1).is_air);
    }

    #[test]
    fn after_place_stops_at_solid_non_liquid() {
        let mut level = TestLevel::new(0);
        let pieces_box = BoundingBox::new(0, 5, 0, 0, 10, 0);
        level.set(
            0,
            5,
            0,
            BlockState {
                is_air: false,
                is_liquid: false,
                is_cobblestone: false,
            },
        );
        level.set(
            0,
            2,
            0,
            BlockState {
                is_air: false,
                is_liquid: false,
                is_cobblestone: false,
            },
        );
        let pieces = TestPieces::new(pieces_box, vec![(0, 5, 0)]);
        let chunk_box = BoundingBox::new(0, 0, 0, 15, 15, 15);
        let s = WoodlandMansionStructure::new(StructureSettings);
        s.after_place(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &RandomSource,
            chunk_box,
            ChunkPos::new(0, 0),
            &pieces,
        );

        assert!(level.get(0, 4, 0).is_cobblestone);
        assert!(level.get(0, 3, 0).is_cobblestone);
        assert!(!level.get(0, 2, 0).is_cobblestone);
        assert!(level.get(0, 1, 0).is_air);
    }

    #[test]
    fn after_place_fills_through_liquid() {
        let mut level = TestLevel::new(-64);
        let pieces_box = BoundingBox::new(0, 5, 0, 0, 10, 0);
        level.set(
            0,
            5,
            0,
            BlockState {
                is_air: false,
                is_liquid: false,
                is_cobblestone: false,
            },
        );
        level.set(
            0,
            3,
            0,
            BlockState {
                is_air: false,
                is_liquid: true,
                is_cobblestone: false,
            },
        );
        let pieces = TestPieces::new(pieces_box, vec![(0, 5, 0)]);
        let chunk_box = BoundingBox::new(0, 0, 0, 15, 15, 15);
        let s = WoodlandMansionStructure::new(StructureSettings);
        s.after_place(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &RandomSource,
            chunk_box,
            ChunkPos::new(0, 0),
            &pieces,
        );

        assert!(level.get(0, 4, 0).is_cobblestone);
        assert!(level.get(0, 3, 0).is_cobblestone);
        assert!(level.get(0, 2, 0).is_cobblestone);
    }

    #[test]
    fn structure_type_is_woodland_mansion() {
        let s = WoodlandMansionStructure::new(StructureSettings);
        assert_eq!(s.structure_type(), StructureType::WoodlandMansion);
    }
}
