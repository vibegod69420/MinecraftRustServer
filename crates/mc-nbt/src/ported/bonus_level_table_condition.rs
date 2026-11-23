use std::collections::HashSet;

/// Minimal context key for loot parameter references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub &'static str);

/// `LootContextParams` constants used by this condition.
pub mod loot_context_params {
    use super::ContextKey;

    pub const TOOL: ContextKey = ContextKey("minecraft:tool");
}

/// Minimal loot item condition type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootItemConditionType {
    TableBonus,
}

/// Minimal resource location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation(pub String);

/// Minimal registry holder wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Holder<T>(pub T);

/// Minimal enchantment representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Enchantment {
    pub id: ResourceLocation,
}

/// Minimal item stack carrying enchantment levels for condition evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub enchantments: Vec<(Holder<Enchantment>, i32)>,
}

/// Minimal enchantment helper.
pub struct EnchantmentHelper;

impl EnchantmentHelper {
    // JAVA: net.minecraft.world.item.enchantment.EnchantmentHelper#getItemEnchantmentLevel
    pub fn get_item_enchantment_level(enchantment: &Holder<Enchantment>, stack: &ItemStack) -> i32 {
        for (holder, lvl) in &stack.enchantments {
            if holder == enchantment {
                return *lvl;
            }
        }
        0
    }
}

/// Minimal random source providing Java `Random.nextFloat()` semantics.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

/// Minimal loot context providing tool parameter and random source.
pub trait LootContext {
    fn get_tool(&self) -> Option<&ItemStack>;
    fn random(&mut self) -> &mut dyn RandomSource;
}

/// Port of `net.minecraft.world.level.storage.loot.predicates.BonusLevelTableCondition`.
///
/// Record fields: `enchantment`, `values` (`chances` in codec).
#[derive(Debug, Clone)]
pub struct BonusLevelTableCondition {
    pub enchantment: Holder<Enchantment>,
    pub values: Vec<f32>,
}

