use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Map(BTreeMap<String, DynamicValue>),
    Int(i32),
    Long(i64),
    String(String),
    IntList(Vec<i32>),
    Other,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynamicValue,
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Self {
            value: DynamicValue::Map(BTreeMap::new()),
        }
    }

    pub fn of_map(map: BTreeMap<String, DynamicValue>) -> Self {
        Self {
            value: DynamicValue::Map(map),
        }
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match &self.value {
            DynamicValue::Map(m) => OptionalDynamic {
                value: m.get(key).cloned(),
            },
            _ => OptionalDynamic { value: None },
        }
    }

    pub fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self.value {
            DynamicValue::Map(mut m) => {
                if let Some(v) = m.remove(key) {
                    let updated = f(Dynamic { value: v });
                    m.insert(key.to_string(), updated.value);
                }
                Dynamic {
                    value: DynamicValue::Map(m),
                }
            }
            other => Dynamic { value: other },
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self.value {
            DynamicValue::Map(mut m) => {
                m.insert(key.to_string(), value.value);
                Dynamic {
                    value: DynamicValue::Map(m),
                }
            }
            other => Dynamic { value: other },
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match &self.value {
            DynamicValue::Int(v) => Some(*v),
            DynamicValue::Long(v) => i32::try_from(*v).ok(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic {
    value: Option<DynamicValue>,
}

impl OptionalDynamic {
    pub fn get(&self, key: &str) -> OptionalDynamic {
        match &self.value {
            Some(DynamicValue::Map(m)) => OptionalDynamic {
                value: m.get(key).cloned(),
            },
            _ => OptionalDynamic { value: None },
        }
    }

    pub fn result(&self) -> Option<Dynamic> {
        self.value.clone().map(|value| Dynamic { value })
    }

    pub fn or_else_empty_map(self) -> Dynamic {
        match self.value {
            Some(value) => Dynamic { value },
            None => Dynamic::empty_map(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug)]
pub struct TypeRewriteRule {
    pub name: &'static str,
    pub fix: fn(Dynamic) -> Dynamic,
}

// JAVA: net.minecraft.util.datafix.ExtraDataFixUtils#fixBlockPos
pub fn fix_block_pos(d: Dynamic) -> Dynamic {
    let x = d.get("X").result().and_then(|v| v.as_i32());
    let y = d.get("Y").result().and_then(|v| v.as_i32());
    let z = d.get("Z").result().and_then(|v| v.as_i32());
    match (x, y, z) {
        (Some(x), Some(y), Some(z)) => Dynamic {
            value: DynamicValue::IntList(vec![x, y, z]),
        },
        _ => d,
    }
}

pub struct LegacyDragonFightFix {
    _output_schema: Schema,
}

impl LegacyDragonFightFix {
    // JAVA: net.minecraft.util.datafix.fixes.LegacyDragonFightFix#<init>
    pub fn new(output_schema: Schema) -> Self {
        Self {
            _output_schema: output_schema,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.LegacyDragonFightFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            name: "LegacyDragonFightFix",
            fix: fix_level,
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.LegacyDragonFightFix#fixDragonFight
fn fix_dragon_fight(d: Dynamic) -> Dynamic {
    d.update("ExitPortalLocation", fix_block_pos)
}

fn fix_level(level: Dynamic) -> Dynamic {
    let dragon_fight = level.get("DragonFight");
    if dragon_fight.result().is_some() {
        return level;
    }

    let legacy = level
        .get("DimensionData")
        .get("1")
        .get("DragonFight")
        .or_else_empty_map();
    level.set("DragonFight", fix_dragon_fight(legacy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn make_exit_portal(x: i32, y: i32, z: i32) -> DynamicValue {
        let mut m = BTreeMap::new();
        m.insert("X".to_string(), DynamicValue::Int(x));
        m.insert("Y".to_string(), DynamicValue::Int(y));
        m.insert("Z".to_string(), DynamicValue::Int(z));
        DynamicValue::Map(m)
    }

    #[test]
    fn existing_dragon_fight_unchanged() {
        let mut df = BTreeMap::new();
        df.insert("DragonKilled".to_string(), DynamicValue::Int(1));
        let mut level_map = BTreeMap::new();
        level_map.insert("DragonFight".to_string(), DynamicValue::Map(df));
        let level = Dynamic::of_map(level_map);

        let rule = LegacyDragonFightFix::new(Schema).make_rule();
        let out = (rule.fix)(level.clone());
        assert_eq!(out, level);
    }

    #[test]
    fn migrates_legacy_dragon_fight_and_fixes_exit_portal() {
        let mut dragon = BTreeMap::new();
        dragon.insert("ExitPortalLocation".to_string(), make_exit_portal(1, 2, 3));
        let mut dim1 = BTreeMap::new();
        dim1.insert("DragonFight".to_string(), DynamicValue::Map(dragon));
        let mut dim_data = BTreeMap::new();
        dim_data.insert("1".to_string(), DynamicValue::Map(dim1));
        let mut level_map = BTreeMap::new();
        level_map.insert("DimensionData".to_string(), DynamicValue::Map(dim_data));
        let level = Dynamic::of_map(level_map);

        let out = (LegacyDragonFightFix::new(Schema).make_rule().fix)(level);
        let df = out.get("DragonFight").result().unwrap();
        let exit = df.get("ExitPortalLocation").result().unwrap();
        assert_eq!(exit.value, DynamicValue::IntList(vec![1, 2, 3]));
    }

    #[test]
    fn missing_legacy_path_creates_empty_dragon_fight() {
        let level = Dynamic::empty_map();
        let out = (LegacyDragonFightFix::new(Schema).make_rule().fix)(level);
        let df = out.get("DragonFight").result().unwrap();
        assert_eq!(df.value, DynamicValue::Map(BTreeMap::new()));
    }

    #[test]
    fn fix_block_pos_negative_and_extreme_coords() {
        let d = Dynamic {
            value: make_exit_portal(-1, i32::MIN, i32::MAX),
        };
        let out = fix_block_pos(d);
        assert_eq!(
            out.value,
            DynamicValue::IntList(vec![-1, i32::MIN, i32::MAX])
        );
    }

    #[test]
    fn fix_block_pos_missing_field_returns_original() {
        let mut m = BTreeMap::new();
        m.insert("X".to_string(), DynamicValue::Int(1));
        m.insert("Y".to_string(), DynamicValue::Int(2));
        let d = Dynamic {
            value: DynamicValue::Map(m),
        };
        let out = fix_block_pos(d.clone());
        assert_eq!(out, d);
    }

    #[test]
    fn fix_dragon_fight_updates_only_exit_portal() {
        let mut dragon = BTreeMap::new();
        dragon.insert("ExitPortalLocation".to_string(), make_exit_portal(10, 64, -20));
        dragon.insert("DragonKilled".to_string(), DynamicValue::Int(0));
        let d = Dynamic {
            value: DynamicValue::Map(dragon),
        };
        let out = fix_dragon_fight(d);
        assert_eq!(
            out.get("ExitPortalLocation").result().unwrap().value,
            DynamicValue::IntList(vec![10, 64, -20])
        );
        assert_eq!(
            out.get("DragonKilled").result().unwrap().value,
            DynamicValue::Int(0)
        );
    }
}
