//! Port of `net.minecraft.world.level.levelgen.feature.treedecorators.TrunkVineDecorator`.

use std::cell::RefCell;
use std::rc::Rc;

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
    pub const fn west(self) -> Self {
        Self {
            x: self.x.wrapping_sub(1),
            y: self.y,
            z: self.z,
        }
    }

    #[inline]
    pub const fn east(self) -> Self {
        Self {
            x: self.x.wrapping_add(1),
            y: self.y,
            z: self.z,
        }
    }

    #[inline]
    pub const fn north(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z.wrapping_sub(1),
        }
    }

    #[inline]
    pub const fn south(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z.wrapping_add(1),
        }
    }
}

/// Vine facing property used by `TrunkVineDecorator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VineFacing {
    East,
    West,
    South,
    North,
}

/// Minimal random source matching the Java calls used by this class.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Minimal tree decorator context for trunk vine placement.
pub trait TrunkVineContext {
    type Random: RandomSource;

    fn random(&mut self) -> &mut Self::Random;
    fn logs(&self) -> Vec<BlockPos>;
    fn is_air(&self, pos: BlockPos) -> bool;
    fn place_vine(&mut self, pos: BlockPos, facing: VineFacing);
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

/// Port of `net.minecraft.world.level.levelgen.feature.treedecorators.TrunkVineDecorator`.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrunkVineDecorator;

impl TrunkVineDecorator {
    pub const INSTANCE: TrunkVineDecorator = TrunkVineDecorator;

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TrunkVineDecorator#type
    pub fn decorator_type(&self) -> &'static str {
        "trunk_vine"
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TrunkVineDecorator#place
    pub fn place<C: TrunkVineContext>(&self, ctx: &mut C) {
        let logs = ctx.logs();
        for log_pos in logs {
            let random = ctx.random();
            if random.next_int(3) > 0 {
                let pos = log_pos.west();
                if ctx.is_air(pos) {
                    ctx.place_vine(pos, VineFacing::East);
                }
            }

            let random = ctx.random();
            if random.next_int(3) > 0 {
                let pos = log_pos.east();
                if ctx.is_air(pos) {
                    ctx.place_vine(pos, VineFacing::West);
                }
            }

            let random = ctx.random();
            if random.next_int(3) > 0 {
                let pos = log_pos.north();
                if ctx.is_air(pos) {
                    ctx.place_vine(pos, VineFacing::South);
                }
            }

            let random = ctx.random();
            if random.next_int(3) > 0 {
                let pos = log_pos.south();
                if ctx.is_air(pos) {
                    ctx.place_vine(pos, VineFacing::North);
                }
            }
        }
    }
}

/// Test context implementation.
#[derive(Debug)]
struct TestContext {
    random: SimpleRandomSource,
    logs: Vec<BlockPos>,
    air: Vec<BlockPos>,
    placed: Rc<RefCell<Vec<(BlockPos, VineFacing)>>>,
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

impl TrunkVineContext for TestContext {
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

    fn place_vine(&mut self, pos: BlockPos, facing: VineFacing) {
        self.placed.borrow_mut().push((pos, facing));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decorator_type_is_trunk_vine() {
        assert_eq!(TrunkVineDecorator::INSTANCE.decorator_type(), "trunk_vine");
    }

    #[test]
    fn places_vines_only_in_air_with_expected_facing() {
        let log = BlockPos::new(0, 64, 0);
        let air = vec![log.west(), log.south()];
        let mut ctx = TestContext::new(1, vec![log], air);
        TrunkVineDecorator::INSTANCE.place(&mut ctx);
        let placed = ctx.placed.borrow().clone();
        for (pos, facing) in &placed {
            match facing {
                VineFacing::East => assert_eq!(*pos, log.west()),
                VineFacing::West => assert_eq!(*pos, log.east()),
                VineFacing::South => assert_eq!(*pos, log.north()),
                VineFacing::North => assert_eq!(*pos, log.south()),
            }
            assert!(ctx.air.contains(pos));
        }
    }

    #[test]
    fn no_placement_when_not_air() {
        let log = BlockPos::new(10, 70, -5);
        let mut ctx = TestContext::new(42, vec![log], vec![]);
        TrunkVineDecorator::INSTANCE.place(&mut ctx);
        assert!(ctx.placed.borrow().is_empty());
    }

    #[test]
    fn handles_negative_and_wrapping_positions() {
        let log = BlockPos::new(i32::MIN, -1, i32::MAX);
        let air = vec![log.west(), log.east(), log.north(), log.south()];
        let mut ctx = TestContext::new(7, vec![log], air.clone());
        TrunkVineDecorator::INSTANCE.place(&mut ctx);
        for (pos, _) in ctx.placed.borrow().iter() {
            assert!(air.contains(pos));
        }
    }

    #[test]
    fn deterministic_for_same_seed() {
        let logs = vec![BlockPos::new(1, 2, 3), BlockPos::new(2, 2, 3)];
        let air = vec![
            BlockPos::new(0, 2, 3),
            BlockPos::new(2, 2, 3),
            BlockPos::new(1, 2, 2),
            BlockPos::new(1, 2, 4),
            BlockPos::new(3, 2, 3),
            BlockPos::new(2, 2, 2),
            BlockPos::new(2, 2, 4),
        ];
        let mut ctx1 = TestContext::new(12345, logs.clone(), air.clone());
        let mut ctx2 = TestContext::new(12345, logs, air);
        TrunkVineDecorator::INSTANCE.place(&mut ctx1);
        TrunkVineDecorator::INSTANCE.place(&mut ctx2);
        assert_eq!(*ctx1.placed.borrow(), *ctx2.placed.borrow());
    }
}
