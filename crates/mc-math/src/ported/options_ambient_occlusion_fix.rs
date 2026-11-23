use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix ports.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Null),
            _ => Dynamic::Null,
        }
    }

    pub fn as_string_opt(&self) -> Option<String> {
        match self {
            Dynamic::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }
}

pub struct OptionsAmbientOcclusionFix;

impl OptionsAmbientOcclusionFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsAmbientOcclusionFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsAmbientOcclusionFix#makeRule
    pub fn fix_dynamic(&self, d: Dynamic) -> Dynamic {
        match d.get("ao").as_string_opt() {
            Some(s) => {
                let updated = Self::update_value(&s);
                let new_value = d.create_string(&updated);
                d.set("ao", new_value)
            }
            None => d,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsAmbientOcclusionFix#updateValue
    fn update_value(value: &str) -> String {
        match value {
            "0" => "false".to_string(),
            "1" | "2" => "true".to_string(),
            _ => value.to_string(),
        }
    }
}

impl Default for OptionsAmbientOcclusionFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn maps_zero_to_false() {
        let input = map(vec![("ao", Dynamic::String("0".into()))]);
        let out = OptionsAmbientOcclusionFix::new().fix_dynamic(input);
        assert_eq!(out.get("ao"), Dynamic::String("false".into()));
    }

    #[test]
    fn maps_one_and_two_to_true() {
        let fix = OptionsAmbientOcclusionFix::new();
        let out1 = fix.fix_dynamic(map(vec![("ao", Dynamic::String("1".into()))]));
        let out2 = fix.fix_dynamic(map(vec![("ao", Dynamic::String("2".into()))]));
        assert_eq!(out1.get("ao"), Dynamic::String("true".into()));
        assert_eq!(out2.get("ao"), Dynamic::String("true".into()));
    }

    #[test]
    fn preserves_unknown_string_value() {
        let input = map(vec![("ao", Dynamic::String("max".into()))]);
        let out = OptionsAmbientOcclusionFix::new().fix_dynamic(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_ao_returns_unchanged() {
        let input = map(vec![("fov", Dynamic::String("90".into()))]);
        let out = OptionsAmbientOcclusionFix::new().fix_dynamic(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_string_ao_returns_unchanged() {
        let input = map(vec![("ao", Dynamic::Int(1))]);
        let out = OptionsAmbientOcclusionFix::new().fix_dynamic(input.clone());
        assert_eq!(out, input);
    }
}
