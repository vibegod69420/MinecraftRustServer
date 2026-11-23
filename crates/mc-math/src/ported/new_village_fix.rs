use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Self::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> Option<Self> {
        match self {
            Self::Map(map) => map.get(key).cloned(),
            _ => None,
        }
    }

    pub fn set(mut self, key: &str, value: Self) -> Self {
        if let Self::Map(map) = &mut self {
            map.insert(key.to_string(), value);
        }
        self
    }

    pub fn remove(mut self, key: &str) -> Self {
        if let Self::Map(map) = &mut self {
            map.remove(key);
        }
        self
    }

    pub fn rename_key(self, from: &str, to: &str) -> Self {
        if let Some(value) = self.get(from) {
            self.remove(from).set(to, value)
        } else {
            self
        }
    }

    pub fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if let Some(value) = self.get(key) {
            self.set(key, f(value))
        } else {
            self
        }
    }

    pub fn get_or_empty_map(&self, key: &str) -> Self {
        self.get(key).unwrap_or_else(Self::empty_map)
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Self::String(value) => value.clone(),
            _ => default.to_string(),
        }
    }

    pub fn create_string(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
pub fn ensure_namespaced(name: &str) -> String {
    if name.contains(':') {
        name.to_string()
    } else {
        format!("minecraft:{}", name)
    }
}

fn fix_starts(starts: Dynamic) -> Dynamic {
    match starts {
        Dynamic::Map(map) => {
            let mut result = BTreeMap::new();
            for (key, value) in map {
                if key == "Village" {
                    continue;
                }
                let key = if key == "New_Village" {
                    "Village".to_string()
                } else {
                    key
                };
                result.insert(key, value);
            }
            Dynamic::Map(result)
        }
        Dynamic::List(items) => {
            let mut result = Vec::new();
            for item in items {
                match item {
                    Dynamic::List(pair) if pair.len() >= 2 => {
                        let key = pair[0].as_string("");
                        if key == "Village" {
                            continue;
                        }
                        let key = if key == "New_Village" {
                            Dynamic::create_string("Village")
                        } else {
                            pair[0].clone()
                        };
                        let mut new_pair = vec![key];
                        new_pair.extend_from_slice(&pair[1..]);
                        result.push(Dynamic::List(new_pair));
                    }
                    other => result.push(other),
                }
            }
            Dynamic::List(result)
        }
        other => other,
    }
}

fn fix_references(references: Dynamic) -> Dynamic {
    match references {
        Dynamic::Map(mut map) => {
            let new_village = map.get("New_Village").cloned();
            if let Some(value) = new_village {
                map.remove("New_Village");
                map.insert("Village".to_string(), value);
            }
            map.remove("Village");
            Dynamic::Map(map)
        }
        other => other,
    }
}

// JAVA: NewVillageFix chunk transformation
pub fn fix_chunk(chunk: Dynamic) -> Dynamic {
    chunk.update("Level", |level| {
        level.update("Structures", |structures| {
            let structures = structures.update("Starts", fix_starts);
            structures.update("References", fix_references)
        })
    })
}

// JAVA: NewVillageFix structure feature id transformation
pub fn fix_structure_feature(feature: Dynamic) -> Dynamic {
    feature.update("id", |id| {
        if ensure_namespaced(&id.as_string("")) == "minecraft:new_village" {
            Dynamic::create_string("minecraft:village")
        } else {
            id
        }
    })
}

pub fn fix_new_village_chunk(chunk: Dynamic) -> Dynamic {
    fix_chunk(chunk)
}

pub fn fix_structure_feature_id(feature: Dynamic) -> Dynamic {
    fix_structure_feature(feature)
}

pub fn fix_references_value(references: Dynamic) -> Dynamic {
    fix_references(references)
}

pub struct NewVillageFix {
    pub erase_cache: bool,
}

impl NewVillageFix {
    pub fn new(_schema: (), erase_cache: bool) -> Self {
        Self { erase_cache }
    }

    pub fn fix_chunk(&self, chunk: Dynamic) -> Dynamic {
        fix_chunk(chunk)
    }

    pub fn fix_structure_feature(&self, feature: Dynamic) -> Dynamic {
        fix_structure_feature(feature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_map_filters_village_and_renames_new_village() {
        let starts = Dynamic::empty_map()
            .set("Village", Dynamic::empty_map().set("id", Dynamic::create_string("old")))
            .set("New_Village", Dynamic::empty_map().set("id", Dynamic::create_string("new")))
            .set("Mineshaft", Dynamic::empty_map());

        let chunk = Dynamic::empty_map().set(
            "Level",
            Dynamic::empty_map().set("Structures", Dynamic::empty_map().set("Starts", starts)),
        );

        let fixed = fix_chunk(chunk);
        let starts = fixed
            .get("Level").unwrap()
            .get("Structures").unwrap()
            .get("Starts").unwrap();

        assert!(starts.get("Village").is_some());
        assert!(starts.get("New_Village").is_none());
        assert!(starts.get("Mineshaft").is_some());
        assert_eq!(
            starts.get("Village").unwrap().get("id"),
            Some(Dynamic::create_string("new"))
        );
    }

    #[test]
    fn references_removes_village_after_new_village_handling() {
        let references = Dynamic::empty_map()
            .set("New_Village", Dynamic::List(vec![Dynamic::Int(1)]))
            .set("Village", Dynamic::List(vec![Dynamic::Int(2)]));

        let fixed = fix_references(references);
        assert!(fixed.get("New_Village").is_none());
        assert!(fixed.get("Village").is_none());
    }

    #[test]
    fn references_without_new_village_removes_village() {
        let references = Dynamic::empty_map()
            .set("Village", Dynamic::List(vec![Dynamic::Int(2)]))
            .set("Mineshaft", Dynamic::List(vec![Dynamic::Int(3)]));

        let fixed = fix_references(references);
        assert!(fixed.get("Village").is_none());
        assert!(fixed.get("Mineshaft").is_some());
    }

    #[test]
    fn structure_feature_id_is_converted() {
        let feature = Dynamic::empty_map().set("id", Dynamic::create_string("minecraft:new_village"));
        let fixed = fix_structure_feature(feature);
        assert_eq!(fixed.get("id"), Some(Dynamic::create_string("minecraft:village")));
    }

    #[test]
    fn unnamespaced_new_village_id_is_converted() {
        let feature = Dynamic::empty_map().set("id", Dynamic::create_string("new_village"));
        let fixed = fix_structure_feature(feature);
        assert_eq!(fixed.get("id"), Some(Dynamic::create_string("minecraft:village")));
    }

    #[test]
    fn ensure_namespaced_handles_colon_and_missing_colon() {
        assert_eq!(ensure_namespaced("minecraft:foo"), "minecraft:foo");
        assert_eq!(ensure_namespaced("foo"), "minecraft:foo");
    }

    #[test]
    fn non_village_structure_feature_is_unchanged() {
        let feature = Dynamic::empty_map().set("id", Dynamic::create_string("minecraft:village"));
        assert_eq!(fix_structure_feature(feature.clone()), feature);
    }
}
