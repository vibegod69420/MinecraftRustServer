// JAVA: net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FireworkExplosion {
    pub shape: Shape,
    pub colors: Vec<i32>,
    pub fade_colors: Vec<i32>,
    pub trail: bool,
    pub twinkle: bool,
}

pub const DEFAULT_VALUE: FireworkExplosion = FireworkExplosion {
    shape: Shape::SmallBall,
    colors: Vec::new(),
    fade_colors: Vec::new(),
    trail: false,
    twinkle: false,
};

pub trait LootItemCondition {}

#[derive(Default)]
pub struct LootContext {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub firework_explosion: Option<FireworkExplosion>,
}

impl ItemStack {
    pub fn new() -> Self {
        Self { firework_explosion: None }
    }

    pub fn update_firework_explosion(
        &mut self,
        default: FireworkExplosion,
        f: impl FnOnce(FireworkExplosion) -> FireworkExplosion,
    ) {
        let current = self.firework_explosion.clone().unwrap_or(default);
        self.firework_explosion = Some(f(current));
    }
}

pub struct LootItemConditionalFunction {
    pub conditions: Vec<Box<dyn LootItemCondition>>,
}

pub struct SetFireworkExplosionFunction {
    pub conditions: Vec<Box<dyn LootItemCondition>>,
    pub shape: Option<Shape>,
    pub colors: Option<Vec<i32>>,
    pub fade_colors: Option<Vec<i32>>,
    pub trail: Option<bool>,
    pub twinkle: Option<bool>,
}

impl SetFireworkExplosionFunction {
    pub fn new(
        conditions: Vec<Box<dyn LootItemCondition>>,
        shape: Option<Shape>,
        colors: Option<Vec<i32>>,
        fade_colors: Option<Vec<i32>>,
        trail: Option<bool>,
        twinkle: Option<bool>,
    ) -> Self {
        Self {
            conditions,
            shape,
            colors,
            fade_colors,
            trail,
            twinkle,
        }
    }

    pub fn get_type(&self) -> &'static str {
        "set_firework_explosion"
    }

    pub fn apply_explosion(&self, base: FireworkExplosion) -> FireworkExplosion {
        FireworkExplosion {
            shape: self.shape.unwrap_or(base.shape),
            colors: self.colors.clone().unwrap_or(base.colors),
            fade_colors: self.fade_colors.clone().unwrap_or(base.fade_colors),
            trail: self.trail.unwrap_or(base.trail),
            twinkle: self.twinkle.unwrap_or(base.twinkle),
        }
    }

    pub fn run(&self, mut stack: ItemStack, _ctx: &LootContext) -> ItemStack {
        stack.update_firework_explosion(DEFAULT_VALUE.clone(), |base| self.apply_explosion(base));
        stack
    }

    pub fn apply(&self, stack: ItemStack, ctx: &mut LootContext) -> ItemStack {
        self.run(stack, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_none_keeps_base() {
        let f = SetFireworkExplosionFunction::new(Vec::new(), None, None, None, None, None);
        let base = FireworkExplosion {
            shape: Shape::Star,
            colors: vec![1, 2],
            fade_colors: vec![3],
            trail: true,
            twinkle: true,
        };
        let out = f.apply_explosion(base.clone());
        assert_eq!(out, base);
    }

    #[test]
    fn overrides_selected_fields() {
        let f = SetFireworkExplosionFunction::new(
            Vec::new(),
            Some(Shape::Burst),
            Some(vec![9]),
            None,
            Some(false),
            None,
        );
        let base = DEFAULT_VALUE.clone();
        let out = f.apply_explosion(base);
        assert_eq!(out.shape, Shape::Burst);
        assert_eq!(out.colors, vec![9]);
        assert_eq!(out.fade_colors, Vec::<i32>::new());
        assert!(!out.trail);
        assert!(!out.twinkle);
    }

    #[test]
    fn run_on_empty_stack_uses_default() {
        let f = SetFireworkExplosionFunction::new(
            Vec::new(),
            Some(Shape::Creeper),
            None,
            None,
            Some(true),
            Some(true),
        );
        let stack = ItemStack::new();
        let ctx = LootContext::default();
        let out = f.run(stack, &ctx);
        let explosion = out.firework_explosion.unwrap();
        assert_eq!(explosion.shape, Shape::Creeper);
        assert!(explosion.trail);
        assert!(explosion.twinkle);
    }

    #[test]
    fn run_preserves_existing_unspecified_fields() {
        let f = SetFireworkExplosionFunction::new(Vec::new(), None, Some(vec![7]), None, None, None);
        let mut stack = ItemStack::new();
        stack.firework_explosion = Some(FireworkExplosion {
            shape: Shape::LargeBall,
            colors: vec![1],
            fade_colors: vec![2],
            trail: true,
            twinkle: false,
        });
        let ctx = LootContext::default();
        let out = f.run(stack, &ctx);
        let explosion = out.firework_explosion.unwrap();
        assert_eq!(explosion.shape, Shape::LargeBall);
        assert_eq!(explosion.colors, vec![7]);
        assert_eq!(explosion.fade_colors, vec![2]);
        assert!(explosion.trail);
        assert!(!explosion.twinkle);
    }

    #[test]
    fn default_value_is_small_ball() {
        assert_eq!(DEFAULT_VALUE.shape, Shape::SmallBall);
        assert!(DEFAULT_VALUE.colors.is_empty());
        assert!(!DEFAULT_VALUE.trail);
        assert!(!DEFAULT_VALUE.twinkle);
    }
}
