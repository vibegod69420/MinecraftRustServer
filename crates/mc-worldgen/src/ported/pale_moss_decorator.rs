//! Port of `net.minecraft.world.level.levelgen.feature.treedecorators.PaleMossDecorator` (1.21.11).

use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.Vec3i#<init>
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#getY
    pub const fn get_y(self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub const fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#above
    pub const fn above(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }
}

/// Minimal random source matching the methods used by `PaleMossDecorator`.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    PaleHangingMoss,
}

/// Minimal block state representation for pale hanging moss placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub kind: BlockKind,
    pub tip: bool,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.Blocks#PALE_HANGING_MOSS defaultBlockState + setValue(HangingMossBlock.TIP,...)
    pub const fn pale_hanging_moss(tip: bool) -> Self {
        Self {
            kind: BlockKind::PaleHangingMoss,
            tip,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeDecoratorType {
    PaleMoss,
}

/// Minimal context matching `TreeDecorator.Context` methods used by `PaleMossDecorator`.
pub trait TreeDecoratorContext: RandomSource {
    fn logs(&self) -> &[BlockPos];
    fn leaves(&self) -> &[BlockPos];
    fn is_air(&self, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: BlockState);
    /// Places the `PALE_MOSS_PATCH` configured feature at `pos`, mirroring the
    /// registry lookup and `ConfiguredFeature.place(...)` call in Java.
    fn place_pale_moss_patch(&mut self, pos: BlockPos);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaleMossDecorator {
    pub leaves_probability: f32,
    pub trunk_probability: f32,
    pub ground_probability: f32,
}

impl PaleMossDecorator {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.PaleMossDecorator#<init>
    pub const fn new(leaves_probability: f32, trunk_probability: f32, ground_probability: f32) -> Self {
        Self {
            leaves_probability,
            trunk_probability,
            ground_probability,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.PaleMossDecorator#type
    pub const fn get_type(&self) -> TreeDecoratorType {
        TreeDecoratorType::PaleMoss
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.PaleMossDecorator#place
    pub fn place(&self, ctx: &mut impl TreeDecoratorContext) {
        let logs_copy: Vec<BlockPos> = ctx.logs().to_vec();
        let shuffled_logs = shuffled_copy(logs_copy, ctx);
        if !shuffled_logs.is_empty() {
            let lowest = *shuffled_logs
                .iter()
                .min_by_key(|p| p.get_y())
                .expect("shuffled_logs is not empty");

            if ctx.next_float() < self.ground_probability {
                ctx.place_pale_moss_patch(lowest.above());
            }

            let logs: Vec<BlockPos> = ctx.logs().to_vec();
            for log_pos in logs {
                if ctx.next_float() < self.trunk_probability {
                    let below = log_pos.below();
                    if ctx.is_air(below) {
                        Self::add_moss_hanger(below, ctx);
                    }
                }
            }

            let leaves: Vec<BlockPos> = ctx.leaves().to_vec();
            for leaf_pos in leaves {
                if ctx.next_float() < self.leaves_probability {
                    let below = leaf_pos.below();
                    if ctx.is_air(below) {
                        Self::add_moss_hanger(below, ctx);
                    }
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.PaleMossDecorator#addMossHanger
    fn add_moss_hanger(mut pos: BlockPos, ctx: &mut impl TreeDecoratorContext) {
        while ctx.is_air(pos.below()) && !(f64::from(ctx.next_float()) < 0.5_f64) {
            ctx.set_block(pos, BlockState::pale_hanging_moss(false));
            pos = pos.below();
        }
        ctx.set_block(pos, BlockState::pale_hanging_moss(true));
    }
}

// JAVA: net.minecraft.Util#shuffledCopy
fn shuffled_copy<T: Clone>(mut list: Vec<T>, rng: &mut impl RandomSource) -> Vec<T> {
    // JAVA: net.minecraft.Util#shuffle
    let size = list.len() as i32;
    let mut i = size;
    while i > 1 {
        let j = rng.next_int(i);
        list.swap(i.wrapping_sub(1) as usize, j as usize);
        i = i.wrapping_sub(1);
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        float_idx: usize,
        int_idx: usize,
    }

    impl MockRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                ints,
                float_idx: 0,
                int_idx: 0,
            }
        }

        fn next_float_val(&mut self) -> f32 {
            if self.floats.is_empty() {
                return 0.0;
            }
            let v = self.floats[self.float_idx % self.floats.len()];
            self.float_idx = self.float_idx.wrapping_add(1);
            v
        }

        fn next_int_val(&mut self, bound: i32) -> i32 {
            if self.ints.is_empty() || bound <= 0 {
                return 0;
            }
            let v = self.ints[self.int_idx % self.ints.len()];
            self.int_idx = self.int_idx.wrapping_add(1);
            v.rem_euclid(bound)
        }
    }

    struct MockContext {
        rng: MockRandom,
        logs_vec: Vec<BlockPos>,
        leaves_vec: Vec<BlockPos>,
        air_set: HashSet<(i32, i32, i32)>,
        placed: Vec<(BlockPos, BlockState)>,
        patch_positions: Vec<BlockPos>,
    }

    impl MockContext {
        fn new(rng: MockRandom, logs: Vec<BlockPos>, leaves: Vec<BlockPos>) -> Self {
            Self {
                rng,
                logs_vec: logs,
                leaves_vec: leaves,
                air_set: HashSet::new(),
                placed: Vec::new(),
                patch_positions: Vec::new(),
            }
        }

        fn with_air(mut self, positions: &[(i32, i32, i32)]) -> Self {
            for &(x, y, z) in positions {
                self.air_set.insert((x, y, z));
            }
            self
        }
    }

    impl RandomSource for MockContext {
        fn next_float(&mut self) -> f32 {
            self.rng.next_float_val()
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            self.rng.next_int_val(bound)
        }
    }

    impl TreeDecoratorContext for MockContext {
        fn logs(&self) -> &[BlockPos] {
            &self.logs_vec
        }

        fn leaves(&self) -> &[BlockPos] {
            &self.leaves_vec
        }

        fn is_air(&self, pos: BlockPos) -> bool {
            let key = (pos.x, pos.y, pos.z);
            if self
                .placed
                .iter()
                .any(|(p, _)| p.x == pos.x && p.y == pos.y && p.z == pos.z)
            {
                return false;
            }
            self.air_set.contains(&key)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState) {
            self.placed.push((pos, state));
        }

        fn place_pale_moss_patch(&mut self, pos: BlockPos) {
            self.patch_positions.push(pos);
        }
    }

    #[test]
    fn empty_logs_noop() {
        let mut ctx = MockContext::new(
            MockRandom::new(vec![0.0], vec![0]),
            vec![],
            vec![BlockPos::new(0, 5, 0)],
        )
        .with_air(&[(0, 4, 0)]);
        let decorator = PaleMossDecorator::new(1.0, 1.0, 1.0);
        decorator.place(&mut ctx);
        assert!(ctx.placed.is_empty());
        assert!(ctx.patch_positions.is_empty());
        assert_eq!(decorator.get_type(), TreeDecoratorType::PaleMoss);
    }

    #[test]
    fn ground_patch_at_lowest_log_above() {
        let mut ctx = MockContext::new(
            MockRandom::new(vec![0.0, 1.0, 1.0, 1.0, 1.0], vec![0, 0, 0]),
            vec![
                BlockPos::new(0, 10, 0),
                BlockPos::new(1, 5, 1),
                BlockPos::new(2, 8, 2),
            ],
            vec![],
        );
        let decorator = PaleMossDecorator::new(0.0, 0.0, 1.0);
        decorator.place(&mut ctx);
        assert_eq!(ctx.patch_positions, vec![BlockPos::new(1, 6, 1)]);

        let mut ctx2 = MockContext::new(
            MockRandom::new(vec![0.0], vec![0]),
            vec![BlockPos::new(0, 5, 0)],
            vec![],
        );
        let decorator2 = PaleMossDecorator::new(0.0, 0.0, 0.0);
        decorator2.place(&mut ctx2);
        assert!(ctx2.patch_positions.is_empty());
    }

    #[test]
    fn trunk_and_leaves_hangers_placed() {
        let mut ctx = MockContext::new(
            MockRandom::new(vec![0.0], vec![0]),
            vec![BlockPos::new(0, 5, 0)],
            vec![BlockPos::new(2, 6, 0)],
        )
        .with_air(&[(0, 4, 0), (2, 5, 0)]);
        let decorator = PaleMossDecorator::new(1.0, 1.0, 0.0);
        decorator.place(&mut ctx);
        assert_eq!(ctx.placed.len(), 2);
        assert_eq!(
            ctx.placed[0],
            (
                BlockPos::new(0, 4, 0),
                BlockState::pale_hanging_moss(true)
            )
        );
        assert_eq!(
            ctx.placed[1],
            (
                BlockPos::new(2, 5, 0),
                BlockState::pale_hanging_moss(true)
            )
        );
    }

    #[test]
    fn moss_hanger_extends_downward_and_stops() {
        // Floats consumed in order:
        // 1) ground check: 0.0 < 0.0 => false
        // 2) trunk check: 0.0 < 1.0 => true
        // 3) hanger while: !(0.6 < 0.5) => extend
        // 4) hanger while: !(0.7 < 0.5) => extend
        // 5) hanger while: !(0.3 < 0.5) => stop
        let mut ctx = MockContext::new(
            MockRandom::new(vec![0.0, 0.0, 0.6, 0.7, 0.3], vec![0]),
            vec![BlockPos::new(0, 5, 0)],
            vec![],
        )
        .with_air(&[(0, 4, 0), (0, 3, 0), (0, 2, 0), (0, 1, 0)]);
        let decorator = PaleMossDecorator::new(0.0, 1.0, 0.0);
        decorator.place(&mut ctx);
        assert_eq!(
            ctx.placed,
            vec![
                (BlockPos::new(0, 4, 0), BlockState::pale_hanging_moss(false)),
                (BlockPos::new(0, 3, 0), BlockState::pale_hanging_moss(false)),
                (BlockPos::new(0, 2, 0), BlockState::pale_hanging_moss(true)),
            ]
        );
    }

    #[test]
    fn block_pos_negative_and_wrapping() {
        let p = BlockPos::new(-10, -20, -30);
        assert_eq!(p.below(), BlockPos::new(-10, -21, -30));
        assert_eq!(p.above(), BlockPos::new(-10, -19, -30));
        assert_eq!(p.get_y(), -20);

        let min = BlockPos::new(i32::MIN, i32::MIN, i32::MIN);
        assert_eq!(min.below().y, i32::MAX);

        let max = BlockPos::new(i32::MAX, i32::MAX, i32::MAX);
        assert_eq!(max.above().y, i32::MIN);
    }

    #[test]
    fn hanger_float_boundary_bits_and_no_air() {
        let half = f32::from_bits(0x3F00_0000);
        assert!(!(f64::from(half) < 0.5_f64));
        let just_below = f32::from_bits(0x3EFF_FFFF);
        assert!(f64::from(just_below) < 0.5_f64);

        // Exactly 0.5 should extend: !(0.5 < 0.5) == true
        let mut ctx = MockContext::new(
            MockRandom::new(vec![0.0, 0.0, 0.5, 0.4], vec![0]),
            vec![BlockPos::new(0, 5, 0)],
            vec![],
        )
        .with_air(&[(0, 4, 0), (0, 3, 0), (0, 2, 0)]);
        let decorator = PaleMossDecorator::new(0.0, 1.0, 0.0);
        decorator.place(&mut ctx);
        assert_eq!(
            ctx.placed,
            vec![
                (BlockPos::new(0, 4, 0), BlockState::pale_hanging_moss(false)),
                (BlockPos::new(0, 3, 0), BlockState::pale_hanging_moss(true)),
            ]
        );

        // No air below log -> no hanger placed
        let mut ctx2 = MockContext::new(
            MockRandom::new(vec![0.0, 0.0], vec![0]),
            vec![BlockPos::new(0, 5, 0)],
            vec![],
        );
        decorator.place(&mut ctx2);
        assert!(ctx2.placed.is_empty());
    }
}
