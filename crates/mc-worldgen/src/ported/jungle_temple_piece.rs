//! Port of `net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece` (1.21.11).
//!
//! NOTE: This module is intentionally self-contained and does not depend on unported
//! Minecraft structure/runtime classes. It provides a faithful port of the piece's
//! placement logic against small local traits/types that model the Java operations
//! used by `JungleTemplePiece`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const WIDTH: i32 = 12;
pub const DEPTH: i32 = 15;

/// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructurePieceType {
    JunglePyramidPiece,
}

/// JAVA: net.minecraft.core.Direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    West,
    East,
    Up,
    Down,
}

/// JAVA: net.minecraft.core.Direction#getOpposite()
impl Direction {
    // JAVA: net.minecraft.core.Direction#getOpposite()
    pub fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

/// JAVA: net.minecraft.world.level.levelgen.structure.BoundingBox
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    // JAVA: net.minecraft.world.level.levelgen.structure.BoundingBox#isInside(net.minecraft.core.Vec3i)
    pub fn is_inside(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.min_x
            && x <= self.max_x
            && y >= self.min_y
            && y <= self.max_y
            && z >= self.min_z
            && z <= self.max_z
    }
}

/// JAVA: net.minecraft.core.BlockPos
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#BlockPos(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// JAVA: net.minecraft.world.level.ChunkPos
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// JAVA: net.minecraft.util.RandomSource (minimal interface)
pub trait RandomSource {
    fn next_i32(&mut self) -> i32;
    fn next_i32_bounded(&mut self, bound: i32) -> i32;
    fn next_f32(&mut self) -> f32;
    fn next_boolean(&mut self) -> bool;
}

/// Simple deterministic RNG for tests/self-contained usage.
/// Not a Java `RandomSource` implementation; used only where callers provide it.
#[derive(Debug, Clone)]
pub struct SimpleRandom {
    state: u64,
}

impl SimpleRandom {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.state;
        if x == 0 {
            x = 0x9E3779B97F4A7C15;
        }
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
}

impl RandomSource for SimpleRandom {
    fn next_i32(&mut self) -> i32 {
        self.next_u64() as i32
    }

    fn next_i32_bounded(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        let v = (self.next_u64() % (bound as u64)) as i32;
        v
    }

    fn next_f32(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32;
        (bits as f32) / ((1u32 << 24) as f32)
    }

    fn next_boolean(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }
}

/// JAVA: net.minecraft.world.level.block.state.properties.AttachFace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachFace {
    Floor,
    Wall,
    Ceiling,
}

/// JAVA: net.minecraft.world.level.block.state.properties.RedstoneSide
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RedstoneSide {
    Up,
    Side,
    None,
}

/// Minimal block identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Cobblestone,
    MossyCobblestone,
    CobblestoneStairs,
    TripwireHook,
    Tripwire,
    RedstoneWire,
    Vine,
    ChiseledStoneBricks,
    Lever,
    StickyPiston,
    Repeater,
    Dispenser,
    Chest,
}

/// Minimal property bag for block states used by this piece.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockStateProps {
    pub facing: Option<Direction>,
    pub attached: Option<bool>,
    pub east: Option<bool>,
    pub west: Option<bool>,
    pub north: Option<bool>,
    pub south: Option<bool>,
    pub up: Option<bool>,
    pub redstone_north: Option<RedstoneSide>,
    pub redstone_south: Option<RedstoneSide>,
    pub redstone_east: Option<RedstoneSide>,
    pub redstone_west: Option<RedstoneSide>,
    pub vine_south: Option<bool>,
    pub vine_east: Option<bool>,
    pub lever_face: Option<AttachFace>,
}

