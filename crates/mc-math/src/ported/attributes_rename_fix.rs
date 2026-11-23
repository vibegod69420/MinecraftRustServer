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

    pub fn as_seq(&self) -> Option<&[Dynamic]> {
        match self {
            Dynamic::Seq(v) => Some(v),
            _ => None,
        }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                if let Some(v) = m.remove(key) {
                    m.insert(key.to_string(), f(v));
                }
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Dynamic {
        Dynamic::Seq(items)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    pub fn update_remainder<F: FnOnce(Dynamic) -> Dynamic>(self, f: F) -> Typed {
        Typed { value: f(self.value) }
    }
}

// Minimal DFU stubs for structural fidelity.
#[derive(Clone, Debug, Default)]
pub struct Schema;
#[derive(Clone, Debug, Default)]
pub struct TypeRewriteRule;

pub struct References;
impl References {
    pub const DATA_COMPONENTS: &'static str = "minecraft:data_components";
    pub const ENTITY: &'static str = "minecraft:entity";
    pub const PLAYER: &'static str = "minecraft:player";
}

// JAVA: net.minecraft.util.datafix.ExtraDataFixUtils#fixStringField
fn fix_string_field(d: Dynamic, key: &str, renames: &dyn Fn(&str) -> String) -> Dynamic {
    d.update(key, |v| match v.as_string() {
        Some(s) => Dynamic::Str(renames(s)),
        None => v,
    })
}

pub struct AttributesRenameFix {
    pub name: String,
    renames: Box<dyn Fn(&str) -> String>,
}

impl AttributesRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameFix#<init>
    pub fn new(_schema: Schema, name: String, renames: impl Fn(&str) -> String + 'static) -> Self {
        Self {
            name,
            renames: Box::new(renames),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        // DFU plumbing: in Java this composes fixTypeEverywhereTyped for
        // DATA_COMPONENTS, ENTITY, and PLAYER. Use fix_data_components/fix_entity.
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameFix#fixDataComponents
    pub fn fix_data_components(&self, typed: Typed) -> Typed {
        typed.update_remainder(|d| {
            d.update("minecraft:attribute_modifiers", |am| {
                am.update("modifiers", |mods| match mods.as_seq() {
                    Some(seq) => {
                        let new_seq: Vec<Dynamic> = seq
                            .iter()
                            .cloned()
                            .map(|item| self.fix_type_field(item))
                            .collect();
                        mods.create_list(new_seq)
                    }
                    None => mods,
                })
            })
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameFix#fixEntity
    pub fn fix_entity(&self, typed: Typed) -> Typed {
        typed.update_remainder(|d| {
            d.update("attributes", |attrs| match attrs.as_seq() {
                Some(seq) => {
                    let new_seq: Vec<Dynamic> = seq
                        .iter()
                        .cloned()
                        .map(|item| self.fix_id_field(item))
                        .collect();
                    attrs.create_list(new_seq)
                }
                None => attrs,
            })
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameFix#fixIdField
    fn fix_id_field(&self, d: Dynamic) -> Dynamic {
        fix_string_field(d, "id", self.renames.as_ref())
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributesRenameFix#fixTypeField
    fn fix_type_field(&self, d: Dynamic) -> Dynamic {
        fix_string_field(d, "type", self.renames.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_val(s: &str) -> Dynamic {
        Dynamic::Str(s.to_string())
    }

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    fn seq(items: Vec<Dynamic>) -> Dynamic {
        Dynamic::Seq(items)
    }

    fn rename(s: &str) -> String {
        if s == "old:attr" {
            "new:attr".to_string()
        } else {
            s.to_string()
        }
    }

    #[test]
    fn fix_entity_renames_id_field() {
        let fix = AttributesRenameFix::new(Schema, "test".to_string(), rename);
        let input = Typed {
            value: map(vec![(
                "attributes",
                seq(vec![map(vec![
                    ("id", str_val("old:attr")),
                    ("amount", Dynamic::Int(1)),
                ])]),
            )]),
        };
        let out = fix.fix_entity(input);
        let attrs = out.value.get("attributes").unwrap().as_seq().unwrap();
        assert_eq!(attrs[0].get("id").unwrap().as_string(), Some("new:attr"));
        assert_eq!(attrs[0].get("amount"), Some(&Dynamic::Int(1)));
    }

    #[test]
    fn fix_data_components_renames_type_field() {
        let fix = AttributesRenameFix::new(Schema, "test".to_string(), rename);
        let input = Typed {
            value: map(vec![(
                "minecraft:attribute_modifiers",
                map(vec![(
                    "modifiers",
                    seq(vec![map(vec![("type", str_val("old:attr"))])]),
                )]),
            )]),
        };
        let out = fix.fix_data_components(input);
        let mods = out
            .value
            .get("minecraft:attribute_modifiers")
            .unwrap()
            .get("modifiers")
            .unwrap()
            .as_seq()
            .unwrap();
        assert_eq!(mods[0].get("type").unwrap().as_string(), Some("new:attr"));
    }

    #[test]
    fn missing_fields_unchanged() {
        let fix = AttributesRenameFix::new(Schema, "test".to_string(), rename);
        let input = Typed {
            value: map(vec![("other", Dynamic::Int(1))]),
        };
        let out_entity = fix.fix_entity(input.clone());
        let out_components = fix.fix_data_components(input.clone());
        assert_eq!(out_entity, input);
        assert_eq!(out_components, input);
    }

    #[test]
    fn non_seq_attributes_unchanged() {
        let fix = AttributesRenameFix::new(Schema, "test".to_string(), rename);
        let input = Typed {
            value: map(vec![("attributes", Dynamic::Int(5))]),
        };
        let out = fix.fix_entity(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn unmatched_values_not_renamed() {
        let fix = AttributesRenameFix::new(Schema, "test".to_string(), rename);
        let input = Typed {
            value: map(vec![(
                "attributes",
                seq(vec![map(vec![("id", str_val("keep:me"))])]),
            )]),
        };
        let out = fix.fix_entity(input);
        let attrs = out.value.get("attributes").unwrap().as_seq().unwrap();
        assert_eq!(attrs[0].get("id").unwrap().as_string(), Some("keep:me"));
    }
}
