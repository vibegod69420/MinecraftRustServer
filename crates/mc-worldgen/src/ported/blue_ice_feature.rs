use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    Water,
    PackedIce,
    BlueIce,
    Ice,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
}

impl BlockState {
    pub const fn new(block: Block) -> Self {
        Self { block }
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#is
    pub fn is(&self, block: Block) -> bool {
        self.block == block
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#isAir
    pub fn is_air(&self) -> bool {
        self.block == Block::Air
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const VALUES: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    pub const fn normal(self) -> (i32, i32, i32) {
        match self {
            Direction::Down => (0, -1, 0),
            Direction::Up => (0, 1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
        }
    }
}

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

    // JAVA: net.minecraft.core.BlockPos#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> Self {
        self.offset(0, -1, 0)
    }

    // JAVA: net.minecraft.core.BlockPos#relative
    pub fn relative(&self, dir: Direction) -> Self {
        let (dx, dy, dz) = dir.normal();
        self.offset(dx, dy, dz)
    }

    // JAVA: net.minecraft.core.BlockPos#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait WorldGenLevel {
    fn get_sea_level(&self) -> i32;
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub struct NoneFeatureConfiguration;

pub struct FeaturePlaceContext<'a, L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized> {
    pub origin: BlockPos,
    pub level: &'a mut L,
    pub random: &'a mut R,
    _config: PhantomData<NoneFeatureConfiguration>,
}

impl<'a, L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized> FeaturePlaceContext<'a, L, R> {
    pub fn new(origin: BlockPos, level: &'a mut L, random: &'a mut R) -> Self {
        Self {
            origin,
            level,
            random,
            _config: PhantomData,
        }
    }
}

pub struct BlueIceFeature;

impl BlueIceFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.BlueIceFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BlueIceFeature#place
    pub fn place<L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let origin = ctx.origin;
        let level = &mut *ctx.level;
        let random = &mut *ctx.random;

        if origin.get_y() > level.get_sea_level() - 1 {
            return false;
        }

        if !level.get_block_state(&origin).is(Block::Water)
            && !level.get_block_state(&origin.below()).is(Block::Water)
        {
            return false;
        }

        let mut found_packed_ice = false;
        for dir in Direction::VALUES {
            if dir != Direction::Down
                && level
                    .get_block_state(&origin.relative(dir))
                    .is(Block::PackedIce)
            {
                found_packed_ice = true;
                break;
            }
        }

        if !found_packed_ice {
            return false;
        }

        level.set_block(origin, BlockState::new(Block::BlueIce), 2);

        for _ in 0..200 {
            let dy = random.next_int(5) - random.next_int(6);
            let mut radius = 3i32;
            if dy < 2 {
                radius += dy / 2;
            }

            if radius >= 1 {
                let offset_pos = origin.offset(
                    random.next_int(radius) - random.next_int(radius),
                    dy,
                    random.next_int(radius) - random.next_int(radius),
                );
                let state = level.get_block_state(&offset_pos);
                if state.is_air()
                    || state.is(Block::Water)
                    || state.is(Block::PackedIce)
                    || state.is(Block::Ice)
                {
                    for dir in Direction::VALUES {
                        let neighbor_state = level.get_block_state(&offset_pos.relative(dir));
                        if neighbor_state.is(Block::BlueIce) {
                            level.set_block(offset_pos, BlockState::new(Block::BlueIce), 2);
                            break;
                        }
                    }
                }
            }
        }

        true
    }
}

