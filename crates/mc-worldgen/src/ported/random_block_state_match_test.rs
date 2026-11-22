use std::marker::PhantomData;

/// Placeholder for `net.minecraft.world.level.block.state.BlockState`.
/// Java uses reference equality in this test; this stub uses value equality on an id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub id: u32,
}

/// Placeholder for `net.minecraft.util.RandomSource`.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

/// Placeholder for `net.minecraft.world.level.levelgen.structure.templatesystem.RuleTestType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleTestType {
    pub name: &'static str,
}

/// Placeholder for the registered type constant `RuleTestType.RANDOM_BLOCKSTATE_TEST`.
pub const RANDOM_BLOCKSTATE_TEST_TYPE: RuleTestType = RuleTestType {
    name: "random_blockstate_match",
};

/// Placeholder for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<T: ?Sized>(PhantomData<fn() -> T>);

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockStateMatchTest`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RandomBlockStateMatchTest {
    block_state: BlockState,
    probability: f32,
}

/// Placeholder for `RandomBlockStateMatchTest.CODEC`.
pub const CODEC: MapCodec<RandomBlockStateMatchTest> = MapCodec(PhantomData);

impl RandomBlockStateMatchTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockStateMatchTest#<init>
    pub fn new(block_state: BlockState, probability: f32) -> Self {
        Self {
            block_state,
            probability,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockStateMatchTest#test
    pub fn test(&self, state: BlockState, random: &mut impl RandomSource) -> bool {
        state == self.block_state && random.next_float() < self.probability
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockStateMatchTest#getType
    pub fn get_type(&self) -> RuleTestType {
        RANDOM_BLOCKSTATE_TEST_TYPE
    }

    pub fn block_state(&self) -> BlockState {
        self.block_state
    }

    pub fn probability(&self) -> f32 {
        self.probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom {
        value: f32,
    }

    impl RandomSource for FixedRandom {
        fn next_float(&mut self) -> f32 {
            self.value
        }
    }

    struct SeqRandom {
        values: Vec<f32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<f32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.values[self.idx];
            self.idx += 1;
            v
        }
    }

    #[test]
    fn matches_same_state_when_random_below_probability() {
        let state = BlockState { id: 42 };
        let rule = RandomBlockStateMatchTest::new(state, 0.5);
        let mut rng = FixedRandom { value: 0.25 };
        assert!(rule.test(state, &mut rng));
    }

    #[test]
    fn rejects_different_state_even_if_random_low() {
        let rule = RandomBlockStateMatchTest::new(BlockState { id: 1 }, 1.0);
        let mut rng = FixedRandom { value: 0.0 };
        assert!(!rule.test(BlockState { id: 2 }, &mut rng));
    }

    #[test]
    fn rejects_when_random_equal_or_above_probability() {
        let state = BlockState { id: 7 };
        let rule = RandomBlockStateMatchTest::new(state, 0.5);
        let mut eq_rng = FixedRandom { value: 0.5 };
        let mut above_rng = FixedRandom { value: 0.75 };
        assert!(!rule.test(state, &mut eq_rng));
        assert!(!rule.test(state, &mut above_rng));
    }

    #[test]
    fn handles_float_edge_cases_probability() {
        let state = BlockState { id: 9 };
        let mut rng = FixedRandom { value: 0.0 };

        let neg = RandomBlockStateMatchTest::new(state, -0.0_f32);
        assert!(!neg.test(state, &mut rng));

        let nan_prob = RandomBlockStateMatchTest::new(state, f32::NAN);
        assert!(!nan_prob.test(state, &mut rng));

        let inf_prob = RandomBlockStateMatchTest::new(state, f32::INFINITY);
        assert!(inf_prob.test(state, &mut rng));

        let neg_inf_prob = RandomBlockStateMatchTest::new(state, f32::NEG_INFINITY);
        assert!(!neg_inf_prob.test(state, &mut rng));
    }

    #[test]
    fn get_type_and_accessors_match() {
        let state = BlockState { id: 123 };
        let prob = f32::from_bits(0x3f00_0000); // 0.5
        let rule = RandomBlockStateMatchTest::new(state, prob);
        assert_eq!(rule.get_type(), RANDOM_BLOCKSTATE_TEST_TYPE);
        assert_eq!(rule.block_state(), state);
        assert_eq!(rule.probability().to_bits(), prob.to_bits());
    }

    #[test]
    fn random_is_consumed_only_when_needed_by_java_short_circuit() {
        // Java: state == blockState && random.nextFloat() < probability
        // If state differs, nextFloat must not be called.
        let rule = RandomBlockStateMatchTest::new(BlockState { id: 1 }, 1.0);
        let mut rng = SeqRandom::new(vec![0.0, 0.0]);
        assert!(!rule.test(BlockState { id: 2 }, &mut rng));
        assert_eq!(rng.idx, 0);
        assert!(rule.test(BlockState { id: 1 }, &mut rng));
        assert_eq!(rng.idx, 1);
    }
}
