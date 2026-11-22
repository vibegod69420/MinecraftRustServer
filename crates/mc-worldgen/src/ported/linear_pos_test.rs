#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#distManhattan
    pub fn dist_manhattan(&self, other: &BlockPos) -> i32 {
        let dx = self.x.wrapping_sub(other.x).wrapping_abs();
        let dy = self.y.wrapping_sub(other.y).wrapping_abs();
        let dz = self.z.wrapping_sub(other.z).wrapping_abs();
        dx.wrapping_add(dy).wrapping_add(dz)
    }
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosRuleTestType {
    LinearPosTest,
}

// JAVA: net.minecraft.util.Mth#inverseLerp
pub fn inverse_lerp(value: f32, start: f32, end: f32) -> f32 {
    (value - start) / (end - start)
}

// JAVA: net.minecraft.util.Mth#clampedLerp
pub fn clamped_lerp(delta: f32, start: f32, end: f32) -> f32 {
    if delta < 0.0 {
        start
    } else if delta > 1.0 {
        end
    } else {
        start + delta * (end - start)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearPosTest {
    pub min_chance: f32,
    pub max_chance: f32,
    pub min_dist: i32,
    pub max_dist: i32,
}

impl LinearPosTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.LinearPosTest#<init>
    pub fn new(min_chance: f32, max_chance: f32, min_dist: i32, max_dist: i32) -> Self {
        if min_dist >= max_dist {
            panic!("Invalid range: [{},{}]", min_dist, max_dist);
        }
        Self {
            min_chance,
            max_chance,
            min_dist,
            max_dist,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.LinearPosTest#test
    pub fn test(
        &self,
        _pos0: &BlockPos,
        pos1: &BlockPos,
        pos2: &BlockPos,
        random: &mut dyn RandomSource,
    ) -> bool {
        let dist = pos1.dist_manhattan(pos2);
        let f = random.next_float();
        let delta = inverse_lerp(dist as f32, self.min_dist as f32, self.max_dist as f32);
        let threshold = clamped_lerp(delta, self.min_chance, self.max_chance);
        f <= threshold
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.LinearPosTest#getType
    pub fn get_type(&self) -> PosRuleTestType {
        PosRuleTestType::LinearPosTest
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

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    #[test]
    #[should_panic(expected = "Invalid range: [5,5]")]
    fn constructor_rejects_equal_range() {
        let _ = LinearPosTest::new(0.0, 1.0, 5, 5);
    }

    #[test]
    #[should_panic(expected = "Invalid range: [10,5]")]
    fn constructor_rejects_inverted_range() {
        let _ = LinearPosTest::new(0.0, 1.0, 10, 5);
    }

    #[test]
    fn test_at_min_and_max_dist_uses_chances() {
        let t = LinearPosTest::new(0.2, 0.8, 0, 10);
        let origin = pos(0, 0, 0);
        let at_min = pos(0, 0, 0);
        let at_max = pos(10, 0, 0);

        let mut r = FixedRandom(0.2);
        assert!(t.test(&origin, &origin, &at_min, &mut r));
        let mut r2 = FixedRandom(0.21);
        assert!(!t.test(&origin, &origin, &at_min, &mut r2));

        let mut r3 = FixedRandom(0.8);
        assert!(t.test(&origin, &origin, &at_max, &mut r3));
        let mut r4 = FixedRandom(0.81);
        assert!(!t.test(&origin, &origin, &at_max, &mut r4));
    }

    #[test]
    fn test_clamps_outside_range_and_negative_coords() {
        let t = LinearPosTest::new(0.25, 0.75, 5, 10);
        let origin = pos(0, 0, 0);
        let below_min = pos(1, 1, 1); // dist 3 < 5
        let above_max = pos(20, 0, 0); // dist 20 > 10

        let mut r = FixedRandom(0.25);
        assert!(t.test(&origin, &origin, &below_min, &mut r));
        let mut r2 = FixedRandom(0.26);
        assert!(!t.test(&origin, &origin, &below_min, &mut r2));

        let mut r3 = FixedRandom(0.75);
        assert!(t.test(&origin, &origin, &above_max, &mut r3));
        let mut r4 = FixedRandom(0.76);
        assert!(!t.test(&origin, &origin, &above_max, &mut r4));

        let t2 = LinearPosTest::new(0.0, 1.0, 0, 10);
        let p1 = pos(-3, -4, 0);
        let p2 = pos(0, 0, 0); // dist 7 -> threshold 0.7
        let mut r5 = FixedRandom(0.7);
        assert!(t2.test(&origin, &p1, &p2, &mut r5));
        let mut r6 = FixedRandom(0.71);
        assert!(!t2.test(&origin, &p1, &p2, &mut r6));
    }

    #[test]
    fn dist_manhattan_wrapping_and_float_bits() {
        let a = BlockPos {
            x: i32::MIN,
            y: 0,
            z: 0,
        };
        let b = BlockPos { x: 0, y: 0, z: 0 };
        let d = a.dist_manhattan(&b);
        assert_eq!(d, i32::MIN);

        let t = LinearPosTest::new(0.0, 1.0, 0, 4);
        let origin = pos(0, 0, 0);
        let mid = pos(1, 0, 0); // dist 1, delta 0.25
        let mut r = FixedRandom(0.25);
        assert!(t.test(&origin, &origin, &mid, &mut r));

        let delta = inverse_lerp(1.0f32, 0.0f32, 4.0f32);
        let threshold = clamped_lerp(delta, 0.0f32, 1.0f32);
        assert_eq!(threshold.to_bits(), 0.25f32.to_bits());
    }

    #[test]
    fn get_type_is_linear_pos_test() {
        let t = LinearPosTest::new(0.0, 1.0, 0, 1);
        assert_eq!(t.get_type(), PosRuleTestType::LinearPosTest);
    }
}
