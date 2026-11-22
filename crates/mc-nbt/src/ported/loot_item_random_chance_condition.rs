/// Local stand-in for `net.minecraft.util.RandomSource`.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

/// Minimal context surface required by `LootItemRandomChanceCondition`.
pub trait LootContext {
    type Rng: RandomSource;
    fn get_random(&mut self) -> &mut Self::Rng;
}

/// Local stand-in for `net.minecraft.world.level.storage.loot.providers.number.NumberProvider`.
pub trait NumberProvider<C: ?Sized> {
    fn get_float(&self, ctx: &mut C) -> f32;
}

/// Local stand-in for `net.minecraft.world.level.storage.loot.providers.number.ConstantValue`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstantValue {
    value: f32,
}

impl ConstantValue {
    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#exactly
    pub fn exactly(value: f32) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.world.level.storage.loot.providers.number.ConstantValue#getFloat
    pub fn get_float<C: ?Sized>(&self, _ctx: &mut C) -> f32 {
        self.value
    }
}

impl<C: ?Sized> NumberProvider<C> for ConstantValue {
    fn get_float(&self, ctx: &mut C) -> f32 {
        ConstantValue::get_float(self, ctx)
    }
}

/// Local stand-in for `net.minecraft.world.level.storage.loot.predicates.LootItemConditionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemConditionType {
    RandomChance,
}

/// Port of `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition`.
pub struct LootItemRandomChanceCondition<C: ?Sized> {
    chance: Box<dyn NumberProvider<C>>,
}

impl<C: LootContext + ?Sized> LootItemRandomChanceCondition<C> {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition#<init>
    pub fn new(chance: Box<dyn NumberProvider<C>>) -> Self {
        Self { chance }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition#chance
    pub fn chance(&self) -> &dyn NumberProvider<C> {
        &*self.chance
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::RandomChance
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition#test
    pub fn test(&self, ctx: &mut C) -> bool {
        let chance: f32 = self.chance.get_float(ctx);
        ctx.get_random().next_float() < chance
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition#randomChance
    pub fn random_chance(chance: f32) -> Self {
        Self::new(Box::new(ConstantValue::exactly(chance)))
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition#randomChance
    pub fn random_chance_provider(chance: Box<dyn NumberProvider<C>>) -> Self {
        Self::new(chance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockRng(f32);
    impl RandomSource for MockRng {
        fn next_float(&mut self) -> f32 {
            self.0
        }
    }

    #[derive(Debug)]
    struct MockCtx {
        rng: MockRng,
    }
    impl LootContext for MockCtx {
        type Rng = MockRng;
        fn get_random(&mut self) -> &mut Self::Rng {
            &mut self.rng
        }
    }

    #[test]
    fn zero_chance_never_passes() {
        let cond = LootItemRandomChanceCondition::<MockCtx>::random_chance(0.0_f32);
        let mut ctx = MockCtx { rng: MockRng(0.0_f32) };
        assert!(!cond.test(&mut ctx));
        assert_eq!(cond.get_type(), LootItemConditionType::RandomChance);
    }

    #[test]
    fn comparison_is_strict_less_than() {
        let cond = LootItemRandomChanceCondition::<MockCtx>::random_chance(0.5_f32);
        let mut eq_ctx = MockCtx { rng: MockRng(0.5_f32) };
        let mut below_ctx = MockCtx { rng: MockRng(0.499_999_97_f32) };
        let mut above_ctx = MockCtx { rng: MockRng(0.500_000_06_f32) };
        assert!(!cond.test(&mut eq_ctx)); // 0.5 < 0.5 is false
        assert!(cond.test(&mut below_ctx));
        assert!(!cond.test(&mut above_ctx));
    }

    #[test]
    fn negative_and_nan_chance_float_bits() {
        let mut ctx = MockCtx { rng: MockRng(0.0_f32) };

        let neg = LootItemRandomChanceCondition::<MockCtx>::random_chance(-1.0_f32);
        assert_eq!((-1.0_f32).to_bits(), neg.chance().get_float(&mut ctx).to_bits());
        assert!(!neg.test(&mut ctx));

        let nan = LootItemRandomChanceCondition::<MockCtx>::random_chance(f32::NAN);
        let mut nan_ctx = MockCtx { rng: MockRng(0.0_f32) };
        let v = nan.chance().get_float(&mut nan_ctx);
        assert!(v.is_nan());
        let mut nan_test_ctx = MockCtx { rng: MockRng(0.0_f32) };
        assert!(!nan.test(&mut nan_test_ctx));
    }

    #[test]
    fn chance_evaluated_before_random() {
        #[derive(Debug)]
        struct OrderRng {
            value: f32,
            log: Vec<&'static str>,
        }
        impl RandomSource for OrderRng {
            fn next_float(&mut self) -> f32 {
                self.log.push("random");
                self.value
            }
        }
        #[derive(Debug)]
        struct OrderCtx {
            rng: OrderRng,
        }
        impl LootContext for OrderCtx {
            type Rng = OrderRng;
            fn get_random(&mut self) -> &mut Self::Rng {
                &mut self.rng
            }
        }
        struct LoggingProvider;
        impl NumberProvider<OrderCtx> for LoggingProvider {
            fn get_float(&self, ctx: &mut OrderCtx) -> f32 {
                ctx.rng.log.push("chance");
                1.0_f32
            }
        }

        let cond = LootItemRandomChanceCondition::<OrderCtx>::random_chance_provider(Box::new(LoggingProvider));
        let mut ctx = OrderCtx {
            rng: OrderRng { value: 0.0_f32, log: Vec::new() },
        };
        assert!(cond.test(&mut ctx));
        assert_eq!(ctx.rng.log, vec!["chance", "random"]);
    }

    #[test]
    fn constant_value_exact_and_provider_path() {
        let mut ctx = MockCtx { rng: MockRng(0.25_f32) };
        let c = ConstantValue::exactly(0.5_f32);
        assert_eq!(c.get_float(&mut ctx).to_bits(), 0.5_f32.to_bits());
        let cond = LootItemRandomChanceCondition::<MockCtx>::random_chance_provider(Box::new(c));
        let mut ctx2 = MockCtx { rng: MockRng(0.25_f32) };
        assert!(cond.test(&mut ctx2));
    }
}