impl BonusLevelTableCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.BonusLevelTableCondition#<init>
    pub fn new(enchantment: Holder<Enchantment>, values: Vec<f32>) -> Self {
        Self { enchantment, values }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.BonusLevelTableCondition#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::TableBonus
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.BonusLevelTableCondition#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        HashSet::from([loot_context_params::TOOL])
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.BonusLevelTableCondition#test
    pub fn test(&self, ctx: &mut dyn LootContext) -> bool {
        let tool: Option<&ItemStack> = ctx.get_tool();
        let level: i32 = match tool {
            Some(stack) => EnchantmentHelper::get_item_enchantment_level(&self.enchantment, stack),
            None => 0,
        };
        let max_idx: i32 = (self.values.len() as i32).wrapping_sub(1);
        let idx: i32 = level.min(max_idx);
        let chance: f32 = self.values[idx as usize];
        ctx.random().next_float() < chance
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.BonusLevelTableCondition#bonusLevelFlatChance
    pub fn bonus_level_flat_chance(
        enchantment: Holder<Enchantment>,
        chances: &[f32],
    ) -> impl FnOnce() -> Self {
        let mut values: Vec<f32> = Vec::with_capacity(chances.len());
        for &chance in chances {
            values.push(chance);
        }
        move || Self::new(enchantment, values)
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

    struct CountingRandom {
        value: f32,
        calls: u32,
    }

    impl RandomSource for CountingRandom {
        fn next_float(&mut self) -> f32 {
            self.calls = self.calls.wrapping_add(1);
            self.value
        }
    }

    struct TestContext<R: RandomSource> {
        tool: Option<ItemStack>,
        rng: R,
    }

    impl<R: RandomSource> LootContext for TestContext<R> {
        fn get_tool(&self) -> Option<&ItemStack> {
            self.tool.as_ref()
        }

        fn random(&mut self) -> &mut dyn RandomSource {
            &mut self.rng
        }
    }

    fn make_enchantment(id: &str) -> Holder<Enchantment> {
        Holder(Enchantment {
            id: ResourceLocation(id.to_string()),
        })
    }

    fn ctx_with(
        enchantment: &Holder<Enchantment>,
        level: i32,
        random: f32,
    ) -> TestContext<FixedRandom> {
        TestContext {
            tool: Some(ItemStack {
                enchantments: vec![(enchantment.clone(), level)],
            }),
            rng: FixedRandom { value: random },
        }
    }

    #[test]
    fn test_basic_selection_and_rng_always_consumed() {
        let ench = make_enchantment("minecraft:fortune");
        let cond = BonusLevelTableCondition::new(ench.clone(), vec![0.0, 0.25, 0.5]);

        let mut ctx = ctx_with(&ench, 0, 0.0);
        assert!(!cond.test(&mut ctx)); // 0.0 < 0.0 is false

        let mut ctx = ctx_with(&ench, 1, 0.1);
        assert!(cond.test(&mut ctx)); // 0.1 < 0.25

        let mut ctx = ctx_with(&ench, 1, 0.25);
        assert!(!cond.test(&mut ctx)); // strict <

        // RNG is consumed even when chance is 0.0
        let mut ctx = TestContext {
            tool: Some(ItemStack {
                enchantments: vec![(ench.clone(), 0)],
            }),
            rng: CountingRandom {
                value: 0.0,
                calls: 0,
            },
        };
        assert!(!cond.test(&mut ctx));
        assert_eq!(ctx.rng.calls, 1);
    }

    #[test]
    fn test_level_clamped_to_last_value() {
        let ench = make_enchantment("minecraft:fortune");
        let cond = BonusLevelTableCondition::new(ench.clone(), vec![0.1, 0.2, 0.3]);

        let mut ctx = ctx_with(&ench, 5, 0.25);
        assert!(cond.test(&mut ctx)); // uses values[2] = 0.3

        let mut ctx = ctx_with(&ench, i32::MAX, 0.35);
        assert!(!cond.test(&mut ctx));
    }

    #[test]
    fn test_no_tool_or_missing_enchantment_defaults_to_level_zero() {
        let ench = make_enchantment("minecraft:fortune");
        let other = make_enchantment("minecraft:looting");
        let cond = BonusLevelTableCondition::new(ench.clone(), vec![0.5, 1.0]);

        let mut ctx = TestContext {
            tool: None,
            rng: FixedRandom { value: 0.4 },
        };
        assert!(cond.test(&mut ctx));

        let mut ctx = TestContext {
            tool: Some(ItemStack {
                enchantments: vec![(other, 3)],
            }),
            rng: FixedRandom { value: 0.4 },
        };
        assert!(cond.test(&mut ctx));
    }

    #[test]
    #[should_panic]
    fn test_negative_enchantment_level_panics_like_java_oob() {
        let ench = make_enchantment("minecraft:fortune");
        let cond = BonusLevelTableCondition::new(ench.clone(), vec![0.5, 1.0]);
        let mut ctx = ctx_with(&ench, -1, 0.0);
        let _ = cond.test(&mut ctx);
    }

    #[test]
    fn test_float_bits_nan_and_signed_zero() {
        let ench = make_enchantment("minecraft:fortune");

        let cond_nan = BonusLevelTableCondition::new(ench.clone(), vec![f32::NAN]);
        let mut ctx = TestContext {
            tool: None,
            rng: FixedRandom { value: 0.0 },
        };
        assert!(!cond_nan.test(&mut ctx)); // 0.0 < NaN is false

        let neg_zero = f32::from_bits(0x8000_0000);
        assert_eq!(neg_zero.to_bits(), 0x8000_0000);
        let cond_neg_zero = BonusLevelTableCondition::new(ench.clone(), vec![neg_zero]);
        let mut ctx = TestContext {
            tool: None,
            rng: FixedRandom { value: 0.0 },
        };
        assert!(!cond_neg_zero.test(&mut ctx));

        let cond_one = BonusLevelTableCondition::new(ench, vec![1.0]);
        let mut ctx = TestContext {
            tool: None,
            rng: FixedRandom {
                value: f32::from_bits(0x3F7F_FFFF), // largest f32 < 1.0
            },
        };
        assert!(cond_one.test(&mut ctx));
    }

    #[test]
    fn test_builder_metadata_and_empty_values_wrapping() {
        let ench = make_enchantment("minecraft:fortune");
        let builder =
            BonusLevelTableCondition::bonus_level_flat_chance(ench.clone(), &[0.1, 0.2, 0.3]);
        let cond = builder();
        assert_eq!(cond.enchantment, ench);
        assert_eq!(cond.values, vec![0.1, 0.2, 0.3]);
        assert_eq!(cond.get_type(), LootItemConditionType::TableBonus);
        let params = cond.get_referenced_context_params();
        assert_eq!(params, HashSet::from([loot_context_params::TOOL]));

        // Empty values: size-1 wraps to -1 in i32, causing OOB like Java.
        let empty = BonusLevelTableCondition::new(make_enchantment("minecraft:fortune"), vec![]);
        let mut ctx = TestContext {
            tool: None,
            rng: FixedRandom { value: 0.0 },
        };
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| empty.test(&mut ctx)));
        assert!(result.is_err());
    }
}
