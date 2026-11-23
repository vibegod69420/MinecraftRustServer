use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Byte(i8),
    Bool(bool),
    Float(f32),
    Double(f64),
    Null,
}

pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    // JAVA: com.mojang.serialization.OptionalDynamic#result
    pub fn result(self) -> Option<Dynamic> {
        self.0
    }
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(m) => OptionalDynamic(m.get(key).cloned()),
            _ => OptionalDynamic(None),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createMap
    pub fn create_map(&self, map: BTreeMap<String, Dynamic>) -> Dynamic {
        Dynamic::Map(map)
    }

    // JAVA: com.mojang.serialization.Dynamic#createString
    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    // JAVA: com.mojang.serialization.Dynamic#remove
    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut new_map = m.clone();
                new_map.remove(key);
                Dynamic::Map(new_map)
            }
            _ => self.clone(),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#set
    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut new_map = m.clone();
                new_map.insert(key.to_string(), value);
                Dynamic::Map(new_map)
            }
            _ => self.clone(),
        }
    }
}

pub struct ReorganizePoi;

impl ReorganizePoi {
    // JAVA: net.minecraft.util.datafix.fixes.ReorganizePoi#<init>
    pub fn new(_schema: (), _changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ReorganizePoi#makeRule
    pub fn make_rule(&self, d: Dynamic) -> Dynamic {
        Self::cap(d)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ReorganizePoi#cap
    pub fn cap(d: Dynamic) -> Dynamic {
        let mut sections: BTreeMap<String, Dynamic> = BTreeMap::new();
        let mut out = d;
        let mut i: i32 = 0;
        while i < 16 {
            let key = i.to_string();
            let maybe = out.get(&key).result();
            if let Some(section) = maybe {
                let mut record_map = BTreeMap::new();
                record_map.insert("Records".to_string(), section);
                let wrapped = out.create_map(record_map);
                sections.insert(i.to_string(), wrapped);
                out = out.remove(&key);
            }
            i = i.wrapping_add(1);
        }
        let sections_dynamic = out.create_map(sections);
        out.set("Sections", sections_dynamic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn reorganizes_present_sections() {
        let data = map(vec![
            ("0".into(), Dynamic::String("s0".into())),
            ("2".into(), Dynamic::Int(42)),
            ("keep".into(), Dynamic::Bool(true)),
        ]);
        let out = ReorganizePoi::cap(data);
        assert_eq!(out.get("0").result(), None);
        assert_eq!(out.get("2").result(), None);
        assert_eq!(out.get("keep").result(), Some(Dynamic::Bool(true)));

        let sections = match out.get("Sections").result() {
            Some(Dynamic::Map(m)) => m,
            other => panic!("expected Sections map, got {:?}", other),
        };
        assert_eq!(sections.len(), 2);
        assert_eq!(
            sections.get("0"),
            Some(&map(vec![("Records".into(), Dynamic::String("s0".into()))]))
        );
        assert_eq!(
            sections.get("2"),
            Some(&map(vec![("Records".into(), Dynamic::Int(42))]))
        );
    }

    #[test]
    fn empty_input_creates_empty_sections() {
        let out = ReorganizePoi::cap(map(vec![]));
        let sections = match out.get("Sections").result() {
            Some(Dynamic::Map(m)) => m,
            other => panic!("expected Sections map, got {:?}", other),
        };
        assert!(sections.is_empty());
    }

    #[test]
    fn all_sixteen_sections_moved() {
        let mut entries = Vec::new();
        for i in 0..16 {
            entries.push((Box::leak(i.to_string().into_boxed_str()) as &str, Dynamic::Int(i)));
        }
        let out = ReorganizePoi::cap(map(entries));
        let sections = match out.get("Sections").result() {
            Some(Dynamic::Map(m)) => m.clone(),
            other => panic!("expected Sections map, got {:?}", other),
        };
        assert_eq!(sections.len(), 16);
        for i in 0..16 {
            let key = i.to_string();
            assert_eq!(out.get(&key).result(), None);
            let wrapped = sections.get(&key).expect("missing section");
            assert_eq!(wrapped.get("Records").result(), Some(Dynamic::Int(i)));
        }
    }

    #[test]
    fn preserves_float_bits_and_negative_values() {
        let nan_bits = f32::NAN.to_bits();
        let neg_inf_bits = f64::NEG_INFINITY.to_bits();
        let data = map(vec![
            ("1".into(), Dynamic::Float(f32::from_bits(nan_bits))),
            ("3".into(), Dynamic::Double(f64::from_bits(neg_inf_bits))),
            ("neg".into(), Dynamic::Int(i32::MIN)),
        ]);
        let out = ReorganizePoi::cap(data);
        assert_eq!(out.get("neg").result(), Some(Dynamic::Int(i32::MIN)));
        let sections = match out.get("Sections").result() {
            Some(Dynamic::Map(m)) => m.clone(),
            _ => panic!("missing sections"),
        };
        match sections.get("1").unwrap().get("Records").result() {
            Some(Dynamic::Float(f)) => assert_eq!(f.to_bits(), nan_bits),
            other => panic!("{:?}", other),
        }
        match sections.get("3").unwrap().get("Records").result() {
            Some(Dynamic::Double(f)) => assert_eq!(f.to_bits(), neg_inf_bits),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn make_rule_applies_cap() {
        let fix = ReorganizePoi::new((), true);
        let data = map(vec![("5".into(), Dynamic::Byte(7))]);
        let out = fix.make_rule(data);
        assert_eq!(out.get("5").result(), None);
        let sections = match out.get("Sections").result() {
            Some(Dynamic::Map(m)) => m.clone(),
            _ => panic!("missing sections"),
        };
        assert_eq!(
            sections.get("5").unwrap().get("Records").result(),
            Some(Dynamic::Byte(7))
        );
    }
}
