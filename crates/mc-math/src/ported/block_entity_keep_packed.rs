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

pub struct BlockEntityKeepPacked {
    pub schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub type_reference: String,
    pub entity_name: String,
}

impl BlockEntityKeepPacked {
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityKeepPacked#BlockEntityKeepPacked
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            schema,
            changes_type,
            name: "BlockEntityKeepPacked".to_string(),
            type_reference: "BLOCK_ENTITY".to_string(),
            entity_name: "DUMMY".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityKeepPacked#fixTag
    fn fix_tag(tag: Dynamic) -> Dynamic {
        tag.set("keepPacked", Dynamic::create_boolean(true))
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityKeepPacked#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(remainder_finder(), Self::fix_tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_tag_sets_keep_packed_on_empty_map() {
        let out = BlockEntityKeepPacked::fix_tag(Dynamic::Map(BTreeMap::new()));
        assert_eq!(
            out,
            Dynamic::Map(BTreeMap::from([(
                "keepPacked".to_string(),
                Dynamic::Bool(true)
            )]))
        );
    }

    #[test]
    fn test_fix_typed_preserves_existing_fields() {
        let fix = BlockEntityKeepPacked::new(Schema, true);
        let mut m = BTreeMap::new();
        m.insert(
            "id".to_string(),
            Dynamic::String("minecraft:chest".to_string()),
        );
        m.insert("x".to_string(), Dynamic::Int(i32::MIN));
        let out = fix.fix(Typed {
            value: Dynamic::Map(m),
        });
        match out.value {
            Dynamic::Map(m) => {
                assert_eq!(m.get("keepPacked"), Some(&Dynamic::Bool(true)));
                assert_eq!(
                    m.get("id"),
                    Some(&Dynamic::String("minecraft:chest".to_string()))
                );
                assert_eq!(m.get("x"), Some(&Dynamic::Int(i32::MIN)));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_fix_tag_overwrites_existing_keep_packed() {
        let mut m = BTreeMap::new();
        m.insert("keepPacked".to_string(), Dynamic::Bool(false));
        let out = BlockEntityKeepPacked::fix_tag(Dynamic::Map(m));
        match out {
            Dynamic::Map(m) => assert_eq!(m.get("keepPacked"), Some(&Dynamic::Bool(true))),
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_fix_tag_non_map_preserved() {
        let out = BlockEntityKeepPacked::fix_tag(Dynamic::Int(-128));
        assert_eq!(out, Dynamic::Int(-128));
    }

    #[test]
    fn test_constructor_fields() {
        let fix = BlockEntityKeepPacked::new(Schema, false);
        assert!(!fix.changes_type);
        assert_eq!(fix.name, "BlockEntityKeepPacked");
        assert_eq!(fix.type_reference, "BLOCK_ENTITY");
        assert_eq!(fix.entity_name, "DUMMY");
    }
}
