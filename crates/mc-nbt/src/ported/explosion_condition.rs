use std::collections::HashSet;

/// Local stand-in for `net.minecraft.util.context.ContextKey<?>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextKey {
    ExplosionRadius,
}

/// Local stand-in for `net.minecraft.world.level.storage.loot.predicates.LootItemConditionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemConditionType {
    SurvivesExplosion,
}

/// Local stand-in for `net.minecraft.util.RandomSource`.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

/// Minimal context surface required by `ExplosionCondition`.
pub trait LootContext {
    type Rng: RandomSource;

    fn get_optional_parameter_explosion_radius(&self) -> Option<f32>;
    fn get_random(&mut self) -> &mut Self::Rng;
}

/// Port of `net.minecraft.world.level.storage.loot.predicates.ExplosionCondition`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExplosionCondition;

impl ExplosionCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.ExplosionCondition#<init>
    fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ExplosionCondition#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::SurvivesExplosion
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ExplosionCondition#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        let mut set = HashSet::new();
        set.insert(ContextKey::ExplosionRadius);
        set
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ExplosionCondition#test
    pub fn test<C: LootContext>(&self, ctx: &mut C) -> bool {
        let radius: Option<f32> = ctx.get_optional_parameter_explosion_radius();
        if let Some(radius) = radius {
            let rng = ctx.get_random();
            let chance: f32 = 1.0_f32 / radius;
            rng.next_float() <= chance
        } else {
            true
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ExplosionCondition#survivesExplosion
    pub fn survives_explosion() -> Self {
        Self::new()
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
        radius: Option<f32>,
        rng: MockRng,
    }

    impl LootContext for MockCtx {
        type Rng = MockRng;

        fn get_optional_parameter_explosion_radius(&self) -> Option<f32> {
            self.radius
        }

        fn get_random(&mut self) -> &mut Self::Rng {
            &mut self.rng
        }
    }

    #[test]
    fn no_radius_returns_true() {
        let cond = ExplosionCondition::survives_explosion();
        let mut ctx = MockCtx {
            radius: None,
            rng: MockRng(0.999),
        };
        assert!(cond.test(&mut ctx));
        assert_eq!(cond.get_type(), LootItemConditionType::SurvivesExplosion);
        let params = cond.get_referenced_context_params();
        assert!(params.contains(&ContextKey::ExplosionRadius));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn normal_radius_uses_java_order_and_comparison() {
        let cond = ExplosionCondition::survives_explosion();
        let mut ctx = MockCtx {
            radius: Some(4.0_f32),
            rng: MockRng(0.25_f32),
        };
        // 1.0 / 4.0 == 0.25, nextFloat() <= 0.25 => true
        assert!(cond.test(&mut ctx));

        let mut ctx2 = MockCtx {
            radius: Some(4.0_f32),
            rng: MockRng(0.250_000_03_f32),
        };
        assert!(!cond.test(&mut ctx2));
    }

    #[test]
    fn zero_radius_produces_infinity_chance() {
        let cond = ExplosionCondition::survives_explosion();
        let mut ctx = MockCtx {
            radius: Some(0.0_f32),
            rng: MockRng(0.5_f32),
        };
        let chance = 1.0_f32 / 0.0_f32;
        assert!(chance.is_infinite() && chance.is_sign_positive());
        assert!(cond.test(&mut ctx));
    }

    #[test]
    fn negative_and_nan_radius_float_bits() {
        let cond = ExplosionCondition::survives_explosion();

        let mut neg_ctx = MockCtx {
            radius: Some(-4.0_f32),
            rng: MockRng(0.0_f32),
        };
        // 1.0 / -4.0 = -0.25, 0.0 <= -0.25 is false
        assert_eq!((1.0_f32 / -4.0_f32).to_bits(), (-0.25_f32).to_bits());
        assert!(!cond.test(&mut neg_ctx));

        let mut nan_ctx = MockCtx {
            radius: Some(f32::NAN),
            rng: MockRng(0.0_f32),
        };
        // 1.0 / NaN = NaN, comparison with NaN is false
        assert!(!cond.test(&mut nan_ctx));
    }

    #[test]
    fn negative_zero_radius_preserves_float_semantics() {
        let cond = ExplosionCondition::survives_explosion();
        let mut ctx = MockCtx {
            radius: Some(-0.0_f32),
            rng: MockRng(0.0_f32),
        };
        let chance = 1.0_f32 / -0.0_f32;
        assert!(chance.is_infinite() && chance.is_sign_negative());
        // 0.0 <= -inf is false
        assert!(!cond.test(&mut ctx));
    }
}
