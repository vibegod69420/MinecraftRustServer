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

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> Self {
        self.offset(0, -1, 0)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
}

pub trait BlockPileRandom {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
    fn next_boolean(&mut self) -> bool;
}

pub trait BlockPileLevel<B: Clone> {
    fn get_min_y(&self) -> i32;
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> B;
    fn set_block(&mut self, pos: BlockPos, state: B, flags: i32) -> bool;
    fn is_dirt_path(&self, state: &B) -> bool;
    fn is_face_sturdy(&self, pos: BlockPos, state: &B, direction: Direction) -> bool;
}

pub struct BlockPileConfiguration<FP> {
    pub state_provider: FP,
}

pub struct BlockPileFeature;

impl BlockPileFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.BlockPileFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BlockPileFeature#place
    pub fn place<B, W, R, FP>(
        &self,
        origin: BlockPos,
        level: &mut W,
        random: &mut R,
        config: &BlockPileConfiguration<FP>,
    ) -> bool
    where
        B: Clone,
        W: BlockPileLevel<B>,
        R: BlockPileRandom,
        FP: Fn(&mut R, BlockPos) -> B,
    {
        if origin.get_y() < level.get_min_y().wrapping_add(5) {
            return false;
        }

        let radius_x = 2i32.wrapping_add(random.next_int(2));
        let radius_z = 2i32.wrapping_add(random.next_int(2));

        let from = origin.offset(radius_x.wrapping_neg(), 0, radius_z.wrapping_neg());
        let to = origin.offset(radius_x, 1, radius_z);
        for pos in BlockPos::between_closed(from, to) {
            let dx = origin.get_x().wrapping_sub(pos.get_x());
            let dz = origin.get_z().wrapping_sub(pos.get_z());
            let dist_sq = dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz));
            let threshold = random.next_float() * 10.0_f32 - random.next_float() * 6.0_f32;
            if (dist_sq as f32) <= threshold {
                self.try_place_block(level, pos, random, config);
            } else if random.next_float() < 0.031_f32 {
                self.try_place_block(level, pos, random, config);
            }
        }

        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BlockPileFeature#mayPlaceOn
    fn may_place_on<B, W, R>(&self, level: &W, pos: BlockPos, random: &mut R) -> bool
    where
        B: Clone,
        W: BlockPileLevel<B>,
        R: BlockPileRandom,
    {
        let below = pos.below();
        let below_state = level.get_block_state(below);
        if level.is_dirt_path(&below_state) {
            random.next_boolean()
        } else {
            level.is_face_sturdy(below, &below_state, Direction::Up)
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BlockPileFeature#tryPlaceBlock
    fn try_place_block<B, W, R, FP>(
        &self,
        level: &mut W,
        pos: BlockPos,
        random: &mut R,
        config: &BlockPileConfiguration<FP>,
    ) where
        B: Clone,
        W: BlockPileLevel<B>,
        R: BlockPileRandom,
        FP: Fn(&mut R, BlockPos) -> B,
    {
        if level.is_empty_block(pos) && self.may_place_on(level, pos, random) {
            let state = (config.state_provider)(random, pos);
            level.set_block(pos, state, 260);
        }
    }
}

impl Default for BlockPileFeature {
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
        ints: Vec<i32>,
        floats: Vec<f32>,
        bools: Vec<bool>,
        i: usize,
        f: usize,
        b: usize,
    }

    impl TestRandom {
        fn new(ints: Vec<i32>, floats: Vec<f32>, bools: Vec<bool>) -> Self {
            Self { ints, floats, bools, i: 0, f: 0, b: 0 }
        }
    }

    impl BlockPileRandom for TestRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = if self.i < self.ints.len() { self.ints[self.i] } else { 0 };
            self.i += 1;
            v
        }
        fn next_float(&mut self) -> f32 {
            let v = if self.f < self.floats.len() { self.floats[self.f] } else { 1.0 };
            self.f += 1;
            v
        }
        fn next_boolean(&mut self) -> bool {
            let v = if self.b < self.bools.len() { self.bools[self.b] } else { false };
            self.b += 1;
            v
        }
    }

    struct TestLevel {
        min_y: i32,
        blocks: HashMap<(i32, i32, i32), Block>,
        sturdy: HashMap<(i32, i32, i32), bool>,
        set_calls: Vec<(BlockPos, Block, i32)>,
    }

    impl TestLevel {
        fn new(min_y: i32) -> Self {
            Self { min_y, blocks: HashMap::new(), sturdy: HashMap::new(), set_calls: Vec::new() }
        }
        fn with_block(mut self, pos: BlockPos, block: Block) -> Self {
            self.blocks.insert((pos.x, pos.y, pos.z), block);
            self
        }
        fn with_sturdy(mut self, pos: BlockPos) -> Self {
            self.sturdy.insert((pos.x, pos.y, pos.z), true);
            self
        }
    }

    impl BlockPileLevel<Block> for TestLevel {
        fn get_min_y(&self) -> i32 { self.min_y }
        fn is_empty_block(&self, pos: BlockPos) -> bool {
            !self.blocks.contains_key(&(pos.x, pos.y, pos.z))
        }
        fn get_block_state(&self, pos: BlockPos) -> Block {
            self.blocks.get(&(pos.x, pos.y, pos.z)).cloned().unwrap_or(Block(0))
        }
        fn set_block(&mut self, pos: BlockPos, state: Block, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state.clone());
            self.set_calls.push((pos, state, flags));
            true
        }
        fn is_dirt_path(&self, state: &Block) -> bool { state.0 == 9 }
        fn is_face_sturdy(&self, pos: BlockPos, _state: &Block, _direction: Direction) -> bool {
            self.sturdy.get(&(pos.x, pos.y, pos.z)).copied().unwrap_or(false)
        }
    }

    #[test]
    fn rejects_origin_too_low() {
        let feature = BlockPileFeature::new();
        let mut level = TestLevel::new(0);
        let mut random = TestRandom::new(vec![0, 0], vec![], vec![]);
        let config = BlockPileConfiguration { state_provider: |_r: &mut TestRandom, _p: BlockPos| Block(1) };
        assert!(!feature.place(BlockPos::new(0, 4, 0), &mut level, &mut random, &config));
    }

    #[test]
    fn places_block_when_empty_and_sturdy_below() {
        let feature = BlockPileFeature::new();
        let origin = BlockPos::new(0, 10, 0);
        let mut level = TestLevel::new(0)
            .with_block(origin.below(), Block(2))
            .with_sturdy(origin.below());
        // radius_x = 2+0, radius_z = 2+0; first pos gives dist_sq 8; threshold 10*1.0 - 6*0.0 = 10.0 -> place
        let mut random = TestRandom::new(vec![0, 0], vec![1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], vec![]);
        let config = BlockPileConfiguration { state_provider: |_r: &mut TestRandom, _p: BlockPos| Block(5) };
        let placed = feature.place(origin, &mut level, &mut random, &config);
        assert!(placed);
        assert!(level.set_calls.iter().any(|(p, b, flags)| *p == origin && b.0 == 5 && *flags == 260));
    }

    #[test]
    fn dirt_path_requires_random_boolean() {
        let feature = BlockPileFeature::new();
        let origin = BlockPos::new(0, 10, 0);
        let mut level = TestLevel::new(0).with_block(origin.below(), Block(9));
        // Make only origin position qualify: radius 2,2; force threshold high for first position (origin offset -2,-2 has dist 8)
        // We'll provide floats so first position passes threshold, then boolean false prevents placement on dirt path
        let mut floats = vec![1.0_f32, 0.0_f32]; // threshold 10
        // fill remaining with high second float so threshold becomes negative
        for _ in 0..200 { floats.push(0.0); floats.push(1.0); }
        let mut random = TestRandom::new(vec![0, 0], floats, vec![false]);
        let config = BlockPileConfiguration { state_provider: |_r: &mut TestRandom, _p: BlockPos| Block(5) };
        feature.place(origin, &mut level, &mut random, &config);
        assert!(!level.set_calls.iter().any(|(p, _, _)| *p == origin));
    }

    #[test]
    fn random_fallback_places_when_below_threshold() {
        let feature = BlockPileFeature::new();
        let origin = BlockPos::new(0, 10, 0);
        let pos = origin;
        let mut level = TestLevel::new(0).with_block(pos.below(), Block(2)).with_sturdy(pos.below());
        // radius 2,2. For origin pos (dx=0,dz=0, dist_sq=0): threshold = 0.0*10 - 1.0*6 = -6, so 0 <= -6 false, then fallback random < 0.031
        let mut floats = Vec::new();
        // first position in betweenClosed is (-2,10,-2), ... we need to find origin. Easier to force all thresholds negative and fallback true only at right time.
        // betweenClosed for x:-2..2, y:0..1, z:-2..2 => 5*2*5 = 50 positions
        for _ in 0..50 {
            floats.push(0.0); // next_float for threshold first
            floats.push(1.0); // next_float for threshold second => threshold -6
            floats.push(0.0); // fallback < 0.031 true
        }
        let mut random = TestRandom::new(vec![0, 0], floats, vec![]);
        let config = BlockPileConfiguration { state_provider: |_r: &mut TestRandom, _p: BlockPos| Block(7) };
        let placed = feature.place(origin, &mut level, &mut random, &config);
        assert!(placed);
        assert!(level.set_calls.iter().any(|(p, b, _)| *p == origin && b.0 == 7));
    }

    #[test]
    fn negative_coordinates_supported() {
        let feature = BlockPileFeature::new();
        let origin = BlockPos::new(-10, 10, -20);
        let mut level = TestLevel::new(-64)
            .with_block(origin.below(), Block(1))
            .with_sturdy(origin.below());
        let mut floats = Vec::new();
        for _ in 0..200 { floats.push(1.0); floats.push(0.0); }
        let mut random = TestRandom::new(vec![0, 0], floats, vec![]);
        let config = BlockPileConfiguration { state_provider: |_r: &mut TestRandom, _p: BlockPos| Block(3) };
        assert!(feature.place(origin, &mut level, &mut random, &config));
        assert!(level.set_calls.iter().any(|(p, _, _)| *p == origin));
    }
}
