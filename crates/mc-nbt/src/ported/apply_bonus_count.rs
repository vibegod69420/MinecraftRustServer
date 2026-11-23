use std::fmt;

/// Minimal self-contained random source matching the Java `RandomSource` methods
/// used by `ApplyBonusCount`.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Minimal self-contained item stack for loot function application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
}

impl ItemStack {
    pub fn new(count: i32) -> Self {
        Self { count }
    }

    pub fn get_count(&self) -> i32 {
        self.count
    }

    pub fn set_count(&mut self, count: i32) {
        self.count = count;
    }
}

/// Minimal self-contained enchantment identifier/holder placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnchantmentId(pub &'static str);

/// Minimal self-contained loot function type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LootItemFunctionType(pub &'static str);

/// Minimal self-contained context key identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormulaType(pub &'static str);

impl FormulaType {
    pub const BINOMIAL_WITH_BONUS_COUNT: Self = Self("minecraft:binomial_with_bonus_count");
    pub const ORE_DROPS: Self = Self("minecraft:ore_drops");
    pub const UNIFORM_BONUS_COUNT: Self = Self("minecraft:uniform_bonus_count");
}

impl fmt::Display for FormulaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

pub trait Formula {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$Formula#calculateNewCount
    fn calculate_new_count(
        &self,
        random: &mut dyn RandomSource,
        count: i32,
        enchantment_level: i32,
    ) -> i32;

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$Formula#getType
    fn get_type(&self) -> FormulaType;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinomialWithBonusCount {
    pub extra_rounds: i32,
    pub probability: f32,
}

impl BinomialWithBonusCount {
    pub const TYPE: FormulaType = FormulaType::BINOMIAL_WITH_BONUS_COUNT;

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$BinomialWithBonusCount#calculateNewCount
    pub fn calculate_new_count(
        &self,
        random: &mut dyn RandomSource,
        count: i32,
        enchantment_level: i32,
    ) -> i32 {
        let mut result = count;
        let rounds = enchantment_level.wrapping_add(self.extra_rounds);
        let mut i: i32 = 0;
        while i < rounds {
            if random.next_float() < self.probability {
                result = result.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        result
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$BinomialWithBonusCount#getType
    pub fn get_type(&self) -> FormulaType {
        Self::TYPE
    }
}

impl Formula for BinomialWithBonusCount {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$BinomialWithBonusCount#calculateNewCount
    fn calculate_new_count(
        &self,
        random: &mut dyn RandomSource,
        count: i32,
        enchantment_level: i32,
    ) -> i32 {
        BinomialWithBonusCount::calculate_new_count(self, random, count, enchantment_level)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$BinomialWithBonusCount#getType
    fn get_type(&self) -> FormulaType {
        BinomialWithBonusCount::get_type(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OreDrops;

impl OreDrops {
    pub const INSTANCE: Self = Self;
    pub const TYPE: FormulaType = FormulaType::ORE_DROPS;

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$OreDrops#calculateNewCount
    pub fn calculate_new_count(
        &self,
        random: &mut dyn RandomSource,
        count: i32,
        enchantment_level: i32,
    ) -> i32 {
        if enchantment_level > 0 {
            let bound = enchantment_level.wrapping_add(2);
            let mut multiplier = random.next_int(bound).wrapping_sub(1);
            if multiplier < 0 {
                multiplier = 0;
            }
            count.wrapping_mul(multiplier.wrapping_add(1))
        } else {
            count
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$OreDrops#getType
    pub fn get_type(&self) -> FormulaType {
        Self::TYPE
    }
}

impl Formula for OreDrops {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$OreDrops#calculateNewCount
    fn calculate_new_count(
        &self,
        random: &mut dyn RandomSource,
        count: i32,
        enchantment_level: i32,
    ) -> i32 {
        OreDrops::calculate_new_count(self, random, count, enchantment_level)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$OreDrops#getType
    fn get_type(&self) -> FormulaType {
        OreDrops::get_type(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformBonusCount {
    pub bonus_multiplier: i32,
}

impl UniformBonusCount {
    pub const TYPE: FormulaType = FormulaType::UNIFORM_BONUS_COUNT;

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$UniformBonusCount#calculateNewCount
    pub fn calculate_new_count(
        &self,
        random: &mut dyn RandomSource,
        count: i32,
        enchantment_level: i32,
    ) -> i32 {
        let bound = self
            .bonus_multiplier
            .wrapping_mul(enchantment_level)
            .wrapping_add(1);
        count.wrapping_add(random.next_int(bound))
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$UniformBonusCount#getType
    pub fn get_type(&self) -> FormulaType {
        Self::TYPE
    }
}

impl Formula for UniformBonusCount {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$UniformBonusCount#calculateNewCount
    fn calculate_new_count(
        &self,
        random: &mut dyn RandomSource,
        count: i32,
        enchantment_level: i32,
    ) -> i32 {
        UniformBonusCount::calculate_new_count(self, random, count, enchantment_level)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$UniformBonusCount#getType
    fn get_type(&self) -> FormulaType {
        UniformBonusCount::get_type(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApplyBonusFormula {
    BinomialWithBonusCount(BinomialWithBonusCount),
    OreDrops(OreDrops),
    UniformBonusCount(UniformBonusCount),
}

impl ApplyBonusFormula {
    pub fn calculate_new_count(
        &self,
        random: &mut dyn RandomSource,
        count: i32,
        enchantment_level: i32,
    ) -> i32 {
        match self {
            Self::BinomialWithBonusCount(f) => {
                f.calculate_new_count(random, count, enchantment_level)
            }
            Self::OreDrops(f) => f.calculate_new_count(random, count, enchantment_level),
            Self::UniformBonusCount(f) => f.calculate_new_count(random, count, enchantment_level),
        }
    }

    pub fn get_type(&self) -> FormulaType {
        match self {
            Self::BinomialWithBonusCount(f) => f.get_type(),
            Self::OreDrops(f) => f.get_type(),
            Self::UniformBonusCount(f) => f.get_type(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyBonusCount {
    pub enchantment: EnchantmentId,
    pub formula: ApplyBonusFormula,
}

impl ApplyBonusCount {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#<init>
    pub fn new(enchantment: EnchantmentId, formula: ApplyBonusFormula) -> Self {
        Self {
            enchantment,
            formula,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType("minecraft:apply_bonus")
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> &'static [ContextKey] {
        const PARAMS: &[ContextKey] = &[ContextKey("minecraft:tool")];
        PARAMS
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#run
    pub fn run<'a>(
        &self,
        stack: &'a mut ItemStack,
        tool_enchantment_level: Option<i32>,
        random: &mut dyn RandomSource,
    ) -> &'a mut ItemStack {
        if let Some(level) = tool_enchantment_level {
            let new_count = self
                .formula
                .calculate_new_count(random, stack.get_count(), level);
            stack.set_count(new_count);
        }
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#addBonusBinomialDistributionCount
    pub fn add_bonus_binomial_distribution_count(
        enchantment: EnchantmentId,
        probability: f32,
        extra_rounds: i32,
    ) -> Self {
        Self::new(
            enchantment,
            ApplyBonusFormula::BinomialWithBonusCount(BinomialWithBonusCount {
                extra_rounds,
                probability,
            }),
        )
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#addOreBonusCount
    pub fn add_ore_bonus_count(enchantment: EnchantmentId) -> Self {
        Self::new(enchantment, ApplyBonusFormula::OreDrops(OreDrops))
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#addUniformBonusCount
    pub fn add_uniform_bonus_count(enchantment: EnchantmentId) -> Self {
        Self::new(
            enchantment,
            ApplyBonusFormula::UniformBonusCount(UniformBonusCount {
                bonus_multiplier: 1,
            }),
        )
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#addUniformBonusCount
    pub fn add_uniform_bonus_count_with_multiplier(
        enchantment: EnchantmentId,
        bonus_multiplier: i32,
    ) -> Self {
        Self::new(
            enchantment,
            ApplyBonusFormula::UniformBonusCount(UniformBonusCount { bonus_multiplier }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        float_idx: usize,
        int_idx: usize,
    }

    impl SeqRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                ints,
                float_idx: 0,
                int_idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.float_idx];
            self.float_idx += 1;
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            let _ = bound;
            let v = self.ints[self.int_idx];
            self.int_idx += 1;
            v
        }
    }

    #[test]
    fn binomial_with_bonus_count_basic_and_negative_rounds() {
        let formula = BinomialWithBonusCount {
            extra_rounds: 3,
            probability: 0.5,
        };
        let mut rng = SeqRandom::new(vec![0.0, 0.5, 0.25, 0.75, 0.499], vec![]);
        // rounds = 2 + 3 = 5; increments on 0.0, 0.25, 0.499
        assert_eq!(formula.calculate_new_count(&mut rng, 10, 2), 13);

        let formula = BinomialWithBonusCount {
            extra_rounds: -5,
            probability: 1.0,
        };
        let mut rng = SeqRandom::new(vec![0.0, 0.0], vec![]);
        // rounds = 2 + (-5) = -3, no iterations
        assert_eq!(formula.calculate_new_count(&mut rng, 10, 2), 10);
        assert_eq!(rng.float_idx, 0);
    }

    #[test]
    fn ore_drops_behavior_and_negative_level() {
        let formula = OreDrops;
        let mut rng = SeqRandom::new(vec![], vec![0]);
        // bound=5, nextInt=0 -> multiplier -1 clamped to 0 -> 4 * 1
        assert_eq!(formula.calculate_new_count(&mut rng, 4, 3), 4);

        let mut rng = SeqRandom::new(vec![], vec![4]);
        // bound=5, nextInt=4 -> multiplier 3 -> 4 * 4
        assert_eq!(formula.calculate_new_count(&mut rng, 4, 3), 16);

        let mut rng = SeqRandom::new(vec![], vec![]);
        assert_eq!(formula.calculate_new_count(&mut rng, 7, 0), 7);
        assert_eq!(formula.calculate_new_count(&mut rng, 7, -1), 7);
    }

    #[test]
    fn uniform_bonus_count_and_run_behavior() {
        let formula = UniformBonusCount {
            bonus_multiplier: 2,
        };
        let mut rng = SeqRandom::new(vec![], vec![5]);
        // bound = 2*3+1 = 7, 10 + 5 = 15
        assert_eq!(formula.calculate_new_count(&mut rng, 10, 3), 15);

        let func = ApplyBonusCount::add_uniform_bonus_count_with_multiplier(
            EnchantmentId("minecraft:fortune"),
            1,
        );
        let mut stack = ItemStack::new(5);
        let mut rng = SeqRandom::new(vec![], vec![2]);
        // level 2, bound = 3, 5 + 2 = 7
        func.run(&mut stack, Some(2), &mut rng);
        assert_eq!(stack.get_count(), 7);

        let mut stack = ItemStack::new(5);
        let mut rng = SeqRandom::new(vec![], vec![2]);
        func.run(&mut stack, None, &mut rng);
        assert_eq!(stack.get_count(), 5);
        assert_eq!(rng.int_idx, 0);
    }

    #[test]
    fn integer_wrapping_and_negative_count() {
        let uniform = UniformBonusCount {
            bonus_multiplier: 1,
        };
        let mut rng = SeqRandom::new(vec![], vec![1]);
        // bound=2, i32::MAX + 1 wraps
        assert_eq!(uniform.calculate_new_count(&mut rng, i32::MAX, 1), i32::MIN);

        let ore = OreDrops;
        let mut rng = SeqRandom::new(vec![], vec![2]);
        // bound=3, nextInt=2 -> multiplier 1 -> i32::MAX * 2 wraps
        assert_eq!(
            ore.calculate_new_count(&mut rng, i32::MAX, 1),
            i32::MAX.wrapping_mul(2)
        );

        let binomial = BinomialWithBonusCount {
            extra_rounds: 1,
            probability: 1.0,
        };
        let mut rng = SeqRandom::new(vec![0.0], vec![]);
        assert_eq!(binomial.calculate_new_count(&mut rng, i32::MAX, 0), i32::MIN);

        let mut rng = SeqRandom::new(vec![], vec![1]);
        assert_eq!(uniform.calculate_new_count(&mut rng, -5, 2), -4);
    }

    #[test]
    fn float_bits_and_nan_comparisons() {
        let formula = BinomialWithBonusCount {
            extra_rounds: 1,
            probability: f32::NAN,
        };
        let mut rng = SeqRandom::new(vec![0.5], vec![]);
        // 0.5 < NaN is false
        assert_eq!(formula.calculate_new_count(&mut rng, 1, 0), 1);

        let formula = BinomialWithBonusCount {
            extra_rounds: 1,
            probability: 0.0,
        };
        let mut rng = SeqRandom::new(vec![-0.0f32], vec![]);
        // -0.0 < 0.0 is false; also verify float bits are preserved in input
        assert_eq!(rng.floats[0].to_bits(), (-0.0f32).to_bits());
        assert_eq!(formula.calculate_new_count(&mut rng, 1, 0), 1);

        let formula = BinomialWithBonusCount {
            extra_rounds: 1,
            probability: 0.5,
        };
        let mut rng = SeqRandom::new(vec![f32::NAN], vec![]);
        // NaN < 0.5 is false
        assert_eq!(formula.calculate_new_count(&mut rng, 1, 0), 1);
    }

    #[test]
    fn formula_types_and_builders() {
        assert_eq!(
            BinomialWithBonusCount::TYPE,
            FormulaType::BINOMIAL_WITH_BONUS_COUNT
        );
        assert_eq!(OreDrops::TYPE, FormulaType::ORE_DROPS);
        assert_eq!(UniformBonusCount::TYPE, FormulaType::UNIFORM_BONUS_COUNT);

        let ench = EnchantmentId("minecraft:fortune");
        let binomial = ApplyBonusCount::add_bonus_binomial_distribution_count(ench, 0.5, 3);
        assert_eq!(
            binomial.formula.get_type(),
            FormulaType::BINOMIAL_WITH_BONUS_COUNT
        );

        let ore = ApplyBonusCount::add_ore_bonus_count(ench);
        assert_eq!(ore.formula.get_type(), FormulaType::ORE_DROPS);

        let uniform = ApplyBonusCount::add_uniform_bonus_count(ench);
        assert_eq!(uniform.formula.get_type(), FormulaType::UNIFORM_BONUS_COUNT);
        assert_eq!(uniform.get_type(), LootItemFunctionType("minecraft:apply_bonus"));
        assert_eq!(
            uniform.get_referenced_context_params(),
            &[ContextKey("minecraft:tool")]
        );
    }
}
