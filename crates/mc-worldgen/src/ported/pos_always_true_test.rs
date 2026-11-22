use std::marker::PhantomData;

/// Minimal `BlockPos` stub for self-contained compilation.
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

/// Minimal `RandomSource` stub for self-contained compilation.
#[derive(Debug, Clone, Copy, Default)]
pub struct RandomSource {
    pub seed: i64,
}

/// Minimal `PosRuleTestType` stub for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PosRuleTestType {
    AlwaysTrueTest,
    LinearPosTest,
    AxisAlignedLinearPosTest,
}

/// Minimal `MapCodec` stub preserving the Java field shape.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> MapCodec<T> {
    pub const fn unit(_factory: fn() -> T) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Base trait mirroring `PosRuleTest`.
pub trait PosRuleTest {
    fn test(
        &self,
        pos0: BlockPos,
        pos1: BlockPos,
        pos2: BlockPos,
        random: &mut RandomSource,
    ) -> bool;

    fn get_type(&self) -> PosRuleTestType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PosAlwaysTrueTest;

impl PosAlwaysTrueTest {
    pub const INSTANCE: PosAlwaysTrueTest = PosAlwaysTrueTest;
    pub const CODEC: MapCodec<PosAlwaysTrueTest> = MapCodec::unit(Self::codec_instance);

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.PosAlwaysTrueTest#<init>
    const fn new() -> Self {
        Self
    }

    fn codec_instance() -> Self {
        Self::INSTANCE
    }
}

impl PosRuleTest for PosAlwaysTrueTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.PosAlwaysTrueTest#test
    fn test(
        &self,
        _pos0: BlockPos,
        _pos1: BlockPos,
        _pos2: BlockPos,
        _random: &mut RandomSource,
    ) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.PosAlwaysTrueTest#getType
    fn get_type(&self) -> PosRuleTestType {
        PosRuleTestType::AlwaysTrueTest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_true_returns_true_for_normal_positions() {
        let t = PosAlwaysTrueTest::INSTANCE;
        let mut rng = RandomSource::default();
        assert!(t.test(
            BlockPos::new(0, 0, 0),
            BlockPos::new(1, 2, 3),
            BlockPos::new(-1, -2, -3),
            &mut rng,
        ));
    }

    #[test]
    fn always_true_returns_true_for_extreme_coordinates() {
        let t = PosAlwaysTrueTest::new();
        let mut rng = RandomSource { seed: i64::MIN };
        assert!(t.test(
            BlockPos::new(i32::MIN, i32::MAX, 0),
            BlockPos::new(i32::MAX, i32::MIN, -1),
            BlockPos::new(0, i32::MIN, i32::MAX),
            &mut rng,
        ));
    }

    #[test]
    fn get_type_is_always_true() {
        let t = PosAlwaysTrueTest::INSTANCE;
        assert_eq!(t.get_type(), PosRuleTestType::AlwaysTrueTest);
    }

    #[test]
    fn instance_equals_default() {
        assert_eq!(PosAlwaysTrueTest::INSTANCE, PosAlwaysTrueTest);
    }
}
