use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Null,
}

impl Dynamic {
    fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::String(s) => Some(s),
            _ => None,
        }
    }

    fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(mut map) => {
                if let Some(value) = map.remove(key) {
                    let new_value = f(value);
                    map.insert(key.to_string(), new_value);
                }
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

pub struct PoiTypeRenameFix {
    renamer: Box<dyn Fn(&str) -> String>,
}

impl PoiTypeRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.PoiTypeRenameFix#<init>
    pub fn new(_schema: (), _name: &str, renamer: Box<dyn Fn(&str) -> String>) -> Self {
        Self { renamer }
    }

    // JAVA: net.minecraft.util.datafix.fixes.PoiTypeRenameFix#processRecords
    pub fn process_records(&self, records: impl IntoIterator<Item = Dynamic>) -> Vec<Dynamic> {
        records
            .into_iter()
            .map(|record| {
                record.update("type", |type_val| {
                    if let Some(s) = type_val.as_string() {
                        Dynamic::String((self.renamer)(s))
                    } else {
                        type_val
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(type_val: Dynamic) -> Dynamic {
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), type_val);
        map.insert("pos".to_string(), Dynamic::Int(0));
        Dynamic::Map(map)
    }

    #[test]
    fn renames_string_type() {
        let fix = PoiTypeRenameFix::new(
            (),
            "test",
            Box::new(|s| if s == "old" { "new".into() } else { s.into() }),
        );
        let out = fix.process_records(vec![record(Dynamic::String("old".into()))]);
        match &out[0] {
            Dynamic::Map(map) => {
                assert_eq!(map.get("type"), Some(&Dynamic::String("new".into())));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn non_string_type_preserved() {
        let fix = PoiTypeRenameFix::new((), "test", Box::new(|_| "x".into()));
        let out = fix.process_records(vec![record(Dynamic::Int(-1))]);
        match &out[0] {
            Dynamic::Map(map) => {
                assert_eq!(map.get("type"), Some(&Dynamic::Int(-1)));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn preserves_negative_and_overflow_numeric_fields() {
        let fix = PoiTypeRenameFix::new((), "test", Box::new(|s| s.to_string()));
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), Dynamic::String("poi".into()));
        map.insert("x".to_string(), Dynamic::Int(i32::MIN));
        map.insert("y".to_string(), Dynamic::Long(i64::MAX));
        let out = fix.process_records(vec![Dynamic::Map(map)]);
        match &out[0] {
            Dynamic::Map(map) => {
                assert_eq!(map.get("x"), Some(&Dynamic::Int(i32::MIN)));
                assert_eq!(map.get("y"), Some(&Dynamic::Long(i64::MAX)));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn preserves_float_bits_in_other_fields() {
        let fix = PoiTypeRenameFix::new((), "test", Box::new(|s| s.to_string()));
        let nan_bits = f64::NAN.to_bits();
        let neg_zero_bits = (-0.0f32).to_bits();
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), Dynamic::String("poi".into()));
        map.insert("d".to_string(), Dynamic::Double(f64::from_bits(nan_bits)));
        map.insert(
            "f".to_string(),
            Dynamic::Float(f32::from_bits(neg_zero_bits)),
        );
        let out = fix.process_records(vec![Dynamic::Map(map)]);
        match &out[0] {
            Dynamic::Map(map) => {
                match map.get("d") {
                    Some(Dynamic::Double(v)) => assert_eq!(v.to_bits(), nan_bits),
                    other => panic!("unexpected d: {:?}", other),
                }
                match map.get("f") {
                    Some(Dynamic::Float(v)) => assert_eq!(v.to_bits(), neg_zero_bits),
                    other => panic!("unexpected f: {:?}", other),
                }
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn missing_type_field_unchanged() {
        let fix = PoiTypeRenameFix::new((), "test", Box::new(|_| "x".into()));
        let mut map = BTreeMap::new();
        map.insert("pos".to_string(), Dynamic::Int(1));
        let out = fix.process_records(vec![Dynamic::Map(map)]);
        match &out[0] {
            Dynamic::Map(map) => {
                assert!(!map.contains_key("type"));
                assert_eq!(map.get("pos"), Some(&Dynamic::Int(1)));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }
}
