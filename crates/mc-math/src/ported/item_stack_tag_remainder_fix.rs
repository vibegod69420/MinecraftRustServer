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

pub struct ItemStackTagRemainderFix<F>
where
    F: Fn(Dynamic) -> Dynamic,
{
    pub schema: Schema,
    pub name: String,
    pub filter: Box<dyn Fn(&str) -> bool>,
    fix_fn: F,
}

impl<F> ItemStackTagRemainderFix<F>
where
    F: Fn(Dynamic) -> Dynamic,
{
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagRemainderFix#ItemStackTagRemainderFix
    pub fn new(schema: Schema, name: String, filter: Box<dyn Fn(&str) -> bool>, fix_fn: F) -> Self {
        Self {
            schema,
            name,
            filter,
            fix_fn,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagRemainderFix#fixItemStackTag
    pub fn fix_item_stack_tag_dynamic(&self, tag: Dynamic) -> Dynamic {
        (self.fix_fn)(tag)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagRemainderFix#fixItemStackTag
    pub fn fix_item_stack_tag_typed(&self, typed: Typed) -> Typed {
        typed.update(remainder_finder(), |d| self.fix_item_stack_tag_dynamic(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fix() -> ItemStackTagRemainderFix<fn(Dynamic) -> Dynamic> {
        // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagRemainderFix#fixItemStackTag (test impl)
        fn fix_fn(d: Dynamic) -> Dynamic {
            d.set("fixed", Dynamic::create_boolean(true))
        }
        ItemStackTagRemainderFix::new(
            Schema,
            "test_fix".to_string(),
            Box::new(|s: &str| s == "minecraft:stick"),
            fix_fn,
        )
    }

    #[test]
    fn test_typed_fix_delegates_to_dynamic() {
        let fix = make_fix();
        let input = Typed {
            value: Dynamic::Map(BTreeMap::new()),
        };
        let out = fix.fix_item_stack_tag_typed(input);
        let expected = Dynamic::Map(BTreeMap::from([(
            "fixed".to_string(),
            Dynamic::Bool(true),
        )]));
        assert_eq!(out.value, expected);
    }

    #[test]
    fn test_dynamic_fix_on_non_map_preserves_value() {
        let fix = make_fix();
        let input = Dynamic::Int(i32::MIN);
        let out = fix.fix_item_stack_tag_dynamic(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn test_constructor_stores_fields_and_filter() {
        let fix = make_fix();
        assert_eq!(fix.name, "test_fix");
        assert!((fix.filter)("minecraft:stick"));
        assert!(!(fix.filter)("minecraft:stone"));
    }

    #[test]
    fn test_overflow_and_negative_int_update() {
        // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagRemainderFix#fixItemStackTag (test impl)
        fn overflow_fix(d: Dynamic) -> Dynamic {
            d.update("v", |v| {
                let i = v.as_int(0);
                Dynamic::create_int(i.wrapping_add(1))
            })
        }
        let fix = ItemStackTagRemainderFix::new(
            Schema,
            "overflow".to_string(),
            Box::new(|_| true),
            overflow_fix as fn(Dynamic) -> Dynamic,
        );
        let mut m = BTreeMap::new();
        m.insert("v".to_string(), Dynamic::Int(i32::MAX));
        let out = fix.fix_item_stack_tag_typed(Typed {
            value: Dynamic::Map(m),
        });
        match out.value {
            Dynamic::Map(m) => assert_eq!(m.get("v"), Some(&Dynamic::Int(i32::MIN))),
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn test_float_bits_preserved_through_fix() {
        let nan_bits = f64::from_bits(0x7FF8_0000_0000_0001u64);
        // JAVA: net.minecraft.util.datafix.fixes.ItemStackTagRemainderFix#fixItemStackTag (test impl)
        fn identity_fix(d: Dynamic) -> Dynamic {
            d
        }
        let fix = ItemStackTagRemainderFix::new(
            Schema,
            "identity".to_string(),
            Box::new(|_| true),
            identity_fix as fn(Dynamic) -> Dynamic,
        );
        let input = Typed {
            value: Dynamic::Double(nan_bits),
        };
        let out = fix.fix_item_stack_tag_typed(input);
        match out.value {
            Dynamic::Double(v) => assert_eq!(v.to_bits(), nan_bits.to_bits()),
            other => panic!("expected double, got {:?}", other),
        }
    }
}
