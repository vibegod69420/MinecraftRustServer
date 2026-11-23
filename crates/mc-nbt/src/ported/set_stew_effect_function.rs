use std::collections::HashSet;
use std::fmt;

pub type ContextKey = String;

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub struct LootContext<'a> {
    rng: &'a mut dyn RandomSource,
}

impl<'a> LootContext<'a> {
    pub fn new(rng: &'a mut dyn RandomSource) -> Self {
        Self { rng }
    }

    pub fn get_random(&mut self) -> &mut dyn RandomSource {
        self.rng
    }
}

pub trait NumberProvider: fmt::Debug {
    fn get_int(&self, ctx: &mut LootContext<'_>) -> i32;
    fn get_referenced_context_params(&self) -> Vec<ContextKey>;
}

pub trait LootItemCondition: fmt::Debug {
    fn test(&self, ctx: &mut LootContext<'_>) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootItemFunctionType {
    SetStewEffect,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MobEffect {
    pub id: String,
    pub instantaneous: bool,
}

impl MobEffect {
    pub fn is_instantenous(&self) -> bool {
        self.instantaneous
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Holder<T>(pub T);

impl<T> Holder<T> {
    pub fn value(&self) -> &T {
        &self.0
    }
}

pub const SUSPICIOUS_STEW: &str = "minecraft:suspicious_stew";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuspiciousStewEntry {
    pub effect: Holder<MobEffect>,
    pub duration: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SuspiciousStewEffects {
    pub effects: Vec<SuspiciousStewEntry>,
}

impl SuspiciousStewEffects {
    pub const EMPTY: Self = Self {
        effects: Vec::new(),
    };

    pub fn with_effect_added(&self, entry: SuspiciousStewEntry) -> Self {
        let mut effects = self.effects.clone();
        effects.push(entry);
        Self { effects }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item: String,
    pub suspicious_stew_effects: Option<SuspiciousStewEffects>,
}

impl ItemStack {
    pub fn new(item: impl Into<String>) -> Self {
        Self {
            item: item.into(),
            suspicious_stew_effects: None,
        }
    }

    pub fn is(&self, item: &str) -> bool {
        self.item == item
    }

    pub fn update_suspicious_stew_effects(
        &mut self,
        default: SuspiciousStewEffects,
        entry: SuspiciousStewEntry,
    ) {
        let current = self.suspicious_stew_effects.take().unwrap_or(default);
        self.suspicious_stew_effects = Some(current.with_effect_added(entry));
    }
}

pub struct EffectEntry {
    pub effect: Holder<MobEffect>,
    pub duration: Box<dyn NumberProvider>,
}

impl fmt::Debug for EffectEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EffectEntry")
            .field("effect", &self.effect)
            .field("duration", &self.duration)
            .finish()
    }
}

pub struct SetStewEffectFunction {
    pub conditions: Vec<Box<dyn LootItemCondition>>,
    pub effects: Vec<EffectEntry>,
}

impl fmt::Debug for SetStewEffectFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SetStewEffectFunction")
            .field("conditions", &self.conditions)
            .field("effects", &self.effects)
            .finish()
    }
}

// JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction#EFFECTS_LIST
pub fn validate_effects_list(effects: &[EffectEntry]) -> Result<(), String> {
    let mut seen: HashSet<Holder<MobEffect>> = HashSet::new();
    for entry in effects {
        if !seen.insert(entry.effect.clone()) {
            return Err(format!(
                "Encountered duplicate mob effect: '{:?}'",
                entry.effect
            ));
        }
    }
    Ok(())
}

impl SetStewEffectFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction#<init>
    pub fn new(conditions: Vec<Box<dyn LootItemCondition>>, effects: Vec<EffectEntry>) -> Self {
        Self { conditions, effects }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::SetStewEffect
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        self.effects
            .iter()
            .flat_map(|e| e.duration.get_referenced_context_params())
            .collect()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction#run
    pub fn run(&self, mut stack: ItemStack, ctx: &mut LootContext<'_>) -> ItemStack {
        if stack.is(SUSPICIOUS_STEW) && !self.effects.is_empty() {
            let idx = ctx.get_random().next_int(self.effects.len() as i32) as usize;
            let entry = &self.effects[idx];
            let effect = entry.effect.clone();
            let mut duration = entry.duration.get_int(ctx);
            if !effect.value().is_instantenous() {
                duration = duration.wrapping_mul(20);
            }
            let stew_entry = SuspiciousStewEntry { effect, duration };
            stack.update_suspicious_stew_effects(SuspiciousStewEffects::EMPTY, stew_entry);
            stack
        } else {
            stack
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction#stewEffect
    pub fn stew_effect() -> Builder {
        Builder::new()
    }
}

pub struct Builder {
    conditions: Vec<Box<dyn LootItemCondition>>,
    effects: Vec<EffectEntry>,
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("conditions", &self.conditions)
            .field("effects", &self.effects)
            .finish()
    }
}

impl Builder {
    fn new() -> Self {
        Self {
            conditions: Vec::new(),
            effects: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction$Builder#getThis
    pub fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction$Builder#withEffect
    pub fn with_effect(mut self, effect: Holder<MobEffect>, duration: Box<dyn NumberProvider>) -> Self {
        self.effects.push(EffectEntry { effect, duration });
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction$Builder#build
    pub fn build(self) -> SetStewEffectFunction {
        SetStewEffectFunction::new(self.conditions, self.effects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FixedRng(i32);
    impl RandomSource for FixedRng {
        fn next_int(&mut self, _bound: i32) -> i32 {
            self.0
        }
    }

    #[derive(Debug)]
    struct ConstNum(i32);
    impl NumberProvider for ConstNum {
        fn get_int(&self, _ctx: &mut LootContext<'_>) -> i32 {
            self.0
        }
        fn get_referenced_context_params(&self) -> Vec<ContextKey> {
            vec![]
        }
    }

    #[derive(Debug)]
    struct KeyNum(i32, Vec<ContextKey>);
    impl NumberProvider for KeyNum {
        fn get_int(&self, _ctx: &mut LootContext<'_>) -> i32 {
            self.0
        }
        fn get_referenced_context_params(&self) -> Vec<ContextKey> {
            self.1.clone()
        }
    }

    fn effect(id: &str, instantaneous: bool) -> Holder<MobEffect> {
        Holder(MobEffect {
            id: id.to_string(),
            instantaneous,
        })
    }

    #[test]
    fn applies_effect_and_multiplies_duration_by_20() {
        let func = SetStewEffectFunction::new(
            vec![],
            vec![EffectEntry {
                effect: effect("minecraft:speed", false),
                duration: Box::new(ConstNum(7)),
            }],
        );
        let mut rng = FixedRng(0);
        let mut ctx = LootContext::new(&mut rng);
        let out = func.run(ItemStack::new(SUSPICIOUS_STEW), &mut ctx);
        let effects = &out.suspicious_stew_effects.as_ref().unwrap().effects;
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].effect.value().id, "minecraft:speed");
        assert_eq!(effects[0].duration, 140);
    }

    #[test]
    fn instantaneous_effect_does_not_multiply_duration() {
        let func = SetStewEffectFunction::new(
            vec![],
            vec![EffectEntry {
                effect: effect("minecraft:instant_health", true),
                duration: Box::new(ConstNum(5)),
            }],
        );
        let mut rng = FixedRng(0);
        let mut ctx = LootContext::new(&mut rng);
        let out = func.run(ItemStack::new(SUSPICIOUS_STEW), &mut ctx);
        let dur = out.suspicious_stew_effects.unwrap().effects[0].duration;
        assert_eq!(dur, 5);
    }

    #[test]
    fn duration_wrapping_overflow_and_negative_inputs() {
        let func = SetStewEffectFunction::new(
            vec![],
            vec![
                EffectEntry {
                    effect: effect("minecraft:speed", false),
                    duration: Box::new(ConstNum(i32::MAX)),
                },
                EffectEntry {
                    effect: effect("minecraft:slowness", false),
                    duration: Box::new(ConstNum(-3)),
                },
            ],
        );

        let mut rng0 = FixedRng(0);
        let mut ctx0 = LootContext::new(&mut rng0);
        let out0 = func.run(ItemStack::new(SUSPICIOUS_STEW), &mut ctx0);
        assert_eq!(
            out0.suspicious_stew_effects.unwrap().effects[0].duration,
            i32::MAX.wrapping_mul(20)
        );

        let mut rng1 = FixedRng(1);
        let mut ctx1 = LootContext::new(&mut rng1);
        let out1 = func.run(ItemStack::new(SUSPICIOUS_STEW), &mut ctx1);
        assert_eq!(
            out1.suspicious_stew_effects.unwrap().effects[0].duration,
            (-3i32).wrapping_mul(20)
        );
    }

    #[test]
    fn duration_float_bits_pattern_preserved_as_i32() {
        let bits = f32::NAN.to_bits() as i32;
        let func = SetStewEffectFunction::new(
            vec![],
            vec![EffectEntry {
                effect: effect("minecraft:jump_boost", false),
                duration: Box::new(ConstNum(bits)),
            }],
        );
        let mut rng = FixedRng(0);
        let mut ctx = LootContext::new(&mut rng);
        let out = func.run(ItemStack::new(SUSPICIOUS_STEW), &mut ctx);
        let dur = out.suspicious_stew_effects.unwrap().effects[0].duration;
        assert_eq!(dur, bits.wrapping_mul(20));
        assert_eq!(dur.to_ne_bytes(), bits.wrapping_mul(20).to_ne_bytes());
    }

    #[test]
    fn non_stew_or_empty_effects_returns_unchanged() {
        let stack = ItemStack::new("minecraft:apple");
        let func = SetStewEffectFunction::new(
            vec![],
            vec![EffectEntry {
                effect: effect("minecraft:speed", false),
                duration: Box::new(ConstNum(1)),
            }],
        );
        let mut rng = FixedRng(0);
        let mut ctx = LootContext::new(&mut rng);
        assert_eq!(func.run(stack.clone(), &mut ctx), stack);

        let stew = ItemStack::new(SUSPICIOUS_STEW);
        let empty_func = SetStewEffectFunction::new(vec![], vec![]);
        let mut rng2 = FixedRng(0);
        let mut ctx2 = LootContext::new(&mut rng2);
        assert_eq!(empty_func.run(stew.clone(), &mut ctx2), stew);
    }

    #[test]
    fn referenced_context_params_and_duplicate_validation() {
        let func = SetStewEffectFunction::new(
            vec![],
            vec![
                EffectEntry {
                    effect: effect("minecraft:speed", false),
                    duration: Box::new(KeyNum(1, vec!["a".into(), "b".into()])),
                },
                EffectEntry {
                    effect: effect("minecraft:strength", false),
                    duration: Box::new(KeyNum(2, vec!["b".into(), "c".into()])),
                },
            ],
        );
        let params = func.get_referenced_context_params();
        let expected: HashSet<ContextKey> = ["a", "b", "c"].into_iter().map(String::from).collect();
        assert_eq!(params, expected);
        assert_eq!(func.get_type(), LootItemFunctionType::SetStewEffect);

        let dup = vec![
            EffectEntry {
                effect: effect("minecraft:speed", false),
                duration: Box::new(ConstNum(1)),
            },
            EffectEntry {
                effect: effect("minecraft:speed", false),
                duration: Box::new(ConstNum(2)),
            },
        ];
        let err = validate_effects_list(&dup).unwrap_err();
        assert!(err.contains("Encountered duplicate mob effect"));
    }
}
