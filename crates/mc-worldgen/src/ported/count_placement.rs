use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(self) -> i32 {
        self.x
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn get_z(self) -> i32 {
        self.z
    }
}

pub trait RandomSource: Debug {
    fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantInt {
    value: i32,
}

impl ConstantInt {
    pub fn of(value: i32) -> Self {
        Self { value }
    }

    pub fn sample(self, _random: &mut dyn RandomSource) -> i32 {
        self.value
    }

    pub fn get_value(self) -> i32 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProvider {
    Constant(ConstantInt),
}

impl IntProvider {
    pub fn sample(self, random: &mut dyn RandomSource) -> i32 {
        match self {
            IntProvider::Constant(c) => c.sample(random),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifierType {
    Count,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlacementContext;

pub trait RepeatingPlacement {
    fn count(&self, random: &mut dyn RandomSource, pos: BlockPos) -> i32;

    // JAVA: net.minecraft.world.level.levelgen.placement.RepeatingPlacement#getPositions
    fn get_positions(
        &self,
        _ctx: &PlacementContext,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        let n = self.count(random, pos);
        if n <= 0 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(n as usize);
        for _ in 0..n {
            out.push(pos);
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountPlacement {
    count: IntProvider,
}

impl CountPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.CountPlacement#<init>
    fn new(count: IntProvider) -> Self {
        Self { count }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountPlacement#of
    pub fn of_provider(count: IntProvider) -> Self {
        Self::new(count)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountPlacement#of
    pub fn of(count: i32) -> Self {
        Self::of_provider(IntProvider::Constant(ConstantInt::of(count)))
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountPlacement#type
    pub fn placement_type(&self) -> PlacementModifierType {
        PlacementModifierType::Count
    }
}

impl RepeatingPlacement for CountPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.CountPlacement#count
    fn count(&self, random: &mut dyn RandomSource, _pos: BlockPos) -> i32 {
        self.count.sample(random)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FixedRandom(i32);

    impl RandomSource for FixedRandom {
        fn next_int_between_inclusive(&mut self, _min: i32, _max: i32) -> i32 {
            self.0
        }
    }

    #[test]
    fn count_constant() {
        let cp = CountPlacement::of(5);
        let mut rng = FixedRandom(0);
        assert_eq!(cp.count(&mut rng, BlockPos::new(1, 2, 3)), 5);
    }

    #[test]
    fn count_zero_returns_empty_positions() {
        let cp = CountPlacement::of(0);
        let mut rng = FixedRandom(0);
        let positions = cp.get_positions(&PlacementContext, &mut rng, BlockPos::new(10, 20, 30));
        assert!(positions.is_empty());
    }

    #[test]
    fn count_negative_returns_empty_positions() {
        let cp = CountPlacement::of(-7);
        let mut rng = FixedRandom(0);
        let positions = cp.get_positions(&PlacementContext, &mut rng, BlockPos::new(0, 0, 0));
        assert!(positions.is_empty());
    }

    #[test]
    fn repeating_returns_same_pos_n_times() {
        let cp = CountPlacement::of(4);
        let mut rng = FixedRandom(0);
        let pos = BlockPos::new(7, -8, 9);
        let positions = cp.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(positions, vec![pos, pos, pos, pos]);
    }

    #[test]
    fn of_provider_uses_provider_sample() {
        let cp = CountPlacement::of_provider(IntProvider::Constant(ConstantInt::of(2)));
        let mut rng = FixedRandom(99);
        assert_eq!(cp.count(&mut rng, BlockPos::new(0, 0, 0)), 2);
    }

    #[test]
    fn placement_type_is_count() {
        let cp = CountPlacement::of(1);
        assert_eq!(cp.placement_type(), PlacementModifierType::Count);
    }
}
