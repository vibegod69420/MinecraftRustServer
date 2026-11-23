use std::collections::BTreeMap;

/// Minimal dynamic value representation for self-contained datafix ports.
#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Map(BTreeMap<String, DynamicValue>),
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Byte(i8),
    Short(i16),
    List(Vec<DynamicValue>),
    Empty,
}

/// Minimal dynamic wrapper mirroring the DFU Dynamic operations used by this fix.
#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    value: DynamicValue,
}

/// Pair of dynamics, mirroring DFU Pair<Dynamic, Dynamic> for map entry rewrites.
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicPair {
    pub first: Dynamic,
    pub second: Dynamic,
}

impl DynamicPair {
    /// Maps the second element of the pair, preserving the first.
    pub fn map_second<F>(self, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Self {
            first: self.first,
            second: f(self.second),
        }
    }
}

impl Dynamic {
    /// Creates a new dynamic from a value.
    pub fn new(value: DynamicValue) -> Self {
        Self { value }
    }

    /// Returns a reference to the underlying value.
    pub fn value(&self) -> &DynamicValue {
        &self.value
    }

    /// Consumes the dynamic and returns the underlying value.
    pub fn into_value(self) -> DynamicValue {
        self.value
    }

    /// Gets a child value by key if this dynamic is a map.
    pub fn get(&self, key: &str) -> Option<&DynamicValue> {
        match &self.value {
            DynamicValue::Map(map) => map.get(key),
            _ => None,
        }
    }

    /// If the key is present in a map, applies the function to its value.
    pub fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        match self.value {
            DynamicValue::Map(mut map) => {
                if let Some(child) = map.remove(key) {
                    let updated = f(Self::new(child));
                    map.insert(key.to_string(), updated.into_value());
                }
                Self::new(DynamicValue::Map(map))
            }
            other => Self::new(other),
        }
    }

    /// Maps over all entries in a map dynamic.
    pub fn update_map_values<F>(self, f: F) -> Self
    where
        F: Fn(DynamicPair) -> DynamicPair,
    {
        match self.value {
            DynamicValue::Map(map) => {
                let new_map = map
                    .into_iter()
                    .map(|(k, v)| {
                        let pair = DynamicPair {
                            first: Self::new(DynamicValue::String(k)),
                            second: Self::new(v),
                        };
                        let pair = f(pair);
                        let key = match pair.first.into_value() {
                            DynamicValue::String(s) => s,
                            _ => String::new(),
                        };
                        (key, pair.second.into_value())
                    })
                    .collect();
                Self::new(DynamicValue::Map(new_map))
            }
            other => Self::new(other),
        }
    }

    /// Removes a key from a map dynamic.
    pub fn remove(self, key: &str) -> Self {
        match self.value {
            DynamicValue::Map(mut map) => {
                map.remove(key);
                Self::new(DynamicValue::Map(map))
            }
            other => Self::new(other),
        }
    }
}

/// Minimal schema placeholder for self-contained DataFix construction.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schema;

/// Port of `net.minecraft.util.datafix.fixes.ForcePoiRebuild`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForcePoiRebuild {
    _schema: Schema,
    _changes_type: bool,
}

impl ForcePoiRebuild {
    // JAVA: net.minecraft.util.datafix.fixes.ForcePoiRebuild#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ForcePoiRebuild#makeRule
    pub fn make_rule(
        &self,
        poi_chunk_type_matches: bool,
    ) -> Result<impl Fn(DynamicPair) -> DynamicPair, String> {
        if !poi_chunk_type_matches {
            return Err("Poi type is not what was expected.".to_string());
        }
        Ok(|pair: DynamicPair| pair.map_second(ForcePoiRebuild::cap))
    }

