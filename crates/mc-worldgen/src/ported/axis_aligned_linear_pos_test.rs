#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn positive_step(self) -> (i32, i32, i32) {
        match self {
            Axis::X => (1, 0, 0),
            Axis::Y => (0, 1, 0),
            Axis::Z => (0, 0, 1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(int,int,int)
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
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PosRuleTestType {
    AxisAlignedLinearPosTest,
}

// JAVA: net.minecraft.util.Mth#inverseLerp
fn mth_inverse_lerp(value: f32, start: f32, end: f32) -> f32 {
    (value - start) / (end - start)
}

// JAVA: net.minecraft.util.Mth#clampedLerp
fn mth_clamped_lerp(start: f32, end: f32, delta: f32) -> f32 {
    if delta < 0.0f32 {
        start
    } else if delta > 1.0f32 {
        end
    } else {
        start + delta * (end - start)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisAlignedLinearPosTest {
    min_chance: f32,
    max_chance: f32,
    min_dist: i32,
    max_dist: i32,
    axis: Axis,
}

impl AxisAlignedLinearPosTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.AxisAlignedLinearPosTest#<init>
    pub fn new(min_chance: f32, max_chance: f32, min_dist: i32, max_dist: i32, axis: Axis) -> Self {
        if min_dist >= max_dist {
            panic!("Invalid range: [{},{}]", min_dist, max_dist);
        }
        Self {
            min_chance,
            max_chance,
            min_dist,
            max_dist,
            axis,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.AxisAlignedLinearPosTest#test
    pub fn test(
        &self,
        _local_pos: BlockPos,
        relative_pos: BlockPos,
        origin: BlockPos,
        random: &mut dyn RandomSource,
    ) -> bool {
        let (sx, sy, sz) = self.axis.positive_step();
        let dx = relative_pos
            .get_x()
            .wrapping_sub(origin.get_x())
            .wrapping_mul(sx)
            .wrapping_abs() as f32;
        let dy = relative_pos
            .get_y()
            .wrapping_sub(origin.get_y())
            .wrapping_mul(sy)
            .wrapping_abs() as f32;
        let dz = relative_pos
            .get_z()
            .wrapping_sub(origin.get_z())
            .wrapping_mul(sz)
            .wrapping_abs() as f32;
        let dist = (dx + dy + dz) as i32;
        let r = random.next_float();
        let chance = mth_clamped_lerp(
            self.min_chance,
            self.max_chance,
            mth_inverse_lerp(dist as f32, self.min_dist as f32, self.max_dist as f32),
        );
        r <= chance
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.AxisAlignedLinearPosTest#getType
    pub fn get_type(&self) -> PosRuleTestType {
        PosRuleTestType::AxisAlignedLinearPosTest
    }

    pub fn min_chance(&self) -> f32 {
        self.min_chance
    }
    pub fn max_chance(&self) -> f32 {
        self.max_chance
    }
    pub fn min_dist(&self) -> i32 {
        self.min_dist
    }
    pub fn max_dist(&self) -> i32 {
        self.max_dist
    }
    pub fn axis(&self) -> Axis {
        self.axis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(f32);
    impl RandomSource for FixedRandom {
        fn next_float(&mut self) -> f32 {
            self.0
        }
    }

    #[test]
    #[should_panic(expected = "Invalid range: [5,2]")]
    fn invalid_range_panics() {
        let _ = AxisAlignedLinearPosTest::new(0.0, 1.0, 5, 2, Axis::Y);
    }

    #[test]
    #[should_panic(expected = "Invalid range: [3,3]")]
    fn equal_range_panics() {
        let _ = AxisAlignedLinearPosTest::new(0.0, 1.0, 3, 3, Axis::X);
    }

    #[test]
    fn y_axis_distance_controls_chance() {
        let rule = AxisAlignedLinearPosTest::new(0.0, 1.0, 0, 10, Axis::Y);
        let origin = BlockPos::new(0, 0, 0);
        let near = BlockPos::new(100, 0, 100); // y distance 0
        let far = BlockPos::new(0, 10, 0); // y distance 10
        let mut r0 = FixedRandom(0.0);
        let mut r1 = FixedRandom(0.5);
        assert!(rule.test(BlockPos::default(), near, origin, &mut r0));
        assert!(rule.test(BlockPos::default(), far, origin, &mut r1)); // at maxDist, chance=1.0
        let mid = BlockPos::new(0, 5, 0);
        let mut r_low = FixedRandom(0.4);
        let mut r_high = FixedRandom(0.6);
        assert!(rule.test(BlockPos::default(), mid, origin, &mut r_low));
        assert!(!rule.test(BlockPos::default(), mid, origin, &mut r_high));
    }

    #[test]
    fn only_selected_axis_matters() {
        let rule = AxisAlignedLinearPosTest::new(0.0, 1.0, 0, 10, Axis::X);
        let origin = BlockPos::new(0, 0, 0);
        let p = BlockPos::new(5, 999, -999);
        let mut r = FixedRandom(0.49);
        assert!(rule.test(BlockPos::default(), p, origin, &mut r));
        let mut r2 = FixedRandom(0.51);
        assert!(!rule.test(BlockPos::default(), p, origin, &mut r2));
    }

    #[test]
    fn negative_coords_and_wrapping_abs() {
        let rule = AxisAlignedLinearPosTest::new(0.25, 0.75, 0, 4, Axis::Z);
        let origin = BlockPos::new(0, 0, -2);
        let p = BlockPos::new(0, 0, 2); // abs((2 - (-2))*1) = 4
        let mut r = FixedRandom(0.75);
        assert!(rule.test(BlockPos::default(), p, origin, &mut r));
        let mut r2 = FixedRandom(0.76);
        assert!(!rule.test(BlockPos::default(), p, origin, &mut r2));
    }

    #[test]
    fn get_type_and_accessors() {
        let rule = AxisAlignedLinearPosTest::new(0.1, 0.9, 1, 5, Axis::Z);
        assert_eq!(rule.get_type(), PosRuleTestType::AxisAlignedLinearPosTest);
        assert_eq!(rule.min_dist(), 1);
        assert_eq!(rule.max_dist(), 5);
        assert_eq!(rule.axis(), Axis::Z);
        assert!((rule.min_chance() - 0.1f32).abs() < 1e-6);
        assert!((rule.max_chance() - 0.9f32).abs() < 1e-6);
    }
}
