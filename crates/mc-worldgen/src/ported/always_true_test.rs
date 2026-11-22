use std::marker::PhantomData;

/// Minimal self-contained placeholder for BlockState.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: i32,
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

/// Minimal self-contained placeholder for RuleTestType.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTestType {
    AlwaysTrueTest,
}

/// Minimal self-contained placeholder for Mojang MapCodec.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T>(PhantomData<T>);

impl<T> MapCodec<T> {
    pub const fn unit(_factory: fn() -> T) -> Self {
        Self(PhantomData)
    }
}

/// Minimal self-contained placeholder for RuleTest base.
pub trait RuleTest {
    fn test(&self, state: &BlockState, random: &mut RandomSource) -> bool;
    fn get_type(&self) -> RuleTestType;
}

fn always_true_test_instance() -> AlwaysTrueTest {
    AlwaysTrueTest::INSTANCE
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlwaysTrueTest;

impl AlwaysTrueTest {
    pub const INSTANCE: AlwaysTrueTest = AlwaysTrueTest;
    pub const CODEC: MapCodec<AlwaysTrueTest> = MapCodec::unit(always_true_test_instance);

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.AlwaysTrueTest#<init>
    const fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.AlwaysTrueTest#test
    pub fn test(&self, _state: &BlockState, _random: &mut RandomSource) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.AlwaysTrueTest#getType
    fn get_type(&self) -> RuleTestType {
        RuleTestType::AlwaysTrueTest
    }
}

impl RuleTest for AlwaysTrueTest {
    fn test(&self, state: &BlockState, random: &mut RandomSource) -> bool {
        AlwaysTrueTest::test(self, state, random)
    }

    fn get_type(&self) -> RuleTestType {
        AlwaysTrueTest::get_type(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_returns_true_for_extreme_states() {
        let t = AlwaysTrueTest::INSTANCE;
        let mut rng = RandomSource::new(0);
        assert!(t.test(&BlockState { id: 0 }, &mut rng));
        assert!(t.test(&BlockState { id: i32::MIN }, &mut rng));
        assert!(t.test(&BlockState { id: i32::MAX }, &mut rng));
        assert!(t.test(&BlockState { id: -1 }, &mut rng));
    }

    #[test]
    fn get_type_is_always_true_test() {
        let t = AlwaysTrueTest::new();
        assert_eq!(AlwaysTrueTest::get_type(&t), RuleTestType::AlwaysTrueTest);
        assert_eq!(RuleTest::get_type(&t), RuleTestType::AlwaysTrueTest);
    }

    #[test]
    fn trait_object_works_with_negative_seed() {
        let t = AlwaysTrueTest::INSTANCE;
        let rt: &dyn RuleTest = &t;
        let mut rng = RandomSource::new(i64::MIN);
        assert!(rt.test(&BlockState { id: -123_456 }, &mut rng));
        assert_eq!(rt.get_type(), RuleTestType::AlwaysTrueTest);
    }

    #[test]
    fn codec_placeholder_and_instance_identity() {
        let _codec: MapCodec<AlwaysTrueTest> = AlwaysTrueTest::CODEC;
        let produced = always_true_test_instance();
        assert_eq!(produced, AlwaysTrueTest::INSTANCE);
        let a = AlwaysTrueTest::INSTANCE;
        let b = a;
        assert_eq!(a, b);
    }
}
