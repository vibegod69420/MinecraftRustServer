use std::marker::PhantomData;

/// Minimal self-contained placeholder for BlockPos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Minimal self-contained placeholder for RandomSource.
#[derive(Debug, Clone)]
pub struct RandomSource {
    seed: i64,
}

impl RandomSource {
    pub fn new(seed: i64) -> Self {
        Self { seed }
    }

    pub fn next_i32(&mut self) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005_i64)
            .wrapping_add(1_442_695_040_888_963_407_i64);
        (self.seed >> 33) as i32
    }
}

/// Minimal self-contained placeholder for PosRuleTestType.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosRuleTestType {
    AlwaysTrue,
    LinearPos,
    AxisAlignedLinearPos,
}

/// Minimal self-contained placeholder for Mojang Codec.
#[derive(Debug, Clone, Copy)]
pub struct Codec<T>(PhantomData<T>);

pub trait PosRuleTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTest#test
    fn test(
        &self,
        original_pos: BlockPos,
        relative_pos: BlockPos,
        transformed_pos: BlockPos,
        random: &mut RandomSource,
    ) -> bool;

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTest#getType
    fn get_type(&self) -> PosRuleTestType;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysTruePos;
    impl PosRuleTest for AlwaysTruePos {
        fn test(
            &self,
            _o: BlockPos,
            _r: BlockPos,
            _t: BlockPos,
            _rng: &mut RandomSource,
        ) -> bool {
            true
        }
        fn get_type(&self) -> PosRuleTestType {
            PosRuleTestType::AlwaysTrue
        }
    }

    struct YAbove(i32);
    impl PosRuleTest for YAbove {
        fn test(
            &self,
            _o: BlockPos,
            _r: BlockPos,
            t: BlockPos,
            _rng: &mut RandomSource,
        ) -> bool {
            t.y > self.0
        }
        fn get_type(&self) -> PosRuleTestType {
            PosRuleTestType::LinearPos
        }
    }

    #[test]
    fn always_true_with_extreme_positions() {
        let rt = AlwaysTruePos;
        let mut rng = RandomSource::new(0);
        assert!(rt.test(
            BlockPos::new(i32::MIN, i32::MIN, i32::MIN),
            BlockPos::new(i32::MAX, i32::MAX, i32::MAX),
            BlockPos::new(-1, 0, 1),
            &mut rng
        ));
    }

    #[test]
    fn y_above_negative_and_min_boundary() {
        let rt = YAbove(-10);
        let mut rng = RandomSource::new(1);
        assert!(rt.test(
            BlockPos::default(),
            BlockPos::default(),
            BlockPos::new(0, -9, 0),
            &mut rng
        ));
        assert!(!rt.test(
            BlockPos::default(),
            BlockPos::default(),
            BlockPos::new(0, -10, 0),
            &mut rng
        ));
        assert!(!rt.test(
            BlockPos::default(),
            BlockPos::default(),
            BlockPos::new(0, i32::MIN, 0),
            &mut rng
        ));
    }

    #[test]
    fn block_pos_equality_and_hash() {
        use std::collections::HashSet;
        let a = BlockPos::new(1, -2, 3);
        let b = BlockPos::new(1, -2, 3);
        let c = BlockPos::new(1, 2, 3);
        assert_eq!(a, b);
        assert_ne!(a, c);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn get_type_variants() {
        assert_eq!(AlwaysTruePos.get_type(), PosRuleTestType::AlwaysTrue);
        assert_eq!(YAbove(0).get_type(), PosRuleTestType::LinearPos);
    }
}
