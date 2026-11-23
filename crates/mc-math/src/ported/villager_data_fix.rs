use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    Seq(Vec<Dynamic>),
    Str(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i32(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            Dynamic::Long(v) => *v as i32,
            _ => default,
        }
    }

    pub fn remove(self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn create_map(&self, entries: Vec<(Dynamic, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            if let Dynamic::Str(s) = k {
                m.insert(s, v);
            }
        }
        Dynamic::Map(m)
    }

    pub fn create_string(&self, s: String) -> Dynamic {
        Dynamic::Str(s)
    }

    pub fn create_int(&self, v: i32) -> Dynamic {
        Dynamic::Int(v)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    pub fn get_remainder(&self) -> &Dynamic {
        &self.value
    }

    pub fn set_remainder(self, value: Dynamic) -> Typed {
        Typed { value }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct VillagerDataFix {
    pub name: String,
}

impl VillagerDataFix {
    // JAVA: net.minecraft.util.datafix.fixes.VillagerDataFix#<init>
    pub fn new(_schema: Schema, name: String) -> Self {
        Self { name }
    }

    // JAVA: net.minecraft.util.datafix.fixes.VillagerDataFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        let d = typed.get_remainder().clone();
        let profession = d.get("Profession").map(|v| v.as_i32(0)).unwrap_or(0);
        let career = d.get("Career").map(|v| v.as_i32(0)).unwrap_or(0);
        let career_level = d.get("CareerLevel").cloned();

        let src = typed.get_remainder();
        let villager_data = src.create_map(vec![
            (
                src.create_string("type".to_string()),
                src.create_string("minecraft:plains".to_string()),
            ),
            (
                src.create_string("profession".to_string()),
                src.create_string(Self::upgrade_data(profession, career).to_string()),
            ),
            (
                src.create_string("level".to_string()),
                career_level.unwrap_or_else(|| src.create_int(1)),
            ),
        ]);

        let new_d = d
            .remove("Profession")
            .remove("Career")
            .remove("CareerLevel")
            .set("VillagerData", villager_data);
        typed.set_remainder(new_d)
    }

    // JAVA: net.minecraft.util.datafix.fixes.VillagerDataFix#upgradeData
    fn upgrade_data(profession: i32, career: i32) -> &'static str {
        if profession == 0 {
            if career == 2 {
                "minecraft:fisherman"
            } else if career == 3 {
                "minecraft:shepherd"
            } else if career == 4 {
                "minecraft:fletcher"
            } else {
                "minecraft:farmer"
            }
        } else if profession == 1 {
            if career == 2 {
                "minecraft:cartographer"
            } else {
                "minecraft:librarian"
            }
        } else if profession == 2 {
            "minecraft:cleric"
        } else if profession == 3 {
            if career == 2 {
                "minecraft:weaponsmith"
            } else if career == 3 {
                "minecraft:toolsmith"
            } else {
                "minecraft:armorer"
            }
        } else if profession == 4 {
            if career == 2 {
                "minecraft:leatherworker"
            } else {
                "minecraft:butcher"
            }
        } else if profession == 5 {
            "minecraft:nitwit"
        } else {
            "minecraft:none"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn farmer_default_and_fields_removed() {
        let fix = VillagerDataFix::new(Schema, "test".to_string());
        let input = Typed {
            value: map(vec![
                ("Profession", Dynamic::Int(0)),
                ("Career", Dynamic::Int(1)),
                ("CareerLevel", Dynamic::Int(5)),
            ]),
        };
        let out = fix.fix(input);
        let vd = out.value.get("VillagerData").unwrap();
        assert_eq!(
            vd.get("type").unwrap().as_string(),
            Some("minecraft:plains")
        );
        assert_eq!(
            vd.get("profession").unwrap().as_string(),
            Some("minecraft:farmer")
        );
        assert_eq!(vd.get("level"), Some(&Dynamic::Int(5)));
        assert_eq!(out.value.get("Profession"), None);
        assert_eq!(out.value.get("Career"), None);
        assert_eq!(out.value.get("CareerLevel"), None);
    }

    #[test]
    fn profession_0_career_variants() {
        let fix = VillagerDataFix::new(Schema, "t".to_string());
        for (career, expected) in [
            (2, "minecraft:fisherman"),
            (3, "minecraft:shepherd"),
            (4, "minecraft:fletcher"),
        ] {
            let input = Typed {
                value: map(vec![
                    ("Profession", Dynamic::Int(0)),
                    ("Career", Dynamic::Int(career)),
                ]),
            };
            let out = fix.fix(input);
            let prof = out
                .value
                .get("VillagerData")
                .unwrap()
                .get("profession")
                .unwrap()
                .as_string()
                .unwrap();
            assert_eq!(prof, expected);
        }
    }

    #[test]
    fn profession_3_and_4_variants() {
        let fix = VillagerDataFix::new(Schema, "t".to_string());
        for (profession, career, expected) in [
            (3, 2, "minecraft:weaponsmith"),
            (3, 3, "minecraft:toolsmith"),
            (3, 1, "minecraft:armorer"),
            (4, 2, "minecraft:leatherworker"),
            (4, 1, "minecraft:butcher"),
        ] {
            let input = Typed {
                value: map(vec![
                    ("Profession", Dynamic::Int(profession)),
                    ("Career", Dynamic::Int(career)),
                ]),
            };
            let out = fix.fix(input);
            let prof = out
                .value
                .get("VillagerData")
                .unwrap()
                .get("profession")
                .unwrap()
                .as_string()
                .unwrap();
            assert_eq!(prof, expected);
        }
    }

    #[test]
    fn missing_career_level_defaults_to_1() {
        let fix = VillagerDataFix::new(Schema, "t".to_string());
        let input = Typed {
            value: map(vec![
                ("Profession", Dynamic::Int(2)),
                ("Career", Dynamic::Int(1)),
            ]),
        };
        let out = fix.fix(input);
        let vd = out.value.get("VillagerData").unwrap();
        assert_eq!(
            vd.get("profession").unwrap().as_string(),
            Some("minecraft:cleric")
        );
        assert_eq!(vd.get("level"), Some(&Dynamic::Int(1)));
    }

    #[test]
    fn nitwit_and_unknown_profession() {
        let fix = VillagerDataFix::new(Schema, "t".to_string());
        let nitwit = fix.fix(Typed {
            value: map(vec![("Profession", Dynamic::Int(5))]),
        });
        assert_eq!(
            nitwit
                .value
                .get("VillagerData")
                .unwrap()
                .get("profession")
                .unwrap()
                .as_string(),
            Some("minecraft:nitwit")
        );
        let none = fix.fix(Typed {
            value: map(vec![("Profession", Dynamic::Int(99))]),
        });
        assert_eq!(
            none.value
                .get("VillagerData")
                .unwrap()
                .get("profession")
                .unwrap()
                .as_string(),
            Some("minecraft:none")
        );
    }
}
