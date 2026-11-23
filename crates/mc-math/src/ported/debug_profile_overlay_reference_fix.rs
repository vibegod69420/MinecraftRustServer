use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix transformations.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicPair(pub Dynamic, pub Dynamic);

impl DynamicPair {
    pub fn map_second(self, f: impl FnOnce(Dynamic) -> Dynamic) -> Self {
        DynamicPair(self.0, f(self.1))
    }
}

impl Dynamic {
    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn create_string(&self, s: &str) -> Self {
        Dynamic::String(s.to_string())
    }

    pub fn update(mut self, key: &str, f: impl FnOnce(Dynamic) -> Dynamic) -> Self {
        if let Dynamic::Map(ref mut m) = self {
            if let Some(v) = m.remove(key) {
                m.insert(key.to_string(), f(v));
            }
        }
        self
    }

    pub fn update_map_values(self, f: impl Fn(DynamicPair) -> DynamicPair) -> Self {
        match self {
            Dynamic::Map(m) => {
                let mut out = BTreeMap::new();
                for (k, v) in m {
                    let pair = f(DynamicPair(Dynamic::String(k.clone()), v));
                    let new_key = match pair.0 {
                        Dynamic::String(s) => s,
                        _ => k,
                    };
                    out.insert(new_key, pair.1);
                }
                Dynamic::Map(out)
            }
            other => other,
        }
    }
}

pub struct DebugProfileOverlayReferenceFix;

impl DebugProfileOverlayReferenceFix {
    // JAVA: net.minecraft.util.datafix.fixes.DebugProfileOverlayReferenceFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.DebugProfileOverlayReferenceFix#makeRule
    pub fn make_rule(&self, input: Dynamic) -> Dynamic {
        // fixTypeEverywhereTyped(..., remainderFinder().update("custom", ...))
        input.update("custom", |custom| {
            custom.update_map_values(|pair| {
                pair.map_second(|v| {
                    if v.as_string("") == "inF3" {
                        v.create_string("inOverlay")
                    } else {
                        v
                    }
                })
            })
        })
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
    fn replaces_inf3_with_inoverlay_in_custom() {
        let fix = DebugProfileOverlayReferenceFix::new(());
        let input = map(vec![(
            "custom",
            map(vec![
                ("a", Dynamic::String("inF3".to_string())),
                ("b", Dynamic::String("inOverlay".to_string())),
            ]),
        )]);
        let out = fix.make_rule(input);
        let expected = map(vec![(
            "custom",
            map(vec![
                ("a", Dynamic::String("inOverlay".to_string())),
                ("b", Dynamic::String("inOverlay".to_string())),
            ]),
        )]);
        assert_eq!(out, expected);
    }

    #[test]
    fn leaves_non_inf3_values_unchanged() {
        let fix = DebugProfileOverlayReferenceFix::new(());
        let input = map(vec![(
            "custom",
            map(vec![
                ("x", Dynamic::String("somethingElse".to_string())),
                ("y", Dynamic::Int(1)),
                ("z", Dynamic::Bool(true)),
            ]),
        )]);
        let out = fix.make_rule(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_custom_field_is_unchanged() {
        let fix = DebugProfileOverlayReferenceFix::new(());
        let input = map(vec![("other", Dynamic::Int(42))]);
        let out = fix.make_rule(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn empty_custom_map_is_unchanged() {
        let fix = DebugProfileOverlayReferenceFix::new(());
        let input = map(vec![("custom", Dynamic::Map(BTreeMap::new()))]);
        let out = fix.make_rule(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn preserves_other_top_level_fields() {
        let fix = DebugProfileOverlayReferenceFix::new(());
        let input = map(vec![
            ("name", Dynamic::String("debug".to_string())),
            (
                "custom",
                map(vec![("flag", Dynamic::String("inF3".to_string()))]),
            ),
            ("enabled", Dynamic::Bool(false)),
        ]);
        let out = fix.make_rule(input);
        let expected = map(vec![
            ("name", Dynamic::String("debug".to_string())),
            (
                "custom",
                map(vec![("flag", Dynamic::String("inOverlay".to_string()))]),
            ),
            ("enabled", Dynamic::Bool(false)),
        ]);
        assert_eq!(out, expected);
    }
}
