use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i32),
    FloatBits(u32),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(m) => OptionalDynamic(m.get(key).cloned()),
            _ => OptionalDynamic(None),
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

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    pub fn as_string(&self, default: &str) -> String {
        match &self.0 {
            Some(Dynamic::String(s)) => s.clone(),
            _ => default.to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    pub fn update_remainder<F: FnOnce(Dynamic) -> Dynamic>(self, f: F) -> Self {
        Typed {
            remainder: f(self.remainder),
        }
    }
}

pub struct EntitySalmonSizeFix {
    _schema: Schema,
}

impl EntitySalmonSizeFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntitySalmonSizeFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntitySalmonSizeFix#fix
    pub fn fix(&self, input: Typed) -> Typed {
        input.update_remainder(|d| {
            let type_val = d.get("type").as_string("medium");
            if type_val == "large" {
                d
            } else {
                let medium = d.create_string("medium");
                d.set("type", medium)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn salmon_map(type_str: &str) -> Dynamic {
        let mut m = BTreeMap::new();
        m.insert("type".to_string(), Dynamic::String(type_str.to_string()));
        Dynamic::Map(m)
    }

    #[test]
    fn large_salmon_unchanged() {
        let fix = EntitySalmonSizeFix::new(Schema);
        let input = Typed {
            remainder: salmon_map("large"),
        };
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn small_salmon_becomes_medium() {
        let fix = EntitySalmonSizeFix::new(Schema);
        let out = fix.fix(Typed {
            remainder: salmon_map("small"),
        });
        assert_eq!(out.remainder.get("type").as_string(""), "medium");
    }

    #[test]
    fn missing_type_defaults_to_medium() {
        let fix = EntitySalmonSizeFix::new(Schema);
        let out = fix.fix(Typed {
            remainder: Dynamic::empty_map(),
        });
        assert_eq!(out.remainder.get("type").as_string(""), "medium");
    }

    #[test]
    fn float_bits_and_negative_int_preserved() {
        let fix = EntitySalmonSizeFix::new(Schema);
        let mut m = BTreeMap::new();
        m.insert("type".to_string(), Dynamic::String("small".to_string()));
        m.insert("scale".to_string(), Dynamic::FloatBits(f32::NAN.to_bits()));
        m.insert("age".to_string(), Dynamic::Int(i32::MIN));
        let out = fix.fix(Typed {
            remainder: Dynamic::Map(m),
        });
        assert_eq!(out.remainder.get("type").as_string(""), "medium");
        if let Dynamic::Map(ref map) = out.remainder {
            match map.get("scale") {
                Some(Dynamic::FloatBits(bits)) => assert!(f32::from_bits(*bits).is_nan()),
                other => panic!("unexpected scale: {:?}", other),
            }
            assert_eq!(map.get("age"), Some(&Dynamic::Int(i32::MIN)));
        } else {
            panic!("expected map");
        }
    }

    #[test]
    fn non_map_remainder_passthrough() {
        let fix = EntitySalmonSizeFix::new(Schema);
        let input = Typed {
            remainder: Dynamic::Int(-1),
        };
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }
}
