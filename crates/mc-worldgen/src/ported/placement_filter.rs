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

pub trait PlacementFilter {
    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementFilter#shouldPlace
    fn should_place(&self, ctx: &PlacementContext, random: &mut dyn RandomSource, pos: BlockPos) -> bool;

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementFilter#getPositions
    fn get_positions(
        &self,
        ctx: &PlacementContext,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        if self.should_place(ctx, random, pos) {
            vec![pos]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Always(bool);
    struct MockRandom {
        float_val: f32,
    }
    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
        fn next_float(&mut self) -> f32 {
            self.float_val
        }
    }

    impl PlacementFilter for Always {
        fn should_place(&self, _ctx: &PlacementContext, _random: &mut dyn RandomSource, _pos: BlockPos) -> bool {
            self.0
        }
    }

    #[test]
    fn returns_pos_when_allowed() {
        let f = Always(true);
        let mut rng = MockRandom { float_val: 0.0 };
        let pos = BlockPos::new(10, -3, 20);
        let out = f.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(out, vec![pos]);
    }

    #[test]
    fn returns_empty_when_rejected() {
        let f = Always(false);
        let mut rng = MockRandom { float_val: 1.0 };
        let pos = BlockPos::new(0, 0, 0);
        let out = f.get_positions(&PlacementContext, &mut rng, pos);
        assert!(out.is_empty());
    }

    #[test]
    fn preserves_negative_and_extreme_coords() {
        let f = Always(true);
        let mut rng = MockRandom { float_val: 0.5 };
        let pos = BlockPos::new(i32::MIN, i32::MAX, -1);
        let out = f.get_positions(&PlacementContext, &mut rng, pos);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], pos);
    }
}
