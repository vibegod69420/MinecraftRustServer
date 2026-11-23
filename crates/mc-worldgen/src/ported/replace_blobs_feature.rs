#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(III)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.Vec3i#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.Vec3i#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.BlockPos#distManhattan
    pub fn dist_manhattan(&self, other: BlockPos) -> i32 {
        let dx = self.get_x().wrapping_sub(other.get_x()).abs();
        let dy = self.get_y().wrapping_sub(other.get_y()).abs();
        let dz = self.get_z().wrapping_sub(other.get_z()).abs();
        dx.wrapping_add(dy).wrapping_add(dz)
    }

    // JAVA: net.minecraft.core.BlockPos#withinManhattan
    pub fn within_manhattan(center: BlockPos, dx: i32, dy: i32, dz: i32) -> Vec<BlockPos> {
        let mut out = Vec::new();
        let min_x = center.x.wrapping_sub(dx);
        let max_x = center.x.wrapping_add(dx);
        let min_y = center.y.wrapping_sub(dy);
        let max_y = center.y.wrapping_add(dy);
        let min_z = center.z.wrapping_sub(dz);
        let max_z = center.z.wrapping_add(dz);
        let mut x = min_x;
        while x <= max_x {
            let mut y = min_y;
            while y <= max_y {
                let mut z = min_z;
                while z <= max_z {
                    let p = BlockPos::new(x, y, z);
                    if p.dist_manhattan(center) <= dx.wrapping_add(dy).wrapping_add(dz) {
                        // Java withinManhattan iterates by Manhattan distance layers/order;
                        // for behavioral correctness of this feature we include all points in the box
                        // and the feature itself breaks on distManhattan > max radius.
                        out.push(p);
                    }
                    if z == max_z { break; }
                    z = z.wrapping_add(1);
                }
                if y == max_y { break; }
                y = y.wrapping_add(1);
            }
            if x == max_x { break; }
            x = x.wrapping_add(1);
        }
        // Java BlockPos.withinManhattan returns positions ordered by manhattan distance from center.
        out.sort_by_key(|p| p.dist_manhattan(center));
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn as_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#clamp
    pub fn clamp(&mut self, axis: Axis, min: i32, max: i32) -> &mut Self {
        match axis {
            Axis::Y => {
                if self.y < min {
                    self.y = min;
                } else if self.y > max {
                    self.y = max;
                }
            }
        }
        self
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#move
    pub fn move_dir(&mut self, direction: Direction) -> &mut Self {
        match direction {
            Direction::Down => {
                self.y = self.y.wrapping_sub(1);
            }
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
}

pub trait ReplaceBlobsRandom {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait ReplaceBlobsLevel<B: Clone + PartialEq> {
    fn get_min_y(&self) -> i32;
    fn get_max_y(&self) -> i32;
    fn get_block_state(&self, pos: BlockPos) -> B;
    fn set_block(&mut self, pos: BlockPos, state: B, flags: i32) -> bool;
    fn is_target(&self, state: &B, target: &B) -> bool;
}

pub struct ReplaceSphereConfiguration<B, FR> {
    pub target_state: B,
    pub replace_state: B,
    pub radius: FR,
}

pub struct ReplaceBlobsFeature;

impl ReplaceBlobsFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.ReplaceBlobsFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.ReplaceBlobsFeature#place
    pub fn place<B, W, R, FR>(
        &self,
        config: &ReplaceSphereConfiguration<B, FR>,
        level: &mut W,
        random: &mut R,
        origin: BlockPos,
    ) -> bool
    where
        B: Clone + PartialEq,
        W: ReplaceBlobsLevel<B>,
        R: ReplaceBlobsRandom,
        FR: Fn(&mut R) -> i32,
    {
        let mut start = MutableBlockPos::new(origin.x, origin.y, origin.z);
        let min_y = level.get_min_y().wrapping_add(1);
        let max_y = level.get_max_y();
        start.clamp(Axis::Y, min_y, max_y);
        let target_pos = Self::find_target(level, &mut start, &config.target_state);
        let Some(target_pos) = target_pos else {
            return false;
        };

        let rx = (config.radius)(random);
        let ry = (config.radius)(random);
        let rz = (config.radius)(random);
        let max_radius = rx.max(ry.max(rz));
        let mut placed_any = false;

        for pos in BlockPos::within_manhattan(target_pos, rx, ry, rz) {
            if pos.dist_manhattan(target_pos) > max_radius {
                break;
            }

            let state = level.get_block_state(pos).clone();
            if level.is_target(&state, &config.target_state) {
                level.set_block(pos, config.replace_state.clone(), 2);
                placed_any = true;
            }
        }

        placed_any
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.ReplaceBlobsFeature#findTarget
    fn find_target<B, W>(level: &W, pos: &mut MutableBlockPos, target: &B) -> Option<BlockPos>
    where
        B: Clone + PartialEq,
        W: ReplaceBlobsLevel<B>,
    {
        while pos.y > level.get_min_y().wrapping_add(1) {
            let block_pos = pos.as_pos();
            let state = level.get_block_state(block_pos);
            if level.is_target(&state, target) {
                return Some(block_pos);
            }
            pos.move_dir(Direction::Down);
        }
        None
    }
}

impl Default for ReplaceBlobsFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Block(u8);

    struct TestRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl TestRandom {
        fn new(values: Vec<i32>) -> Self { Self { values, idx: 0 } }
    }

    impl ReplaceBlobsRandom for TestRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = if self.idx < self.values.len() { self.values[self.idx] } else { 0 };
            self.idx += 1;
            v
        }
    }

    struct TestLevel {
        min_y: i32,
        max_y: i32,
        blocks: HashMap<(i32, i32, i32), Block>,
        set_calls: Vec<(BlockPos, Block, i32)>,
    }

    impl TestLevel {
        fn new(min_y: i32, max_y: i32) -> Self {
            Self { min_y, max_y, blocks: HashMap::new(), set_calls: Vec::new() }
        }
        fn with_block(mut self, pos: BlockPos, b: Block) -> Self {
            self.blocks.insert((pos.x, pos.y, pos.z), b);
            self
        }
    }

    impl ReplaceBlobsLevel<Block> for TestLevel {
        fn get_min_y(&self) -> i32 { self.min_y }
        fn get_max_y(&self) -> i32 { self.max_y }
        fn get_block_state(&self, pos: BlockPos) -> Block {
            self.blocks.get(&(pos.x, pos.y, pos.z)).cloned().unwrap_or(Block(0))
        }
        fn set_block(&mut self, pos: BlockPos, state: Block, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state.clone());
            self.set_calls.push((pos, state, flags));
            true
        }
        fn is_target(&self, state: &Block, target: &Block) -> bool { state == target }
    }

    #[test]
    fn returns_false_when_no_target_found() {
        let feature = ReplaceBlobsFeature::new();
        let mut level = TestLevel::new(-64, 320);
        let mut random = TestRandom::new(vec![1, 1, 1]);
        let config = ReplaceSphereConfiguration {
            target_state: Block(1),
            replace_state: Block(2),
            radius: |r: &mut TestRandom| r.next_int(0),
        };
        assert!(!feature.place(&config, &mut level, &mut random, BlockPos::new(0, 10, 0)));
    }

    #[test]
    fn replaces_target_blocks_within_manhattan_radius() {
        let feature = ReplaceBlobsFeature::new();
        let target = BlockPos::new(0, 10, 0);
        let mut level = TestLevel::new(-64, 320)
            .with_block(target, Block(1))
            .with_block(BlockPos::new(1, 10, 0), Block(1))
            .with_block(BlockPos::new(2, 10, 0), Block(1));
        let mut random = TestRandom::new(vec![1, 1, 1]);
        let config = ReplaceSphereConfiguration {
            target_state: Block(1),
            replace_state: Block(2),
            radius: |r: &mut TestRandom| r.next_int(0),
        };
        let placed = feature.place(&config, &mut level, &mut random, BlockPos::new(0, 10, 0));
        assert!(placed);
        assert_eq!(level.get_block_state(target), Block(2));
        assert_eq!(level.get_block_state(BlockPos::new(1, 10, 0)), Block(2));
        // dist 2 from center, max radius is 1, so it should not be replaced
        assert_eq!(level.get_block_state(BlockPos::new(2, 10, 0)), Block(1));
    }

    #[test]
    fn find_target_clamps_and_searches_downward() {
        let mut level = TestLevel::new(0, 10).with_block(BlockPos::new(0, 2, 0), Block(9));
        let mut pos = MutableBlockPos::new(0, 100, 0);
        pos.clamp(Axis::Y, level.get_min_y() + 1, level.get_max_y());
        let found = ReplaceBlobsFeature::find_target(&level, &mut pos, &Block(9));
        assert_eq!(found, Some(BlockPos::new(0, 2, 0)));
    }

    #[test]
    fn negative_coordinates_and_manhattan() {
        let a = BlockPos::new(-5, -10, -3);
        let b = BlockPos::new(-2, -7, -1);
        assert_eq!(a.dist_manhattan(b), 8);
        let feature = ReplaceBlobsFeature::new();
        let mut level = TestLevel::new(-64, 320).with_block(a, Block(1));
        let mut random = TestRandom::new(vec![0, 0, 0]);
        let config = ReplaceSphereConfiguration {
            target_state: Block(1),
            replace_state: Block(4),
            radius: |_r: &mut TestRandom| 0,
        };
        assert!(feature.place(&config, &mut level, &mut random, a));
        assert_eq!(level.get_block_state(a), Block(4));
    }

    #[test]
    fn set_block_uses_flag_2() {
        let feature = ReplaceBlobsFeature::new();
        let mut level = TestLevel::new(-64, 320).with_block(BlockPos::new(0, 5, 0), Block(1));
        let mut random = TestRandom::new(vec![0, 0, 0]);
        let config = ReplaceSphereConfiguration {
            target_state: Block(1),
            replace_state: Block(3),
            radius: |_r: &mut TestRandom| 0,
        };
        feature.place(&config, &mut level, &mut random, BlockPos::new(0, 5, 0));
        assert_eq!(level.set_calls.len(), 1);
        assert_eq!(level.set_calls[0].2, 2);
    }
}
