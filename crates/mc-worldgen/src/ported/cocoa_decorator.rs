//! Port of `net.minecraft.world.level.levelgen.feature.treedecorators.CocoaDecorator`.

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

    #[inline]
    pub const fn get_y(self) -> i32 {
        self.y
    }

    #[inline]
    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

/// Horizontal directions used by cocoa placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const HORIZONTAL: [Direction; 4] = [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    #[inline]
    pub const fn get_opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    #[inline]
    pub const fn step(self) -> BlockPos {
        match self {
            Direction::North => BlockPos::new(0, 0, -1),
            Direction::South => BlockPos::new(0, 0, 1),
            Direction::West => BlockPos::new(-1, 0, 0),
            Direction::East => BlockPos::new(1, 0, 0),
        }
    }
}

/// Minimal block state representation.
/// Encodes cocoa facing and age for faithful testability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub block: u32,
    pub cocoa_age: Option<i32>,
    pub cocoa_facing: Option<Direction>,
}

impl BlockState {
    pub const COCOA_BLOCK_ID: u32 = 1;

    pub fn cocoa_default() -> Self {
        Self {
            block: Self::COCOA_BLOCK_ID,
            cocoa_age: Some(0),
            cocoa_facing: Some(Direction::North),
        }
    }

    pub fn with_cocoa_age(mut self, age: i32) -> Self {
        self.cocoa_age = Some(age);
        self
    }

    pub fn with_cocoa_facing(mut self, facing: Direction) -> Self {
        self.cocoa_facing = Some(facing);
        self
    }
}

/// Minimal random source matching the Java calls used by this class.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Minimal tree decorator context.
pub trait CocoaContext {
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

/// Port of `net.minecraft.world.level.levelgen.feature.treedecorators.CocoaDecorator`.
#[derive(Debug, Clone, Copy)]
pub struct CocoaDecorator {
    pub probability: f32,
}

impl CocoaDecorator {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.CocoaDecorator#<init>
    pub fn new(probability: f32) -> Self {
        Self { probability }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.CocoaDecorator#type
    pub fn decorator_type(&self) -> &'static str {
        "cocoa"
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.CocoaDecorator#place
    pub fn place<C: CocoaContext>(&self, ctx: &mut C) {
        let random = ctx.random();
        if !(random.next_float() >= self.probability) {
            let logs = ctx.logs();
            if !logs.is_empty() {
                let first_y = logs[0].get_y();
                let filtered: Vec<BlockPos> = logs
                    .into_iter()
                    .filter(|p| p.get_y().wrapping_sub(first_y) <= 2)
                    .collect();

                for log_pos in filtered {
                    for dir in Direction::HORIZONTAL {
                        let random = ctx.random();
                        if random.next_float() <= 0.25_f32 {
                            let opposite = dir.get_opposite();
                            let step = opposite.step();
                            let target = log_pos.offset(step.x, 0, step.z);
                            if ctx.is_air(target) {
                                let random = ctx.random();
                                let age = random.next_int(3);
                                let state = BlockState::cocoa_default()
                                    .with_cocoa_age(age)
                                    .with_cocoa_facing(dir);
                                ctx.set_block(target, state);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

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

    impl CocoaContext for TestContext {
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
    fn decorator_type_is_cocoa() {
        assert_eq!(CocoaDecorator::new(1.0).decorator_type(), "cocoa");
    }

    #[test]
    fn probability_gate_blocks_placement() {
        let log = BlockPos::new(0, 64, 0);
        let air = vec![
            log.offset(-1, 0, 0),
            log.offset(1, 0, 0),
            log.offset(0, 0, -1),
            log.offset(0, 0, 1),
        ];
        let mut ctx = TestContext::new(0, vec![log], air);
        CocoaDecorator::new(0.0).place(&mut ctx);
        assert!(ctx.placed.borrow().is_empty());
    }

    #[test]
    fn places_only_on_logs_within_two_y_levels_of_first() {
        let logs = vec![
            BlockPos::new(0, 64, 0),
            BlockPos::new(0, 66, 0),
            BlockPos::new(0, 67, 0), // excluded: 67 - 64 > 2
        ];
        let mut air = Vec::new();
        for log in &logs {
            for dir in Direction::HORIZONTAL {
                let step = dir.get_opposite().step();
                air.push(log.offset(step.x, 0, step.z));
            }
        }
        let mut ctx = TestContext::new(123, logs, air);
        CocoaDecorator::new(1.0).place(&mut ctx);
        for (pos, _) in ctx.placed.borrow().iter() {
            assert_ne!(pos.y, 67);
        }
    }

    #[test]
    fn placed_cocoa_has_valid_age_and_facing() {
        let log = BlockPos::new(5, 70, -2);
        let mut air = Vec::new();
        for dir in Direction::HORIZONTAL {
            let step = dir.get_opposite().step();
            air.push(log.offset(step.x, 0, step.z));
        }
        let mut ctx = TestContext::new(42, vec![log], air);
        CocoaDecorator::new(1.0).place(&mut ctx);
        let placed = ctx.placed.borrow();
        for (_, state) in placed.iter() {
            assert_eq!(state.block, BlockState::COCOA_BLOCK_ID);
            let age = state.cocoa_age.unwrap();
            assert!(age >= 0 && age < 3);
            assert!(state.cocoa_facing.is_some());
        }
    }

    #[test]
    fn handles_negative_and_wrapping_coordinates() {
        let log = BlockPos::new(i32::MIN, -1, i32::MAX);
        let mut air = Vec::new();
        for dir in Direction::HORIZONTAL {
            let step = dir.get_opposite().step();
            air.push(log.offset(step.x, 0, step.z));
        }
        let mut ctx = TestContext::new(7, vec![log], air.clone());
        CocoaDecorator::new(1.0).place(&mut ctx);
        for (pos, _) in ctx.placed.borrow().iter() {
            assert!(air.contains(pos));
        }
    }

    #[test]
    fn deterministic_for_same_seed() {
        let logs = vec![BlockPos::new(0, 64, 0), BlockPos::new(0, 65, 0)];
        let mut air = Vec::new();
        for log in &logs {
            for dir in Direction::HORIZONTAL {
                let step = dir.get_opposite().step();
                air.push(log.offset(step.x, 0, step.z));
            }
        }
        let mut ctx1 = TestContext::new(9999, logs.clone(), air.clone());
        let mut ctx2 = TestContext::new(9999, logs, air);
        CocoaDecorator::new(1.0).place(&mut ctx1);
        CocoaDecorator::new(1.0).place(&mut ctx2);
        assert_eq!(*ctx1.placed.borrow(), *ctx2.placed.borrow());
    }
}
