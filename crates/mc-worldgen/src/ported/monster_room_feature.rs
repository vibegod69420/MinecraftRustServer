//! Port of `net.minecraft.world.level.levelgen.feature.MonsterRoomFeature` (1.21.11).

use std::fmt;

/// JAVA: net.minecraft.core.BlockPos
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }

    pub fn above(self) -> Self {
        self.offset(0, 1, 0)
    }

    pub fn below(self) -> Self {
        self.offset(0, -1, 0)
    }

    pub fn relative(self, dir: Direction) -> Self {
        let (dx, dy, dz) = dir.normal();
        self.offset(dx, dy, dz)
    }

    pub fn get_x(self) -> i32 {
        self.x
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn get_z(self) -> i32 {
        self.z
    }
}

/// JAVA: net.minecraft.core.Direction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
    Up,
    Down,
}

impl Direction {
    pub const fn normal(self) -> (i32, i32, i32) {
        match self {
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
            Direction::Up => (0, 1, 0),
            Direction::Down => (0, -1, 0),
        }
    }
}

/// JAVA: net.minecraft.core.Direction.Plane#HORIZONTAL
pub const HORIZONTAL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// Minimal block identity used by MonsterRoomFeature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    CaveAir,
    Cobblestone,
    MossyCobblestone,
    Chest,
    Spawner,
    Other,
}

/// Minimal block state representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub kind: BlockKind,
    pub solid: bool,
    pub features_cannot_replace: bool,
}

impl BlockState {
    pub const fn new(kind: BlockKind, solid: bool, features_cannot_replace: bool) -> Self {
        Self {
            kind,
            solid,
            features_cannot_replace,
        }
    }

    pub const fn cave_air() -> Self {
        Self::new(BlockKind::CaveAir, false, false)
    }

    pub const fn cobblestone() -> Self {
        Self::new(BlockKind::Cobblestone, true, false)
    }

    pub const fn mossy_cobblestone() -> Self {
        Self::new(BlockKind::MossyCobblestone, true, false)
    }

    pub const fn chest() -> Self {
        Self::new(BlockKind::Chest, true, false)
    }

    pub const fn spawner() -> Self {
        Self::new(BlockKind::Spawner, false, false)
    }

    pub fn is_solid(self) -> bool {
        self.solid
    }

    pub fn is(self, kind: BlockKind) -> bool {
        self.kind == kind
    }
}

/// JAVA: net.minecraft.world.level.block.Blocks#CAVE_AIR.defaultBlockState()
pub const AIR: BlockState = BlockState::cave_air();

/// JAVA: net.minecraft.world.entity.EntityType
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityType {
    Skeleton,
    Zombie,
    Spider,
}

/// JAVA: net.minecraft.world.level.levelgen.feature.MonsterRoomFeature#MOBS
const MOBS: [EntityType; 4] = [
    EntityType::Skeleton,
    EntityType::Zombie,
    EntityType::Zombie,
    EntityType::Spider,
];

/// JAVA: net.minecraft.world.level.storage.loot.BuiltInLootTables#SIMPLE_DUNGEON
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LootTable {
    SimpleDungeon,
}

/// Minimal random source interface matching Java RandomSource usage.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Minimal level interface required by MonsterRoomFeature.
pub trait MonsterRoomLevel<R: RandomSource> {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32);
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_min_y(&self) -> i32;
    fn try_set_spawner_entity(
        &mut self,
        pos: BlockPos,
        entity: EntityType,
        random: &mut R,
    ) -> bool;
    fn set_chest_loot_table(&mut self, pos: BlockPos, random: &mut R, loot_table: LootTable);
}

/// Minimal feature place context.
pub struct FeaturePlaceContext<'a, R: RandomSource, L: MonsterRoomLevel<R>> {
    pub origin: BlockPos,
    pub random: &'a mut R,
    pub level: &'a mut L,
}

/// JAVA: net.minecraft.world.level.levelgen.feature.Feature#isReplaceable
pub fn is_replaceable_features_cannot_replace(state: BlockState) -> bool {
    !state.features_cannot_replace
}

