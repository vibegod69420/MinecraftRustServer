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

pub struct EntityWolfColorFix {
    pub schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub type_reference: String,
    pub entity_name: String,
}

impl EntityWolfColorFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityWolfColorFix#EntityWolfColorFix
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            schema,
            changes_type,
            name: "EntityWolfColorFix".to_string(),
            type_reference: "ENTITY".to_string(),
            entity_name: "minecraft:wolf".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityWolfColorFix#fixTag
    pub fn fix_tag(&self, tag: Dynamic) -> Dynamic {
        tag.update("CollarColor", |v: Dynamic| {
            let old = v.as_int(0);
            let new_val = 15_i32.wrapping_sub(old) as i8;
            Dynamic::create_byte(new_val)
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityWolfColorFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(remainder_finder(), |d| self.fix_tag(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_tag_inverts_collar_color() {
        let fix = EntityWolfColorFix::new(Schema, false);
        let mut m = BTreeMap::new();
        m.insert("CollarColor".to_string(), Dynamic::Int(14));
        let out = fix.fix_tag(Dynamic::Map(m));
        match out {
            Dynamic::Map(m) => assert_eq!(m.get("CollarColor"), Some(&Dynamic::Byte(1))),
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_fix_tag_default_when_missing() {
        let fix = EntityWolfColorFix::new(Schema, false);
        let out = fix.fix_tag(Dynamic::Map(BTreeMap::new()));
        match out {
            Dynamic::Map(m) => assert_eq!(m.get("CollarColor"), Some(&Dynamic::Byte(15))),
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_fix_tag_negative_and_overflow_wrapping() {
        let fix = EntityWolfColorFix::new(Schema, false);
        let mut m = BTreeMap::new();
        m.insert("CollarColor".to_string(), Dynamic::Int(i32::MIN));
        let out = fix.fix_tag(Dynamic::Map(m));
        let expected = 15_i32.wrapping_sub(i32::MIN) as i8;
        match out {
            Dynamic::Map(m) => assert_eq!(m.get("CollarColor"), Some(&Dynamic::Byte(expected))),
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_fix_tag_byte_input_and_max_color() {
        let fix = EntityWolfColorFix::new(Schema, false);
        let mut m = BTreeMap::new();
        m.insert("CollarColor".to_string(), Dynamic::Byte(15));
        let out = fix.fix_tag(Dynamic::Map(m));
        match out {
            Dynamic::Map(m) => assert_eq!(m.get("CollarColor"), Some(&Dynamic::Byte(0))),
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_fix_typed_and_float_bits_preserved_in_other_fields() {
        let fix = EntityWolfColorFix::new(Schema, false);
        let bits = f32::from_bits(0x7FC0_0001u32);
        let mut m = BTreeMap::new();
        m.insert("CollarColor".to_string(), Dynamic::Int(0));
        m.insert("scale".to_string(), Dynamic::Float(bits));
        let out = fix.fix(Typed {
            value: Dynamic::Map(m),
        });
        match out.value {
            Dynamic::Map(m) => {
                assert_eq!(m.get("CollarColor"), Some(&Dynamic::Byte(15)));
                match m.get("scale") {
                    Some(Dynamic::Float(v)) => assert_eq!(v.to_bits(), bits.to_bits()),
                    other => panic!("expected float scale, got {:?}", other),
                }
            }
            other => panic!("expected map, got {:?}", other),
        }
    }
}
