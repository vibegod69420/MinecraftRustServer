use std::fmt;

// ---------------------------------------------------------------------------
// Minimal self-contained types (stubs for unported Minecraft classes)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    pub fn relative(self, dir: Direction) -> Self {
        let n = dir.normal();
        Self {
            x: self.x.wrapping_add(n.x),
            y: self.y.wrapping_add(n.y),
            z: self.z.wrapping_add(n.z),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn below(self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    pub fn relative(self, dir: Direction) -> BlockPos {
        let n = dir.normal();
        BlockPos {
            x: self.x.wrapping_add(n.x),
            y: self.y.wrapping_add(n.y),
            z: self.z.wrapping_add(n.z),
        }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, dz: i32) {
        self.x = self.x.wrapping_add(dx);
        self.y = self.y.wrapping_add(dy);
        self.z = self.z.wrapping_add(dz);
    }

    pub fn as_pos(self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn values() -> [Direction; 6] {
        [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ]
    }

    pub fn normal(self) -> BlockPos {
        match self {
            Direction::Down => BlockPos::new(0, -1, 0),
            Direction::Up => BlockPos::new(0, 1, 0),
            Direction::North => BlockPos::new(0, 0, -1),
            Direction::South => BlockPos::new(0, 0, 1),
            Direction::West => BlockPos::new(-1, 0, 0),
            Direction::East => BlockPos::new(1, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Air,
    Sandstone,
    Stone,
    Andesite,
    Granite,
    Diorite,
    Sand,
    Water,
    Lava,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub kind: BlockKind,
}

impl BlockState {
    pub fn is_air(self) -> bool {
        self.kind == BlockKind::Air
    }
}

pub struct Blocks;

impl Blocks {
    pub fn sandstone_default() -> BlockState {
        BlockState {
            kind: BlockKind::Sandstone,
        }
    }

    pub fn stone_default() -> BlockState {
        BlockState {
            kind: BlockKind::Stone,
        }
    }

    pub fn andesite_default() -> BlockState {
        BlockState {
            kind: BlockKind::Andesite,
        }
    }

    pub fn granite_default() -> BlockState {
        BlockState {
            kind: BlockKind::Granite,
        }
    }

    pub fn diorite_default() -> BlockState {
        BlockState {
            kind: BlockKind::Diorite,
        }
    }

    pub fn sand_default() -> BlockState {
        BlockState {
            kind: BlockKind::Sand,
        }
    }

    pub fn water_default() -> BlockState {
        BlockState {
            kind: BlockKind::Water,
        }
    }

    pub fn lava_default() -> BlockState {
        BlockState {
            kind: BlockKind::Lava,
        }
    }

    pub fn air_default() -> BlockState {
        BlockState {
            kind: BlockKind::Air,
        }
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
    pub fn from_pos(pos: BlockPos) -> Self {
        Self {
            min_x: pos.x,
            min_y: pos.y,
            min_z: pos.z,
            max_x: pos.x,
            max_y: pos.y,
            max_z: pos.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    OceanFloorWg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StructureManager;

#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkGenerator;

#[derive(Debug, Clone, Default)]
pub struct CompoundTag;

#[derive(Debug, Clone, Copy, Default)]
pub struct StructurePieceSerializationContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructurePieceType(pub u32);

pub mod structure_piece_types {
    use super::StructurePieceType;
    pub const BURIED_TREASURE_PIECE: StructurePieceType = StructurePieceType(0);
}

pub const BURIED_TREASURE_LOOT: &str = "minecraft:chests/buried_treasure";

pub trait RandomSource {}

pub trait WorldGenLevel {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
    fn get_min_y(&self) -> i32;
    fn create_chest<R: RandomSource>(
        &mut self,
        bounds: &BoundingBox,
        random: &mut R,
        pos: BlockPos,
        loot_table: &str,
        block_state: Option<BlockState>,
    ) -> bool;
}

// ---------------------------------------------------------------------------
// Ported: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuriedTreasurePiece {
    pub piece_type: StructurePieceType,
    pub gen_depth: i32,
    pub bounding_box: BoundingBox,
}

impl fmt::Display for BuriedTreasurePiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BuriedTreasurePiece(type={:?}, depth={}, bbox={:?})",
            self.piece_type, self.gen_depth, self.bounding_box
        )
    }
}

impl BuriedTreasurePiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces#BuriedTreasurePiece(BlockPos)
    pub fn new(pos: BlockPos) -> Self {
        Self {
            piece_type: structure_piece_types::BURIED_TREASURE_PIECE,
            gen_depth: 0,
            bounding_box: BoundingBox::from_pos(pos),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces#BuriedTreasurePiece(CompoundTag)
    pub fn from_tag(_tag: &CompoundTag) -> Self {
        Self {
            piece_type: structure_piece_types::BURIED_TREASURE_PIECE,
            gen_depth: 0,
            bounding_box: BoundingBox::from_pos(BlockPos::new(0, 0, 0)),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces#addAdditionalSaveData
    pub fn add_additional_save_data(
        &self,
        _context: &StructurePieceSerializationContext,
        _tag: &mut CompoundTag,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces#postProcess
    pub fn post_process<L: WorldGenLevel, R: RandomSource>(
        &mut self,
        level: &mut L,
        _structure_manager: &StructureManager,
        _chunk_generator: &ChunkGenerator,
        random: &mut R,
        bounding_box: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        let height = level.get_height(
            HeightmapType::OceanFloorWg,
            self.bounding_box.min_x,
            self.bounding_box.min_z,
        );
        let mut cursor = MutableBlockPos::new(
            self.bounding_box.min_x,
            height,
            self.bounding_box.min_z,
        );

        while cursor.get_y() > level.get_min_y() {
            let state_at = level.get_block_state(cursor.as_pos());
            let state_below = level.get_block_state(cursor.below());
            if state_below == Blocks::sandstone_default()
                || state_below == Blocks::stone_default()
                || state_below == Blocks::andesite_default()
                || state_below == Blocks::granite_default()
                || state_below == Blocks::diorite_default()
            {
                let fill_state = if !state_at.is_air() && !self.is_liquid(state_at) {
                    state_at
                } else {
                    Blocks::sand_default()
                };

                for dir in Direction::values() {
                    let neighbor_pos = cursor.relative(dir);
                    let neighbor_state = level.get_block_state(neighbor_pos);
                    if neighbor_state.is_air() || self.is_liquid(neighbor_state) {
                        let below_neighbor = neighbor_pos.below();
                        let below_neighbor_state = level.get_block_state(below_neighbor);
                        if (below_neighbor_state.is_air() || self.is_liquid(below_neighbor_state))
                            && dir != Direction::Up
                        {
                            level.set_block(neighbor_pos, state_below, 3);
                        } else {
                            level.set_block(neighbor_pos, fill_state, 3);
                        }
                    }
                }

                self.bounding_box = BoundingBox::from_pos(cursor.as_pos());
                level.create_chest(
                    bounding_box,
                    random,
                    cursor.as_pos(),
                    BURIED_TREASURE_LOOT,
                    None,
                );
                return;
            }

            cursor.move_by(0, -1, 0);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces#isLiquid
    fn is_liquid(&self, state: BlockState) -> bool {
        state == Blocks::water_default() || state == Blocks::lava_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        min_y: i32,
        height: i32,
        blocks: HashMap<(i32, i32, i32), BlockState>,
        chest_placed: Option<(BlockPos, String)>,
    }

    impl MockLevel {
        fn new(min_y: i32, height: i32) -> Self {
            Self {
                min_y,
                height,
                blocks: HashMap::new(),
                chest_placed: None,
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, state: BlockState) {
            self.blocks.insert((x, y, z), state);
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_height(&self, _heightmap: HeightmapType, _x: i32, _z: i32) -> i32 {
            self.height
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            *self
                .blocks
                .get(&(pos.x, pos.y, pos.z))
                .unwrap_or(&Blocks::air_default())
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            true
        }

        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn create_chest<R: RandomSource>(
            &mut self,
            _bounds: &BoundingBox,
            _random: &mut R,
            pos: BlockPos,
            loot_table: &str,
            _block_state: Option<BlockState>,
        ) -> bool {
            self.chest_placed = Some((pos, loot_table.to_string()));
            true
        }
    }

    struct DummyRandom;
    impl RandomSource for DummyRandom {}

    #[test]
    fn places_treasure_on_stone_floor() {
        let mut level = MockLevel::new(0, 5);
        level.set(10, 2, 20, Blocks::stone_default());
        level.set(10, 3, 20, Blocks::water_default());

        let mut piece = BuriedTreasurePiece::new(BlockPos::new(10, 0, 20));
        let bounds = piece.bounding_box;
        let mut rng = DummyRandom;
        piece.post_process(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &mut rng,
            &bounds,
            ChunkPos { x: 0, z: 0 },
            BlockPos::new(10, 0, 20),
        );

        let (chest_pos, loot) = level.chest_placed.expect("chest should be placed");
        assert_eq!(chest_pos, BlockPos::new(10, 3, 20));
        assert_eq!(loot, BURIED_TREASURE_LOOT);
        assert_eq!(
            piece.bounding_box,
            BoundingBox::from_pos(BlockPos::new(10, 3, 20))
        );
    }

    #[test]
    fn no_placement_without_solid_ground() {
        let mut level = MockLevel::new(0, 5);
        let mut piece = BuriedTreasurePiece::new(BlockPos::new(1, 0, 2));
        let bounds = piece.bounding_box;
        let mut rng = DummyRandom;
        piece.post_process(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &mut rng,
            &bounds,
            ChunkPos { x: 0, z: 0 },
            BlockPos::new(1, 0, 2),
        );
        assert!(level.chest_placed.is_none());
    }

    #[test]
    fn works_with_negative_coordinates() {
        let mut level = MockLevel::new(-64, -10);
        level.set(-5, -20, -7, Blocks::sandstone_default());

        let mut piece = BuriedTreasurePiece::new(BlockPos::new(-5, -64, -7));
        let bounds = piece.bounding_box;
        let mut rng = DummyRandom;
        piece.post_process(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &mut rng,
            &bounds,
            ChunkPos { x: -1, z: -1 },
            BlockPos::new(-5, -64, -7),
        );

        let (pos, _) = level.chest_placed.expect("chest should be placed");
        assert_eq!(pos, BlockPos::new(-5, -19, -7));
    }

    #[test]
    fn fill_state_preserves_solid_block() {
        let mut level = MockLevel::new(0, 5);
        level.set(0, 2, 0, Blocks::stone_default());
        level.set(0, 3, 0, Blocks::sand_default());
        level.set(1, 3, 0, Blocks::air_default());
        level.set(1, 2, 0, Blocks::stone_default());

        let mut piece = BuriedTreasurePiece::new(BlockPos::new(0, 0, 0));
        let bounds = piece.bounding_box;
        let mut rng = DummyRandom;
        piece.post_process(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &mut rng,
            &bounds,
            ChunkPos { x: 0, z: 0 },
            BlockPos::new(0, 0, 0),
        );

        assert_eq!(
            level.get_block_state(BlockPos::new(1, 3, 0)),
            Blocks::sand_default()
        );
    }

    #[test]
    fn air_neighbor_over_air_gets_ground_support() {
        let mut level = MockLevel::new(0, 5);
        level.set(0, 2, 0, Blocks::stone_default());
        level.set(1, 3, 0, Blocks::water_default());
        level.set(1, 2, 0, Blocks::air_default());

        let mut piece = BuriedTreasurePiece::new(BlockPos::new(0, 0, 0));
        let bounds = piece.bounding_box;
        let mut rng = DummyRandom;
        piece.post_process(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &mut rng,
            &bounds,
            ChunkPos { x: 0, z: 0 },
            BlockPos::new(0, 0, 0),
        );

        assert_eq!(
            level.get_block_state(BlockPos::new(1, 3, 0)),
            Blocks::stone_default()
        );
    }

    #[test]
    fn wrapping_coordinates_do_not_panic() {
        let mut level = MockLevel::new(i32::MIN, i32::MIN.wrapping_add(2));
        let mut piece = BuriedTreasurePiece::new(BlockPos::new(i32::MAX, 0, i32::MIN));
        let bounds = piece.bounding_box;
        let mut rng = DummyRandom;
        piece.post_process(
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            &mut rng,
            &bounds,
            ChunkPos { x: 0, z: 0 },
            BlockPos::new(0, 0, 0),
        );
        assert!(level.chest_placed.is_none());
    }
}