/// JAVA: net.minecraft.world.level.levelgen.feature.Feature#safeSetBlock
pub fn safe_set_block<R: RandomSource, L: MonsterRoomLevel<R>, P: Fn(BlockState) -> bool>(
    level: &mut L,
    pos: BlockPos,
    state: BlockState,
    predicate: P,
) {
    if predicate(level.get_block_state(pos)) {
        level.set_block(pos, state, 2);
    }
}

/// JAVA: net.minecraft.world.level.levelgen.structure.StructurePiece#reorient
pub fn reorient_chest<R: RandomSource, L: MonsterRoomLevel<R>>(
    _level: &L,
    _pos: BlockPos,
    state: BlockState,
) -> BlockState {
    state
}

fn log_spawner_error(pos: BlockPos) {
    eprintln!(
        "Failed to fetch mob spawner entity at ({}, {}, {})",
        pos.get_x(),
        pos.get_y(),
        pos.get_z()
    );
}

/// JAVA: net.minecraft.world.level.levelgen.feature.MonsterRoomFeature
#[derive(Debug, Clone, Copy, Default)]
pub struct MonsterRoomFeature;

impl fmt::Display for MonsterRoomFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MonsterRoomFeature")
    }
}

impl MonsterRoomFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.MonsterRoomFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.MonsterRoomFeature#place
    pub fn place<R: RandomSource, L: MonsterRoomLevel<R>>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, R, L>,
    ) -> bool {
        let origin = ctx.origin;
        let random = &mut *ctx.random;
        let level = &mut *ctx.level;

        let size_x: i32 = random.next_int(2).wrapping_add(2);
        let min_x: i32 = size_x.wrapping_neg().wrapping_sub(1);
        let max_x: i32 = size_x.wrapping_add(1);
        let size_z: i32 = random.next_int(2).wrapping_add(2);
        let min_z: i32 = size_z.wrapping_neg().wrapping_sub(1);
        let max_z: i32 = size_z.wrapping_add(1);
        let mut openings: i32 = 0;

        let mut dx = min_x;
        while dx <= max_x {
            let mut dy = -1i32;
            while dy <= 4 {
                let mut dz = min_z;
                while dz <= max_z {
                    let pos = origin.offset(dx, dy, dz);
                    let solid = level.get_block_state(pos).is_solid();
                    if dy == -1 && !solid {
                        return false;
                    }
                    if dy == 4 && !solid {
                        return false;
                    }
                    if (dx == min_x || dx == max_x || dz == min_z || dz == max_z)
                        && dy == 0
                        && level.is_empty_block(pos)
                        && level.is_empty_block(pos.above())
                    {
                        openings = openings.wrapping_add(1);
                    }
                    dz = dz.wrapping_add(1);
                }
                dy = dy.wrapping_add(1);
            }
            dx = dx.wrapping_add(1);
        }

        if openings < 1 || openings > 5 {
            return false;
        }

        let mut dx = min_x;
        while dx <= max_x {
            let mut dy = 3i32;
            while dy >= -1 {
                let mut dz = min_z;
                while dz <= max_z {
                    let pos = origin.offset(dx, dy, dz);
                    let block_state = level.get_block_state(pos);
                    if dx == min_x
                        || dy == -1
                        || dz == min_z
                        || dx == max_x
                        || dy == 4
                        || dz == max_z
                    {
                        if pos.get_y() >= level.get_min_y()
                            && !level.get_block_state(pos.below()).is_solid()
                        {
                            level.set_block(pos, AIR, 2);
                        } else if block_state.is_solid() && !block_state.is(BlockKind::Chest) {
                            if dy == -1 && random.next_int(4) != 0 {
                                safe_set_block(
                                    level,
                                    pos,
                                    BlockState::mossy_cobblestone(),
                                    is_replaceable_features_cannot_replace,
                                );
                            } else {
                                safe_set_block(
                                    level,
                                    pos,
                                    BlockState::cobblestone(),
                                    is_replaceable_features_cannot_replace,
                                );
                            }
                        }
                    } else if !block_state.is(BlockKind::Chest)
                        && !block_state.is(BlockKind::Spawner)
                    {
                        safe_set_block(level, pos, AIR, is_replaceable_features_cannot_replace);
                    }
                    dz = dz.wrapping_add(1);
                }
                dy = dy.wrapping_sub(1);
            }
            dx = dx.wrapping_add(1);
        }

        let mut chest_attempts = 0i32;
        while chest_attempts < 2 {
            let mut j = 0i32;
            while j < 3 {
                let x = origin
                    .get_x()
                    .wrapping_add(random.next_int(size_x.wrapping_mul(2).wrapping_add(1)))
                    .wrapping_sub(size_x);
                let y = origin.get_y();
                let z = origin
                    .get_z()
                    .wrapping_add(random.next_int(size_z.wrapping_mul(2).wrapping_add(1)))
                    .wrapping_sub(size_z);
                let chest_pos = BlockPos::new(x, y, z);
                if level.is_empty_block(chest_pos) {
                    let mut solid_neighbors: i32 = 0;
                    for dir in HORIZONTAL_DIRECTIONS {
                        if level.get_block_state(chest_pos.relative(dir)).is_solid() {
                            solid_neighbors = solid_neighbors.wrapping_add(1);
                        }
                    }
                    if solid_neighbors == 1 {
                        let chest_state = reorient_chest(&*level, chest_pos, BlockState::chest());
                        safe_set_block(
                            level,
                            chest_pos,
                            chest_state,
                            is_replaceable_features_cannot_replace,
                        );
                        level.set_chest_loot_table(chest_pos, random, LootTable::SimpleDungeon);
                        break;
                    }
                }
                j = j.wrapping_add(1);
            }
            chest_attempts = chest_attempts.wrapping_add(1);
        }

        safe_set_block(
            level,
            origin,
            BlockState::spawner(),
            is_replaceable_features_cannot_replace,
        );
        let entity = self.random_entity_id(random);
        if !level.try_set_spawner_entity(origin, entity, random) {
            log_spawner_error(origin);
        }

        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.MonsterRoomFeature#randomEntityId
    fn random_entity_id<R: RandomSource>(&self, random: &mut R) -> EntityType {
        let idx = random.next_int(MOBS.len() as i32);
        MOBS[idx as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.idx < self.values.len() {
                self.values[self.idx]
            } else {
                0
            };
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    #[derive(Clone)]
    struct MockLevel {
        blocks: HashMap<BlockPos, BlockState>,
        default_state: BlockState,
        min_y: i32,
        spawner_entity: Option<(BlockPos, EntityType)>,
        chest_loot_positions: Vec<BlockPos>,
    }

    impl MockLevel {
        fn new(default_state: BlockState, min_y: i32) -> Self {
            Self {
                blocks: HashMap::new(),
                default_state,
                min_y,
                spawner_entity: None,
                chest_loot_positions: Vec::new(),
            }
        }

        fn set(&mut self, pos: BlockPos, state: BlockState) {
            self.blocks.insert(pos, state);
        }
    }

    impl MonsterRoomLevel<SeqRandom> for MockLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.blocks.get(&pos).copied().unwrap_or(self.default_state)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) {
            self.blocks.insert(pos, state);
        }

        fn is_empty_block(&self, pos: BlockPos) -> bool {
            matches!(self.get_block_state(pos).kind, BlockKind::Air | BlockKind::CaveAir)
        }

        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn try_set_spawner_entity(
            &mut self,
            pos: BlockPos,
            entity: EntityType,
            _random: &mut SeqRandom,
        ) -> bool {
            if self.get_block_state(pos).kind == BlockKind::Spawner {
                self.spawner_entity = Some((pos, entity));
                true
            } else {
                false
            }
        }

        fn set_chest_loot_table(
            &mut self,
            pos: BlockPos,
            _random: &mut SeqRandom,
            _loot_table: LootTable,
        ) {
            self.chest_loot_positions.push(pos);
        }
    }

    fn solid_stone() -> BlockState {
        BlockState::new(BlockKind::Other, true, false)
    }

    #[test]
    fn block_pos_offset_wraps_and_negative_inputs() {
        let pos = BlockPos::new(i32::MIN, -1, i32::MAX);
        let off = pos.offset(-1, -1, 1);
        assert_eq!(off.get_x(), i32::MAX);
        assert_eq!(off.get_y(), -2);
        assert_eq!(off.get_z(), i32::MIN);
        assert_eq!(pos.above(), BlockPos::new(i32::MIN, 0, i32::MAX));
        assert_eq!(pos.below(), BlockPos::new(i32::MIN, -2, i32::MAX));
    }

    #[test]
    fn place_fails_when_floor_not_solid() {
        let mut level = MockLevel::new(solid_stone(), 0);
        let origin = BlockPos::new(0, 64, 0);
        // Make one floor cell non-solid.
        level.set(origin.offset(0, -1, 0), BlockState::cave_air());
        let mut random = SeqRandom::new(vec![0, 0]);
        let mut ctx = FeaturePlaceContext {
            origin,
            random: &mut random,
            level: &mut level,
        };
        assert!(!MonsterRoomFeature::new().place(&mut ctx));
    }

    #[test]
    fn place_fails_when_openings_out_of_range() {
        let mut level = MockLevel::new(solid_stone(), 0);
        let origin = BlockPos::new(10, 64, -20);
        // All solid, no empty border openings at dy==0 -> openings == 0.
        let mut random = SeqRandom::new(vec![0, 0]);
        let mut ctx = FeaturePlaceContext {
            origin,
            random: &mut random,
            level: &mut level,
        };
        assert!(!MonsterRoomFeature::new().place(&mut ctx));
    }

    #[test]
    fn place_succeeds_sets_spawner_and_handles_wrapping_coords() {
        let mut level = MockLevel::new(solid_stone(), i32::MIN);
        let origin = BlockPos::new(-5, 10, 7);
        // size_x = next_int(2)+2 = 2, size_z = 2 => min=-3,max=3
        // Create exactly one valid opening on border at dx=min_x, dy=0, dz=0.
        let opening = origin.offset(-3, 0, 0);
        level.set(opening, BlockState::cave_air());
        level.set(opening.above(), BlockState::cave_air());

        // Random sequence: size_x, size_z, chest placement attempts (many zeros), randomEntityId
        let mut values = vec![0, 0];
        values.extend(std::iter::repeat(0).take(32));
        // randomEntityId index for MOBS len 4 -> 0 => Skeleton
        values.push(0);
        let mut random = SeqRandom::new(values);

        let mut ctx = FeaturePlaceContext {
            origin,
            random: &mut random,
            level: &mut level,
        };
        let ok = MonsterRoomFeature::new().place(&mut ctx);
        assert!(ok);
        assert_eq!(
            ctx.level.get_block_state(origin).kind,
            BlockKind::Spawner
        );
        assert_eq!(ctx.level.spawner_entity, Some((origin, EntityType::Skeleton)));
    }

    #[test]
    fn random_entity_id_and_float_bits_edge_cases() {
        let feature = MonsterRoomFeature::new();
        let mut random = SeqRandom::new(vec![0, 1, 2, 3, 4, -1]);
        assert_eq!(feature.random_entity_id(&mut random), EntityType::Skeleton);
        assert_eq!(feature.random_entity_id(&mut random), EntityType::Zombie);
        assert_eq!(feature.random_entity_id(&mut random), EntityType::Zombie);
        assert_eq!(feature.random_entity_id(&mut random), EntityType::Spider);
        assert_eq!(feature.random_entity_id(&mut random), EntityType::Skeleton);
        assert_eq!(feature.random_entity_id(&mut random), EntityType::Spider);

        // No float ops in MonsterRoomFeature; verify float bit-pattern edge awareness.
        assert_eq!(f32::NAN.to_bits() & 0x7fc0_0000, 0x7fc0_0000);
        assert_ne!(f32::to_bits(-0.0_f32), f32::to_bits(0.0_f32));
        assert_eq!(f64::INFINITY.to_bits(), 0x7ff0_0000_0000_0000_u64);
    }
}
