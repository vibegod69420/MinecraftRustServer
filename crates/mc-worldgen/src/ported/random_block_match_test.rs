use std::marker::PhantomData;

/// Placeholder for `net.minecraft.world.level.block.Block`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: u32,
}

/// Placeholder for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block_id: u32,
}

impl BlockState {
    /// Minimal stub for `BlockState#is(Block)`.
    pub fn is(&self, block: &Block) -> bool {
        self.block_id == block.id
    }
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

/// Placeholder for the registered type constant `RuleTestType.RANDOM_BLOCK_TEST`.
pub const RANDOM_BLOCK_TEST_TYPE: RuleTestType = RuleTestType {
    name: "random_block_match",
};

/// Placeholder for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<T: ?Sized>(PhantomData<fn() -> T>);

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockMatchTest`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RandomBlockMatchTest {
    block: Block,
    probability: f32,
}

/// Placeholder for `RandomBlockMatchTest.CODEC`.
pub const CODEC: MapCodec<RandomBlockMatchTest> = MapCodec(PhantomData);

impl RandomBlockMatchTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockMatchTest#<init>
    pub fn new(block: Block, probability: f32) -> Self {
        Self { block, probability }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockMatchTest#test
    pub fn test(&self, state: BlockState, random: &mut impl RandomSource) -> bool {
        state.is(&self.block) && random.next_float() < self.probability
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockMatchTest#getType
    pub fn get_type(&self) -> RuleTestType {
        RANDOM_BLOCK_TEST_TYPE
    }

    pub fn block(&self) -> Block {
        self.block
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
    fn matches_state_with_same_block_when_random_below_probability() {
        let block = Block { id: 5 };
        let rule = RandomBlockMatchTest::new(block, 0.5);
        let mut rng = FixedRandom { value: 0.1 };
        assert!(rule.test(BlockState { block_id: 5 }, &mut rng));
    }

    #[test]
    fn rejects_state_with_different_block() {
        let rule = RandomBlockMatchTest::new(Block { id: 5 }, 1.0);
        let mut rng = FixedRandom { value: 0.0 };
        assert!(!rule.test(BlockState { block_id: 6 }, &mut rng));
    }

    #[test]
    fn rejects_when_random_equal_or_above_probability() {
        let block = Block { id: 10 };
        let rule = RandomBlockMatchTest::new(block, 0.25);
        let mut eq_rng = FixedRandom { value: 0.25 };
        let mut above_rng = FixedRandom { value: 0.9 };
        assert!(!rule.test(BlockState { block_id: 10 }, &mut eq_rng));
        assert!(!rule.test(BlockState { block_id: 10 }, &mut above_rng));
    }

    #[test]
    fn handles_float_edge_cases_probability_and_bits() {
        let block = Block { id: 3 };
        let state = BlockState { block_id: 3 };
        let mut rng_zero = FixedRandom { value: 0.0 };

        let neg_prob = RandomBlockMatchTest::new(block, -1.0_f32);
        assert!(!neg_prob.test(state, &mut rng_zero));

        let nan_prob = RandomBlockMatchTest::new(block, f32::NAN);
        assert!(!nan_prob.test(state, &mut rng_zero));

        let inf_prob = RandomBlockMatchTest::new(block, f32::INFINITY);
        assert!(inf_prob.test(state, &mut rng_zero));

        let bits = 0xBF80_0000_u32; // -1.0f32
        let from_bits = RandomBlockMatchTest::new(block, f32::from_bits(bits));
        assert_eq!(from_bits.probability().to_bits(), bits);
    }

    #[test]
    fn get_type_and_accessors_match() {
        let block = Block { id: 77 };
        let rule = RandomBlockMatchTest::new(block, 0.75);
        assert_eq!(rule.get_type(), RANDOM_BLOCK_TEST_TYPE);
        assert_eq!(rule.block(), block);
        assert_eq!(rule.probability().to_bits(), 0.75_f32.to_bits());
    }

    #[test]
    fn random_is_consumed_only_when_block_matches() {
        // Java: state.is(block) && random.nextFloat() < probability
        // If block does not match, nextFloat must not be called.
        let rule = RandomBlockMatchTest::new(Block { id: 1 }, 1.0);
        let mut rng = SeqRandom::new(vec![0.0, 0.0]);
        assert!(!rule.test(BlockState { block_id: 2 }, &mut rng));
        assert_eq!(rng.idx, 0);
        assert!(rule.test(BlockState { block_id: 1 }, &mut rng));
        assert_eq!(rng.idx, 1);
    }
}
