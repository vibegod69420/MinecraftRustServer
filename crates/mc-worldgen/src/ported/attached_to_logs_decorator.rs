//! Port of `net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLogsDecorator`.

/// Minimal block position used by tree decorators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#relative(net.minecraft.core.Direction)
    #[inline]
    pub fn relative(self, direction: Direction) -> Self {
        let step = direction.step();
        Self {
            x: self.x.wrapping_add(step.x),
            y: self.y.wrapping_add(step.y),
            z: self.z.wrapping_add(step.z),
        }
    }
}

/// Minimal direction representation for attached-to-logs placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    #[inline]
    pub const fn step(self) -> BlockPos {
        match self {
            Direction::Down => BlockPos::new(0, -1, 0),
            Direction::Up => BlockPos::new(0, 1, 0),
            Direction::North => BlockPos::new(0, 0, -1),
            Direction::South => BlockPos::new(0, 0, 1),
            Direction::West => BlockPos::new(-1, 0, 0),
            Direction::East => BlockPos::new(1, 0, 0),
        }
    }
}

/// Opaque block state handle for self-contained decorator logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState(pub u32);

/// Minimal random source matching the Java calls used by this class.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Minimal block state provider.
pub trait BlockStateProvider {
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState;
}

/// Minimal tree decorator context.
pub trait AttachedToLogsContext {
    type Random: RandomSource;

    fn random(&mut self) -> &mut Self::Random;
    fn logs(&self) -> Vec<BlockPos>;
    fn is_air(&self, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: BlockState);
}

/// Simple deterministic random source for tests and self-contained use.
#[derive(Debug, Clone)]
pub struct SimpleRandomSource {
    state: i64,
}

impl SimpleRandomSource {
    pub fn new(seed: i64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self, bits: u32) -> i32 {
        const MULTIPLIER: i64 = 0x5DEECE66D;
        const ADDEND: i64 = 0xB;
        const MASK: i64 = (1_i64 << 48) - 1;
        self.state = self
            .state
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(ADDEND)
            & MASK;
        ((self.state as u64) >> (48 - bits)) as i32
    }
}

impl RandomSource for SimpleRandomSource {
    fn next_float(&mut self) -> f32 {
        let bits = self.next(24) as f32;
        bits / ((1_i32 << 24) as f32)
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        let m = bound.wrapping_sub(1);
        if (bound & m) == 0 {
            let r = ((bound as i64).wrapping_mul(self.next(31) as i64)) >> 31;
            return r as i32;
        }

        loop {
            let bits = self.next(31);
            let val = bits % bound;
            if bits.wrapping_sub(val).wrapping_add(m) >= 0 {
                return val;
            }
        }
    }
}

// JAVA: net.minecraft.util.Util#shuffle(java.util.List, net.minecraft.util.RandomSource)
fn shuffle<T>(list: &mut [T], random: &mut impl RandomSource) {
    let mut i = list.len() as i32;
    while i > 1 {
        i -= 1;
        let j = random.next_int(i + 1) as usize;
        list.swap(i as usize, j);
    }
}

// JAVA: net.minecraft.util.Util#shuffledCopy(java.util.List, net.minecraft.util.RandomSource)
fn shuffled_copy<T: Clone>(list: &[T], random: &mut impl RandomSource) -> Vec<T> {
    let mut out = list.to_vec();
    shuffle(&mut out, random);
    out
}

// JAVA: net.minecraft.util.Util#getRandom(java.util.List, net.minecraft.util.RandomSource)
fn get_random<T: Copy>(list: &[T], random: &mut impl RandomSource) -> T {
    assert!(!list.is_empty(), "list must not be empty");
    list[random.next_int(list.len() as i32) as usize]
}

/// Port of `net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLogsDecorator`.
#[derive(Debug, Clone)]
pub struct AttachedToLogsDecorator<P: BlockStateProvider> {
    pub probability: f32,
    pub block_provider: P,
    pub directions: Vec<Direction>,
}