    // JAVA: net.minecraft.util.datafix.fixes.ForcePoiRebuild#cap
    pub fn cap(dynamic: Dynamic) -> Dynamic {
        dynamic.update("Sections", |sections| {
            sections.update_map_values(|entry| {
                entry.map_second(|section| section.remove("Valid"))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, DynamicValue)>) -> DynamicValue {
        DynamicValue::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn cap_removes_valid_from_sections() {
        let root = map(vec![(
            "Sections",
            map(vec![(
                "0,0,0",
                map(vec![
                    ("Valid", DynamicValue::Byte(1)),
                    ("Records", DynamicValue::Int(42)),
                ]),
            )]),
        )]);
        let result = ForcePoiRebuild::cap(Dynamic::new(root)).into_value();
        let DynamicValue::Map(root_map) = result else {
            panic!("expected root map");
        };
        let Some(DynamicValue::Map(sections)) = root_map.get("Sections") else {
            panic!("expected Sections map");
        };
        let Some(DynamicValue::Map(section)) = sections.get("0,0,0") else {
            panic!("expected section map");
        };
        assert!(!section.contains_key("Valid"));
        assert_eq!(section.get("Records"), Some(&DynamicValue::Int(42)));
    }

    #[test]
    fn cap_no_sections_is_noop() {
        let root = map(vec![("DataVersion", DynamicValue::Int(1))]);
        let original = Dynamic::new(root.clone());
        let result = ForcePoiRebuild::cap(original).into_value();
        assert_eq!(result, root);
    }

    #[test]
    fn cap_section_without_valid_unchanged() {
        let section = map(vec![("Records", DynamicValue::Int(7))]);
        let root = map(vec![(
            "Sections",
            map(vec![("1,2,3", section.clone())]),
        )]);
        let result = ForcePoiRebuild::cap(Dynamic::new(root)).into_value();
        let DynamicValue::Map(root_map) = result else {
            panic!("expected root map");
        };
        let Some(DynamicValue::Map(sections)) = root_map.get("Sections") else {
            panic!("expected Sections map");
        };
        assert_eq!(sections.get("1,2,3"), Some(&section));
    }

    #[test]
    fn cap_multiple_sections_and_preserves_other_fields() {
        let root = map(vec![
            (
                "Sections",
                map(vec![
                    ("a", map(vec![("Valid", DynamicValue::Byte(1))])),
                    (
                        "b",
                        map(vec![
                            ("Valid", DynamicValue::Byte(0)),
                            ("Data", DynamicValue::Long(-1)),
                        ]),
                    ),
                ]),
            ),
            ("Keep", DynamicValue::String("yes".to_string())),
        ]);
        let result = ForcePoiRebuild::cap(Dynamic::new(root)).into_value();
        let DynamicValue::Map(root_map) = result else {
            panic!("expected root map");
        };
        assert_eq!(
            root_map.get("Keep"),
            Some(&DynamicValue::String("yes".to_string()))
        );
        let Some(DynamicValue::Map(sections)) = root_map.get("Sections") else {
            panic!("expected Sections map");
        };
        let Some(DynamicValue::Map(a)) = sections.get("a") else {
            panic!("expected section a");
        };
        let Some(DynamicValue::Map(b)) = sections.get("b") else {
            panic!("expected section b");
        };
        assert!(!a.contains_key("Valid"));
        assert!(!b.contains_key("Valid"));
        assert_eq!(b.get("Data"), Some(&DynamicValue::Long(-1)));
    }

    #[test]
    fn make_rule_rejects_mismatched_type() {
        let fix = ForcePoiRebuild::new(Schema, false);
        let err = match fix.make_rule(false) {
            Ok(_) => panic!("expected error"),
            Err(e) => e,
        };
        assert_eq!(err, "Poi type is not what was expected.");
    }

    #[test]
    fn make_rule_applies_cap_to_pair_second() {
        let root = map(vec![(
            "Sections",
            map(vec![("s", map(vec![("Valid", DynamicValue::Byte(1))]))]),
        )]);
        let pair = DynamicPair {
            first: Dynamic::new(DynamicValue::String("minecraft:poi_chunk".to_string())),
            second: Dynamic::new(root),
        };
        let fix = ForcePoiRebuild::new(Schema, false);
        let rule = fix.make_rule(true).expect("rule should be created");
        let out = rule(pair);
        assert_eq!(
            out.first.value(),
            &DynamicValue::String("minecraft:poi_chunk".to_string())
        );
        let DynamicValue::Map(root_map) = out.second.into_value() else {
            panic!("expected root map");
        };
        let Some(DynamicValue::Map(sections)) = root_map.get("Sections") else {
            panic!("expected Sections map");
        };
        let Some(DynamicValue::Map(section)) = sections.get("s") else {
            panic!("expected section");
        };
        assert!(!section.contains_key("Valid"));
    }
}
