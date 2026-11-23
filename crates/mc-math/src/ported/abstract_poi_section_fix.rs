use std::collections::BTreeMap;

/// Minimal self-contained dynamic value representation for datafix porting.
/// Mirrors the subset of `com.mojang.serialization.Dynamic` operations used by this fix.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Byte(i8),
    Short(i16),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn get_or_empty(&self, key: &str) -> Dynamic {
        self.get(key).cloned().unwrap_or(Dynamic::Null)
    }

    pub fn as_bool(&self, default: bool) -> bool {
        match self {
            Dynamic::Bool(b) => *b,
            Dynamic::Byte(v) => *v != 0,
            Dynamic::Short(v) => *v != 0,
            Dynamic::Int(v) => *v != 0,
            Dynamic::Long(v) => *v != 0,
            _ => default,
        }
    }

    pub fn as_stream_opt(&self) -> Option<Vec<Dynamic>> {
        match self {
            Dynamic::List(list) => Some(list.clone()),
            _ => None,
        }
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Dynamic {
        Dynamic::List(items)
    }

    pub fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(mut map) => {
                let child = map.remove(key).unwrap_or(Dynamic::Null);
                let updated = f(child);
                map.insert(key.to_string(), updated);
                Dynamic::Map(map)
            }
            other => other,
        }
    }

    pub fn update_map_values<F>(self, mut f: F) -> Self
    where
        F: FnMut(String, Dynamic) -> (String, Dynamic),
    {
        match self {
            Dynamic::Map(map) => {
                let mut new_map = BTreeMap::new();
                for (k, v) in map {
                    let (nk, nv) = f(k, v);
                    new_map.insert(nk, nv);
                }
                Dynamic::Map(new_map)
            }
            other => other,
        }
    }

    pub fn remove(self, key: &str) -> Self {
        match self {
            Dynamic::Map(mut map) => {
                map.remove(key);
                Dynamic::Map(map)
            }
            other => other,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut map) => {
                map.insert(key.to_string(), value);
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

fn or_else<T>(opt: Option<T>, fallback: T) -> T {
    opt.unwrap_or(fallback)
}

/// Faithful structural port of `net.minecraft.util.datafix.fixes.AbstractPoiSectionFix`.
/// This is an abstract datafix that processes POI chunk section records.
pub struct AbstractPoiSectionFix {
    pub name: String,
}

impl AbstractPoiSectionFix {
    // JAVA: net.minecraft.util.datafix.fixes.AbstractPoiSectionFix#<init>
    pub fn new(name: String) -> Self {
        Self { name }
    }

    // JAVA: net.minecraft.util.datafix.fixes.AbstractPoiSectionFix#cap
    fn cap<F>(&self, value: Dynamic, process_records: &F) -> Dynamic
    where
        F: Fn(Vec<Dynamic>) -> Vec<Dynamic>,
    {
        value.update("Sections", |sections| {
            sections.update_map_values(|key, section_pair_value| {
                // Java: mapSecond(processSection) on each map entry value
                (key, self.process_section(section_pair_value, process_records))
            })
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.AbstractPoiSectionFix#processSection
    fn process_section<F>(&self, section: Dynamic, process_records: &F) -> Dynamic
    where
        F: Fn(Vec<Dynamic>) -> Vec<Dynamic>,
    {
        section.update("Records", |records| self.process_section_records(records, process_records))
    }

    // JAVA: net.minecraft.util.datafix.fixes.AbstractPoiSectionFix#processSectionRecords
    fn process_section_records<F>(&self, records: Dynamic, process_records: &F) -> Dynamic
    where
        F: Fn(Vec<Dynamic>) -> Vec<Dynamic>,
    {
        let original = records.clone();
        let mapped = records
            .as_stream_opt()
            .map(|stream| original.create_list(process_records(stream)));
        or_else(mapped, original)
    }

    /// Applies this fix to a POI chunk dynamic value by processing its section records.
    /// `process_records` corresponds to the abstract `processRecords` method in Java.
    // JAVA: net.minecraft.util.datafix.fixes.AbstractPoiSectionFix#processRecords
    pub fn apply_with<F>(&self, poi_chunk: Dynamic, process_records: F) -> Dynamic
    where
        F: Fn(Vec<Dynamic>) -> Vec<Dynamic>,
    {
        self.cap(poi_chunk, &process_records)
    }

    // JAVA: net.minecraft.util.datafix.fixes.AbstractPoiSectionFix#makeRule
    pub fn make_rule_description(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_poi_chunk(sections: Vec<(String, Vec<Dynamic>)>) -> Dynamic {
        let mut sections_map = BTreeMap::new();
        for (key, records) in sections {
            let mut section = BTreeMap::new();
            section.insert("Records".to_string(), Dynamic::List(records));
            sections_map.insert(key, Dynamic::Map(section));
        }
        let mut root = BTreeMap::new();
        root.insert("Sections".to_string(), Dynamic::Map(sections_map));
        Dynamic::Map(root)
    }

    fn record(name: &str) -> Dynamic {
        let mut m = BTreeMap::new();
        m.insert("type".to_string(), Dynamic::String(name.to_string()));
        Dynamic::Map(m)
    }

    #[test]
    fn test_process_records_applied() {
        let fix = AbstractPoiSectionFix::new("TestPoiFix".to_string());
        let input = make_poi_chunk(vec![("0,0,0".to_string(), vec![record("a"), record("b")])]);
        let output = fix.apply_with(input, |records| {
            records
                .into_iter()
                .filter(|r| r.get("type") != Some(&Dynamic::String("a".to_string())))
                .collect()
        });
        let records = output
            .get("Sections")
            .and_then(|s| s.get("0,0,0"))
            .and_then(|s| s.get("Records"))
            .and_then(|r| r.as_stream_opt())
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get("type"), Some(&Dynamic::String("b".to_string())));
    }

    #[test]
    fn test_missing_records_kept_as_is() {
        let fix = AbstractPoiSectionFix::new("TestPoiFix".to_string());
        let mut section = BTreeMap::new();
        section.insert("Valid".to_string(), Dynamic::Byte(1));
        let mut sections = BTreeMap::new();
        sections.insert("0".to_string(), Dynamic::Map(section.clone()));
        let mut root = BTreeMap::new();
        root.insert("Sections".to_string(), Dynamic::Map(sections));
        let input = Dynamic::Map(root);
        let output = fix.apply_with(input.clone(), |r| r);
        // Records is Null/missing, so process_section_records returns original (Null)
        let sec = output.get("Sections").unwrap().get("0").unwrap();
        assert_eq!(sec.get("Records"), Some(&Dynamic::Null));
        assert_eq!(sec.get("Valid"), Some(&Dynamic::Byte(1)));
    }

    #[test]
    fn test_empty_sections() {
        let fix = AbstractPoiSectionFix::new("Empty".to_string());
        let mut root = BTreeMap::new();
        root.insert("Sections".to_string(), Dynamic::empty_map());
        let input = Dynamic::Map(root);
        let output = fix.apply_with(input, |_r| vec![record("should_not_appear")]);
        assert_eq!(output.get("Sections"), Some(&Dynamic::empty_map()));
    }

    #[test]
    fn test_multiple_sections_processed() {
        let fix = AbstractPoiSectionFix::new("Multi".to_string());
        let input = make_poi_chunk(vec![
            ("a".to_string(), vec![record("x")]),
            ("b".to_string(), vec![record("y"), record("z")]),
        ]);
        let output = fix.apply_with(input, |mut records| {
            records.push(record("added"));
            records
        });
        let a_records = output.get("Sections").unwrap().get("a").unwrap().get("Records").unwrap().as_stream_opt().unwrap();
        let b_records = output.get("Sections").unwrap().get("b").unwrap().get("Records").unwrap().as_stream_opt().unwrap();
        assert_eq!(a_records.len(), 2);
        assert_eq!(b_records.len(), 3);
    }

    #[test]
    fn test_make_rule_description() {
        let fix = AbstractPoiSectionFix::new("PoiFixName".to_string());
        assert_eq!(fix.make_rule_description(), "PoiFixName");
    }
}
