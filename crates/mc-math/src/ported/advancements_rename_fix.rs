use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    List(Vec<Dynamic>),
}

impl Dynamic {
    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn create_string(&self, s: String) -> Dynamic {
        Dynamic::String(s)
    }

    pub fn update_map_values<F: FnMut((Dynamic, Dynamic)) -> (Dynamic, Dynamic)>(
        self,
        mut f: F,
    ) -> Self {
        match self {
            Dynamic::Map(m) => {
                let mut new_m = BTreeMap::new();
                for (k, v) in m {
                    let (nk, nv) = f((Dynamic::String(k), v));
                    let nk_str = nk.as_string("");
                    new_m.insert(nk_str, nv);
                }
                Dynamic::Map(new_m)
            }
            other => other,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct AdvancementsRenameFix {
    name: String,
    renamer: Box<dyn Fn(String) -> String>,
}

impl AdvancementsRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsRenameFix#<init>
    pub fn new(_schema: Schema, _changes_type: bool, name: String, renamer: Box<dyn Fn(String) -> String>) -> Self {
        Self { name, renamer }
    }

    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsRenameFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic + '_ {
        move |input: Dynamic| {
            input.update_map_values(|pair: (Dynamic, Dynamic)| {
                let key = pair.0.as_string("");
                let renamed = (self.renamer)(key);
                (Dynamic::String(renamed), pair.1)
            })
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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
    fn renames_advancement_keys() {
        let fix = AdvancementsRenameFix::new(
            Schema,
            false,
            "test".to_string(),
            Box::new(|s| if s == "minecraft:old" { "minecraft:new".to_string() } else { s }),
        );
        let input = map(vec![
            ("minecraft:old", Dynamic::Int(1)),
            ("minecraft:keep", Dynamic::Int(2)),
        ]);
        let out = (fix.make_rule())(input);
        if let Dynamic::Map(m) = out {
            assert!(m.contains_key("minecraft:new"));
            assert!(!m.contains_key("minecraft:old"));
            assert_eq!(m.get("minecraft:keep"), Some(&Dynamic::Int(2)));
        } else {
            panic!("expected map");
        }
    }

    #[test]
    fn preserves_values_and_handles_empty_map() {
        let fix = AdvancementsRenameFix::new(Schema, false, "n".to_string(), Box::new(|s| s + "_x"));
        let out = (fix.make_rule())(Dynamic::Map(BTreeMap::new()));
        assert_eq!(out, Dynamic::Map(BTreeMap::new()));
    }

    #[test]
    fn renamer_receives_original_key() {
        let fix = AdvancementsRenameFix::new(
            Schema,
            true,
            "rename".to_string(),
            Box::new(|s| s.replace("foo", "bar")),
        );
        let input = map(vec![("foo/test", Dynamic::String("v".to_string()))]);
        let out = (fix.make_rule())(input);
        if let Dynamic::Map(m) = out {
            assert_eq!(m.get("bar/test"), Some(&Dynamic::String("v".to_string())));
        } else {
            panic!("expected map");
        }
        assert_eq!(fix.name(), "rename");
    }

    #[test]
    fn non_map_input_is_preserved() {
        let fix = AdvancementsRenameFix::new(Schema, false, "n".to_string(), Box::new(|s| s));
        let input = Dynamic::Int(42);
        let out = (fix.make_rule())(input.clone());
        assert_eq!(out, input);
    }
}
