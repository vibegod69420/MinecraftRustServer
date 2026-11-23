use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Compound(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get(&self, key: &str) -> OptionalDynamic<'_> {
        match self {
            Dynamic::Compound(m) => OptionalDynamic { value: m.get(key) },
            _ => OptionalDynamic { value: None },
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#set
    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Compound(m) => {
                let mut new_m = m.clone();
                new_m.insert(key.to_string(), value);
                Dynamic::Compound(new_m)
            }
            _ => self.clone(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createBoolean
    pub fn create_boolean(&self, value: bool) -> Dynamic {
        Dynamic::Bool(value)
    }
}

pub struct OptionalDynamic<'a> {
    value: Option<&'a Dynamic>,
}

impl<'a> OptionalDynamic<'a> {
    // JAVA: com.mojang.serialization.Dynamic#asBoolean
    pub fn as_boolean(&self, default: bool) -> bool {
        match self.value {
            Some(Dynamic::Bool(b)) => *b,
            _ => default,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeReference(pub &'static str);

pub struct References;

impl References {
    pub const ENTITY: TypeReference = TypeReference("entity");
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    // JAVA: com.mojang.datafixers.Typed#update
    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, f: F) -> Self {
        Typed { value: f(self.value) }
    }
}

pub struct StriderGravityFix {
    pub output_schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub type_reference: TypeReference,
    pub entity_name: String,
}

impl StriderGravityFix {
    // JAVA: net.minecraft.util.datafix.fixes.StriderGravityFix#<init>
    pub fn new(output_schema: Schema, changes_type: bool) -> Self {
        Self {
            output_schema,
            changes_type,
            name: "StriderGravityFix".to_string(),
            type_reference: References::ENTITY,
            entity_name: "minecraft:strider".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.StriderGravityFix#fixTag
    pub fn fix_tag(&self, tag: Dynamic) -> Dynamic {
        if tag.get("NoGravity").as_boolean(false) {
            let false_val = tag.create_boolean(false);
            tag.set("NoGravity", false_val)
        } else {
            tag
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.StriderGravityFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(|d| self.fix_tag(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_compound(no_gravity: Option<bool>, extra: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        if let Some(v) = no_gravity {
            m.insert("NoGravity".to_string(), Dynamic::Bool(v));
        }
        for (k, v) in extra {
            m.insert(k.to_string(), v);
        }
        Dynamic::Compound(m)
    }

    #[test]
    fn no_gravity_true_becomes_false() {
        let fix = StriderGravityFix::new(Schema { version: 1 }, true);
        let input = make_compound(Some(true), vec![]);
        let out = fix.fix_tag(input);
        assert_eq!(out.get("NoGravity").as_boolean(true), false);
    }

    #[test]
    fn no_gravity_false_unchanged() {
        let fix = StriderGravityFix::new(Schema { version: 1 }, true);
        let input = make_compound(Some(false), vec![]);
        let out = fix.fix_tag(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn no_gravity_missing_unchanged() {
        let fix = StriderGravityFix::new(Schema { version: 1 }, true);
        let input = make_compound(
            None,
            vec![("id", Dynamic::String("minecraft:strider".to_string()))],
        );
        let out = fix.fix_tag(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn constructor_fields_match_java() {
        let fix = StriderGravityFix::new(Schema { version: 42 }, false);
        assert_eq!(fix.name, "StriderGravityFix");
        assert_eq!(fix.type_reference, References::ENTITY);
        assert_eq!(fix.entity_name, "minecraft:strider");
        assert!(!fix.changes_type);
        assert_eq!(fix.output_schema.version, 42);
    }

    #[test]
    fn preserves_negative_ints_and_float_bits() {
        let fix = StriderGravityFix::new(Schema { version: 2 }, true);
        let nan_bits = f64::NAN.to_bits();
        let neg_zero_bits = (-0.0f32).to_bits();
        let input = make_compound(
            Some(true),
            vec![
                ("Health", Dynamic::Int(-1)),
                ("Min", Dynamic::Int(i32::MIN)),
                ("Big", Dynamic::Long(i64::MIN)),
                ("N", Dynamic::Double(f64::from_bits(nan_bits))),
                ("NZ", Dynamic::Float(f32::from_bits(neg_zero_bits))),
            ],
        );
        let out = fix.fix_tag(input);
        assert_eq!(out.get("NoGravity").as_boolean(true), false);
        match out {
            Dynamic::Compound(m) => {
                assert_eq!(m.get("Health"), Some(&Dynamic::Int(-1)));
                assert_eq!(m.get("Min"), Some(&Dynamic::Int(i32::MIN)));
                assert_eq!(m.get("Big"), Some(&Dynamic::Long(i64::MIN)));
                match m.get("N") {
                    Some(Dynamic::Double(d)) => assert_eq!(d.to_bits(), nan_bits),
                    _ => panic!("expected double NaN"),
                }
                match m.get("NZ") {
                    Some(Dynamic::Float(f)) => assert_eq!(f.to_bits(), neg_zero_bits),
                    _ => panic!("expected float -0.0"),
                }
            }
            _ => panic!("expected compound"),
        }
    }

    #[test]
    fn fix_updates_typed_and_non_compound_is_unchanged() {
        let fix = StriderGravityFix::new(Schema { version: 1 }, true);
        let typed = Typed {
            value: make_compound(Some(true), vec![]),
        };
        let out = fix.fix(typed);
        assert_eq!(out.value.get("NoGravity").as_boolean(true), false);

        let non_compound = Dynamic::Int(i32::MIN);
        let out2 = fix.fix_tag(non_compound.clone());
        assert_eq!(out2, non_compound);
    }
}
