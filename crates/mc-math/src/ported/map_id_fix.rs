use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Byte(i8),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#createBoolean
    pub fn create_boolean(value: bool) -> Self {
        Dynamic::Bool(value)
    }

    // JAVA: com.mojang.serialization.Dynamic#createByte
    pub fn create_byte(value: i8) -> Self {
        Dynamic::Byte(value)
    }

    // JAVA: com.mojang.serialization.Dynamic#createInt
    pub fn create_int(value: i32) -> Self {
        Dynamic::Int(value)
    }

    // JAVA: com.mojang.serialization.Dynamic#createString
    pub fn create_string(value: &str) -> Self {
        Dynamic::String(value.to_string())
    }

    // JAVA: com.mojang.serialization.Dynamic#createMap
    pub fn create_map(entries: Vec<(Dynamic, Dynamic)>) -> Self {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            if let Dynamic::String(key) = k {
                map.insert(key, v);
            }
        }
        Dynamic::Map(map)
    }

    // JAVA: com.mojang.serialization.Dynamic#set
    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#update
    pub fn update(self, key: &str, f: impl FnOnce(Dynamic) -> Dynamic) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                let current = m.get(key).cloned().unwrap_or(Dynamic::Null);
                m.insert(key.to_string(), f(current));
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asInt
    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            Dynamic::Byte(v) => *v as i32,
            Dynamic::Long(v) => *v as i32,
            _ => default,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemainderFinder;

// JAVA: com.mojang.datafixers.DSL#remainderFinder
pub fn remainder_finder() -> RemainderFinder {
    RemainderFinder
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    // JAVA: com.mojang.datafixers.Typed#update
    pub fn update(self, _finder: RemainderFinder, f: impl FnOnce(Dynamic) -> Dynamic) -> Self {
        Typed {
            value: f(self.value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Schema;

pub struct TypeRewriteRule {
    pub name: String,
    pub apply: Box<dyn Fn(Typed) -> Typed>,
}

pub struct MapIdFix {
    pub schema: Schema,
    pub changes_type: bool,
}

impl MapIdFix {
    // JAVA: net.minecraft.util.datafix.fixes.MapIdFix#MapIdFix
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            changes_type: false,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.MapIdFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            name: "Map id fix".to_string(),
            apply: Box::new(|typed: Typed| {
                typed.update(remainder_finder(), |d: Dynamic| {
                    Dynamic::create_map(vec![(Dynamic::create_string("data"), d)])
                })
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_rule_wraps_dynamic_in_data_map() {
        let fix = MapIdFix::new(Schema);
        let rule = fix.make_rule();
        assert_eq!(rule.name, "Map id fix");
        let input = Typed {
            value: Dynamic::Int(42),
        };
        let out = (rule.apply)(input);
        let expected = Dynamic::Map(BTreeMap::from([(
            "data".to_string(),
            Dynamic::Int(42),
        )]));
        assert_eq!(out.value, expected);
    }

    #[test]
    fn test_make_rule_preserves_negative_and_min_int() {
        let fix = MapIdFix::new(Schema);
        let rule = fix.make_rule();
        let input = Typed {
            value: Dynamic::Int(i32::MIN),
        };
        let out = (rule.apply)(input);
        match out.value {
            Dynamic::Map(m) => assert_eq!(m.get("data"), Some(&Dynamic::Int(i32::MIN))),
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_make_rule_preserves_nested_map_and_byte() {
        let fix = MapIdFix::new(Schema);
        let rule = fix.make_rule();
        let mut inner = BTreeMap::new();
        inner.insert("x".to_string(), Dynamic::Byte(-128));
        inner.insert("y".to_string(), Dynamic::Int(i32::MAX));
        let input = Typed {
            value: Dynamic::Map(inner.clone()),
        };
        let out = (rule.apply)(input);
        match out.value {
            Dynamic::Map(m) => assert_eq!(m.get("data"), Some(&Dynamic::Map(inner))),
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_make_rule_preserves_float_bits() {
        let fix = MapIdFix::new(Schema);
        let rule = fix.make_rule();
        let bits = f64::from_bits(0xFFF0_0000_0000_0001u64);
        let input = Typed {
            value: Dynamic::Double(bits),
        };
        let out = (rule.apply)(input);
        match out.value {
            Dynamic::Map(m) => match m.get("data") {
                Some(Dynamic::Double(v)) => assert_eq!(v.to_bits(), bits.to_bits()),
                other => panic!("expected double data, got {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }
}
