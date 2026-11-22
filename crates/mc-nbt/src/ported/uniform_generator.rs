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

pub struct Mth;
impl Mth {
    // JAVA: net.minecraft.util.Mth#nextInt(net.minecraft.util.RandomSource,int,int)
    pub fn next_int(random: &mut dyn RandomSource, minimum: i32, maximum: i32) -> i32 {
        if minimum >= maximum {
            minimum
        } else {
            random
                .next_int_bound(maximum.wrapping_sub(minimum).wrapping_add(1))
                .wrapping_add(minimum)
        }
    }

    // JAVA: net.minecraft.util.Mth#nextFloat(net.minecraft.util.RandomSource,float,float)
    pub fn next_float(random: &mut dyn RandomSource, minimum: f32, maximum: f32) -> f32 {
        if minimum >= maximum {
            minimum
        } else {
            random.next_float() * (maximum - minimum) + minimum
        }
    }
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
pub struct UniformGenerator {
    pub min: Box<dyn NumberProvider>,
    pub max: Box<dyn NumberProvider>,
}

impl UniformGenerator {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.UniformGenerator#between
    pub fn between(min: f32, max: f32) -> Self {
        Self {
            min: Box::new(ConstantValue::exactly(min)),
            max: Box::new(ConstantValue::exactly(max)),
        }
    }
}

impl NumberProvider for UniformGenerator {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.UniformGenerator#getType
    fn get_type(&self) -> LootNumberProviderType {
        NumberProviders::UNIFORM
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.UniformGenerator#getInt
    fn get_int(&self, ctx: &mut LootContext<'_>) -> i32 {
        let min_val = self.min.get_int(ctx);
        let max_val = self.max.get_int(ctx);
        Mth::next_int(ctx.random, min_val, max_val)
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.UniformGenerator#getFloat
    fn get_float(&self, ctx: &mut LootContext<'_>) -> f32 {
        let min_val = self.min.get_float(ctx);
        let max_val = self.max.get_float(ctx);
        Mth::next_float(ctx.random, min_val, max_val)
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.UniformGenerator#getReferencedContextParams
    fn get_referenced_context_params(&self) -> BTreeSet<ContextKey> {
        let mut set = self.min.get_referenced_context_params();
        set.extend(self.max.get_referenced_context_params());
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct SeqRandom {
        ints: Vec<i32>,
        floats: Vec<f32>,
        int_idx: usize,
        float_idx: usize,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>, floats: Vec<f32>) -> Self {
            Self {
                ints,
                floats,
                int_idx: 0,
                float_idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int_bound(&mut self, _bound: i32) -> i32 {
            let v = self.ints[self.int_idx];
            self.int_idx += 1;
            v
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
    fn uniform_between_int() {
        let uniform = UniformGenerator::between(1.0, 5.0);
        let mut rng = SeqRandom::new(vec![2], vec![]);
        let mut ctx = LootContext { random: &mut rng };
        assert_eq!(uniform.get_int(&mut ctx), 3);
        assert_eq!(uniform.get_type(), NumberProviders::UNIFORM);
    }

    #[test]
    fn uniform_min_gte_max() {
        let uniform = UniformGenerator::between(5.0, 3.0);
        let mut rng = SeqRandom::new(vec![99], vec![0.9]);
        {
            let mut ctx = LootContext { random: &mut rng };
            assert_eq!(uniform.get_int(&mut ctx), 5);
        }
        {
            let mut ctx = LootContext { random: &mut rng };
            assert_eq!(uniform.get_float(&mut ctx).to_bits(), 5.0f32.to_bits());
        }
    }

    #[test]
    fn uniform_negative_range() {
        let uniform = UniformGenerator::between(-10.0, -5.0);
        let mut rng = SeqRandom::new(vec![3], vec![]);
        let mut ctx = LootContext { random: &mut rng };
        assert_eq!(uniform.get_int(&mut ctx), -7);
    }

    #[test]
    fn uniform_float_order_and_bits() {
        let uniform = UniformGenerator::between(2.0, 6.0);
        let mut rng = SeqRandom::new(vec![], vec![0.5]);
        let mut ctx = LootContext { random: &mut rng };
        let result = uniform.get_float(&mut ctx);
        assert_eq!(result.to_bits(), 4.0f32.to_bits());
    }

    #[test]
    fn uniform_referenced_params_union() {
        let mut s1 = BTreeSet::new();
        s1.insert(ContextKey("a".into()));
        let mut s2 = BTreeSet::new();
        s2.insert(ContextKey("b".into()));
        s2.insert(ContextKey("a".into()));
        let uniform = UniformGenerator {
            min: Box::new(ParamProvider(s1)),
            max: Box::new(ParamProvider(s2)),
        };
        let params = uniform.get_referenced_context_params();
        assert_eq!(params.len(), 2);
        assert!(params.contains(&ContextKey("a".into())));
        assert!(params.contains(&ContextKey("b".into())));
    }
}
