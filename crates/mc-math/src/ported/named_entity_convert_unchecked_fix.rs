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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#getChoiceType
    pub fn get_choice_type(&self, reference: &TypeReference, choice_name: &str) -> TypeDescriptor {
        TypeDescriptor {
            reference: reference.clone(),
            choice_name: choice_name.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeReference(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDescriptor {
    pub reference: TypeReference,
    pub choice_name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub type_desc: TypeDescriptor,
    pub value: Dynamic,
}

pub struct ExtraDataFixUtils;

impl ExtraDataFixUtils {
    // JAVA: net.minecraft.util.datafix.ExtraDataFixUtils#cast
    pub fn cast(target_type: TypeDescriptor, typed: Typed) -> Typed {
        Typed {
            type_desc: target_type,
            value: typed.value,
        }
    }
}

pub struct NamedEntityConvertUncheckedFix {
    pub output_schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub type_reference: TypeReference,
    pub entity_name: String,
}

impl NamedEntityConvertUncheckedFix {
    // JAVA: net.minecraft.util.datafix.fixes.NamedEntityConvertUncheckedFix#<init>
    pub fn new(
        output_schema: Schema,
        name: String,
        type_reference: TypeReference,
        entity_name: String,
    ) -> Self {
        Self {
            output_schema,
            changes_type: true,
            name,
            type_reference,
            entity_name,
        }
    }

    // JAVA: com.mojang.datafixers.DataFix#getOutputSchema
    pub fn get_output_schema(&self) -> &Schema {
        &self.output_schema
    }

    // JAVA: net.minecraft.util.datafix.fixes.NamedEntityConvertUncheckedFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        let target_type = self
            .get_output_schema()
            .get_choice_type(&self.type_reference, &self.entity_name);
        ExtraDataFixUtils::cast(target_type, typed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_typed() -> Typed {
        let mut m = BTreeMap::new();
        m.insert(
            "id".to_string(),
            Dynamic::String("minecraft:zombie".to_string()),
        );
        m.insert("Health".to_string(), Dynamic::Int(20));
        Typed {
            type_desc: TypeDescriptor {
                reference: TypeReference("entity".to_string()),
                choice_name: "minecraft:old".to_string(),
            },
            value: Dynamic::Compound(m),
        }
    }

    #[test]
    fn fix_casts_to_target_choice_type_preserving_value() {
        let fix = NamedEntityConvertUncheckedFix::new(
            Schema { version: 3 },
            "RenameZombie".to_string(),
            TypeReference("entity".to_string()),
            "minecraft:zombie".to_string(),
        );
        let input = sample_typed();
        let out = fix.fix(input.clone());
        assert_eq!(out.type_desc.reference, TypeReference("entity".to_string()));
        assert_eq!(out.type_desc.choice_name, "minecraft:zombie");
        assert_eq!(out.value, input.value);
        assert!(fix.changes_type);
    }

    #[test]
    fn constructor_stores_fields() {
        let fix = NamedEntityConvertUncheckedFix::new(
            Schema { version: -7 },
            "N".to_string(),
            TypeReference("e".to_string()),
            "ent".to_string(),
        );
        assert_eq!(fix.output_schema.version, -7);
        assert_eq!(fix.name, "N");
        assert_eq!(fix.type_reference, TypeReference("e".to_string()));
        assert_eq!(fix.entity_name, "ent");
        assert!(fix.changes_type);
    }

    #[test]
    fn cast_preserves_negative_and_extreme_integers() {
        let mut m = BTreeMap::new();
        m.insert("min_i32".to_string(), Dynamic::Int(i32::MIN));
        m.insert("neg".to_string(), Dynamic::Int(-1));
        m.insert("min_i64".to_string(), Dynamic::Long(i64::MIN));
        m.insert("max_i64".to_string(), Dynamic::Long(i64::MAX));
        let typed = Typed {
            type_desc: TypeDescriptor {
                reference: TypeReference("t".to_string()),
                choice_name: "old".to_string(),
            },
            value: Dynamic::Compound(m.clone()),
        };
        let target = TypeDescriptor {
            reference: TypeReference("t".to_string()),
            choice_name: "new".to_string(),
        };
        let out = ExtraDataFixUtils::cast(target.clone(), typed);
        assert_eq!(out.type_desc, target);
        assert_eq!(out.value, Dynamic::Compound(m));
    }

    #[test]
    fn cast_preserves_float_bits() {
        let nan_bits = f64::NAN.to_bits();
        let neg_zero_bits = (-0.0f64).to_bits();
        let inf_bits = f32::INFINITY.to_bits();
        let mut m = BTreeMap::new();
        m.insert("nan".to_string(), Dynamic::Double(f64::from_bits(nan_bits)));
        m.insert(
            "nz".to_string(),
            Dynamic::Double(f64::from_bits(neg_zero_bits)),
        );
        m.insert(
            "inf".to_string(),
            Dynamic::Float(f32::from_bits(inf_bits)),
        );
        let typed = Typed {
            type_desc: TypeDescriptor {
                reference: TypeReference("t".to_string()),
                choice_name: "a".to_string(),
            },
            value: Dynamic::Compound(m),
        };
        let out = ExtraDataFixUtils::cast(
            TypeDescriptor {
                reference: TypeReference("t".to_string()),
                choice_name: "b".to_string(),
            },
            typed.clone(),
        );
        match (&typed.value, &out.value) {
            (Dynamic::Compound(in_m), Dynamic::Compound(out_m)) => {
                match (in_m.get("nan"), out_m.get("nan")) {
                    (Some(Dynamic::Double(a)), Some(Dynamic::Double(b))) => {
                        assert_eq!(a.to_bits(), nan_bits);
                        assert_eq!(b.to_bits(), nan_bits);
                    }
                    _ => panic!("expected nan double"),
                }
                match out_m.get("nz") {
                    Some(Dynamic::Double(d)) => assert_eq!(d.to_bits(), neg_zero_bits),
                    _ => panic!("expected -0.0 double"),
                }
                match out_m.get("inf") {
                    Some(Dynamic::Float(f)) => assert_eq!(f.to_bits(), inf_bits),
                    _ => panic!("expected inf float"),
                }
            }
            _ => panic!("expected compound"),
        }
    }

    #[test]
    fn empty_entity_name_and_null_value_edge_case() {
        let fix = NamedEntityConvertUncheckedFix::new(
            Schema { version: 0 },
            String::new(),
            TypeReference(String::new()),
            String::new(),
        );
        let typed = Typed {
            type_desc: TypeDescriptor {
                reference: TypeReference("x".to_string()),
                choice_name: "y".to_string(),
            },
            value: Dynamic::Null,
        };
        let out = fix.fix(typed);
        assert_eq!(out.type_desc.choice_name, "");
        assert_eq!(out.type_desc.reference, TypeReference(String::new()));
        assert_eq!(out.value, Dynamic::Null);
    }
}
