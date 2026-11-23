use std::collections::{BTreeMap, HashMap};

/// Minimal self-contained dynamic value for datafix porting.
#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Bool(bool),
    Null,
}

pub struct VariantRenameFix {
    renames: HashMap<String, String>,
}

impl VariantRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.VariantRenameFix#<init>
    pub fn new(renames: HashMap<String, String>) -> Self {
        Self { renames }
    }

    // JAVA: net.minecraft.util.datafix.fixes.VariantRenameFix#fix
    pub fn fix(&self, data: Dynamic) -> Dynamic {
        match data {
            Dynamic::Map(mut map) => {
                if let Some(variant) = map.remove("variant") {
                    let fixed = match variant {
                        Dynamic::String(s) => {
                            let renamed = self.renames.get(&s).cloned().unwrap_or(s);
                            Dynamic::String(renamed)
                        }
                        other => other,
                    };
                    map.insert("variant".to_string(), fixed);
                }
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(pairs: &[(&str, Dynamic)]) -> Dynamic {
        let mut map = BTreeMap::new();
        for (k, v) in pairs {
            map.insert((*k).to_string(), v.clone());
        }
        Dynamic::Map(map)
    }

    fn get_variant_str(d: &Dynamic) -> Option<&str> {
        match d {
            Dynamic::Map(m) => match m.get("variant") {
                Some(Dynamic::String(s)) => Some(s.as_str()),
                _ => None,
            },
            _ => None,
        }
    }

    #[test]
    fn renames_matching_variant() {
        let mut renames = HashMap::new();
        renames.insert("old".to_string(), "new".to_string());
        let fix = VariantRenameFix::new(renames);
        let input = obj(&[("variant", Dynamic::String("old".to_string()))]);
        let out = fix.fix(input);
        assert_eq!(get_variant_str(&out), Some("new"));
    }

    #[test]
    fn keeps_unmapped_variant_string() {
        let mut renames = HashMap::new();
        renames.insert("old".to_string(), "new".to_string());
        let fix = VariantRenameFix::new(renames);
        let input = obj(&[("variant", Dynamic::String("other".to_string()))]);
        let out = fix.fix(input);
        assert_eq!(get_variant_str(&out), Some("other"));
    }

    #[test]
    fn non_string_variant_preserved() {
        let fix = VariantRenameFix::new(HashMap::new());
        let input = obj(&[("variant", Dynamic::Int(7))]);
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_variant_unchanged() {
        let fix = VariantRenameFix::new(HashMap::new());
        let input = obj(&[("name", Dynamic::String("x".to_string()))]);
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn empty_renames_preserves_string_variant() {
        let fix = VariantRenameFix::new(HashMap::new());
        let input = obj(&[("variant", Dynamic::String("same".to_string()))]);
        let out = fix.fix(input);
        assert_eq!(get_variant_str(&out), Some("same"));
    }
}
