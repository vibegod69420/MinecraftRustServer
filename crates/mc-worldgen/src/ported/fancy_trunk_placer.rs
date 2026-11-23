// === FILE: fancy_trunk_placer.rs ===

use std::f64::consts::PI;

pub const TRUNK_HEIGHT_SCALE: f64 = 0.618;
pub const CLUSTER_DENSITY_MAGIC: f64 = 1.382;
pub const BRANCH_SLOPE: f64 = 0.381;
pub const BRANCH_LENGTH_MAGIC: f64 = 0.328;

#[inline]
fn mth_floor_f64(value: f64) -> i32 {
    let i = value as i32;
    if value < i as f64 {
        i.wrapping_sub(1)
    } else {
        i
    }
}

#[inline]
fn mth_floor_f32(value: f32) -> i32 {
    let i = value as i32;
    if value < i as f32 {
        i.wrapping_sub(1)
    } else {
        i
    }
}

#[inline]
fn mth_abs(value: i32) -> i32 {
    if value >= 0 {
        value
    } else {
        value.wrapping_neg()
    }
}

#[inline]
fn java_math_abs(value: i32) -> i32 {
    if value >= 0 {
        value
    } else {
        value.wrapping_neg()
    }
}

#[inline]
fn mth_sqrt(value: f32) -> f32 {
    (f64::from(value)).sqrt() as f32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    pub fn above(&self, n: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(n),
            z: self.z,
        }
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoliageAttachment {
    pub pos: BlockPos,
    pub radius_offset: i32,
    pub double_trunk: bool,
}

impl FoliageAttachment {
    pub fn new(pos: BlockPos, radius_offset: i32, double_trunk: bool) -> Self {
        Self {
            pos,
            radius_offset,
            double_trunk,
        }
    }

    pub fn pos(&self) -> BlockPos {
        self.pos
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoliageCoords {
    pub attachment: FoliageAttachment,
    branch_base: i32,
}

impl FoliageCoords {
    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer$FoliageCoords#<init>
    pub fn new(pos: BlockPos, branch_base: i32) -> Self {
        Self {
            attachment: FoliageAttachment::new(pos, 0, false),
            branch_base,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer$FoliageCoords#getBranchBase
    pub fn get_branch_base(&self) -> i32 {
        self.branch_base
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrunkPlacerType {
    FancyTrunkPlacer,
}

/// Minimal abstraction over the world/random/config interactions required by
/// `FancyTrunkPlacer`. This keeps the module self-contained while preserving
/// the Java algorithm's call order and semantics.
pub trait TreePlacerContext {
    fn next_float(&mut self) -> f32;
    fn set_dirt_at(&mut self, pos: BlockPos);
    fn place_log(&mut self, pos: BlockPos, axis: DirectionAxis) -> bool;
    fn is_free(&self, pos: BlockPos) -> bool;
}

#[derive(Clone, Debug)]
pub struct FancyTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl FancyTrunkPlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#<init>
    pub fn new(base_height: i32, height_rand_a: i32, height_rand_b: i32) -> Self {
        Self {
            base_height,
            height_rand_a,
            height_rand_b,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#type
    pub fn placer_type(&self) -> TrunkPlacerType {
        TrunkPlacerType::FancyTrunkPlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#placeTrunk
    pub fn place_trunk<C: TreePlacerContext>(
        &self,
        ctx: &mut C,
        free_tree_height: i32,
        pos: BlockPos,
    ) -> Vec<FoliageAttachment> {
        let _j6: i32 = 5;
        let j7: i32 = free_tree_height.wrapping_add(2);
        let j8: i32 = mth_floor_f64(f64::from(j7) * TRUNK_HEIGHT_SCALE);
        ctx.set_dirt_at(pos.below());
        let _j9: f64 = 1.0;
        let j10: i32 = 1i32.min(mth_floor_f64(
            CLUSTER_DENSITY_MAGIC + (1.0 * f64::from(j7) / 13.0).powf(2.0),
        ));
        let j11: i32 = pos.get_y().wrapping_add(j8);
        let mut j12: i32 = j7.wrapping_sub(5);
        let mut foliage_coords: Vec<FoliageCoords> = Vec::new();
        foliage_coords.push(FoliageCoords::new(pos.above(j12), j11));

        while j12 >= 0 {
            let j14: f32 = Self::tree_shape(j7, j12);
            if !(j14 < 0.0f32) {
                let mut j15: i32 = 0;
                while j15 < j10 {
                    let _j16: f64 = 1.0;
                    let j17: f64 = 1.0
                        * f64::from(j14)
                        * (f64::from(ctx.next_float()) + BRANCH_LENGTH_MAGIC);
                    let j18: f64 = f64::from(ctx.next_float() * 2.0f32) * PI;
                    let j19: f64 = j17 * j18.sin() + 0.5;
                    let j20: f64 = j17 * j18.cos() + 0.5;
                    let j21: BlockPos = pos.offset(
                        mth_floor_f64(j19),
                        j12.wrapping_sub(1),
                        mth_floor_f64(j20),
                    );
                    let j22: BlockPos = j21.above(5);
                    if self.make_limb(ctx, j21, j22, false) {
                        let j23: i32 = pos.get_x().wrapping_sub(j21.get_x());
                        let j24: i32 = pos.get_z().wrapping_sub(j21.get_z());
                        let dist_sq: i32 =
                            j23.wrapping_mul(j23).wrapping_add(j24.wrapping_mul(j24));
                        let j25: f64 =
                            f64::from(j21.get_y()) - f64::from(dist_sq).sqrt() * BRANCH_SLOPE;
                        let j26: i32 = if j25 > f64::from(j11) {
                            j11
                        } else {
                            j25 as i32
                        };
                        let j27: BlockPos = BlockPos::new(pos.get_x(), j26, pos.get_z());
                        if self.make_limb(ctx, j27, j21, false) {
                            foliage_coords.push(FoliageCoords::new(j21, j27.get_y()));
                        }
                    }
                    j15 = j15.wrapping_add(1);
                }
            }
            j12 = j12.wrapping_sub(1);
        }

        self.make_limb(ctx, pos, pos.above(j8), true);
        self.make_branches(ctx, j7, pos, &foliage_coords);
        let mut attachments: Vec<FoliageAttachment> = Vec::new();

        for coord in &foliage_coords {
            if self.trim_branches(j7, coord.get_branch_base().wrapping_sub(pos.get_y())) {
                attachments.push(coord.attachment);
            }
        }

        attachments
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#makeLimb
    fn make_limb<C: TreePlacerContext>(
        &self,
        ctx: &mut C,
        start: BlockPos,
        end: BlockPos,
        is_trunk: bool,
    ) -> bool {
        if !is_trunk && start == end {
            return true;
        }

        let diff: BlockPos = end.offset(
            start.get_x().wrapping_neg(),
            start.get_y().wrapping_neg(),
            start.get_z().wrapping_neg(),
        );
        let steps: i32 = self.get_steps(diff);
        let dx: f32 = (diff.get_x() as f32) / (steps as f32);
        let dy: f32 = (diff.get_y() as f32) / (steps as f32);
        let dz: f32 = (diff.get_z() as f32) / (steps as f32);

        let mut i: i32 = 0;
        while i <= steps {
            let log_pos: BlockPos = start.offset(
                mth_floor_f32(0.5f32 + (i as f32) * dx),
                mth_floor_f32(0.5f32 + (i as f32) * dy),
                mth_floor_f32(0.5f32 + (i as f32) * dz),
            );
            if is_trunk {
                ctx.place_log(log_pos, self.get_log_axis(start, log_pos));
            } else if !ctx.is_free(log_pos) {
                return false;
            }
            i = i.wrapping_add(1);
        }

        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#getSteps
    fn get_steps(&self, pos: BlockPos) -> i32 {
        let x: i32 = mth_abs(pos.get_x());
        let y: i32 = mth_abs(pos.get_y());
        let z: i32 = mth_abs(pos.get_z());
        x.max(y.max(z))
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#getLogAxis
    fn get_log_axis(&self, start: BlockPos, end: BlockPos) -> DirectionAxis {
        let mut axis: DirectionAxis = DirectionAxis::Y;
        let dx: i32 = java_math_abs(end.get_x().wrapping_sub(start.get_x()));
        let dz: i32 = java_math_abs(end.get_z().wrapping_sub(start.get_z()));
        let max_horiz: i32 = dx.max(dz);
        if max_horiz > 0 {
            if dx == max_horiz {
                axis = DirectionAxis::X;
            } else {
                axis = DirectionAxis::Z;
            }
        }
        axis
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#trimBranches
    fn trim_branches(&self, tree_height: i32, branch_y_offset: i32) -> bool {
        f64::from(branch_y_offset) >= f64::from(tree_height) * 0.2
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#makeBranches
    fn make_branches<C: TreePlacerContext>(
        &self,
        ctx: &mut C,
        tree_height: i32,
        pos: BlockPos,
        foliage_coords: &[FoliageCoords],
    ) {
        for coord in foliage_coords {
            let branch_base: i32 = coord.get_branch_base();
            let base_pos: BlockPos = BlockPos::new(pos.get_x(), branch_base, pos.get_z());
            if base_pos != coord.attachment.pos()
                && self.trim_branches(tree_height, branch_base.wrapping_sub(pos.get_y()))
            {
                self.make_limb(ctx, base_pos, coord.attachment.pos(), true);
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer#treeShape
    fn tree_shape(tree_height: i32, branch_y: i32) -> f32 {
        if (branch_y as f32) < (tree_height as f32) * 0.3f32 {
            return -1.0f32;
        }

        let radius: f32 = (tree_height as f32) / 2.0f32;
        let offset: f32 = radius - (branch_y as f32);
        let mut shape: f32 = mth_sqrt(radius * radius - offset * offset);
        if offset == 0.0f32 {
            shape = radius;
        } else if offset.abs() >= radius {
            return 0.0f32;
        }

        shape * 0.5f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCtx {
        floats: Vec<f32>,
        idx: usize,
        dirt: Vec<BlockPos>,
        logs: Vec<(BlockPos, DirectionAxis)>,
        blocked: std::collections::HashSet<BlockPos>,
    }

    impl MockCtx {
        fn new(floats: Vec<f32>) -> Self {
            Self {
                floats,
                idx: 0,
                dirt: Vec::new(),
                logs: Vec::new(),
                blocked: std::collections::HashSet::new(),
            }
        }
    }

    impl TreePlacerContext for MockCtx {
        fn next_float(&mut self) -> f32 {
            let v = self.floats.get(self.idx).copied().unwrap_or(0.0);
            self.idx += 1;
            v
        }

        fn set_dirt_at(&mut self, pos: BlockPos) {
            self.dirt.push(pos);
        }

        fn place_log(&mut self, pos: BlockPos, axis: DirectionAxis) -> bool {
            self.logs.push((pos, axis));
            true
        }

        fn is_free(&self, pos: BlockPos) -> bool {
            !self.blocked.contains(&pos)
        }
    }

    #[test]
    fn test_tree_shape_and_float_bits() {
        // Below threshold: -1.0
        assert_eq!(
            FancyTrunkPlacer::tree_shape(10, 2).to_bits(),
            (-1.0f32).to_bits()
        );
        // At center: offset == 0 => shape = radius, return radius * 0.5
        let center = FancyTrunkPlacer::tree_shape(12, 6);
        assert_eq!(center.to_bits(), 3.0f32.to_bits());
        // Normal case: exact float bits
        let s = FancyTrunkPlacer::tree_shape(10, 3);
        let expected = mth_sqrt(21.0f32) * 0.5f32;
        assert_eq!(s.to_bits(), expected.to_bits());
        // abs(offset) >= radius => 0.0
        let edge = FancyTrunkPlacer::tree_shape(10, 10);
        assert_eq!(edge.to_bits(), 0.0f32.to_bits());
        // Negative branch_y
        assert_eq!(
            FancyTrunkPlacer::tree_shape(10, -1).to_bits(),
            (-1.0f32).to_bits()
        );
    }

    #[test]
    fn test_mth_helpers_negative_and_overflow() {
        assert_eq!(mth_floor_f64(2.7), 2);
        assert_eq!(mth_floor_f64(-2.7), -3);
        assert_eq!(mth_floor_f64(0.0), 0);
        assert_eq!(mth_floor_f32(-0.1f32), -1);
        assert_eq!(mth_floor_f32(0.5f32), 0);
        // Java Math.abs(Integer.MIN_VALUE) == Integer.MIN_VALUE
        assert_eq!(mth_abs(i32::MIN), i32::MIN);
        assert_eq!(java_math_abs(i32::MIN), i32::MIN);
        assert_eq!(mth_abs(-42), 42);
    }

    #[test]
    fn test_make_limb_axis_and_blocked() {
        let placer = FancyTrunkPlacer::new(0, 0, 0);

        // Trunk limb places logs along Y
        let mut ctx = MockCtx::new(vec![]);
        let ok = placer.make_limb(
            &mut ctx,
            BlockPos::new(0, 0, 0),
            BlockPos::new(0, 3, 0),
            true,
        );
        assert!(ok);
        assert_eq!(ctx.logs.len(), 4);
        assert!(ctx.logs.iter().all(|(_, a)| *a == DirectionAxis::Y));

        // Axis selection
        assert_eq!(
            placer.get_log_axis(BlockPos::new(0, 0, 0), BlockPos::new(5, 0, 0)),
            DirectionAxis::X
        );
        assert_eq!(
            placer.get_log_axis(BlockPos::new(0, 0, 0), BlockPos::new(0, 0, 5)),
            DirectionAxis::Z
        );
        assert_eq!(
            placer.get_log_axis(BlockPos::new(0, 0, 0), BlockPos::new(0, 5, 0)),
            DirectionAxis::Y
        );
        assert_eq!(
            placer.get_log_axis(BlockPos::new(0, 0, 0), BlockPos::new(1, 0, 0)),
            DirectionAxis::X
        );

        // Blocked path for non-trunk limb
        let mut ctx2 = MockCtx::new(vec![]);
        ctx2.blocked.insert(BlockPos::new(0, 2, 0));
        let ok2 = placer.make_limb(
            &mut ctx2,
            BlockPos::new(0, 0, 0),
            BlockPos::new(0, 3, 0),
            false,
        );
        assert!(!ok2);
    }

    #[test]
    fn test_place_trunk_deterministic() {
        let placer = FancyTrunkPlacer::new(0, 0, 0);
        let mut ctx = MockCtx::new(vec![0.0f32; 64]);
        let attachments = placer.place_trunk(&mut ctx, 5, BlockPos::new(0, 0, 0));
        // free_tree_height=5: j7=7, j8=4, no side branches (treeShape < 0 for y=0..2)
        // trunk limb (0,0,0)->(0,4,0), branch limb (0,4,0)->(0,2,0)
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].pos(), BlockPos::new(0, 2, 0));
        assert_eq!(attachments[0].radius_offset, 0);
        assert!(!attachments[0].double_trunk);
        assert!(ctx.dirt.contains(&BlockPos::new(0, -1, 0)));
        assert!(!ctx.logs.is_empty());
    }

    #[test]
    fn test_negative_inputs_and_overflow() {
        let placer = FancyTrunkPlacer::new(0, 0, 0);
        // Negative free_tree_height should not panic
        let mut ctx = MockCtx::new(vec![0.5f32; 128]);
        let _ = placer.place_trunk(&mut ctx, -3, BlockPos::new(0, 10, 0));

        // getSteps with i32::MIN: Math.abs(MIN) wraps to MIN in Java, max(MIN,0,0)==0
        let steps_min = placer.get_steps(BlockPos::new(i32::MIN, 0, 0));
        assert_eq!(steps_min, 0);

        // Negative coordinates in getLogAxis
        let axis = placer.get_log_axis(BlockPos::new(5, 0, 0), BlockPos::new(-5, 0, 0));
        assert_eq!(axis, DirectionAxis::X);
    }
}
