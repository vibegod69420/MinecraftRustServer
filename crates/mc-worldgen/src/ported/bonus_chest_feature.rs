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

    // JAVA: net.minecraft.core.Direction$Plane#HORIZONTAL (iterator order)
    pub fn horizontal() -> [Direction; 4] {
        [Direction::North, Direction::South, Direction::West, Direction::East]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#<init>(net.minecraft.core.BlockPos)
    pub fn from_block_pos(pos: BlockPos) -> Self {
        Self {
            x: pos.x >> 4,
            z: pos.z >> 4,
        }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockX()
    pub fn get_min_block_x(self) -> i32 {
        self.x << 4
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMaxBlockX()
    pub fn get_max_block_x(self) -> i32 {
        (self.x << 4).wrapping_add(15)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockZ()
    pub fn get_min_block_z(self) -> i32 {
        self.z << 4
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMaxBlockZ()
    pub fn get_max_block_z(self) -> i32 {
        (self.z << 4).wrapping_add(15)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    MotionBlockingNoLeaves,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    pub const AIR: BlockState = BlockState { id: 0 };
    pub const CHEST: BlockState = BlockState { id: 1 };
    pub const TORCH: BlockState = BlockState { id: 2 };
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait BonusChestLevel {
    fn get_heightmap_pos(&self, ty: HeightmapType, pos: BlockPos) -> BlockPos;
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn block_state_collision_shape_is_empty(&self, state: BlockState, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
    fn set_block_entity_loot_table(&mut self, pos: BlockPos);
    fn can_survive(&self, state: BlockState, pos: BlockPos) -> bool;
}

// JAVA: net.minecraft.Util#toShuffledList(java.util.stream.IntStream, net.minecraft.util.RandomSource)
pub fn to_shuffled_list(mut values: Vec<i32>, random: &mut dyn RandomSource) -> Vec<i32> {
    let mut i = values.len() as i32;
    while i > 1 {
        i = i.wrapping_sub(1);
        let j = random.next_int(i.wrapping_add(1));
        let iu = i as usize;
        let ju = j as usize;
        values.swap(iu, ju);
    }
    values
}

fn range_closed(min: i32, max: i32) -> Vec<i32> {
    let mut out = Vec::new();
    let mut v = min;
    loop {
        out.push(v);
        if v == max {
            break;
        }
        v = v.wrapping_add(1);
    }
    out
}

pub struct BonusChestFeature;

impl BonusChestFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.BonusChestFeature#place(net.minecraft.world.level.levelgen.feature.FeaturePlaceContext)
    pub fn place<L: BonusChestLevel + ?Sized>(
        level: &mut L,
        origin: BlockPos,
        random: &mut dyn RandomSource,
    ) -> bool {
        let chunk_pos = ChunkPos::from_block_pos(origin);
        let x_list = to_shuffled_list(
            range_closed(chunk_pos.get_min_block_x(), chunk_pos.get_max_block_x()),
            random,
        );
        let z_list = to_shuffled_list(
            range_closed(chunk_pos.get_min_block_z(), chunk_pos.get_max_block_z()),
            random,
        );
        let mut mutable = BlockPos::new(0, 0, 0);

        for &x in &x_list {
            for &z in &z_list {
                mutable.x = x;
                mutable.y = 0;
                mutable.z = z;
                let top = level.get_heightmap_pos(HeightmapType::MotionBlockingNoLeaves, mutable);
                let top_state = level.get_block_state(top);
                if level.is_empty_block(top)
                    || level.block_state_collision_shape_is_empty(top_state, top)
                {
                    level.set_block(top, BlockState::CHEST, 2);
                    level.set_block_entity_loot_table(top);
                    let torch_state = BlockState::TORCH;

                    for dir in Direction::horizontal() {
                        let side_pos = top.relative(dir);
                        if level.can_survive(torch_state, side_pos) {
                            level.set_block(side_pos, torch_state, 2);
                        }
                    }

                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl TestRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for TestRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.idx < self.values.len() {
                let out = self.values[self.idx];
                self.idx += 1;
                out
            } else {
                0
            };
            v.rem_euclid(bound)
        }
    }

    #[derive(Default)]
    struct TestLevel {
        heightmap_y: i32,
        empty: HashMap<(i32, i32, i32), bool>,
        collision_empty: HashMap<(i32, i32, i32), bool>,
        survivable: HashMap<(i32, i32, i32), bool>,
        placed: Vec<(BlockPos, BlockState, i32)>,
        loot_tables: Vec<BlockPos>,
    }

    impl TestLevel {
        fn new(heightmap_y: i32) -> Self {
            Self {
                heightmap_y,
                ..Default::default()
            }
        }
    }

    impl BonusChestLevel for TestLevel {
        fn get_heightmap_pos(&self, _ty: HeightmapType, pos: BlockPos) -> BlockPos {
            BlockPos::new(pos.x, self.heightmap_y, pos.z)
        }

        fn is_empty_block(&self, pos: BlockPos) -> bool {
            *self.empty.get(&(pos.x, pos.y, pos.z)).unwrap_or(&false)
        }

        fn get_block_state(&self, _pos: BlockPos) -> BlockState {
            BlockState::AIR
        }

        fn block_state_collision_shape_is_empty(&self, _state: BlockState, pos: BlockPos) -> bool {
            *self
                .collision_empty
                .get(&(pos.x, pos.y, pos.z))
                .unwrap_or(&false)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.placed.push((pos, state, flags));
            true
        }

        fn set_block_entity_loot_table(&mut self, pos: BlockPos) {
            self.loot_tables.push(pos);
        }

        fn can_survive(&self, _state: BlockState, pos: BlockPos) -> bool {
            *self.survivable.get(&(pos.x, pos.y, pos.z)).unwrap_or(&false)
        }
    }

    #[test]
    fn chunk_pos_from_block_pos_negative() {
        let cp = ChunkPos::from_block_pos(BlockPos::new(-1, 64, -16));
        assert_eq!(cp.x, -1);
        assert_eq!(cp.z, -1);
        assert_eq!(cp.get_min_block_x(), -16);
        assert_eq!(cp.get_max_block_x(), -1);
        assert_eq!(cp.get_min_block_z(), -16);
        assert_eq!(cp.get_max_block_z(), -1);
    }

    #[test]
    fn to_shuffled_list_deterministic() {
        let mut rng = TestRandom::new(vec![0, 0, 0, 0]);
        let out = to_shuffled_list(vec![1, 2, 3, 4, 5], &mut rng);
        assert_eq!(out.len(), 5);
        let mut sorted = out.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn place_places_chest_and_surviving_torches() {
        let mut level = TestLevel::new(64);
        let origin = BlockPos::new(0, 0, 0);
        let top = BlockPos::new(0, 64, 0);
        level.empty.insert((top.x, top.y, top.z), true);
        let north = top.relative(Direction::North);
        let east = top.relative(Direction::East);
        level.survivable.insert((north.x, north.y, north.z), true);
        level.survivable.insert((east.x, east.y, east.z), true);

        let mut rng = TestRandom::new(vec![0; 128]);
        let placed = BonusChestFeature::place(&mut level, origin, &mut rng);

        assert!(placed);
        assert_eq!(level.loot_tables.len(), 1);
        assert!(level.placed.iter().any(|(p, s, f)| *p == top && *s == BlockState::CHEST && *f == 2));
        assert!(level.placed.iter().any(|(p, s, f)| *p == north && *s == BlockState::TORCH && *f == 2));
        assert!(level.placed.iter().any(|(p, s, f)| *p == east && *s == BlockState::TORCH && *f == 2));
        assert!(!level.placed.iter().any(|(p, s, _)| *s == BlockState::TORCH && (*p == top.relative(Direction::South) || *p == top.relative(Direction::West))));
    }

    #[test]
    fn place_returns_false_when_no_valid_position() {
        let mut level = TestLevel::new(64);
        let mut rng = TestRandom::new(vec![0; 128]);
        let placed = BonusChestFeature::place(&mut level, BlockPos::new(16, 0, 16), &mut rng);
        assert!(!placed);
        assert!(level.placed.is_empty());
        assert!(level.loot_tables.is_empty());
    }

    #[test]
    fn place_accepts_empty_collision_shape() {
        let mut level = TestLevel::new(70);
        let top = BlockPos::new(0, 70, 0);
        level.collision_empty.insert((top.x, top.y, top.z), true);
        let mut rng = TestRandom::new(vec![0; 128]);
        let placed = BonusChestFeature::place(&mut level, BlockPos::new(0, 0, 0), &mut rng);
        assert!(placed);
        assert_eq!(level.loot_tables[0], top);
    }
}
