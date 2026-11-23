use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(III)
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#relative(Direction)
    pub fn relative(&self, direction: Direction) -> Self {
        match direction {
            Direction::Down => Self::new(self.x, self.y.wrapping_sub(1), self.z),
            Direction::Up => Self::new(self.x, self.y.wrapping_add(1), self.z),
            Direction::North => Self::new(self.x, self.y, self.z.wrapping_sub(1)),
            Direction::South => Self::new(self.x, self.y, self.z.wrapping_add(1)),
            Direction::West => Self::new(self.x.wrapping_sub(1), self.y, self.z),
            Direction::East => Self::new(self.x.wrapping_add(1), self.y, self.z),
        }
    }

    // JAVA: net.minecraft.core.Vec3i#getY()
    pub fn get_y(&self) -> i32 {
        self.y
    }
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
    // JAVA: net.minecraft.core.Direction#values()
    pub const fn values() -> [Direction; 6] {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: u16,
}

impl Block {
    pub const AIR: Block = Block { id: 0 };
    pub const CREAKING_HEART: Block = Block { id: 1 };
    pub const OAK_LOG: Block = Block { id: 2 };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CreakingHeartState {
    Dormant,
    Awake,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
    pub creaking_heart_state: Option<CreakingHeartState>,
    pub natural: bool,
}

impl BlockState {
    pub fn new(block: Block) -> Self {
        Self {
            block,
            creaking_heart_state: None,
            natural: false,
        }
    }

    pub fn is(&self, tag: BlockTag) -> bool {
        match tag {
            BlockTag::Logs => matches!(self.block.id, 2),
        }
    }

    pub fn set_creaking_heart_state(mut self, state: CreakingHeartState) -> Self {
        self.creaking_heart_state = Some(state);
        self
    }

