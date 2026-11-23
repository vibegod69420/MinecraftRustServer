// Self-contained port of net.minecraft.world.level.levelgen.feature.TwistingVinesFeature

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: BlockPos = BlockPos { x: 0, y: 0, z: 0 };

    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    #[inline]
    pub fn above(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn set(&mut self, pos: BlockPos) -> &mut Self {
        self.x = pos.x;
        self.y = pos.y;
        self.z = pos.z;
        self
    }

    #[inline]
    pub fn move_by(&mut self, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = self.x.wrapping_add(dx);
        self.y = self.y.wrapping_add(dy);
        self.z = self.z.wrapping_add(dz);
        self
    }

    #[inline]
    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        self.x = self.x.wrapping_add(dir.step_x());
        self.y = self.y.wrapping_add(dir.step_y());
        self.z = self.z.wrapping_add(dir.step_z());
        self
    }

    #[inline]
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    #[inline]
    pub fn above(&self) -> BlockPos {
        BlockPos::new(self.x, self.y.wrapping_add(1), self.z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    North,
    South,
    West,
    East,
}

impl Direction {
    #[inline]
    pub const fn step_x(self) -> i32 {
        match self {
            Direction::East => 1,
            Direction::West => -1,
            _ => 0,
        }
    }

    #[inline]
    pub const fn step_y(self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
            _ => 0,
        }
    }

    #[inline]
    pub const fn step_z(self) -> i32 {
        match self {
            Direction::South => 1,
            Direction::North => -1,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Netherrack,
    WarpedNylium,
    WarpedWartBlock,
    TwistingVines,
    TwistingVinesPlant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
    pub age: i32,
}

impl BlockState {
    #[inline]
    pub const fn new(block: Block) -> Self {
        Self { block, age: 0 }
    }

    #[inline]
    pub const fn twisting_vines(age: i32) -> Self {
        Self {
            block: Block::TwistingVines,
            age,
        }
    }

    #[inline]
    pub const fn twisting_vines_plant() -> Self {
        Self {
            block: Block::TwistingVinesPlant,
            age: 0,
        }
    }

    #[inline]
    pub fn is_air(&self) -> bool {
        self.block == Block::Air
    }

    #[inline]
    pub fn is_block(&self, block: Block) -> bool {
        self.block == block
    }
}

impl fmt::Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}(age={})", self.block, self.age)
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub struct Mth;

impl Mth {
    // JAVA: net.minecraft.util.Mth#nextInt
    pub fn next_int<R: RandomSource + ?Sized>(random: &mut R, min: i32, max: i32) -> i32 {
        if min >= max {
            min
        } else {
            random
                .next_int(max.wrapping_sub(min).wrapping_add(1))
                .wrapping_add(min)
        }
    }
}

pub trait LevelAccessor {
    fn is_outside_build_height(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TwistingVinesConfig {
    pub spread_width: i32,
    pub spread_height: i32,
    pub max_height: i32,
}

pub struct FeaturePlaceContext<'a, L: ?Sized, R: ?Sized> {
    pub level: &'a mut L,
    pub origin: BlockPos,
    pub random: &'a mut R,
    pub config: &'a TwistingVinesConfig,
}

pub struct TwistingVinesFeature;

impl TwistingVinesFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.TwistingVinesFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.TwistingVinesFeature#place
    pub fn place<L: LevelAccessor + ?Sized, R: RandomSource + ?Sized>(
        &self,
        mut ctx: FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let level = &mut *ctx.level;
        let origin = ctx.origin;
        let random = &mut *ctx.random;
        let config = ctx.config;

        if is_invalid_placement_location(level, origin) {
            return false;
        }

        let spread_width = config.spread_width;
        let spread_height = config.spread_height;
        let max_height = config.max_height;
        let mut pos = MutableBlockPos::new(0, 0, 0);

        let mut i: i32 = 0;
        let limit = spread_width.wrapping_mul(spread_width);
        while i < limit {
            pos.set(origin);
            pos.move_by(
                Mth::next_int(random, -spread_width, spread_width),
                Mth::next_int(random, -spread_height, spread_height),
                Mth::next_int(random, -spread_width, spread_width),
            );
            if find_first_air_block_above_ground(level, &mut pos)
                && !is_invalid_placement_location(level, pos.to_block_pos())
            {
                let mut height = Mth::next_int(random, 1, max_height);
                if random.next_int(6) == 0 {
                    height = height.wrapping_mul(2);
                }
                if random.next_int(5) == 0 {
                    height = 1;
                }
                place_weeping_vines_column(level, random, &mut pos, height, 17, 25);
            }
            i = i.wrapping_add(1);
        }

        true
    }
}

impl Default for TwistingVinesFeature {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.TwistingVinesFeature#findFirstAirBlockAboveGround
fn find_first_air_block_above_ground<L: LevelAccessor + ?Sized>(
    level: &L,
    pos: &mut MutableBlockPos,
) -> bool {
    loop {
        pos.move_by(0, -1, 0);
        if level.is_outside_build_height(pos.to_block_pos()) {
            return false;
        }
        if !level.get_block_state(pos.to_block_pos()).is_air() {
            break;
        }
    }
    pos.move_by(0, 1, 0);
    true
}

// JAVA: net.minecraft.world.level.levelgen.feature.TwistingVinesFeature#placeWeepingVinesColumn
pub fn place_weeping_vines_column<L: LevelAccessor + ?Sized, R: RandomSource + ?Sized>(
    level: &mut L,
    random: &mut R,
    pos: &mut MutableBlockPos,
    height: i32,
    min_age: i32,
    max_age: i32,
) {
    let mut i: i32 = 1;
    while i <= height {
        if level.is_empty_block(pos.to_block_pos()) {
            if i == height || !level.is_empty_block(pos.above()) {
                let age = Mth::next_int(random, min_age, max_age);
                level.set_block(pos.to_block_pos(), BlockState::twisting_vines(age), 2);
                break;
            }
            level.set_block(pos.to_block_pos(), BlockState::twisting_vines_plant(), 2);
        }
        pos.move_dir(Direction::Up);
        i = i.wrapping_add(1);
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.TwistingVinesFeature#isInvalidPlacementLocation
fn is_invalid_placement_location<L: LevelAccessor + ?Sized>(level: &L, pos: BlockPos) -> bool {
    if !level.is_empty_block(pos) {
        return true;
    }
    let below_state = level.get_block_state(pos.below());
    !below_state.is_block(Block::Netherrack)
        && !below_state.is_block(Block::WarpedNylium)
        && !below_state.is_block(Block::WarpedWartBlock)
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
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    struct MockLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
        min_y: i32,
        max_y: i32,
    }

    impl MockLevel {
        fn new(min_y: i32, max_y: i32) -> Self {
            Self {
                blocks: HashMap::new(),
                min_y,
                max_y,
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, block: Block) {
            self.blocks
                .insert((x, y, z), BlockState::new(block));
        }

        fn get(&self, x: i32, y: i32, z: i32) -> BlockState {
            self.blocks
                .get(&(x, y, z))
                .copied()
                .unwrap_or(BlockState::new(Block::Air))
        }
    }

    impl LevelAccessor for MockLevel {
        fn is_outside_build_height(&self, pos: BlockPos) -> bool {
            pos.y < self.min_y || pos.y > self.max_y
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.get(pos.x, pos.y, pos.z)
        }

        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.get_block_state(pos).is_air()
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            true
        }
    }

    #[test]
    fn test_invalid_placement_location() {
        let mut level = MockLevel::new(-64, 320);
        let pos = BlockPos::new(0, 10, 0);
        // No ground below -> invalid
        assert!(is_invalid_placement_location(&level, pos));

        level.set(0, 9, 0, Block::Netherrack);
        assert!(!is_invalid_placement_location(&level, pos));

        level.set(0, 9, 0, Block::WarpedNylium);
        assert!(!is_invalid_placement_location(&level, pos));

        level.set(0, 9, 0, Block::WarpedWartBlock);
        assert!(!is_invalid_placement_location(&level, pos));

        level.set(0, 9, 0, Block::TwistingVinesPlant);
        assert!(is_invalid_placement_location(&level, pos));

        level.set(0, 9, 0, Block::Netherrack);
        level.set(0, 10, 0, Block::TwistingVines);
        assert!(is_invalid_placement_location(&level, pos));
    }

    #[test]
    fn test_find_first_air_block_above_ground() {
        let mut level = MockLevel::new(0, 32);
        level.set(0, 5, 0, Block::Netherrack);
        let mut pos = MutableBlockPos::new(0, 10, 0);
        assert!(find_first_air_block_above_ground(&level, &mut pos));
        assert_eq!(pos.to_block_pos(), BlockPos::new(0, 6, 0));

        // Outside build height
        let mut pos2 = MutableBlockPos::new(1, 1, 1);
        assert!(!find_first_air_block_above_ground(&level, &mut pos2));
    }

    #[test]
    fn test_place_weeping_vines_column_head_and_plant() {
        let mut level = MockLevel::new(0, 32);
        // Solid block above height 3 so column stops at 3
        level.set(0, 4, 0, Block::Netherrack);
        let mut pos = MutableBlockPos::new(0, 1, 0);
        let mut rng = SeqRandom::new(vec![5, 0, 0, 0, 0]);
        place_weeping_vines_column(&mut level, &mut rng, &mut pos, 3, 17, 25);

        assert_eq!(level.get(0, 1, 0).block, Block::TwistingVinesPlant);
        assert_eq!(level.get(0, 2, 0).block, Block::TwistingVinesPlant);
        assert_eq!(level.get(0, 3, 0).block, Block::TwistingVines);
        let age = level.get(0, 3, 0).age;
        assert!(age >= 17 && age <= 25);
    }

    #[test]
    fn test_mth_next_int_edge_cases() {
        let mut rng = SeqRandom::new(vec![0, 1, 2]);
        assert_eq!(Mth::next_int(&mut rng, 5, 5), 5);
        assert_eq!(Mth::next_int(&mut rng, 10, 5), 10);
        let v = Mth::next_int(&mut rng, -3, 3);
        assert!(v >= -3 && v <= 3);
    }

    #[test]
    fn test_place_returns_false_on_invalid_origin() {
        let mut level = MockLevel::new(0, 32);
        // origin empty but below is air -> invalid
        let mut rng = SeqRandom::new(vec![0]);
        let config = TwistingVinesConfig {
            spread_width: 1,
            spread_height: 1,
            max_height: 5,
        };
        let feature = TwistingVinesFeature::new();
        let placed = feature.place(FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(0, 10, 0),
            random: &mut rng,
            config: &config,
        });
        assert!(!placed);
    }

    #[test]
    fn test_place_with_negative_and_wrapping_inputs() {
        let mut level = MockLevel::new(-64, 320);
        // Make a valid ground at origin
        level.set(0, 9, 0, Block::Netherrack);
        // Also provide ground around for attempts
        for x in -2..=2 {
            for z in -2..=2 {
                level.set(x, 9, z, Block::Netherrack);
            }
        }
        let mut rng = SeqRandom::new(vec![1, 2, 3, 4, 5, 0, 1, 2]);
        let config = TwistingVinesConfig {
            spread_width: 2,
            spread_height: 1,
            max_height: 3,
        };
        let feature = TwistingVinesFeature::new();
        let placed = feature.place(FeaturePlaceContext {
            level: &mut level,
            origin: BlockPos::new(0, 10, 0),
            random: &mut rng,
            config: &config,
        });
        assert!(placed);
        // Should have placed at least some twisting vines blocks
        let has_vines = level
            .blocks
            .values()
            .any(|s| s.block == Block::TwistingVines || s.block == Block::TwistingVinesPlant);
        assert!(has_vines);
    }
}