impl<P: BlockStateProvider> AttachedToLogsDecorator<P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLogsDecorator#<init>
    pub fn new(probability: f32, block_provider: P, directions: Vec<Direction>) -> Self {
        Self {
            probability,
            block_provider,
            directions,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLogsDecorator#type
    pub fn decorator_type(&self) -> &'static str {
        "attached_to_logs"
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLogsDecorator#place
    pub fn place<C: AttachedToLogsContext>(&self, ctx: &mut C) {
        let logs = ctx.logs();
        let random = ctx.random();
        let shuffled_logs = shuffled_copy(&logs, random);

        for log_pos in shuffled_logs {
            let random = ctx.random();
            let direction = get_random(&self.directions, random);
            let target_pos = log_pos.relative(direction);

            let random = ctx.random();
            let roll = random.next_float();
            if roll <= self.probability && ctx.is_air(target_pos) {
                let state = self.block_provider.get_state(ctx.random(), target_pos);
                ctx.set_block(target_pos, state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, Clone, Copy)]
    struct FixedProvider(BlockState);

    impl BlockStateProvider for FixedProvider {
        fn get_state(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
            self.0
        }
    }

    #[derive(Debug)]
    struct TestContext {
        random: SimpleRandomSource,
        logs: Vec<BlockPos>,
        air: Vec<BlockPos>,
        placed: Rc<RefCell<Vec<(BlockPos, BlockState)>>>,
    }

    impl TestContext {
        fn new(seed: i64, logs: Vec<BlockPos>, air: Vec<BlockPos>) -> Self {
            Self {
                random: SimpleRandomSource::new(seed),
                logs,
                air,
                placed: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl AttachedToLogsContext for TestContext {
        type Random = SimpleRandomSource;

        fn random(&mut self) -> &mut Self::Random {
            &mut self.random
        }

        fn logs(&self) -> Vec<BlockPos> {
            self.logs.clone()
        }

        fn is_air(&self, pos: BlockPos) -> bool {
            self.air.contains(&pos)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState) {
            self.placed.borrow_mut().push((pos, state));
        }
    }

    #[test]
    fn decorator_type_is_attached_to_logs() {
        let dec = AttachedToLogsDecorator::new(1.0, FixedProvider(BlockState(1)), vec![Direction::Up]);
        assert_eq!(dec.decorator_type(), "attached_to_logs");
    }

    #[test]
    fn probability_zero_places_nothing() {
        let log = BlockPos::new(0, 64, 0);
        let mut ctx = TestContext::new(0, vec![log], vec![log.relative(Direction::Up)]);
        let dec = AttachedToLogsDecorator::new(0.0, FixedProvider(BlockState(7)), vec![Direction::Up]);
        dec.place(&mut ctx);
        assert!(ctx.placed.borrow().is_empty());
    }

    #[test]
    fn places_only_when_air() {
        let log = BlockPos::new(1, 64, 1);
        let target = log.relative(Direction::North);
        let mut ctx = TestContext::new(1, vec![log], vec![]);
        let dec = AttachedToLogsDecorator::new(1.0, FixedProvider(BlockState(9)), vec![Direction::North]);
        dec.place(&mut ctx);
        assert!(ctx.placed.borrow().is_empty());

        let mut ctx_air = TestContext::new(1, vec![log], vec![target]);
        dec.place(&mut ctx_air);
        assert_eq!(*ctx_air.placed.borrow(), vec![(target, BlockState(9))]);
    }

    #[test]
    fn handles_negative_and_wrapping_relative_positions() {
        let log = BlockPos::new(i32::MIN, -10, i32::MAX);
        let target = log.relative(Direction::West);
        let mut ctx = TestContext::new(2, vec![log], vec![target]);
        let dec = AttachedToLogsDecorator::new(1.0, FixedProvider(BlockState(3)), vec![Direction::West]);
        dec.place(&mut ctx);
        assert_eq!(*ctx.placed.borrow(), vec![(target, BlockState(3))]);
    }

    #[test]
    fn deterministic_for_same_seed() {
        let logs = vec![BlockPos::new(0, 0, 0), BlockPos::new(1, 0, 0), BlockPos::new(2, 0, 0)];
        let air = vec![
            BlockPos::new(0, 1, 0),
            BlockPos::new(1, 1, 0),
            BlockPos::new(2, 1, 0),
            BlockPos::new(0, 0, 1),
            BlockPos::new(1, 0, 1),
            BlockPos::new(2, 0, 1),
        ];
        let dirs = vec![Direction::Up, Direction::South];
        let dec = AttachedToLogsDecorator::new(0.75, FixedProvider(BlockState(5)), dirs);
        let mut ctx1 = TestContext::new(999, logs.clone(), air.clone());
        let mut ctx2 = TestContext::new(999, logs, air);
        dec.place(&mut ctx1);
        dec.place(&mut ctx2);
        assert_eq!(*ctx1.placed.borrow(), *ctx2.placed.borrow());
    }
}