    pub fn set_natural(mut self, natural: bool) -> Self {
        self.natural = natural;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockTag {
    Logs,
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

// JAVA: net.minecraft.util.Util#shuffle(List, RandomSource)
pub fn util_shuffle<T>(list: &mut [T], random: &mut (impl RandomSource + ?Sized)) {
    let mut i = (list.len() as i32).wrapping_sub(1);
    while i > 0 {
        let j = random.next_int(i.wrapping_add(1));
        list.swap(i as usize, j as usize);
        i = i.wrapping_sub(1);
    }
}

pub trait TreeDecoratorContext {
    fn random(&mut self) -> &mut dyn RandomSource;
    fn logs(&self) -> &[BlockPos];
    fn check_block(&self, pos: BlockPos, predicate: &dyn Fn(BlockState) -> bool) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: BlockState);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeDecoratorType {
    CreakingHeart,
    AlterGround,
}

#[derive(Clone, Copy, Debug)]
pub struct CreakingHeartDecorator {
    pub probability: f32,
}

impl CreakingHeartDecorator {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.CreakingHeartDecorator#<init>(float)
    pub fn new(probability: f32) -> Self {
        Self { probability }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.CreakingHeartDecorator#type()
    pub fn type_(&self) -> TreeDecoratorType {
        TreeDecoratorType::CreakingHeart
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.CreakingHeartDecorator#place(TreeDecorator$Context)
    pub fn place(&self, ctx: &mut dyn TreeDecoratorContext) {
        let random = ctx.random();
        let next = random.next_float();
        if next >= self.probability {
            return;
        }

        let logs = ctx.logs();
        if logs.is_empty() {
            return;
        }

        let mut shuffled: Vec<BlockPos> = logs.to_vec();
        // Shuffle uses the context's random source, matching Java Util.shuffle(list, random)
        util_shuffle(&mut shuffled, ctx.random());

        let selected = shuffled.into_iter().find(|pos| {
            for direction in Direction::values() {
                let neighbor = pos.relative(direction);
                let ok = ctx.check_block(neighbor, &|state| state.is(BlockTag::Logs));
                if !ok {
                    return false;
                }
            }
            true
        });

        if let Some(pos) = selected {
            let state = BlockState::new(Block::CREAKING_HEART)
                .set_creaking_heart_state(CreakingHeartState::Dormant)
                .set_natural(true);
            ctx.set_block(pos, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        float_idx: usize,
        int_idx: usize,
    }

    impl TestRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                ints,
                float_idx: 0,
                int_idx: 0,
            }
        }
    }

    impl RandomSource for TestRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats.get(self.float_idx).copied().unwrap_or(0.0);
            self.float_idx += 1;
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.ints.get(self.int_idx).copied().unwrap_or(0);
            self.int_idx += 1;
            v.rem_euclid(bound)
        }
    }

    struct TestContext {
        random: TestRandom,
        logs: Vec<BlockPos>,
        blocks: HashMap<BlockPos, BlockState>,
        set_blocks: Vec<(BlockPos, BlockState)>,
    }

    impl TestContext {
        fn new(random: TestRandom, logs: Vec<BlockPos>, log_positions: &[BlockPos]) -> Self {
            let mut blocks = HashMap::new();
            for &p in log_positions {
                blocks.insert(p, BlockState::new(Block::OAK_LOG));
            }
            Self {
                random,
                logs,
                blocks,
                set_blocks: Vec::new(),
            }
        }
    }

    impl TreeDecoratorContext for TestContext {
        fn random(&mut self) -> &mut dyn RandomSource {
            &mut self.random
        }

        fn logs(&self) -> &[BlockPos] {
            &self.logs
        }

        fn check_block(&self, pos: BlockPos, predicate: &dyn Fn(BlockState) -> bool) -> bool {
            match self.blocks.get(&pos) {
                Some(state) => predicate(*state),
                None => predicate(BlockState::new(Block::AIR)),
            }
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState) {
            self.blocks.insert(pos, state);
            self.set_blocks.push((pos, state));
        }
    }

    fn make_log_cluster(center: BlockPos) -> Vec<BlockPos> {
        let mut positions = vec![center];
        for dir in Direction::values() {
            positions.push(center.relative(dir));
        }
        positions
    }

    #[test]
    fn type_is_creaking_heart() {
        let decorator = CreakingHeartDecorator::new(1.0);
        assert_eq!(decorator.type_(), TreeDecoratorType::CreakingHeart);
    }

    #[test]
    fn skips_when_probability_fails() {
        let center = BlockPos::new(0, 64, 0);
        let logs = vec![center];
        let cluster = make_log_cluster(center);
        let mut ctx = TestContext::new(TestRandom::new(vec![0.5], vec![]), logs, &cluster);
        let decorator = CreakingHeartDecorator::new(0.5);
        decorator.place(&mut ctx);
        assert!(ctx.set_blocks.is_empty());
    }

    #[test]
    fn skips_when_no_logs() {
        let mut ctx = TestContext::new(TestRandom::new(vec![0.0], vec![]), vec![], &[]);
        let decorator = CreakingHeartDecorator::new(1.0);
        decorator.place(&mut ctx);
        assert!(ctx.set_blocks.is_empty());
    }

    #[test]
    fn places_dormant_natural_creaking_heart_when_surrounded_by_logs() {
        let center = BlockPos::new(10, 70, -5);
        let logs = vec![center];
        let cluster = make_log_cluster(center);
        let mut ctx = TestContext::new(TestRandom::new(vec![0.0], vec![0]), logs, &cluster);
        let decorator = CreakingHeartDecorator::new(1.0);
        decorator.place(&mut ctx);
        assert_eq!(ctx.set_blocks.len(), 1);
        let (pos, state) = ctx.set_blocks[0];
        assert_eq!(pos, center);
        assert_eq!(state.block, Block::CREAKING_HEART);
        assert_eq!(state.creaking_heart_state, Some(CreakingHeartState::Dormant));
        assert!(state.natural);
    }

    #[test]
    fn does_not_place_when_neighbor_not_log() {
        let center = BlockPos::new(0, 64, 0);
        let logs = vec![center];
        // Only center is a log; neighbors are air/missing.
        let mut ctx = TestContext::new(TestRandom::new(vec![0.0], vec![0]), logs, &[center]);
        let decorator = CreakingHeartDecorator::new(1.0);
        decorator.place(&mut ctx);
        assert!(ctx.set_blocks.is_empty());
    }

    #[test]
    fn util_shuffle_is_deterministic_and_handles_edge_cases() {
        let mut values = vec![1, 2, 3, 4];
        let mut rng = TestRandom::new(vec![], vec![0, 0, 0]);
        util_shuffle(&mut values, &mut rng);
        assert_eq!(values.len(), 4);
        // All elements still present
        let mut sorted = values.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3, 4]);

        let mut empty: Vec<i32> = vec![];
        let mut rng2 = TestRandom::new(vec![], vec![]);
        util_shuffle(&mut empty, &mut rng2);
        assert!(empty.is_empty());
    }
}
