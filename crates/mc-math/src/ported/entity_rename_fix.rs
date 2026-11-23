use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug)]
pub struct TypeRewriteRule;

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicOps;

#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Double(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: Option<DynamicValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub ty: Type,
    pub ops: DynamicOps,
    pub value: Dynamic,
}

pub struct EntityRenameFixBase {
    pub name: String,
    pub schema: Schema,
    pub changes_type: bool,
}

impl EntityRenameFixBase {
    // JAVA: net.minecraft.util.datafix.fixes.EntityRenameFix#<init>
    pub fn new(name: String, schema: Schema, changes_type: bool) -> Self {
        Self {
            name,
            schema,
            changes_type,
        }
    }
}

pub trait EntityRenameFix {
    fn base(&self) -> &EntityRenameFixBase;

    // JAVA: net.minecraft.util.datafix.fixes.EntityRenameFix#makeRule
    fn make_rule(&self) -> TypeRewriteRule {
        // DFU type-rewrite wiring stub; concrete implementations provide `fix`.
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityRenameFix#getEntity
    fn get_entity(&self, value: Dynamic, ops: DynamicOps, ty: Type) -> Typed {
        Typed { ty, ops, value }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityRenameFix#fix
    fn fix(&self, name: String, entity: Typed) -> (String, Typed);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleRename {
        base: EntityRenameFixBase,
    }

    impl SimpleRename {
        fn new() -> Self {
            Self {
                base: EntityRenameFixBase::new("TestFix".to_string(), Schema, true),
            }
        }
    }

    impl EntityRenameFix for SimpleRename {
        fn base(&self) -> &EntityRenameFixBase {
            &self.base
        }

        fn fix(&self, name: String, entity: Typed) -> (String, Typed) {
            if name == "minecraft:old" {
                ("minecraft:new".to_string(), entity)
            } else {
                (name, entity)
            }
        }
    }

    fn dyn_str(s: &str) -> Dynamic {
        Dynamic {
            value: Some(DynamicValue::String(s.to_string())),
        }
    }

    #[test]
    fn base_stores_name_and_flags() {
        let r = SimpleRename::new();
        assert_eq!(r.base().name, "TestFix");
        assert!(r.base().changes_type);
    }

    #[test]
    fn get_entity_wraps_value() {
        let r = SimpleRename::new();
        let ty = Type {
            name: "entity".to_string(),
        };
        let ops = DynamicOps;
        let val = dyn_str("hello");
        let typed = r.get_entity(val.clone(), ops.clone(), ty.clone());
        assert_eq!(typed.ty, ty);
        assert_eq!(typed.ops, ops);
        assert_eq!(typed.value, val);
    }

    #[test]
    fn fix_renames_matching_entity() {
        let r = SimpleRename::new();
        let typed = Typed {
            ty: Type { name: "t".into() },
            ops: DynamicOps,
            value: dyn_str("data"),
        };
        let (name, out) = r.fix("minecraft:old".to_string(), typed.clone());
        assert_eq!(name, "minecraft:new");
        assert_eq!(out, typed);
    }

    #[test]
    fn fix_preserves_unknown_and_negative_int() {
        let r = SimpleRename::new();
        let typed = Typed {
            ty: Type { name: "t".into() },
            ops: DynamicOps,
            value: Dynamic {
                value: Some(DynamicValue::Int(i32::MIN)),
            },
        };
        let (name, out) = r.fix("minecraft:other".to_string(), typed.clone());
        assert_eq!(name, "minecraft:other");
        assert_eq!(out.value, typed.value);
    }

    #[test]
    fn fix_preserves_float_bits() {
        let r = SimpleRename::new();
        let bits = f64::INFINITY.to_bits();
        let typed = Typed {
            ty: Type { name: "t".into() },
            ops: DynamicOps,
            value: Dynamic {
                value: Some(DynamicValue::Double(f64::from_bits(bits))),
            },
        };
        let (_, out) = r.fix("minecraft:old".to_string(), typed);
        match &out.value.value {
            Some(DynamicValue::Double(d)) => assert_eq!(d.to_bits(), bits),
            other => panic!("expected double, got {:?}", other),
        }
    }
}
