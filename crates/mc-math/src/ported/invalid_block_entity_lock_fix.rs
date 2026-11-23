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

    pub fn result(&self) -> Option<Dynamic> {
        if matches!(self, Dynamic::Null) {
            None
        } else {
            Some(self.clone())
        }
    }

    pub fn remove(self, key: &str) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
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
}

/// Stand-in for `InvalidLockComponentFix.fixLock`.
/// Returns `Some(d)` by default to keep this module self-contained.
fn invalid_lock_component_fix_lock(d: Dynamic) -> Option<Dynamic> {
    Some(d)
}

pub struct InvalidBlockEntityLockFix;

impl InvalidBlockEntityLockFix {
    // JAVA: net.minecraft.util.datafix.fixes.InvalidBlockEntityLockFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.InvalidBlockEntityLockFix#makeRule
    pub fn fix_dynamic(&self, d: Dynamic) -> Dynamic {
        fix_dynamic_with(d, invalid_lock_component_fix_lock)
    }
}

impl Default for InvalidBlockEntityLockFix {
    fn default() -> Self {
        Self::new()
    }
}

fn fix_dynamic_with<F: FnOnce(Dynamic) -> Option<Dynamic>>(d: Dynamic, fix_lock: F) -> Dynamic {
    let Some(lock) = d.get("lock").result() else {
        return d;
    };

    match fix_lock(lock) {
        Some(fixed) => d.set("lock", fixed),
        None => d.remove("lock"),
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
    fn no_lock_returns_unchanged() {
        let input = map(vec![("id", Dynamic::String("minecraft:chest".into()))]);
        let out = InvalidBlockEntityLockFix::new().fix_dynamic(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn valid_lock_is_preserved_or_replaced() {
        let input = map(vec![("lock", Dynamic::String("key".into()))]);
        let out = InvalidBlockEntityLockFix::new().fix_dynamic(input);
        assert_eq!(
            out.get("lock").result(),
            Some(Dynamic::String("key".into()))
        );
    }

    #[test]
    fn invalid_lock_is_removed_when_fix_returns_none() {
        let input = map(vec![
            ("lock", Dynamic::String("bad".into())),
            ("id", Dynamic::String("minecraft:chest".into())),
        ]);
        let out = fix_dynamic_with(input, |_| None);
        assert_eq!(out.get("lock").result(), None);
        assert_eq!(
            out.get("id").result(),
            Some(Dynamic::String("minecraft:chest".into()))
        );
    }

    #[test]
    fn invalid_lock_is_replaced_when_fix_returns_some() {
        let input = map(vec![("lock", Dynamic::String("old".into()))]);
        let out = fix_dynamic_with(input, |_| Some(Dynamic::String("new".into())));
        assert_eq!(
            out.get("lock").result(),
            Some(Dynamic::String("new".into()))
        );
    }

    #[test]
    fn handles_non_map_dynamic() {
        let input = Dynamic::Int(-1);
        let out = InvalidBlockEntityLockFix::new().fix_dynamic(input.clone());
        assert_eq!(out, input);
    }
}
