use std::collections::BTreeMap;

const POTION_ITEMS: &[&str] = &[
    "minecraft:potion",
    "minecraft:splash_potion",
    "minecraft:lingering_potion",
    "minecraft:tipped_arrow",
];

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Int(i32),
    String(String),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> &Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).unwrap_or(&Dynamic::Null),
            _ => &Dynamic::Null,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            _ => default,
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.remove(key);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.insert(key.to_string(), value);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }

    pub fn update<F>(&self, key: &str, f: F) -> Dynamic
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(m) => {
                if let Some(v) = m.get(key) {
                    let mut out = m.clone();
                    out.insert(key.to_string(), f(v.clone()));
                    Dynamic::Map(out)
                } else {
                    self.clone()
                }
            }
            _ => self.clone(),
        }
    }

    pub fn create_int(&self, _v: i32) -> Dynamic {
        Dynamic::Int(_v)
    }

    pub fn create_list<I>(&self, items: I) -> Dynamic
    where
        I: IntoIterator<Item = Dynamic>,
    {
        Dynamic::List(items.into_iter().collect())
    }

    pub fn as_stream(&self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(v) => v.clone(),
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Schema;

#[derive(Clone, Copy, Debug)]
pub struct TypeRewriteRule {
    fix: EffectDurationFix,
}

impl TypeRewriteRule {
    pub fn fix_entity(&self, d: Dynamic) -> Dynamic {
        self.fix.update_entity(d)
    }

    pub fn fix_player(&self, d: Dynamic) -> Dynamic {
        self.fix.update_entity(d)
    }

    pub fn fix_item_stack(&self, d: Dynamic) -> Dynamic {
        let id = d.get("id").as_string("");
        if !POTION_ITEMS.contains(&id.as_str()) {
            return d;
        }
        match d.get("tag") {
            Dynamic::Map(_) => d.update("tag", |tag| {
                tag.update("CustomPotionEffects", |cpe| self.fix.fix(cpe))
            }),
            _ => d,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EffectDurationFix;

impl EffectDurationFix {
    // JAVA: net.minecraft.util.datafix.fixes.EffectDurationFix#<init>
    pub fn new(_schema: Schema) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EffectDurationFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule { fix: *self }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EffectDurationFix#fixEffect
    fn fix_effect(&self, effect: Dynamic) -> Dynamic {
        effect.update("FactorCalculationData", |fcd| {
            let effect_changed_timestamp = fcd.get("effect_changed_timestamp").as_int(-1);
            let fcd = fcd.remove("effect_changed_timestamp");
            let duration = effect.get("Duration").as_int(-1);
            let ticks_active = effect_changed_timestamp.wrapping_sub(duration);
            let ticks_active_val = fcd.create_int(ticks_active);
            fcd.set("ticks_active", ticks_active_val)
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.EffectDurationFix#fix
    fn fix(&self, d: Dynamic) -> Dynamic {
        let fixed: Vec<Dynamic> = d
            .as_stream()
            .into_iter()
            .map(|e| self.fix_effect(e))
            .collect();
        d.create_list(fixed)
    }

    // JAVA: net.minecraft.util.datafix.fixes.EffectDurationFix#updateEntity
    fn update_entity(&self, entity: Dynamic) -> Dynamic {
        let entity = entity.update("Effects", |d| self.fix(d));
        let entity = entity.update("ActiveEffects", |d| self.fix(d));
        entity.update("CustomPotionEffects", |d| self.fix(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    fn effect(duration: i32, changed_ts: i32) -> Dynamic {
        map(vec![
            ("Duration", Dynamic::Int(duration)),
            (
                "FactorCalculationData",
                map(vec![("effect_changed_timestamp", Dynamic::Int(changed_ts))]),
            ),
        ])
    }

    #[test]
    fn fix_effect_computes_ticks_active() {
        let fix = EffectDurationFix::new(Schema);
        let fixed = fix.fix_effect(effect(100, 500));
        let fcd = fixed.get("FactorCalculationData");
        assert_eq!(fcd.get("ticks_active").as_int(-999), 400);
        assert!(matches!(fcd.get("effect_changed_timestamp"), Dynamic::Null));
    }

    #[test]
    fn fix_effect_wrapping_and_negative_inputs() {
        let fix = EffectDurationFix::new(Schema);
        let fixed = fix.fix_effect(effect(i32::MAX, i32::MIN));
        let expected = i32::MIN.wrapping_sub(i32::MAX);
        assert_eq!(
            fixed
                .get("FactorCalculationData")
                .get("ticks_active")
                .as_int(0),
            expected
        );

        let fixed_default = fix.fix_effect(map(vec![("FactorCalculationData", map(vec![]))]));
        // defaults: -1 - (-1) == 0
        assert_eq!(
            fixed_default
                .get("FactorCalculationData")
                .get("ticks_active")
                .as_int(1),
            0
        );
    }

    #[test]
    fn update_entity_fixes_all_effect_lists() {
        let fix = EffectDurationFix::new(Schema);
        let entity = map(vec![
            ("Effects", Dynamic::List(vec![effect(3, 10)])),
            ("ActiveEffects", Dynamic::List(vec![effect(5, 20)])),
            ("CustomPotionEffects", Dynamic::List(vec![effect(7, 30)])),
        ]);
        let fixed = fix.update_entity(entity);
        let ticks = |key: &str| {
            fixed.get(key).as_stream()[0]
                .get("FactorCalculationData")
                .get("ticks_active")
                .as_int(i32::MIN)
        };
        assert_eq!(ticks("Effects"), 7);
        assert_eq!(ticks("ActiveEffects"), 15);
        assert_eq!(ticks("CustomPotionEffects"), 23);
    }

    #[test]
    fn make_rule_fixes_potion_item_stack() {
        let rule = EffectDurationFix::new(Schema).make_rule();
        let item = map(vec![
            ("id", Dynamic::String("minecraft:potion".to_string())),
            (
                "tag",
                map(vec![(
                    "CustomPotionEffects",
                    Dynamic::List(vec![effect(50, 200)]),
                )]),
            ),
        ]);
        let fixed = rule.fix_item_stack(item);
        let ticks = fixed.get("tag").get("CustomPotionEffects").as_stream()[0]
            .get("FactorCalculationData")
            .get("ticks_active")
            .as_int(0);
        assert_eq!(ticks, 150);
    }

    #[test]
    fn make_rule_ignores_non_potion_and_missing_tag() {
        let rule = EffectDurationFix::new(Schema).make_rule();
        let non_potion = map(vec![
            ("id", Dynamic::String("minecraft:apple".to_string())),
            (
                "tag",
                map(vec![(
                    "CustomPotionEffects",
                    Dynamic::List(vec![effect(50, 200)]),
                )]),
            ),
        ]);
        assert_eq!(rule.fix_item_stack(non_potion.clone()), non_potion);

        let no_tag = map(vec![(
            "id",
            Dynamic::String("minecraft:splash_potion".to_string()),
        )]);
        assert_eq!(rule.fix_item_stack(no_tag.clone()), no_tag);
    }
}
