#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    ExplosionDecay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
}

impl ItemStack {
    pub fn new(count: i32) -> Self {
        Self { count }
    }
}

pub trait LootContextLike {
    fn get_explosion_radius(&self) -> Option<f32>;
    fn next_float(&mut self) -> f32;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyExplosionDecay {
    pub conditions: Vec<LootItemCondition>,
}

impl ApplyExplosionDecay {
    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyExplosionDecay#<init>
    fn new(conditions: Vec<LootItemCondition>) -> Self {
        Self { conditions }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyExplosionDecay#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::ExplosionDecay
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyExplosionDecay#run
    pub fn run(&self, mut stack: ItemStack, ctx: &mut impl LootContextLike) -> ItemStack {
        if let Some(radius) = ctx.get_explosion_radius() {
            let chance: f32 = 1.0f32 / radius;
            let count: i32 = stack.count;
            let mut kept: i32 = 0;
            let mut i: i32 = 0;
            while i < count {
                if ctx.next_float() <= chance {
                    kept = kept.wrapping_add(1);
                }
                i = i.wrapping_add(1);
            }
            stack.count = kept;
        }
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.ApplyExplosionDecay#explosionDecay
    pub fn explosion_decay() -> impl FnOnce(Vec<LootItemCondition>) -> ApplyExplosionDecay {
        ApplyExplosionDecay::new
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCtx {
        radius: Option<f32>,
        floats: Vec<f32>,
        idx: usize,
    }

    impl MockCtx {
        fn new(radius: Option<f32>, floats: Vec<f32>) -> Self {
            Self { radius, floats, idx: 0 }
        }
    }

    impl LootContextLike for MockCtx {
        fn get_explosion_radius(&self) -> Option<f32> {
            self.radius
        }

        fn next_float(&mut self) -> f32 {
            if self.floats.is_empty() {
                return 0.0;
            }
            let v = self.floats[self.idx % self.floats.len()];
            self.idx = self.idx.wrapping_add(1);
            v
        }
    }

    #[test]
    fn no_explosion_radius_unchanged() {
        let func = ApplyExplosionDecay::new(vec![]);
        let mut ctx = MockCtx::new(None, vec![0.0]);
        let out = func.run(ItemStack::new(5), &mut ctx);
        assert_eq!(out.count, 5);
        assert_eq!(func.get_type(), LootItemFunctionType::ExplosionDecay);
    }

    #[test]
    fn explosion_decay_keeps_by_chance() {
        let func = ApplyExplosionDecay::new(vec![]);
        // radius 4.0 => chance 0.25; values <= 0.25 are kept
        let mut ctx = MockCtx::new(Some(4.0), vec![0.1, 0.25, 0.26, 0.9]);
        let out = func.run(ItemStack::new(4), &mut ctx);
        assert_eq!(out.count, 2);
    }

    #[test]
    fn zero_radius_gives_infinity_chance_and_float_bits() {
        let func = ApplyExplosionDecay::new(vec![]);
        let mut ctx = MockCtx::new(Some(0.0), vec![0.5, 1.0]);
        let out = func.run(ItemStack::new(2), &mut ctx);
        assert_eq!(out.count, 2);

        let chance = 1.0f32 / 4.0f32;
        assert_eq!(chance.to_bits(), 0.25f32.to_bits());
        let inf_chance = 1.0f32 / 0.0f32;
        assert!(inf_chance.is_infinite() && inf_chance.is_sign_positive());
    }

    #[test]
    fn negative_count_no_items_kept() {
        let func = ApplyExplosionDecay::new(vec![]);
        let mut ctx = MockCtx::new(Some(2.0), vec![0.0]);
        let out = func.run(ItemStack::new(-3), &mut ctx);
        assert_eq!(out.count, 0);
    }

    #[test]
    fn explosion_decay_builder_creates_function() {
        let builder = ApplyExplosionDecay::explosion_decay();
        let func = builder(vec![LootItemCondition]);
        assert_eq!(func.conditions.len(), 1);
    }
}
