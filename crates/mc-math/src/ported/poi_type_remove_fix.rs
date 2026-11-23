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

    fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(map) => map.get(key),
            _ => None,
        }
    }
}

pub struct PoiTypeRemoveFix {
    types_to_keep: Box<dyn Fn(&str) -> bool>,
}

impl PoiTypeRemoveFix {
    // JAVA: net.minecraft.util.datafix.fixes.PoiTypeRemoveFix#<init>
    pub fn new(_schema: (), _name: &str, types_to_remove: Box<dyn Fn(&str) -> bool>) -> Self {
        Self {
            types_to_keep: Box::new(move |s| !types_to_remove(s)),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.PoiTypeRemoveFix#processRecords
    pub fn process_records(&self, records: impl IntoIterator<Item = Dynamic>) -> Vec<Dynamic> {
        records
            .into_iter()
            .filter(|r| self.should_keep_record(r))
            .collect()
    }

    // JAVA: net.minecraft.util.datafix.fixes.PoiTypeRemoveFix#shouldKeepRecord
    fn should_keep_record(&self, record: &Dynamic) -> bool {
        record
            .get("type")
            .and_then(|d| d.as_string())
            .is_some_and(|s| (self.types_to_keep)(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(type_name: &str, extra: i32) -> Dynamic {
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), Dynamic::String(type_name.to_string()));
        map.insert("extra".to_string(), Dynamic::Int(extra));
        Dynamic::Map(map)
    }

    #[test]
    fn removes_matching_types() {
        let fix = PoiTypeRemoveFix::new((), "test", Box::new(|s| s == "minecraft:old_poi"));
        let out = fix.process_records(vec![
            record("minecraft:old_poi", 1),
            record("minecraft:keep", 2),
        ]);
        assert_eq!(out.len(), 1);
        match &out[0] {
            Dynamic::Map(map) => {
                assert_eq!(
                    map.get("type"),
                    Some(&Dynamic::String("minecraft:keep".into()))
                );
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn drops_non_string_or_missing_type() {
        let fix = PoiTypeRemoveFix::new((), "test", Box::new(|_| false));
        let mut missing = BTreeMap::new();
        missing.insert("extra".to_string(), Dynamic::Int(1));
        let mut non_string = BTreeMap::new();
        non_string.insert("type".to_string(), Dynamic::Int(i32::MIN));
        let out = fix.process_records(vec![Dynamic::Map(missing), Dynamic::Map(non_string)]);
        assert!(out.is_empty());
    }

    #[test]
    fn preserves_negative_and_overflow_fields_in_kept_records() {
        let fix = PoiTypeRemoveFix::new((), "test", Box::new(|_| false));
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), Dynamic::String("keep".into()));
        map.insert("a".to_string(), Dynamic::Int(i32::MIN));
        map.insert("b".to_string(), Dynamic::Long(i64::MIN));
        let out = fix.process_records(vec![Dynamic::Map(map)]);
        assert_eq!(out.len(), 1);
        match &out[0] {
            Dynamic::Map(map) => {
                assert_eq!(map.get("a"), Some(&Dynamic::Int(i32::MIN)));
                assert_eq!(map.get("b"), Some(&Dynamic::Long(i64::MIN)));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn preserves_float_bits_in_kept_records() {
        let fix = PoiTypeRemoveFix::new((), "test", Box::new(|_| false));
        let nan_bits = f64::NAN.to_bits();
        let neg_zero_bits = (-0.0f32).to_bits();
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), Dynamic::String("keep".into()));
        map.insert("d".to_string(), Dynamic::Double(f64::from_bits(nan_bits)));
        map.insert(
            "f".to_string(),
            Dynamic::Float(f32::from_bits(neg_zero_bits)),
        );
        let out = fix.process_records(vec![Dynamic::Map(map)]);
        assert_eq!(out.len(), 1);
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
}
