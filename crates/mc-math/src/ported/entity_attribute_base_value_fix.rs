use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Int(i32),
    Long(i64),
    Short(i16),
    Byte(i8),
    Float(f32),
    Double(f64),
    String(String),
    Bool(bool),
    Map(BTreeMap<String, DynValue>),
    List(Vec<DynValue>),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynValue,
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#new
    pub fn new(value: DynValue) -> Self {
        Self { value }
    }

    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => Dynamic::new(m.get(key).cloned().unwrap_or(DynValue::Null)),
            _ => Dynamic::new(DynValue::Null),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#update(String, Function)
    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(&self, key: &str, f: F) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => match m.get(key) {
                Some(old) => {
                    let mut new_map = m.clone();
                    new_map.insert(key.to_string(), f(Dynamic::new(old.clone())).value);
                    Dynamic::new(DynValue::Map(new_map))
                }
                None => self.clone(),
            },
            _ => self.clone(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createList
    pub fn create_list(&self, values: Vec<Dynamic>) -> Dynamic {
        Dynamic::new(DynValue::List(
            values.into_iter().map(|d| d.value).collect(),
        ))
    }

    // JAVA: com.mojang.serialization.Dynamic#createDouble
    pub fn create_double(&self, v: f64) -> Dynamic {
        Dynamic::new(DynValue::Double(v))
    }

    // JAVA: com.mojang.serialization.Dynamic#asStream
    pub fn as_stream(&self) -> Vec<Dynamic> {
        match &self.value {
            DynValue::List(list) => list.iter().cloned().map(Dynamic::new).collect(),
            _ => Vec::new(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#set
    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                let mut new_map = m.clone();
                new_map.insert(key.to_string(), value.value);
                Dynamic::new(DynValue::Map(new_map))
            }
            _ => self.clone(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asDouble
    pub fn as_double(&self, default: f64) -> f64 {
        match &self.value {
            DynValue::Double(v) => *v,
            DynValue::Float(v) => f64::from(*v),
            DynValue::Int(v) => f64::from(*v),
            DynValue::Long(v) => *v as f64,
            DynValue::Short(v) => f64::from(*v),
            DynValue::Byte(v) => f64::from(*v),
            _ => default,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asString
    pub fn as_string(&self, default: &str) -> String {
        match &self.value {
            DynValue::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    // JAVA: com.mojang.datafixers.Typed#update
    pub fn update_remainder<F: FnOnce(Dynamic) -> Dynamic>(&self, f: F) -> Typed {
        Typed {
            remainder: f(self.remainder.clone()),
        }
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
fn ensure_namespaced(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    }
}

pub struct EntityAttributeBaseValueFix {
    pub attribute_id: String,
    pub value_fixer: fn(f64) -> f64,
}

impl EntityAttributeBaseValueFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityAttributeBaseValueFix#<init>
    pub fn new(
        _schema: (),
        _name: String,
        _entity_name: String,
        attribute_id: String,
        value_fixer: fn(f64) -> f64,
    ) -> Self {
        Self {
            attribute_id,
            value_fixer,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityAttributeBaseValueFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update_remainder(|d| self.fix_value(d))
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityAttributeBaseValueFix#fixValue
    fn fix_value(&self, dynamic: Dynamic) -> Dynamic {
        dynamic.update("attributes", |attrs| {
            let mapped: Vec<Dynamic> = attrs
                .as_stream()
                .into_iter()
                .map(|attr| {
                    let id = ensure_namespaced(&attr.get("id").as_string(""));
                    if id != self.attribute_id {
                        return attr;
                    }
                    let base = attr.get("base").as_double(0.0);
                    let new_base = (self.value_fixer)(base);
                    let new_base_dyn = attr.create_double(new_base);
                    attr.set("base", new_base_dyn)
                })
                .collect();
            attrs.create_list(mapped)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity_with_attrs(attrs: &[(&str, f64)]) -> Dynamic {
        let list: Vec<DynValue> = attrs
            .iter()
            .map(|(id, base)| {
                let mut m = BTreeMap::new();
                m.insert("id".to_string(), DynValue::String((*id).to_string()));
                m.insert("base".to_string(), DynValue::Double(*base));
                DynValue::Map(m)
            })
            .collect();
        let mut root = BTreeMap::new();
        root.insert("attributes".to_string(), DynValue::List(list));
        Dynamic::new(DynValue::Map(root))
    }

    #[test]
    fn updates_matching_attribute_and_normalizes_namespace() {
        let fix = EntityAttributeBaseValueFix::new(
            (),
            "test".to_string(),
            "Entity".to_string(),
            "minecraft:generic.max_health".to_string(),
            |v| v * 2.0,
        );
        let dyn_val = entity_with_attrs(&[
            ("generic.max_health", 20.0),
            ("minecraft:movement_speed", 0.7),
        ]);
        let out = fix.fix(Typed { remainder: dyn_val });
        let attrs = out.remainder.get("attributes").as_stream();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].get("base").as_double(0.0).to_bits(), 40.0f64.to_bits());
        assert_eq!(attrs[1].get("base").as_double(0.0).to_bits(), 0.7f64.to_bits());
    }

    #[test]
    fn non_matching_attributes_unchanged() {
        let fix = EntityAttributeBaseValueFix::new(
            (),
            "t".to_string(),
            "E".to_string(),
            "minecraft:attack_damage".to_string(),
            |v| v + 100.0,
        );
        let dyn_val = entity_with_attrs(&[("minecraft:movement_speed", 0.1)]);
        let out = fix.fix(Typed {
            remainder: dyn_val.clone(),
        });
        assert_eq!(out.remainder, dyn_val);
    }

    #[test]
    fn missing_base_defaults_to_zero() {
        let fix = EntityAttributeBaseValueFix::new(
            (),
            "t".to_string(),
            "E".to_string(),
            "minecraft:attr".to_string(),
            |v| v + 1.0,
        );
        let mut attr = BTreeMap::new();
        attr.insert("id".to_string(), DynValue::String("minecraft:attr".to_string()));
        let mut root = BTreeMap::new();
        root.insert("attributes".to_string(), DynValue::List(vec![DynValue::Map(attr)]));
        let out = fix.fix(Typed {
            remainder: Dynamic::new(DynValue::Map(root)),
        });
        let attrs = out.remainder.get("attributes").as_stream();
        assert_eq!(attrs[0].get("base").as_double(f64::NAN).to_bits(), 1.0f64.to_bits());
    }

    #[test]
    fn negative_and_float_bits_preserved() {
        let fix = EntityAttributeBaseValueFix::new(
            (),
            "t".to_string(),
            "E".to_string(),
            "minecraft:attr".to_string(),
            |v| v - 1.0,
        );
        let bits = (-0.0f64).to_bits();
        let val = f64::from_bits(bits);
        let dyn_val = entity_with_attrs(&[("minecraft:attr", val)]);
        let out = fix.fix(Typed { remainder: dyn_val });
        let attrs = out.remainder.get("attributes").as_stream();
        assert_eq!(attrs[0].get("base").as_double(0.0).to_bits(), (-1.0f64).to_bits());

        let fix2 = EntityAttributeBaseValueFix::new(
            (),
            "t".to_string(),
            "E".to_string(),
            "minecraft:attr".to_string(),
            |v| v,
        );
        let neg = -5.5f64;
        let out2 = fix2.fix(Typed {
            remainder: entity_with_attrs(&[("minecraft:attr", neg)]),
        });
        let attrs2 = out2.remainder.get("attributes").as_stream();
        assert_eq!(attrs2[0].get("base").as_double(0.0).to_bits(), neg.to_bits());
    }

    #[test]
    fn missing_attributes_field_returns_unchanged() {
        let fix = EntityAttributeBaseValueFix::new(
            (),
            "t".to_string(),
            "E".to_string(),
            "minecraft:attr".to_string(),
            |v| v + 1.0,
        );
        let mut root = BTreeMap::new();
        root.insert("id".to_string(), DynValue::String("test".to_string()));
        let original = Dynamic::new(DynValue::Map(root));
        let out = fix.fix(Typed {
            remainder: original.clone(),
        });
        assert_eq!(out.remainder, original);
    }
}