/// JAVA: net.minecraft.world.level.block.state.BlockState (minimal)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
    pub props: BlockStateProps,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.Block#defaultBlockState()
    pub fn default(block: Block) -> Self {
        Self {
            block,
            props: BlockStateProps::default(),
        }
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_facing(mut self, facing: Direction) -> Self {
        self.props.facing = Some(facing);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_attached(mut self, attached: bool) -> Self {
        self.props.attached = Some(attached);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_east(mut self, v: bool) -> Self {
        self.props.east = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_west(mut self, v: bool) -> Self {
        self.props.west = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_north(mut self, v: bool) -> Self {
        self.props.north = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_south(mut self, v: bool) -> Self {
        self.props.south = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_redstone_north(mut self, v: RedstoneSide) -> Self {
        self.props.redstone_north = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_redstone_south(mut self, v: RedstoneSide) -> Self {
        self.props.redstone_south = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_redstone_east(mut self, v: RedstoneSide) -> Self {
        self.props.redstone_east = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_redstone_west(mut self, v: RedstoneSide) -> Self {
        self.props.redstone_west = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_vine_south(mut self, v: bool) -> Self {
        self.props.vine_south = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_vine_east(mut self, v: bool) -> Self {
        self.props.vine_east = Some(v);
        self
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#setValue
    pub fn set_lever_face(mut self, face: AttachFace) -> Self {
        self.props.lever_face = Some(face);
        self
    }
}

/// JAVA: net.minecraft.world.level.block.Blocks (minimal constants used by this piece)
pub mod blocks {
    use super::{Block, BlockState};

    pub fn air() -> BlockState {
        BlockState::default(Block::Air)
    }
    pub fn cobblestone() -> BlockState {
        BlockState::default(Block::Cobblestone)
    }
    pub fn mossy_cobblestone() -> BlockState {
        BlockState::default(Block::MossyCobblestone)
    }
    pub fn cobblestone_stairs() -> BlockState {
        BlockState::default(Block::CobblestoneStairs)
    }
    pub fn tripwire_hook() -> BlockState {
        BlockState::default(Block::TripwireHook)
    }
    pub fn tripwire() -> BlockState {
        BlockState::default(Block::Tripwire)
    }
    pub fn redstone_wire() -> BlockState {
        BlockState::default(Block::RedstoneWire)
    }
    pub fn vine() -> BlockState {
        BlockState::default(Block::Vine)
    }
    pub fn chiseled_stone_bricks() -> BlockState {
        BlockState::default(Block::ChiseledStoneBricks)
    }
    pub fn lever() -> BlockState {
        BlockState::default(Block::Lever)
    }
    pub fn sticky_piston() -> BlockState {
        BlockState::default(Block::StickyPiston)
    }
    pub fn repeater() -> BlockState {
        BlockState::default(Block::Repeater)
    }
    pub fn dispenser() -> BlockState {
        BlockState::default(Block::Dispenser)
    }
    pub fn chest() -> BlockState {
        BlockState::default(Block::Chest)
    }
}

/// JAVA: net.minecraft.world.level.storage.loot.BuiltInLootTables (minimal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltInLootTable {
    JungleTemple,
    JungleTempleDispenser,
}

/// Minimal world/level access used by structure piece placement.
pub trait WorldGenLevel {
    fn get_block(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32);
}

/// Minimal structure manager placeholder.
pub trait StructureManager {}

/// Minimal chunk generator placeholder.
pub trait ChunkGenerator {}

/// JAVA: net.minecraft.world.level.levelgen.structure.pieces.StructurePieceSerializationContext (minimal)
#[derive(Debug, Default, Clone)]
pub struct StructurePieceSerializationContext;

/// Minimal NBT-like tag reader used by this self-contained port.
pub trait CompoundTagReader {
    fn get_boolean_or(&self, key: &str, default: bool) -> bool;
}

/// Minimal NBT-like tag writer used by this self-contained port.
pub trait CompoundTagWriter {
    fn put_boolean(&mut self, key: &str, value: bool);
}

/// Simple in-memory compound tag for tests/self-contained serialization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SimpleCompoundTag {
    booleans: HashMap<String, bool>,
}

impl SimpleCompoundTag {
    pub fn new() -> Self {
        Self {
            booleans: HashMap::new(),
        }
    }
}

impl CompoundTagReader for SimpleCompoundTag {
    fn get_boolean_or(&self, key: &str, default: bool) -> bool {
        self.booleans.get(key).copied().unwrap_or(default)
    }
}

impl CompoundTagWriter for SimpleCompoundTag {
    fn put_boolean(&mut self, key: &str, value: bool) {
        self.booleans.insert(key.to_string(), value);
    }
}

/// JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece.BlockSelector
pub trait BlockSelector {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece.BlockSelector#next
    fn next(&mut self, random: &mut dyn RandomSource, x: i32, y: i32, z: i32, has_next: bool);

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece.BlockSelector#getNext()
    fn get_next(&self) -> BlockState;
}

/// JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece (partial)
#[derive(Debug, Clone)]
pub struct StructurePieceBase {
    pub piece_type: StructurePieceType,
    pub bounding_box: BoundingBox,
    pub orientation: Direction,
}

impl StructurePieceBase {
    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#getWorldX
    pub fn get_world_x(&self, x: i32, z: i32) -> i32 {
        match self.orientation {
            Direction::North | Direction::South => self.bounding_box.min_x.wrapping_add(x),
            Direction::West => self.bounding_box.max_x.wrapping_sub(z),
            Direction::East => self.bounding_box.min_x.wrapping_add(z),
            _ => self.bounding_box.min_x.wrapping_add(x),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#getWorldY
    pub fn get_world_y(&self, y: i32) -> i32 {
        self.bounding_box.min_y.wrapping_add(y)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#getWorldZ
    pub fn get_world_z(&self, x: i32, z: i32) -> i32 {
        match self.orientation {
            Direction::North => self.bounding_box.max_z.wrapping_sub(z),
            Direction::South => self.bounding_box.min_z.wrapping_add(z),
            Direction::West | Direction::East => self.bounding_box.min_z.wrapping_add(x),
            _ => self.bounding_box.min_z.wrapping_add(z),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#placeBlock
    pub fn place_block(
        &self,
        level: &mut dyn WorldGenLevel,
        state: BlockState,
        x: i32,
        y: i32,
        z: i32,
        bounding_box: &BoundingBox,
    ) {
        let wx = self.get_world_x(x, z);
        let wy = self.get_world_y(y);
        let wz = self.get_world_z(x, z);
        if bounding_box.is_inside(wx, wy, wz) {
            level.set_block(BlockPos::new(wx, wy, wz), state, 2);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#getBlock
    pub fn get_block(
        &self,
        level: &dyn WorldGenLevel,
        x: i32,
        y: i32,
        z: i32,
        bounding_box: &BoundingBox,
    ) -> Option<BlockState> {
        let wx = self.get_world_x(x, z);
        let wy = self.get_world_y(y);
        let wz = self.get_world_z(x, z);
        if !bounding_box.is_inside(wx, wy, wz) {
            return None;
        }
        Some(level.get_block(BlockPos::new(wx, wy, wz)))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#generateAirBox
    pub fn generate_air_box(
        &self,
        level: &mut dyn WorldGenLevel,
        bounding_box: &BoundingBox,
        x1: i32,
        y1: i32,
        z1: i32,
        x2: i32,
        y2: i32,
        z2: i32,
    ) {
        for y in y1..=y2 {
            for x in x1..=x2 {
                for z in z1..=z2 {
                    self.place_block(level, blocks::air(), x, y, z, bounding_box);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#generateBox(..., BlockSelector)
    #[allow(clippy::too_many_arguments)]
    pub fn generate_box(
        &self,
        level: &mut dyn WorldGenLevel,
        bounding_box: &BoundingBox,
        x1: i32,
        y1: i32,
        z1: i32,
        x2: i32,
        y2: i32,
        z2: i32,
        _skip_air: bool,
        random: &mut dyn RandomSource,
        selector: &mut dyn BlockSelector,
    ) {
        for y in y1..=y2 {
            for x in x1..=x2 {
                for z in z1..=z2 {
                    selector.next(random, x, y, z, true);
                    self.place_block(level, selector.get_next(), x, y, z, bounding_box);
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#maybeGenerateBlock
    pub fn maybe_generate_block(
        &self,
        level: &mut dyn WorldGenLevel,
        bounding_box: &BoundingBox,
        random: &mut dyn RandomSource,
        chance: f32,
        x: i32,
        y: i32,
        z: i32,
        state: BlockState,
    ) {
        if random.next_f32() < chance {
            self.place_block(level, state, x, y, z, bounding_box);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#createChest
    pub fn create_chest(
        &self,
        level: &mut dyn WorldGenLevel,
        bounding_box: &BoundingBox,
        _random: &mut dyn RandomSource,
        x: i32,
        y: i32,
        z: i32,
        _loot_table: BuiltInLootTable,
    ) -> bool {
        let wx = self.get_world_x(x, z);
        let wy = self.get_world_y(y);
        let wz = self.get_world_z(x, z);
        if !bounding_box.is_inside(wx, wy, wz) {
            return false;
        }
        let pos = BlockPos::new(wx, wy, wz);
        // Faithful enough for self-contained port: place chest if inside bounds.
        // In Java this also checks for existing chest and sets loot table/block entity.
        if level.get_block(pos).block == Block::Chest {
            return false;
        }
        level.set_block(pos, blocks::chest(), 2);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#createDispenser
    #[allow(clippy::too_many_arguments)]
    pub fn create_dispenser(
        &self,
        level: &mut dyn WorldGenLevel,
        bounding_box: &BoundingBox,
        _random: &mut dyn RandomSource,
        x: i32,
        y: i32,
        z: i32,
        facing: Direction,
        _loot_table: BuiltInLootTable,
    ) -> bool {
        let wx = self.get_world_x(x, z);
        let wy = self.get_world_y(y);
        let wz = self.get_world_z(x, z);
        if !bounding_box.is_inside(wx, wy, wz) {
            return false;
        }
        let pos = BlockPos::new(wx, wy, wz);
        if level.get_block(pos).block == Block::Dispenser {
            return false;
        }
        level.set_block(pos, blocks::dispenser().set_facing(facing), 2);
        true
    }
}

/// JAVA: net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece (partial)
#[derive(Debug, Clone)]
pub struct ScatteredFeaturePiece {
    pub base: StructurePieceBase,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub height_position: i32,
}

impl ScatteredFeaturePiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#updateAverageGroundHeight
    pub fn update_average_ground_height(
        &mut self,
        level: &dyn WorldGenLevel,
        bounding_box: &BoundingBox,
        height_offset: i32,
    ) -> bool {
        if self.height_position >= 0 {
            return true;
        }

        let mut total = 0_i32;
        let mut count = 0_i32;

        for z in 0..self.depth {
            for x in 0..self.width {
                let wx = self.base.get_world_x(x, z);
                let wz = self.base.get_world_z(x, z);
                let mut wy = bounding_box.min_y.wrapping_add(self.height);
                // Find top solid-ish: in self-contained port, treat non-air as ground.
                while wy > level_min_y() {
                    let pos = BlockPos::new(wx, wy.wrapping_sub(1), wz);
                    let state = level.get_block(pos);
                    if state.block != Block::Air {
                        break;
                    }
                    wy = wy.wrapping_sub(1);
                }
                total = total.wrapping_add(wy);
                count = count.wrapping_add(1);
            }
        }

        if count == 0 {
            return false;
        }

        self.height_position = total / count;
        self.base.bounding_box.min_y = self.height_position.wrapping_add(height_offset);
        self.base.bounding_box.max_y = self
            .base
            .bounding_box
            .min_y
            .wrapping_add(self.height)
            .wrapping_sub(1);
        true
    }
}

fn level_min_y() -> i32 {
    -64
}

// JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece.MossStoneSelector
#[derive(Debug, Clone)]
pub struct MossStoneSelector {
    next: BlockState,
}

impl Default for MossStoneSelector {
    fn default() -> Self {
        Self {
            next: blocks::cobblestone(),
        }
    }
}

impl BlockSelector for MossStoneSelector {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece.MossStoneSelector#next
    fn next(&mut self, random: &mut dyn RandomSource, _x: i32, _y: i32, _z: i32, _has_next: bool) {
        if random.next_f32() < 0.4_f32 {
            self.next = blocks::cobblestone();
        } else {
            self.next = blocks::mossy_cobblestone();
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece.BlockSelector#getNext()
    fn get_next(&self) -> BlockState {
        self.next.clone()
    }
}

/// JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece
#[derive(Debug, Clone)]
pub struct JungleTemplePiece {
    pub scattered: ScatteredFeaturePiece,
    pub placed_main_chest: bool,
    pub placed_hidden_chest: bool,
    pub placed_trap1: bool,
    pub placed_trap2: bool,
}

// JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#getRandomHorizontalDirection
fn get_random_horizontal_direction(random: &mut dyn RandomSource) -> Direction {
    match random.next_i32_bounded(4) {
        0 => Direction::North,
        1 => Direction::South,
        2 => Direction::West,
        _ => Direction::East,
    }
}

fn make_bounding_box(x: i32, y: i32, z: i32, orientation: Direction, width: i32, height: i32, depth: i32) -> BoundingBox {
    // Standard horizontal orientation mapping used by ScatteredFeaturePiece constructors.
    match orientation {
        Direction::North | Direction::South => BoundingBox {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x.wrapping_add(width).wrapping_sub(1),
            max_y: y.wrapping_add(height).wrapping_sub(1),
            max_z: z.wrapping_add(depth).wrapping_sub(1),
        },
        Direction::West | Direction::East => BoundingBox {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x.wrapping_add(depth).wrapping_sub(1),
            max_y: y.wrapping_add(height).wrapping_sub(1),
            max_z: z.wrapping_add(width).wrapping_sub(1),
        },
        _ => BoundingBox {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x.wrapping_add(width).wrapping_sub(1),
            max_y: y.wrapping_add(height).wrapping_sub(1),
            max_z: z.wrapping_add(depth).wrapping_sub(1),
        },
    }
}

impl JungleTemplePiece {
    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece#<init>(net.minecraft.util.RandomSource,int,int)
    pub fn new(random: &mut dyn RandomSource, x: i32, z: i32) -> Self {
        let orientation = get_random_horizontal_direction(random);
        let bounding_box = make_bounding_box(x, 64, z, orientation, WIDTH, 10, DEPTH);
        Self {
            scattered: ScatteredFeaturePiece {
                base: StructurePieceBase {
                    piece_type: StructurePieceType::JunglePyramidPiece,
                    bounding_box,
                    orientation,
                },
                width: WIDTH,
                height: 10,
                depth: DEPTH,
                height_position: -1,
            },
            placed_main_chest: false,
            placed_hidden_chest: false,
            placed_trap1: false,
            placed_trap2: false,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece#<init>(net.minecraft.nbt.CompoundTag)
    pub fn from_tag(tag: &dyn CompoundTagReader) -> Self {
        // In Java, ScatteredFeaturePiece/StructurePiece read type, bb, orientation, etc.
        // This self-contained port preserves the JungleTemplePiece-specific fields and
        // provides a default base; callers in a full port should hydrate base from tag.
        let orientation = Direction::North;
        let bounding_box = make_bounding_box(0, 64, 0, orientation, WIDTH, 10, DEPTH);
        Self {
            scattered: ScatteredFeaturePiece {
                base: StructurePieceBase {
                    piece_type: StructurePieceType::JunglePyramidPiece,
                    bounding_box,
                    orientation,
                },
                width: WIDTH,
                height: 10,
                depth: DEPTH,
                height_position: -1,
            },
            placed_main_chest: tag.get_boolean_or("placedMainChest", false),
            placed_hidden_chest: tag.get_boolean_or("placedHiddenChest", false),
            placed_trap1: tag.get_boolean_or("placedTrap1", false),
            placed_trap2: tag.get_boolean_or("placedTrap2", false),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece#addAdditionalSaveData
    pub fn add_additional_save_data(
        &self,
        _context: &StructurePieceSerializationContext,
        tag: &mut dyn CompoundTagWriter,
    ) {
        // super.addAdditionalSaveData($$0, $$1); omitted in self-contained port.
        tag.put_boolean("placedMainChest", self.placed_main_chest);
        tag.put_boolean("placedHiddenChest", self.placed_hidden_chest);
        tag.put_boolean("placedTrap1", self.placed_trap1);
        tag.put_boolean("placedTrap2", self.placed_trap2);
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece#postProcess
    #[allow(clippy::too_many_arguments)]
    pub fn post_process(
        &mut self,
        level: &mut dyn WorldGenLevel,
        _structure_manager: &dyn StructureManager,
        _chunk_generator: &dyn ChunkGenerator,
        random: &mut dyn RandomSource,
        bounding_box: BoundingBox,
        _chunk_pos: ChunkPos,
        _block_pos: BlockPos,
    ) {
        if !self
            .scattered
            .update_average_ground_height(level, &bounding_box, 0)
        {
            return;
        }

        let mut selector = MossStoneSelector::default();
        let base = self.scattered.base.clone();
        let width = self.scattered.width;
        let depth = self.scattered.depth;

        base.generate_box(
            level,
            &bounding_box,
            0,
            -4,
            0,
            width.wrapping_sub(1),
            0,
            depth.wrapping_sub(1),
            false,
            random,
            &mut selector,
        );
        base.generate_box(level, &bounding_box, 2, 1, 2, 9, 2, 2, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 2, 1, 12, 9, 2, 12, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 2, 1, 3, 2, 2, 11, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 9, 1, 3, 9, 2, 11, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 1, 3, 1, 10, 6, 1, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 1, 3, 13, 10, 6, 13, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 1, 3, 2, 1, 6, 12, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 10, 3, 2, 10, 6, 12, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 2, 3, 2, 9, 3, 12, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 2, 6, 2, 9, 6, 12, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 3, 7, 3, 8, 7, 11, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 4, 8, 4, 7, 8, 10, false, random, &mut selector);
        base.generate_air_box(level, &bounding_box, 3, 1, 3, 8, 2, 11);
        base.generate_air_box(level, &bounding_box, 4, 3, 6, 7, 3, 9);
        base.generate_air_box(level, &bounding_box, 2, 4, 2, 9, 5, 12);
        base.generate_air_box(level, &bounding_box, 4, 6, 5, 7, 6, 9);
        base.generate_air_box(level, &bounding_box, 5, 7, 6, 6, 7, 8);
        base.generate_air_box(level, &bounding_box, 5, 1, 2, 6, 2, 2);
        base.generate_air_box(level, &bounding_box, 5, 2, 12, 6, 2, 12);
        base.generate_air_box(level, &bounding_box, 5, 5, 1, 6, 5, 1);
        base.generate_air_box(level, &bounding_box, 5, 5, 13, 6, 5, 13);
        base.place_block(level, blocks::air(), 1, 5, 5, &bounding_box);
        base.place_block(level, blocks::air(), 10, 5, 5, &bounding_box);
        base.place_block(level, blocks::air(), 1, 5, 9, &bounding_box);
        base.place_block(level, blocks::air(), 10, 5, 9, &bounding_box);

        let mut z = 0_i32;
        while z <= 14 {
            base.generate_box(level, &bounding_box, 2, 4, z, 2, 5, z, false, random, &mut selector);
            base.generate_box(level, &bounding_box, 4, 4, z, 4, 5, z, false, random, &mut selector);
            base.generate_box(level, &bounding_box, 7, 4, z, 7, 5, z, false, random, &mut selector);
            base.generate_box(level, &bounding_box, 9, 4, z, 9, 5, z, false, random, &mut selector);
            z = z.wrapping_add(14);
        }

        base.generate_box(level, &bounding_box, 5, 6, 0, 6, 6, 0, false, random, &mut selector);

        let mut x = 0_i32;
        while x <= 11 {
            let mut zz = 2_i32;
            while zz <= 12 {
                base.generate_box(level, &bounding_box, x, 4, zz, x, 5, zz, false, random, &mut selector);
                zz = zz.wrapping_add(2);
            }
            base.generate_box(level, &bounding_box, x, 6, 5, x, 6, 5, false, random, &mut selector);
            base.generate_box(level, &bounding_box, x, 6, 9, x, 6, 9, false, random, &mut selector);
            x = x.wrapping_add(11);
        }

        base.generate_box(level, &bounding_box, 2, 7, 2, 2, 9, 2, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 9, 7, 2, 9, 9, 2, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 2, 7, 12, 2, 9, 12, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 9, 7, 12, 9, 9, 12, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 4, 9, 4, 4, 9, 4, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 7, 9, 4, 7, 9, 4, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 4, 9, 10, 4, 9, 10, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 7, 9, 10, 7, 9, 10, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 5, 9, 7, 6, 9, 7, false, random, &mut selector);

        let stairs_east = blocks::cobblestone_stairs().set_facing(Direction::East);
        let stairs_west = blocks::cobblestone_stairs().set_facing(Direction::West);
        let stairs_south = blocks::cobblestone_stairs().set_facing(Direction::South);
        let stairs_north = blocks::cobblestone_stairs().set_facing(Direction::North);

        base.place_block(level, stairs_north.clone(), 5, 9, 6, &bounding_box);
        base.place_block(level, stairs_north.clone(), 6, 9, 6, &bounding_box);
        base.place_block(level, stairs_south.clone(), 5, 9, 8, &bounding_box);
        base.place_block(level, stairs_south.clone(), 6, 9, 8, &bounding_box);
        base.place_block(level, stairs_north.clone(), 4, 0, 0, &bounding_box);
        base.place_block(level, stairs_north.clone(), 5, 0, 0, &bounding_box);
        base.place_block(level, stairs_north.clone(), 6, 0, 0, &bounding_box);
        base.place_block(level, stairs_north.clone(), 7, 0, 0, &bounding_box);
        base.place_block(level, stairs_north.clone(), 4, 1, 8, &bounding_box);
        base.place_block(level, stairs_north.clone(), 4, 2, 9, &bounding_box);
        base.place_block(level, stairs_north.clone(), 4, 3, 10, &bounding_box);
        base.place_block(level, stairs_north.clone(), 7, 1, 8, &bounding_box);
        base.place_block(level, stairs_north.clone(), 7, 2, 9, &bounding_box);
        base.place_block(level, stairs_north, 7, 3, 10, &bounding_box);
        base.generate_box(level, &bounding_box, 4, 1, 9, 4, 1, 9, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 7, 1, 9, 7, 1, 9, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 4, 1, 10, 7, 2, 10, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 5, 4, 5, 6, 4, 5, false, random, &mut selector);
        base.place_block(level, stairs_east, 4, 4, 5, &bounding_box);
        base.place_block(level, stairs_west, 7, 4, 5, &bounding_box);

        let mut i = 0_i32;
        while i < 4 {
            let y = 0_i32.wrapping_sub(i);
            let zz = 6_i32.wrapping_add(i);
            base.place_block(level, stairs_south.clone(), 5, y, zz, &bounding_box);
            base.place_block(level, stairs_south.clone(), 6, y, zz, &bounding_box);
            base.generate_air_box(
                level,
                &bounding_box,
                5,
                0_i32.wrapping_sub(i),
                7_i32.wrapping_add(i),
                6,
                0_i32.wrapping_sub(i),
                9_i32.wrapping_add(i),
            );
            i = i.wrapping_add(1);
        }

        base.generate_air_box(level, &bounding_box, 1, -3, 12, 10, -1, 13);
        base.generate_air_box(level, &bounding_box, 1, -3, 1, 3, -1, 13);
        base.generate_air_box(level, &bounding_box, 1, -3, 1, 9, -1, 5);

        let mut xx = 1_i32;
        while xx <= 13 {
            base.generate_box(level, &bounding_box, 1, -3, xx, 1, -2, xx, false, random, &mut selector);
            xx = xx.wrapping_add(2);
        }

        let mut xx2 = 2_i32;
        while xx2 <= 12 {
            base.generate_box(level, &bounding_box, 1, -1, xx2, 3, -1, xx2, false, random, &mut selector);
            xx2 = xx2.wrapping_add(2);
        }

        base.generate_box(level, &bounding_box, 2, -2, 1, 5, -2, 1, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 7, -2, 1, 9, -2, 1, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 6, -3, 1, 6, -3, 1, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 6, -1, 1, 6, -1, 1, false, random, &mut selector);

        base.place_block(
            level,
            blocks::tripwire_hook()
                .set_facing(Direction::East)
                .set_attached(true),
            1,
            -3,
            8,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::tripwire_hook()
                .set_facing(Direction::West)
                .set_attached(true),
            4,
            -3,
            8,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::tripwire()
                .set_east(true)
                .set_west(true)
                .set_attached(true),
            2,
            -3,
            8,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::tripwire()
                .set_east(true)
                .set_west(true)
                .set_attached(true),
            3,
            -3,
            8,
            &bounding_box,
        );

        let redstone_ns = blocks::redstone_wire()
            .set_redstone_north(RedstoneSide::Side)
            .set_redstone_south(RedstoneSide::Side);
        base.place_block(level, redstone_ns.clone(), 5, -3, 7, &bounding_box);
        base.place_block(level, redstone_ns.clone(), 5, -3, 6, &bounding_box);
        base.place_block(level, redstone_ns.clone(), 5, -3, 5, &bounding_box);
        base.place_block(level, redstone_ns.clone(), 5, -3, 4, &bounding_box);
        base.place_block(level, redstone_ns.clone(), 5, -3, 3, &bounding_box);
        base.place_block(level, redstone_ns.clone(), 5, -3, 2, &bounding_box);
        base.place_block(
            level,
            blocks::redstone_wire()
                .set_redstone_north(RedstoneSide::Side)
                .set_redstone_west(RedstoneSide::Side),
            5,
            -3,
            1,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::redstone_wire()
                .set_redstone_east(RedstoneSide::Side)
                .set_redstone_west(RedstoneSide::Side),
            4,
            -3,
            1,
            &bounding_box,
        );
        base.place_block(level, blocks::mossy_cobblestone(), 3, -3, 1, &bounding_box);
        if !self.placed_trap1 {
            self.placed_trap1 = base.create_dispenser(
                level,
                &bounding_box,
                random,
                3,
                -2,
                1,
                Direction::North,
                BuiltInLootTable::JungleTempleDispenser,
            );
        }

        base.place_block(
            level,
            blocks::vine().set_vine_south(true),
            3,
            -2,
            2,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::tripwire_hook()
                .set_facing(Direction::North)
                .set_attached(true),
            7,
            -3,
            1,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::tripwire_hook()
                .set_facing(Direction::South)
                .set_attached(true),
            7,
            -3,
            5,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::tripwire()
                .set_north(true)
                .set_south(true)
                .set_attached(true),
            7,
            -3,
            2,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::tripwire()
                .set_north(true)
                .set_south(true)
                .set_attached(true),
            7,
            -3,
            3,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::tripwire()
                .set_north(true)
                .set_south(true)
                .set_attached(true),
            7,
            -3,
            4,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::redstone_wire()
                .set_redstone_east(RedstoneSide::Side)
                .set_redstone_west(RedstoneSide::Side),
            8,
            -3,
            6,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::redstone_wire()
                .set_redstone_west(RedstoneSide::Side)
                .set_redstone_south(RedstoneSide::Side),
            9,
            -3,
            6,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::redstone_wire()
                .set_redstone_north(RedstoneSide::Side)
                .set_redstone_south(RedstoneSide::Up),
            9,
            -3,
            5,
            &bounding_box,
        );
        base.place_block(level, blocks::mossy_cobblestone(), 9, -3, 4, &bounding_box);
        base.place_block(level, redstone_ns.clone(), 9, -2, 4, &bounding_box);
        if !self.placed_trap2 {
            self.placed_trap2 = base.create_dispenser(
                level,
                &bounding_box,
                random,
                9,
                -2,
                3,
                Direction::West,
                BuiltInLootTable::JungleTempleDispenser,
            );
        }

        base.place_block(
            level,
            blocks::vine().set_vine_east(true),
            8,
            -1,
            3,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::vine().set_vine_east(true),
            8,
            -2,
            3,
            &bounding_box,
        );
        if !self.placed_main_chest {
            self.placed_main_chest = base.create_chest(
                level,
                &bounding_box,
                random,
                8,
                -3,
                3,
                BuiltInLootTable::JungleTemple,
            );
        }

        base.place_block(level, blocks::mossy_cobblestone(), 9, -3, 2, &bounding_box);
        base.place_block(level, blocks::mossy_cobblestone(), 8, -3, 1, &bounding_box);
        base.place_block(level, blocks::mossy_cobblestone(), 4, -3, 5, &bounding_box);
        base.place_block(level, blocks::mossy_cobblestone(), 5, -2, 5, &bounding_box);
        base.place_block(level, blocks::mossy_cobblestone(), 5, -1, 5, &bounding_box);
        base.place_block(level, blocks::mossy_cobblestone(), 6, -3, 5, &bounding_box);
        base.place_block(level, blocks::mossy_cobblestone(), 7, -2, 5, &bounding_box);
        base.place_block(level, blocks::mossy_cobblestone(), 7, -1, 5, &bounding_box);
        base.place_block(level, blocks::mossy_cobblestone(), 8, -3, 5, &bounding_box);
        base.generate_box(level, &bounding_box, 9, -1, 1, 9, -1, 5, false, random, &mut selector);
        base.generate_air_box(level, &bounding_box, 8, -3, 8, 10, -1, 10);
        base.place_block(level, blocks::chiseled_stone_bricks(), 8, -2, 11, &bounding_box);
        base.place_block(level, blocks::chiseled_stone_bricks(), 9, -2, 11, &bounding_box);
        base.place_block(level, blocks::chiseled_stone_bricks(), 10, -2, 11, &bounding_box);
        let lever = blocks::lever()
            .set_facing(Direction::North)
            .set_lever_face(AttachFace::Wall);
        base.place_block(level, lever.clone(), 8, -2, 12, &bounding_box);
        base.place_block(level, lever.clone(), 9, -2, 12, &bounding_box);
        base.place_block(level, lever, 10, -2, 12, &bounding_box);
        base.generate_box(level, &bounding_box, 8, -3, 8, 8, -3, 10, false, random, &mut selector);
        base.generate_box(level, &bounding_box, 10, -3, 8, 10, -3, 10, false, random, &mut selector);
        base.place_block(level, blocks::mossy_cobblestone(), 10, -2, 9, &bounding_box);
        base.place_block(level, redstone_ns.clone(), 8, -2, 9, &bounding_box);
        base.place_block(level, redstone_ns, 8, -2, 10, &bounding_box);
        base.place_block(
            level,
            blocks::redstone_wire()
                .set_redstone_north(RedstoneSide::Side)
                .set_redstone_south(RedstoneSide::Side)
                .set_redstone_east(RedstoneSide::Side)
                .set_redstone_west(RedstoneSide::Side),
            10,
            -1,
            9,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::sticky_piston().set_facing(Direction::Up),
            9,
            -2,
            8,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::sticky_piston().set_facing(Direction::West),
            10,
            -2,
            8,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::sticky_piston().set_facing(Direction::West),
            10,
            -1,
            8,
            &bounding_box,
        );
        base.place_block(
            level,
            blocks::repeater().set_facing(Direction::North),
            10,
            -2,
            10,
            &bounding_box,
        );
        if !self.placed_hidden_chest {
            self.placed_hidden_chest = base.create_chest(
                level,
                &bounding_box,
                random,
                9,
                -3,
                10,
                BuiltInLootTable::JungleTemple,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default)]
    struct TestLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
    }

    impl TestLevel {
        fn key(pos: BlockPos) -> (i32, i32, i32) {
            (pos.x, pos.y, pos.z)
        }
    }

    impl WorldGenLevel for TestLevel {
        fn get_block(&self, pos: BlockPos) -> BlockState {
            self.blocks
                .get(&Self::key(pos))
                .cloned()
                .unwrap_or_else(blocks::air)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) {
            self.blocks.insert(Self::key(pos), state);
        }
    }

    struct NoopStructureManager;
    impl StructureManager for NoopStructureManager {}
    struct NoopChunkGenerator;
    impl ChunkGenerator for NoopChunkGenerator {}

    #[derive(Clone, Debug)]
    struct SeqRandom {
        f32_vals: Vec<f32>,
        i32_vals: Vec<i32>,
        bool_vals: Vec<bool>,
        f32_idx: usize,
        i32_idx: usize,
        bool_idx: usize,
    }

    impl SeqRandom {
        fn new(f32_vals: Vec<f32>, i32_vals: Vec<i32>, bool_vals: Vec<bool>) -> Self {
            Self {
                f32_vals,
                i32_vals,
                bool_vals,
                f32_idx: 0,
                i32_idx: 0,
                bool_idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_i32(&mut self) -> i32 {
            let v = self.i32_vals.get(self.i32_idx).copied().unwrap_or(0);
            self.i32_idx = self.i32_idx.saturating_add(1);
            v
        }

        fn next_i32_bounded(&mut self, bound: i32) -> i32 {
            let v = self.next_i32();
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }

        fn next_f32(&mut self) -> f32 {
            let v = self.f32_vals.get(self.f32_idx).copied().unwrap_or(0.0);
            self.f32_idx = self.f32_idx.saturating_add(1);
            v
        }

        fn next_boolean(&mut self) -> bool {
            let v = self.bool_vals.get(self.bool_idx).copied().unwrap_or(false);
            self.bool_idx = self.bool_idx.saturating_add(1);
            v
        }
    }

    fn piece_at_origin() -> JungleTemplePiece {
        let mut rng = SimpleRandom::new(1);
        let mut piece = JungleTemplePiece::new(&mut rng, 0, 0);
        // Make tests deterministic and local-space friendly.
        piece.scattered.base.orientation = Direction::South;
        piece.scattered.base.bounding_box = BoundingBox {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: WIDTH - 1,
            max_y: 73,
            max_z: DEPTH - 1,
        };
        piece.scattered.height_position = 64;
        piece
    }

    #[test]
    fn moss_stone_selector_matches_java_threshold() {
        let mut selector = MossStoneSelector::default();
        let mut rng = SeqRandom::new(vec![0.39, 0.4, 0.99], vec![], vec![]);
        selector.next(&mut rng, 0, 0, 0, true);
        assert_eq!(selector.get_next().block, Block::Cobblestone);
        selector.next(&mut rng, 0, 0, 0, true);
        assert_eq!(selector.get_next().block, Block::MossyCobblestone);
        selector.next(&mut rng, 0, 0, 0, true);
        assert_eq!(selector.get_next().block, Block::MossyCobblestone);
    }

    #[test]
    fn add_additional_save_data_roundtrip_fields() {
        let mut tag = SimpleCompoundTag::new();
        tag.put_boolean("placedMainChest", true);
        tag.put_boolean("placedHiddenChest", false);
        tag.put_boolean("placedTrap1", true);
        tag.put_boolean("placedTrap2", false);

        let piece = JungleTemplePiece::from_tag(&tag);
        assert!(piece.placed_main_chest);
        assert!(!piece.placed_hidden_chest);
        assert!(piece.placed_trap1);
        assert!(!piece.placed_trap2);

        let mut out = SimpleCompoundTag::new();
        piece.add_additional_save_data(&StructurePieceSerializationContext, &mut out);
        assert_eq!(out.get_boolean_or("placedMainChest", false), true);
        assert_eq!(out.get_boolean_or("placedHiddenChest", true), false);
        assert_eq!(out.get_boolean_or("placedTrap1", false), true);
        assert_eq!(out.get_boolean_or("placedTrap2", true), false);
    }

    #[test]
    fn post_process_places_key_features_and_sets_flags() {
        let mut level = TestLevel::default();
        let mut piece = piece_at_origin();
        let bb = BoundingBox {
            min_x: -16,
            min_y: 0,
            min_z: -16,
            max_x: 32,
            max_y: 128,
            max_z: 32,
        };
        let mut rng = SeqRandom::new(vec![0.5; 4096], vec![0; 64], vec![false; 64]);

        piece.post_process(
            &mut level,
            &NoopStructureManager,
            &NoopChunkGenerator,
            &mut rng,
            bb,
            ChunkPos { x: 0, z: 0 },
            BlockPos::new(0, 64, 0),
        );

        assert!(piece.placed_main_chest);
        assert!(piece.placed_hidden_chest);
        assert!(piece.placed_trap1);
        assert!(piece.placed_trap2);

        // Entrance air carved by generateAirBox($$0, $$4, 5, 1, 2, 6, 2, 2)
        assert_eq!(level.get_block(BlockPos::new(5, 65, 2)).block, Block::Air);
        assert_eq!(level.get_block(BlockPos::new(6, 66, 2)).block, Block::Air);

        // Bottom lever at (8, -2, 12) with NORTH/WALL
        let lever = level.get_block(BlockPos::new(8, 62, 12));
        assert_eq!(lever.block, Block::Lever);
        assert_eq!(lever.props.facing, Some(Direction::North));
        assert_eq!(lever.props.lever_face, Some(AttachFace::Wall));

        // Hidden chest at (9, -3, 10)
        assert_eq!(level.get_block(BlockPos::new(9, 61, 10)).block, Block::Chest);
        // Main chest at (8, -3, 3)
        assert_eq!(level.get_block(BlockPos::new(8, 61, 3)).block, Block::Chest);

        // Dispenser trap1 at (3, -2, 1) facing NORTH
        let disp1 = level.get_block(BlockPos::new(3, 62, 1));
        assert_eq!(disp1.block, Block::Dispenser);
        assert_eq!(disp1.props.facing, Some(Direction::North));

        // Dispenser trap2 at (9, -2, 3) facing WEST
        let disp2 = level.get_block(BlockPos::new(9, 62, 3));
        assert_eq!(disp2.block, Block::Dispenser);
        assert_eq!(disp2.props.facing, Some(Direction::West));
    }

    #[test]
    fn post_process_respects_bounding_box() {
        let mut level = TestLevel::default();
        let mut piece = piece_at_origin();
        let tiny_bb = BoundingBox {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 0,
            max_y: 64,
            max_z: 0,
        };
        let mut rng = SeqRandom::new(vec![0.5; 4096], vec![0; 64], vec![false; 64]);

        piece.post_process(
            &mut level,
            &NoopStructureManager,
            &NoopChunkGenerator,
            &mut rng,
            tiny_bb,
            ChunkPos { x: 0, z: 0 },
            BlockPos::new(0, 64, 0),
        );

        // Only positions inside the tiny bounding box may be written.
        for (&(x, y, z), _) in &level.blocks {
            assert!(tiny_bb.is_inside(x, y, z), "wrote outside bb: {x},{y},{z}");
        }
    }

    #[test]
    fn coordinate_transform_wrapping_and_negative() {
        let mut base = StructurePieceBase {
            piece_type: StructurePieceType::JunglePyramidPiece,
            bounding_box: BoundingBox {
                min_x: i32::MAX - 2,
                min_y: -10,
                min_z: i32::MIN + 2,
                max_x: i32::MAX - 2,
                max_y: 10,
                max_z: i32::MIN + 2,
            },
            orientation: Direction::South,
        };

        let wx = base.get_world_x(5, 1);
        let wy = base.get_world_y(-3);
        let wz = base.get_world_z(5, 1);
        assert_eq!(wx, (i32::MAX - 2).wrapping_add(5));
        assert_eq!(wy, (-10_i32).wrapping_add(-3));
        assert_eq!(wz, (i32::MIN + 2).wrapping_add(1));

        base.orientation = Direction::West;
        base.bounding_box.max_x = 100;
        base.bounding_box.min_z = -20;
        assert_eq!(base.get_world_x(1, 3), 100_i32.wrapping_sub(3));
        assert_eq!(base.get_world_z(1, 3), (-20_i32).wrapping_add(1));
    }

    #[test]
    fn from_tag_defaults_missing_booleans_to_false() {
        let tag = SimpleCompoundTag::new();
        let piece = JungleTemplePiece::from_tag(&tag);
        assert!(!piece.placed_main_chest);
        assert!(!piece.placed_hidden_chest);
        assert!(!piece.placed_trap1);
        assert!(!piece.placed_trap2);
    }
}
