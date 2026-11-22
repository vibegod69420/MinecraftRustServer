use std::iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlacementContext;

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

pub trait RepeatingPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.RepeatingPlacement#count
    fn count(&self, random: &mut dyn RandomSource, pos: BlockPos) -> i32;

    // JAVA: net.minecraft.world.level.levelgen.placement.RepeatingPlacement#getPositions
    fn get_positions(
        &self,
        _ctx: &PlacementContext,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        let n = self.count(random, pos);
        let take_count = if n > 0 { n as usize } else { 0usize };
        iter::repeat(pos).take(take_count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedCount(i32);

    struct MockRandom;
    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    impl RepeatingPlacement for FixedCount {
        fn count(&self, _random: &mut dyn RandomSource, _pos: BlockPos) -> i32 {
            self.0
        }
    }

    #[test]
    fn repeats_exact_count() {
        let p = FixedCount(4);
        let mut rng = MockRandom;
        let pos = BlockPos::new(1, 2, 3);
        let out = p.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(out, vec![pos, pos, pos, pos]);
    }

    #[test]
    fn zero_count_returns_empty() {
        let p = FixedCount(0);
        let mut rng = MockRandom;
        let pos = BlockPos::new(-5, 64, 7);
        assert!(p.get_positions(&PlacementContext, &mut rng, pos).is_empty());
    }

    #[test]
    fn negative_count_returns_empty() {
        let p = FixedCount(-10);
        let mut rng = MockRandom;
        let pos = BlockPos::new(i32::MIN, -1, i32::MAX);
        assert!(p.get_positions(&PlacementContext, &mut rng, pos).is_empty());
    }

    #[test]
    fn preserves_position_values() {
        let p = FixedCount(2);
        let mut rng = MockRandom;
        let pos = BlockPos::new(i32::MIN, -64, i32::MAX);
        let out = p.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].x, i32::MIN);
        assert_eq!(out[0].y, -64);
        assert_eq!(out[0].z, i32::MAX);
    }
}
