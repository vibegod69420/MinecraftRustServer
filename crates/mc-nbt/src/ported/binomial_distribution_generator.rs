use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextKey(pub String);

pub trait RandomSource: std::fmt::Debug {
    // JAVA: net.minecraft.util.RandomSource#nextInt(int)
    fn next_int_bound(&mut self, bound: i32) -> i32;
    // JAVA: net.minecraft.util.RandomSource#nextFloat
    fn next_float(&mut self) -> f32;
}

#[derive(Debug)]
pub struct LootContext<'a> {
    pub random: &'a mut dyn RandomSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootNumberProviderType(pub u8);

pub struct NumberProviders;
impl NumberProviders {
    pub const CONSTANT: LootNumberProviderType = LootNumberProviderType(0);
    pub const UNIFORM: LootNumberProviderType = LootNumberProviderType(1);
    pub const BINOMIAL: LootNumberProviderType = LootNumberProviderType(2);
}

pub trait NumberProvider: std::fmt::Debug {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getType
    fn get_type(&self) -> LootNumberProviderType;
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getInt
    fn get_int(&self, ctx: &mut LootContext<'_>) -> i32;
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getFloat
    fn get_float(&self, ctx: &mut LootContext<'_>) -> f32;
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.NumberProvider#getReferencedContextParams
    fn get_referenced_context_params(&self) -> BTreeSet<ContextKey>;
}

#[derive(Debug, Clone)]
pub struct ConstantValue {
    pub value: f32,
}

impl ConstantValue {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#exactly(float)
    pub fn exactly(value: f32) -> Self {
        Self { value }
    }
}

impl NumberProvider for ConstantValue {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getType
    fn get_type(&self) -> LootNumberProviderType {
        NumberProviders::CONSTANT
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getInt
    fn get_int(&self, _ctx: &mut LootContext<'_>) -> i32 {
        self.value as i32
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getFloat
    fn get_float(&self, _ctx: &mut LootContext<'_>) -> f32 {
        self.value
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getReferencedContextParams
    fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        BTreeSet::new()
    }
}

#[derive(Debug)]
pub struct BinomialDistributionGenerator {
    pub n: Box<dyn NumberProvider>,
    pub p: Box<dyn NumberProvider>,
}

impl BinomialDistributionGenerator {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.BinomialDistributionGenerator#binomial
    pub fn binomial(n: i32, p: f32) -> Self {
        Self {
            n: Box::new(ConstantValue::exactly(n as f32)),
            p: Box::new(ConstantValue::exactly(p)),
        }
    }
}

impl NumberProvider for BinomialDistributionGenerator {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.BinomialDistributionGenerator#getType
    fn get_type(&self) -> LootNumberProviderType {
        NumberProviders::BINOMIAL
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.BinomialDistributionGenerator#getInt
    fn get_int(&self, ctx: &mut LootContext<'_>) -> i32 {
        let n_val = self.n.get_int(ctx);
        let p_val = self.p.get_float(ctx);
        let mut count: i32 = 0;
        let mut i: i32 = 0;
        while i < n_val {
            if ctx.random.next_float() < p_val {
                count = count.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        count
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.BinomialDistributionGenerator#getFloat
    fn get_float(&self, ctx: &mut LootContext<'_>) -> f32 {
        self.get_int(ctx) as f32
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.BinomialDistributionGenerator#getReferencedContextParams
    fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        let mut set = self.n.get_referenced_context_params();
        set.extend(self.p.get_referenced_context_params());
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct SeqRandom {
        floats: Vec<f32>,
        float_idx: usize,
    }

    impl SeqRandom {
        fn new(floats: Vec<f32>) -> Self {
            Self { floats, float_idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int_bound(&mut self, _bound: i32) -> i32 {
            0
        }

        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.float_idx];
            self.float_idx += 1;
            v
        }
    }

    #[derive(Debug)]
    struct ParamProvider(BTreeSet<ContextKey>);
    impl NumberProvider for ParamProvider {
        fn get_type(&self) -> LootNumberProviderType {
            NumberProviders::CONSTANT
        }
        fn get_int(&self, _ctx: &mut LootContext<'_>) -> i32 {
            0
        }
        fn get_float(&self, _ctx: &mut LootContext<'_>) -> f32 {
            0.0
        }
        fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
            self.0.clone()
        }
    }

    #[test]
    fn binomial_basic() {
        let dist = BinomialDistributionGenerator::binomial(5, 0.5);
        let mut rng = SeqRandom::new(vec![0.1, 0.6, 0.3, 0.5, 0.4]);
        let mut ctx = LootContext { random: &mut rng };
        assert_eq!(dist.get_int(&mut ctx), 3);
        assert_eq!(dist.get_type(), NumberProviders::BINOMIAL);
    }

    #[test]
    fn binomial_zero_n() {
        let dist = BinomialDistributionGenerator::binomial(0, 0.5);
        let mut rng = SeqRandom::new(vec![]);
        let mut ctx = LootContext { random: &mut rng };
        assert_eq!(dist.get_int(&mut ctx), 0);
    }

    #[test]
    fn binomial_extreme_p() {
        let dist_all = BinomialDistributionGenerator::binomial(4, 1.0);
        let mut rng = SeqRandom::new(vec![0.0, 0.5, 0.99, 0.999]);
        let mut ctx = LootContext { random: &mut rng };
        assert_eq!(dist_all.get_int(&mut ctx), 4);

        let dist_none = BinomialDistributionGenerator::binomial(4, 0.0);
        let mut rng2 = SeqRandom::new(vec![0.0, 0.1, 0.5, 0.99]);
        let mut ctx2 = LootContext { random: &mut rng2 };
        assert_eq!(dist_none.get_int(&mut ctx2), 0);
    }

    #[test]
    fn binomial_get_float_returns_int_as_f32() {
        let dist = BinomialDistributionGenerator::binomial(3, 1.0);
        let mut rng = SeqRandom::new(vec![0.0, 0.0, 0.0]);
        let mut ctx = LootContext { random: &mut rng };
        let result = dist.get_float(&mut ctx);
        assert_eq!(result.to_bits(), 3.0f32.to_bits());
    }

    #[test]
    fn binomial_referenced_params_union() {
        let mut s_n = BTreeSet::new();
        s_n.insert(ContextKey("n".into()));
        let mut s_p = BTreeSet::new();
        s_p.insert(ContextKey("p".into()));
        let dist = BinomialDistributionGenerator {
            n: Box::new(ParamProvider(s_n)),
            p: Box::new(ParamProvider(s_p)),
        };
        let params = dist.get_referenced_context_params();
        assert_eq!(params.len(), 2);
        assert!(params.contains(&ContextKey("n".into())));
        assert!(params.contains(&ContextKey("p".into())));
    }
}
