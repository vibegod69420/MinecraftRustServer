use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix logic.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Missing,
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn str(s: &str) -> Self {
        Dynamic::String(s.to_string())
    }

    pub fn object(entries: &[(&str, Dynamic)]) -> Self {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        Dynamic::Map(m)
    }

    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Dynamic::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_int_or(&self, default: i64) -> i64 {
        match self {
            Dynamic::Int(v) => *v,
            _ => default,
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                let old = m.remove(key).unwrap_or(Dynamic::Missing);
                let new_val = f(old);
                if !matches!(new_val, Dynamic::Missing) {
                    m.insert(key.to_string(), new_val);
                }
                Dynamic::Map(m)
            }
            other => other,
        }
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
pub fn ensure_namespaced(s: &str) -> String {
    if s.contains(':') {
        s.to_string()
    } else {
        format!("minecraft:{s}")
    }
}

pub struct RemapChunkStatusFix {
    #[allow(dead_code)]
    name: String,
    mapper: Box<dyn Fn(&str) -> String>,
}

impl RemapChunkStatusFix {
    // JAVA: net.minecraft.util.datafix.fixes.RemapChunkStatusFix#RemapChunkStatusFix
    pub fn new(_schema: (), name: &str, mapper: Box<dyn Fn(&str) -> String>) -> Self {
        Self {
            name: name.to_string(),
            mapper,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.RemapChunkStatusFix#makeRule
    pub fn apply(&self, chunk: Dynamic) -> Dynamic {
        chunk.update("Status", |d| self.fix_status(d)).update(
            "below_zero_retrogen",
            |d| d.update("target_status", |d2| self.fix_status(d2)),
        )
    }

    // JAVA: net.minecraft.util.datafix.fixes.RemapChunkStatusFix#fixStatus
    fn fix_status(&self, d: Dynamic) -> Dynamic {
        match d.as_string() {
            Some(s) => {
                let namespaced = ensure_namespaced(&s);
                let mapped = (self.mapper)(&namespaced);
                d.create_string(&mapped)
            }
            None => d,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fix() -> RemapChunkStatusFix {
        RemapChunkStatusFix::new(
            (),
            "test",
            Box::new(|s: &str| {
                if s == "minecraft:old" {
                    "minecraft:new".to_string()
                } else {
                    s.to_string()
                }
            }),
        )
    }

    #[test]
    fn remaps_status() {
        let fix = make_fix();
        let chunk = Dynamic::object(&[("Status", Dynamic::str("old"))]);
        let out = fix.apply(chunk);
        assert_eq!(
            out.get("Status").as_string().as_deref(),
            Some("minecraft:new")
        );
    }

    #[test]
    fn remaps_below_zero_retrogen_target_status() {
        let fix = make_fix();
        let chunk = Dynamic::object(&[
            ("Status", Dynamic::str("minecraft:old")),
            (
                "below_zero_retrogen",
                Dynamic::object(&[("target_status", Dynamic::str("old"))]),
            ),
        ]);
        let out = fix.apply(chunk);
        assert_eq!(
            out.get("below_zero_retrogen")
                .get("target_status")
                .as_string()
                .as_deref(),
            Some("minecraft:new")
        );
    }

    #[test]
    fn missing_status_unchanged() {
        let fix = make_fix();
        let chunk = Dynamic::object(&[("xPos", Dynamic::Int(1))]);
        let out = fix.apply(chunk.clone());
        assert_eq!(out.get("Status"), Dynamic::Missing);
        assert_eq!(out.get("xPos").as_int_or(0), 1);
    }

    #[test]
    fn non_string_status_unchanged() {
        let fix = make_fix();
        let chunk = Dynamic::object(&[("Status", Dynamic::Int(42))]);
        let out = fix.apply(chunk.clone());
        assert_eq!(out.get("Status"), Dynamic::Int(42));
    }

    #[test]
    fn below_zero_retrogen_without_target_status_unchanged() {
        let fix = make_fix();
        let chunk = Dynamic::object(&[(
            "below_zero_retrogen",
            Dynamic::object(&[("other", Dynamic::Int(1))]),
        )]);
        let out = fix.apply(chunk);
        let bz = out.get("below_zero_retrogen");
        assert_eq!(bz.get("other").as_int_or(0), 1);
        assert_eq!(bz.get("target_status"), Dynamic::Missing);
    }

    #[test]
    fn ensure_namespaced_adds_default_namespace() {
        assert_eq!(ensure_namespaced("full"), "minecraft:full");
        assert_eq!(ensure_namespaced("minecraft:full"), "minecraft:full");
    }
}