impl Default for BlueIceFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        sea_level: i32,
        blocks: HashMap<BlockPos, BlockState>,
    }

    impl MockLevel {
        fn new(sea_level: i32) -> Self {
            Self {
                sea_level,
                blocks: HashMap::new(),
            }
        }

        fn set(&mut self, pos: BlockPos, block: Block) {
            self.blocks.insert(pos, BlockState::new(block));
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_sea_level(&self) -> i32 {
            self.sea_level
        }

        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.blocks
                .get(pos)
                .copied()
                .unwrap_or(BlockState::new(Block::Air))
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert(pos, state);
            true
        }
    }

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

    #[test]
    fn fails_when_above_sea_level() {
        let mut level = MockLevel::new(63);
        let pos = BlockPos::new(0, 63, 0);
        level.set(pos, Block::Water);
        level.set(pos.below(), Block::Water);
        let mut rng = SeqRandom::new(vec![]);
        let mut ctx = FeaturePlaceContext::new(pos, &mut level, &mut rng);
        assert!(!BlueIceFeature::new().place(&mut ctx));
    }

    #[test]
    fn fails_when_no_water_at_origin_or_below() {
        let mut level = MockLevel::new(63);
        let pos = BlockPos::new(0, 10, 0);
        level.set(pos, Block::Other);
        level.set(pos.below(), Block::Other);
        let mut rng = SeqRandom::new(vec![]);
        let mut ctx = FeaturePlaceContext::new(pos, &mut level, &mut rng);
        assert!(!BlueIceFeature::new().place(&mut ctx));
    }

    #[test]
    fn fails_when_no_packed_ice_neighbor() {
        let mut level = MockLevel::new(63);
        let pos = BlockPos::new(0, 10, 0);
        level.set(pos, Block::Water);
        let mut rng = SeqRandom::new(vec![]);
        let mut ctx = FeaturePlaceContext::new(pos, &mut level, &mut rng);
        assert!(!BlueIceFeature::new().place(&mut ctx));
    }

    #[test]
    fn places_blue_ice_when_valid_and_accepts_water_below() {
        let mut level = MockLevel::new(63);
        let pos = BlockPos::new(0, 62, 0); // at sea level - 1 boundary
        level.set(pos, Block::Air);
        level.set(pos.below(), Block::Water);
        level.set(pos.relative(Direction::East), Block::PackedIce);
        let mut rng = SeqRandom::new(vec![0; 4000]);
        let mut ctx = FeaturePlaceContext::new(pos, &mut level, &mut rng);
        assert!(BlueIceFeature::new().place(&mut ctx));
        assert_eq!(level.get_block_state(&pos).block, Block::BlueIce);
    }

    #[test]
    fn spreads_to_adjacent_water_near_blue_ice() {
        let mut level = MockLevel::new(63);
        let pos = BlockPos::new(0, 10, 0);
        level.set(pos, Block::Water);
        level.set(pos.relative(Direction::North), Block::PackedIce);
        // Place a water block at offset (1,0,0) from origin; with dy=0, radius=3,
        // randoms: nextInt(5)=0, nextInt(6)=0 => dy=0; nextInt(3)=1,nextInt(3)=0 => dx=1;
        // nextInt(3)=0,nextInt(3)=0 => dz=0
        let target = pos.offset(1, 0, 0);
        level.set(target, Block::Water);
        // Make target adjacent to origin which becomes blue ice, so it can convert.
        let mut seq = Vec::new();
        // First iteration: dy=0, dx=1, dz=0
        seq.extend([0, 0, 1, 0, 0, 0]);
        // Remaining iterations: all zeros
        seq.extend(std::iter::repeat(0).take(4000));
        let mut rng = SeqRandom::new(seq);
        let mut ctx = FeaturePlaceContext::new(pos, &mut level, &mut rng);
        assert!(BlueIceFeature::new().place(&mut ctx));
        assert_eq!(level.get_block_state(&target).block, Block::BlueIce);
    }

    #[test]
    fn block_pos_offset_wraps_on_overflow() {
        let pos = BlockPos::new(i32::MAX, i32::MIN, 0);
        let off = pos.offset(1, -1, 5);
        assert_eq!(off.x, i32::MIN);
        assert_eq!(off.y, i32::MAX);
        assert_eq!(off.z, 5);
        let below_min = BlockPos::new(0, i32::MIN, 0).below();
        assert_eq!(below_min.y, i32::MAX);
    }
}
