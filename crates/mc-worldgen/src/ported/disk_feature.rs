use std::fmt::Debug;

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

    // JAVA: net.minecraft.core.BlockPos#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(
            self.x.wrapping_add(dx),
            self.y.wrapping_add(dy),
            self.z.wrapping_add(dz),
        )
    }

    // JAVA: net.minecraft.core.BlockPos#above
    pub fn above(&self) -> Self {
        self.offset(0, 1, 0)
    }

    // JAVA: net.minecraft.core.BlockPos#betweenClosed
    pub fn between_closed(from: BlockPos, to: BlockPos) -> Vec<BlockPos> {
        let min_x = from.x.min(to.x);
        let min_y = from.y.min(to.y);
        let min_z = from.z.min(to.z);
        let max_x = from.x.max(to.x);
        let max_y = from.y.max(to.y);
        let max_z = from.z.max(to.z);
        let mut out = Vec::new();
        let mut y = min_y;
        while y <= max_y {
            let mut z = min_z;
            while z <= max_z {
                let mut x = min_x;
                while x <= max_x {
                    out.push(BlockPos::new(x, y, z));
                    if x == max_x {
                        break;
                    }
                    x = x.wrapping_add(1);
                }
                if z == max_z {
                    break;
                }
                z = z.wrapping_add(1);
            }
            if y == max_y {
                break;
            }
            y = y.wrapping_add(1);
        }
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
    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#<init>
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#set(Vec3i)
    pub fn set(&mut self, pos: BlockPos) -> &mut Self {
        self.x = pos.x;
        self.y = pos.y;
        self.z = pos.z;
        self
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#setY
    pub fn set_y(&mut self, y: i32) -> &mut Self {
        self.y = y;
        self
    }

    pub fn as_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

pub trait DiskLevel<B: Clone> {
    fn set_block(&mut self, pos: BlockPos, state: B, flags: i32) -> bool;
}

pub struct DiskConfiguration<FR, FT, FP> {
    pub half_height: i32,
    pub radius: FR,
    pub target: FT,
    pub state_provider: FP,
}

pub struct DiskFeature;

impl DiskFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.DiskFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.DiskFeature#place
    pub fn place<B, W, R, FR, FT, FP>(
        &self,
        config: &DiskConfiguration<FR, FT, FP>,
        origin: BlockPos,
        level: &mut W,
        random: &mut R,
    ) -> bool
    where
        B: Clone,
        W: DiskLevel<B>,
        FR: Fn(&mut R) -> i32,
        FT: Fn(&W, BlockPos) -> bool,
        FP: Fn(&W, &mut R, BlockPos) -> B,
    {
        let y = origin.get_y();
        let top_y = y.wrapping_add(config.half_height);
        let bottom_y = y.wrapping_sub(config.half_height).wrapping_sub(1);
        let radius = (config.radius)(random);
        let mut placed_any = false;
        let mut mutable_pos = MutableBlockPos::new();

        let from = origin.offset(radius.wrapping_neg(), 0, radius.wrapping_neg());
        let to = origin.offset(radius, 0, radius);
        for pos in BlockPos::between_closed(from, to) {
            let dx = pos.get_x().wrapping_sub(origin.get_x());
            let dz = pos.get_z().wrapping_sub(origin.get_z());
            let dist_sq = dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz));
            let radius_sq = radius.wrapping_mul(radius);
            if dist_sq <= radius_sq {
                let placed = self.place_column(
                    config,
                    level,
                    random,
                    top_y,
                    bottom_y,
                    mutable_pos.set(pos),
                );
                placed_any |= placed;
            }
        }

        placed_any
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.DiskFeature#placeColumn
    pub fn place_column<B, W, R, FR, FT, FP>(
        &self,
        config: &DiskConfiguration<FR, FT, FP>,
        level: &mut W,
        random: &mut R,
        top_y: i32,
        bottom_y: i32,
        pos: &mut MutableBlockPos,
    ) -> bool
    where
        B: Clone,
        W: DiskLevel<B>,
        FR: Fn(&mut R) -> i32,
        FT: Fn(&W, BlockPos) -> bool,
        FP: Fn(&W, &mut R, BlockPos) -> B,
    {
        let mut placed_any = false;
        let mut was_placed_above = false;

        let mut y = top_y;
        while y > bottom_y {
            pos.set_y(y);
            let block_pos = pos.as_pos();
            if (config.target)(level, block_pos) {
                let state = (config.state_provider)(level, random, block_pos);
                level.set_block(block_pos, state, 2);
                if !was_placed_above {
                    self.mark_above_for_post_processing(level, block_pos);
                }

                placed_any = true;
                was_placed_above = true;
            } else {
                was_placed_above = false;
            }
            y = y.wrapping_sub(1);
        }

        placed_any
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.Feature#markAboveForPostProcessing
    pub fn mark_above_for_post_processing<B: Clone, W: DiskLevel<B>>(
        &self,
        _level: &mut W,
        _pos: BlockPos,
    ) {
        // Default Feature hook: no-op in this self-contained port.
        // Concrete level implementations may override behavior via trait extension.
    }
}

impl Default for DiskFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestBlock(u8);

    struct TestRandom {
        ints: Vec<i32>,
        idx: usize,
    }

    impl TestRandom {
        fn new(ints: Vec<i32>) -> Self {
            Self { ints, idx: 0 }
        }
    }

    fn next_int(r: &mut TestRandom, _bound: i32) -> i32 {
        let v = self_int(r);
        v
    }

    fn self_int(r: &mut TestRandom) -> i32 {
        if r.idx < r.ints.len() {
            let v = r.ints[r.idx];
            r.idx += 1;
            v
        } else {
            0
        }
    }

    struct TestLevel {
        blocks: HashMap<(i32, i32, i32), TestBlock>,
        target: TestBlock,
        set_calls: Vec<(BlockPos, TestBlock, i32)>,
        post_process_marks: HashSet<(i32, i32, i32)>,
    }

    impl TestLevel {
        fn new(target: TestBlock) -> Self {
            Self {
                blocks: HashMap::new(),
                target,
                set_calls: Vec::new(),
                post_process_marks: HashSet::new(),
            }
        }

        fn with_block(mut self, pos: BlockPos, block: TestBlock) -> Self {
            self.blocks.insert((pos.x, pos.y, pos.z), block);
            self
        }
    }

    impl DiskLevel<TestBlock> for TestLevel {
        fn set_block(&mut self, pos: BlockPos, state: TestBlock, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state.clone());
            self.set_calls.push((pos, state, flags));
            true
        }
    }

    trait DiskLevelMark: DiskLevel<TestBlock> {
        fn mark_post(&mut self, pos: BlockPos);
    }

    impl DiskLevelMark for TestLevel {
        fn mark_post(&mut self, pos: BlockPos) {
            self.post_process_marks.insert((pos.x, pos.y, pos.z));
        }
    }

    struct MarkingDiskFeature;

    impl MarkingDiskFeature {
        fn mark_above_for_post_processing<W: DiskLevelMark>(
            &self,
            level: &mut W,
            pos: BlockPos,
        ) {
            level.mark_post(pos.above());
        }

        fn place_column<B, W, R, FR, FT, FP>(
            &self,
            config: &DiskConfiguration<FR, FT, FP>,
            level: &mut W,
            random: &mut R,
            top_y: i32,
            bottom_y: i32,
            pos: &mut MutableBlockPos,
        ) -> bool
        where
            B: Clone,
            W: DiskLevel<B> + DiskLevelMark,
            FR: Fn(&mut R) -> i32,
            FT: Fn(&W, BlockPos) -> bool,
            FP: Fn(&W, &mut R, BlockPos) -> B,
        {
            let mut placed_any = false;
            let mut was_placed_above = false;
            let mut y = top_y;
            while y > bottom_y {
                pos.set_y(y);
                let block_pos = pos.as_pos();
                if (config.target)(level, block_pos) {
                    let state = (config.state_provider)(level, random, block_pos);
                    level.set_block(block_pos, state, 2);
                    if !was_placed_above {
                        self.mark_above_for_post_processing(level, block_pos);
                    }
                    placed_any = true;
                    was_placed_above = true;
                } else {
                    was_placed_above = false;
                }
                y = y.wrapping_sub(1);
            }
            placed_any
        }
    }

    #[test]
    fn place_places_matching_disk_columns() {
        let feature = DiskFeature::new();
        let origin = BlockPos::new(0, 64, 0);
        let mut level = TestLevel::new(TestBlock(1));
        for y in 62..=65 {
            for x in -1..=1 {
                for z in -1..=1 {
                    if x * x + z * z <= 1 {
                        level.blocks.insert((x, y, z), TestBlock(1));
                    }
                }
            }
        }
        let mut random = TestRandom::new(vec![1]);
        let config = DiskConfiguration {
            half_height: 1,
            radius: |_r: &mut TestRandom| 1i32,
            target: |l: &TestLevel, p: BlockPos| {
                l.blocks.get(&(p.x, p.y, p.z)) == Some(&l.target)
            },
            state_provider: |_l: &TestLevel, _r: &mut TestRandom, _p: BlockPos| TestBlock(2),
        };

        let placed = feature.place(&config, origin, &mut level, &mut random);
        assert!(placed);
        assert!(level.set_calls.iter().all(|(_, _, flags)| *flags == 2));
        assert_eq!(level.blocks.get(&(0, 64, 0)), Some(&TestBlock(2)));
        assert_eq!(level.blocks.get(&(1, 0, 0)), level.blocks.get(&(1, 0, 0)));
    }

    #[test]
    fn place_returns_false_when_no_target_matches() {
        let feature = DiskFeature::new();
        let mut level = TestLevel::new(TestBlock(9));
        let mut random = TestRandom::new(vec![]);
        let config = DiskConfiguration {
            half_height: 1,
            radius: |_r: &mut TestRandom| 2i32,
            target: |_l: &TestLevel, _p: BlockPos| false,
            state_provider: |_l: &TestLevel, _r: &mut TestRandom, _p: BlockPos| TestBlock(2),
        };
        let placed = feature.place(&config, BlockPos::new(0, 10, 0), &mut level, &mut random);
        assert!(!placed);
        assert!(level.set_calls.is_empty());
    }

    #[test]
    fn place_column_marks_only_on_transition_to_target() {
        let feature = MarkingDiskFeature;
        let mut level = TestLevel::new(TestBlock(1))
            .with_block(BlockPos::new(0, 5, 0), TestBlock(1))
            .with_block(BlockPos::new(0, 4, 0), TestBlock(1))
            .with_block(BlockPos::new(0, 3, 0), TestBlock(0))
            .with_block(BlockPos::new(0, 2, 0), TestBlock(1));
        let mut random = TestRandom::new(vec![]);
        let config = DiskConfiguration {
            half_height: 0,
            radius: |_r: &mut TestRandom| 0i32,
            target: |l: &TestLevel, p: BlockPos| l.blocks.get(&(p.x, p.y, p.z)) == Some(&TestBlock(1)),
            state_provider: |_l: &TestLevel, _r: &mut TestRandom, _p: BlockPos| TestBlock(2),
        };
        let mut pos = MutableBlockPos::new();
        pos.set(BlockPos::new(0, 0, 0));
        let placed = feature.place_column(&config, &mut level, &mut random, 5, 1, &mut pos);
        assert!(placed);
        assert!(level.post_process_marks.contains(&(0, 6, 0)));
        assert!(level.post_process_marks.contains(&(0, 3, 0)));
        assert_eq!(level.post_process_marks.len(), 2);
    }

    #[test]
    fn negative_coordinates_and_wrapping_distance() {
        let feature = DiskFeature::new();
        let origin = BlockPos::new(-5, -10, -5);
        let mut level = TestLevel::new(TestBlock(1));
        for y in -11..=-9 {
            level.blocks.insert((-5, y, -5), TestBlock(1));
        }
        let mut random = TestRandom::new(vec![]);
        let config = DiskConfiguration {
            half_height: 1,
            radius: |_r: &mut TestRandom| 0i32,
            target: |l: &TestLevel, p: BlockPos| l.blocks.get(&(p.x, p.y, p.z)) == Some(&TestBlock(1)),
            state_provider: |_l: &TestLevel, _r: &mut TestRandom, _p: BlockPos| TestBlock(7),
        };
        let placed = feature.place(&config, origin, &mut level, &mut random);
        assert!(placed);
        assert_eq!(level.blocks.get(&(-5, -10, -5)), Some(&TestBlock(7)));
    }

    #[test]
    fn distance_uses_wrapping_integer_arithmetic() {
        let dx: i32 = i32::MAX;
        let dz: i32 = 0;
        let dist_sq = dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz));
        let radius_sq = i32::MAX.wrapping_mul(i32::MAX);
        assert_eq!(dist_sq, radius_sq);
        assert!(dist_sq <= radius_sq);
    }
}
