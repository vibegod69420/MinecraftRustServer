use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Byte(i8),
    Short(i16),
    Int(i32),
    Bool(bool),
    Missing,
}

impl Dynamic {
    pub fn map(entries: Vec<(String, Dynamic)>) -> Self {
        Self::Map(entries.into_iter().collect())
    }

    pub fn get(&self, key: &str) -> &Dynamic {
        match self {
            Self::Map(m) => m.get(key).unwrap_or(&Self::Missing),
            _ => &Self::Missing,
        }
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn as_boolean(&self, default: bool) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Byte(b) => *b != 0,
            Self::Short(s) => *s != 0,
            Self::Int(i) => *i != 0,
            _ => default,
        }
    }

    pub fn empty_map(&self) -> Dynamic {
        Self::Map(BTreeMap::new())
    }

    pub fn create_byte(&self, v: i8) -> Dynamic {
        Self::Byte(v)
    }

    pub fn create_short(&self, v: i16) -> Dynamic {
        Self::Short(v)
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Self::Map(m) => {
                let mut out = m.clone();
                out.insert(key.to_string(), value);
                Self::Map(out)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Self::Map(m) => {
                let mut out = m.clone();
                out.remove(key);
                Self::Map(out)
            }
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

pub struct EntityHorseSaddleFix;

impl EntityHorseSaddleFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityHorseSaddleFix#<init>
    pub fn new(_schema: &Schema, _changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityHorseSaddleFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        let remainder = &typed.remainder;
        let has_saddle_item = !remainder.get("SaddleItem").is_missing();
        let saddle = remainder.get("Saddle").as_boolean(false);
        if !has_saddle_item && saddle {
            let saddle_item = Dynamic::map(vec![
                (
                    "id".to_string(),
                    Dynamic::String("minecraft:saddle".to_string()),
                ),
                ("Count".to_string(), Dynamic::Byte(1)),
                ("Damage".to_string(), Dynamic::Short(0)),
            ]);
            let new_remainder = remainder.remove("Saddle").set("SaddleItem", saddle_item);
            Typed {
                remainder: new_remainder,
            }
        } else {
            typed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_saddle_item_when_saddle_true() {
        let fix = EntityHorseSaddleFix::new(&Schema, false);
        let input = Typed {
            remainder: Dynamic::map(vec![
                ("id".to_string(), Dynamic::String("EntityHorse".to_string())),
                ("Saddle".to_string(), Dynamic::Bool(true)),
            ]),
        };
        let out = fix.fix(input);
        let saddle_item = out.remainder.get("SaddleItem");
        assert_eq!(
            saddle_item.get("id"),
            &Dynamic::String("minecraft:saddle".to_string())
        );
        assert_eq!(saddle_item.get("Count"), &Dynamic::Byte(1));
        assert_eq!(saddle_item.get("Damage"), &Dynamic::Short(0));
        assert!(out.remainder.get("Saddle").is_missing());
    }

    #[test]
    fn saddle_byte_one_treated_as_true() {
        let fix = EntityHorseSaddleFix::new(&Schema, false);
        let input = Typed {
            remainder: Dynamic::map(vec![("Saddle".to_string(), Dynamic::Byte(1))]),
        };
        let out = fix.fix(input);
        assert!(!out.remainder.get("SaddleItem").is_missing());
    }

    #[test]
    fn no_change_when_saddle_item_exists() {
        let fix = EntityHorseSaddleFix::new(&Schema, false);
        let existing_item = Dynamic::map(vec![
            (
                "id".to_string(),
                Dynamic::String("minecraft:saddle".to_string()),
            ),
            ("Count".to_string(), Dynamic::Byte(1)),
            ("Damage".to_string(), Dynamic::Short(0)),
        ]);
        let input = Typed {
            remainder: Dynamic::map(vec![
                ("Saddle".to_string(), Dynamic::Bool(true)),
                ("SaddleItem".to_string(), existing_item),
            ]),
        };
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn no_change_when_saddle_false() {
        let fix = EntityHorseSaddleFix::new(&Schema, false);
        let input = Typed {
            remainder: Dynamic::map(vec![("Saddle".to_string(), Dynamic::Bool(false))]),
        };
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn no_change_when_saddle_missing() {
        let fix = EntityHorseSaddleFix::new(&Schema, false);
        let input = Typed {
            remainder: Dynamic::map(vec![(
                "id".to_string(),
                Dynamic::String("EntityHorse".to_string()),
            )]),
        };
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn saddle_byte_zero_and_negative_byte() {
        let fix = EntityHorseSaddleFix::new(&Schema, false);
        let zero = Typed {
            remainder: Dynamic::map(vec![("Saddle".to_string(), Dynamic::Byte(0))]),
        };
        assert_eq!(fix.fix(zero.clone()), zero);

        let neg = Typed {
            remainder: Dynamic::map(vec![("Saddle".to_string(), Dynamic::Byte(-1))]),
        };
        let out = fix.fix(neg);
        assert!(!out.remainder.get("SaddleItem").is_missing());
    }
}
