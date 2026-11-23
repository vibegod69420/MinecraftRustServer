use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#above()
    pub fn above(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#below()
    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#relative(net.minecraft.core.Direction)
    pub fn relative(&self, dir: Direction) -> Self {
        let n = dir.normal();
        Self {
            x: self.x.wrapping_add(n.x),
            y: self.y.wrapping_add(n.y),
            z: self.z.wrapping_add(n.z),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    // JAVA: net.minecraft.core.Direction#getOpposite()
    pub fn opposite(self) -> Self {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    // JAVA: net.minecraft.core.Direction#getNormal()
    pub fn normal(self) -> Vec3i {
        match self {
            Direction::Down => Vec3i { x: 0, y: -1, z: 0 },
            Direction::Up => Vec3i { x: 0, y: 1, z: 0 },
            Direction::North => Vec3i { x: 0, y: 0, z: -1 },
            Direction::South => Vec3i { x: 0, y: 0, z: 1 },
            Direction::West => Vec3i { x: -1, y: 0, z: 0 },
            Direction::East => Vec3i { x: 1, y: 0, z: 0 },
        }
    }

    // JAVA: net.minecraft.core.Direction$Plane#HORIZONTAL
    pub fn horizontal() -> [Direction; 4] {
        [Direction::North, Direction::South, Direction::West, Direction::East]
    }

    // JAVA: net.minecraft.core.Direction#getRandom(net.minecraft.util.RandomSource)
    pub fn get_random(random: &mut dyn RandomSource) -> Direction {
        Self::values()[random.next_int(6) as usize]
    }

    // JAVA: net.minecraft.core.Direction#values()
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    pub const AIR: BlockState = BlockState { id: 0 };
    pub const WATER: BlockState = BlockState { id: 1 };
    pub const DRIPSTONE_BLOCK: BlockState = BlockState { id: 2 };
    pub const POINTED_DRIPSTONE_BASE: BlockState = BlockState { id: 3 };
}

#[derive(Clone, Copy, Debug)]
pub struct PointedDripstoneConfiguration {
    pub chance_of_taller_dripstone: f32,
    pub chance_of_directional_spread: f32,
    pub chance_of_spread_radius2: f32,
    pub chance_of_spread_radius3: f32,
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_boolean(&mut self) -> bool;
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait LevelAccessor {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
    fn is_dripstone_base(&self, state: BlockState) -> bool;
    fn is_empty_or_water(&self, state: BlockState) -> bool;
}

// JAVA: net.minecraft.world.level.levelgen.feature.DripstoneUtils#placeDripstoneBlockIfPossible(net.minecraft.world.level.LevelAccessor, net.minecraft.core.BlockPos)
pub fn place_dripstone_block_if_possible<L: LevelAccessor + ?Sized>(level: &mut L, pos: BlockPos) {
    let state = level.get_block_state(pos);
    if level.is_dripstone_base(state) || level.is_empty_or_water(state) {
        level.set_block(pos, BlockState::DRIPSTONE_BLOCK, 2);
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.DripstoneUtils#growPointedDripstone(net.minecraft.world.level.LevelAccessor, net.minecraft.core.BlockPos, net.minecraft.core.Direction, int, boolean)
pub fn grow_pointed_dripstone<L: LevelAccessor + ?Sized>(
    level: &mut L,
    pos: BlockPos,
    direction: Direction,
    height: i32,
    _merge_tip: bool,
) {
    let mut cur = pos;
    let mut i = 0;
    while i < height {
        level.set_block(cur, BlockState::POINTED_DRIPSTONE_BASE, 2);
        cur = cur.relative(direction);
        i = i.wrapping_add(1);
    }
}

pub struct PointedDripstoneFeature;

impl PointedDripstoneFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.PointedDripstoneFeature#place(net.minecraft.world.level.levelgen.feature.FeaturePlaceContext)
    pub fn place<L: LevelAccessor + ?Sized>(
        level: &mut L,
        origin: BlockPos,
        random: &mut dyn RandomSource,
        config: PointedDripstoneConfiguration,
    ) -> bool {
        let tip_dir = Self::get_tip_direction(level, origin, random);
        let Some(tip_dir) = tip_dir else {
            return false;
        };

        let patch_pos = origin.relative(tip_dir.opposite());
        Self::create_patch_of_dripstone_blocks(level, random, patch_pos, config);
        let taller_roll = random.next_float();
        let tip_state = level.get_block_state(origin.relative(tip_dir));
        let height = if taller_roll < config.chance_of_taller_dripstone
            && level.is_empty_or_water(tip_state)
        {
            2
        } else {
            1
        };
        grow_pointed_dripstone(level, origin, tip_dir, height, false);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.PointedDripstoneFeature#getTipDirection(net.minecraft.world.level.LevelAccessor, net.minecraft.core.BlockPos, net.minecraft.util.RandomSource)
    fn get_tip_direction<L: LevelAccessor + ?Sized>(
        level: &L,
        pos: BlockPos,
        random: &mut dyn RandomSource,
    ) -> Option<Direction> {
        let above_is_base = level.is_dripstone_base(level.get_block_state(pos.above()));
        let below_is_base = level.is_dripstone_base(level.get_block_state(pos.below()));
        if above_is_base && below_is_base {
            Some(if random.next_boolean() {
                Direction::Down
            } else {
                Direction::Up
            })
        } else if above_is_base {
            Some(Direction::Down)
        } else if below_is_base {
            Some(Direction::Up)
        } else {
            None
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.PointedDripstoneFeature#createPatchOfDripstoneBlocks(net.minecraft.world.level.LevelAccessor, net.minecraft.util.RandomSource, net.minecraft.core.BlockPos, net.minecraft.world.level.levelgen.feature.configurations.PointedDripstoneConfiguration)
    fn create_patch_of_dripstone_blocks<L: LevelAccessor + ?Sized>(
        level: &mut L,
        random: &mut dyn RandomSource,
        pos: BlockPos,
        config: PointedDripstoneConfiguration,
    ) {
        place_dripstone_block_if_possible(level, pos);

        for dir in Direction::horizontal() {
            if !(random.next_float() > config.chance_of_directional_spread) {
                let pos2 = pos.relative(dir);
                place_dripstone_block_if_possible(level, pos2);
                if !(random.next_float() > config.chance_of_spread_radius2) {
                    let pos3 = pos2.relative(Direction::get_random(random));
                    place_dripstone_block_if_possible(level, pos3);
                    if !(random.next_float() > config.chance_of_spread_radius3) {
                        let pos4 = pos3.relative(Direction::get_random(random));
                        place_dripstone_block_if_possible(level, pos4);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRandom {
        floats: Vec<f32>,
        bools: Vec<bool>,
        ints: Vec<i32>,
        f_idx: usize,
        b_idx: usize,
        i_idx: usize,
    }

    impl TestRandom {
        fn new(floats: Vec<f32>, bools: Vec<bool>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                bools,
                ints,
                f_idx: 0,
                b_idx: 0,
                i_idx: 0,
            }
        }
    }

    impl RandomSource for TestRandom {
        fn next_float(&mut self) -> f32 {
            let v = if self.f_idx < self.floats.len() {
                let out = self.floats[self.f_idx];
                self.f_idx += 1;
                out
            } else {
                1.0
            };
            v
        }

        fn next_boolean(&mut self) -> bool {
            let v = if self.b_idx < self.bools.len() {
                let out = self.bools[self.b_idx];
                self.b_idx += 1;
                out
            } else {
                false
            };
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.i_idx < self.ints.len() {
                let out = self.ints[self.i_idx];
                self.i_idx += 1;
                out
            } else {
                0
            };
            v.rem_euclid(bound)
        }
    }

    #[derive(Default)]
    struct TestLevel {
        states: HashMap<(i32, i32, i32), BlockState>,
        dripstone_bases: HashMap<(i32, i32, i32), bool>,
        placed: Vec<(BlockPos, BlockState, i32)>,
    }

    impl TestLevel {
        fn set_state(&mut self, pos: BlockPos, state: BlockState) {
            self.states.insert((pos.x, pos.y, pos.z), state);
        }

        fn set_dripstone_base(&mut self, pos: BlockPos, v: bool) {
            self.dripstone_bases.insert((pos.x, pos.y, pos.z), v);
        }
    }

    impl LevelAccessor for TestLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            *self
                .states
                .get(&(pos.x, pos.y, pos.z))
                .unwrap_or(&BlockState::AIR)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.states.insert((pos.x, pos.y, pos.z), state);
            self.placed.push((pos, state, flags));
            true
        }

        fn is_dripstone_base(&self, state: BlockState) -> bool {
            if state == BlockState::DRIPSTONE_BLOCK {
                return true;
            }
            // Allow tests to mark arbitrary positions/states as base via state id lookup.
            // For simplicity, treat POINTED_DRIPSTONE_BASE as base too.
            state == BlockState::POINTED_DRIPSTONE_BASE
                || self
                    .dripstone_bases
                    .values()
                    .copied()
                    .any(|v| v && state.id == 99)
        }

        fn is_empty_or_water(&self, state: BlockState) -> bool {
            state == BlockState::AIR || state == BlockState::WATER
        }
    }

    #[test]
    fn get_tip_direction_prefers_both_with_random() {
        let mut level = TestLevel::default();
        let pos = BlockPos::new(0, 5, 0);
        level.set_state(pos.above(), BlockState::DRIPSTONE_BLOCK);
        level.set_state(pos.below(), BlockState::DRIPSTONE_BLOCK);
        let mut rng = TestRandom::new(vec![], vec![true], vec![]);
        let dir = PointedDripstoneFeature::get_tip_direction(&level, pos, &mut rng);
        assert_eq!(dir, Some(Direction::Down));
    }

    #[test]
    fn get_tip_direction_none_when_no_base() {
        let level = TestLevel::default();
        let mut rng = TestRandom::new(vec![], vec![], vec![]);
        let dir = PointedDripstoneFeature::get_tip_direction(&level, BlockPos::new(0, 0, 0), &mut rng);
        assert_eq!(dir, None);
    }

    #[test]
    fn place_returns_false_without_tip_direction() {
        let mut level = TestLevel::default();
        let mut rng = TestRandom::new(vec![], vec![], vec![]);
        let cfg = PointedDripstoneConfiguration {
            chance_of_taller_dripstone: 1.0,
            chance_of_directional_spread: 0.0,
            chance_of_spread_radius2: 0.0,
            chance_of_spread_radius3: 0.0,
        };
        assert!(!PointedDripstoneFeature::place(&mut level, BlockPos::new(0, 0, 0), &mut rng, cfg));
    }

    #[test]
    fn place_grows_taller_when_chance_and_space() {
        let mut level = TestLevel::default();
        let origin = BlockPos::new(0, 5, 0);
        level.set_state(origin.above(), BlockState::DRIPSTONE_BLOCK); // tip dir DOWN
        level.set_state(origin.below(), BlockState::AIR); // space for taller growth
        let mut rng = TestRandom::new(vec![0.0; 16], vec![], vec![0; 16]);
        let cfg = PointedDripstoneConfiguration {
            chance_of_taller_dripstone: 1.0,
            chance_of_directional_spread: 0.0,
            chance_of_spread_radius2: 0.0,
            chance_of_spread_radius3: 0.0,
        };
        let ok = PointedDripstoneFeature::place(&mut level, origin, &mut rng, cfg);
        assert!(ok);
        let tip_count = level
            .placed
            .iter()
            .filter(|(_, s, _)| *s == BlockState::POINTED_DRIPSTONE_BASE)
            .count();
        assert_eq!(tip_count, 2);
    }

    #[test]
    fn create_patch_places_dripstone_block() {
        let mut level = TestLevel::default();
        let pos = BlockPos::new(1, 2, 3);
        level.set_state(pos, BlockState::AIR);
        let mut rng = TestRandom::new(vec![1.0; 16], vec![], vec![0; 16]);
        let cfg = PointedDripstoneConfiguration {
            chance_of_taller_dripstone: 0.0,
            chance_of_directional_spread: 0.0,
            chance_of_spread_radius2: 0.0,
            chance_of_spread_radius3: 0.0,
        };
        PointedDripstoneFeature::create_patch_of_dripstone_blocks(&mut level, &mut rng, pos, cfg);
        assert!(level.placed.iter().any(|(p, s, _)| *p == pos && *s == BlockState::DRIPSTONE_BLOCK));
    }
}
