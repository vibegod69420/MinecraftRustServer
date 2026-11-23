use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Missing,
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn as_string(&self) -> Option<String> {
        match self {
            Dynamic::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    pub fn empty_map() -> Dynamic {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.insert(key.to_string(), value);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }
}

fn escape_lock_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out
}

pub struct LockComponentPredicateFix;

impl LockComponentPredicateFix {
    // JAVA: net.minecraft.util.datafix.fixes.LockComponentPredicateFix#LockComponentPredicateFix
    pub fn new(_output_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.LockComponentPredicateFix#fixComponent
    pub fn fix_component(dynamic: &Dynamic) -> Option<Dynamic> {
        Self::fix_lock(dynamic)
    }

    // JAVA: net.minecraft.util.datafix.fixes.LockComponentPredicateFix#fixLock
    pub fn fix_lock(dynamic: &Dynamic) -> Option<Dynamic> {
        let lock_str = dynamic.as_string()?;
        if lock_str.is_empty() {
            return None;
        }
        let quoted = dynamic.create_string(&format!("\"{}\"", escape_lock_string(&lock_str)));
        let components_inner = Dynamic::empty_map().set("minecraft:custom_name", quoted);
        Some(Dynamic::empty_map().set("components", components_inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixes_normal_lock_string() {
        let input = Dynamic::String("MyLock".to_string());
        let out = LockComponentPredicateFix::fix_lock(&input).expect("should fix");
        let name = out.get("components").get("minecraft:custom_name");
        assert_eq!(name.as_string(), Some("\"MyLock\"".to_string()));
    }

    #[test]
    fn empty_string_returns_none() {
        let input = Dynamic::String(String::new());
        assert_eq!(LockComponentPredicateFix::fix_lock(&input), None);
    }

    #[test]
    fn non_string_returns_none() {
        let input = Dynamic::Missing;
        assert_eq!(LockComponentPredicateFix::fix_lock(&input), None);
    }

    #[test]
    fn escapes_quotes_and_backslashes() {
        let input = Dynamic::String("a\"b\\c".to_string());
        let out = LockComponentPredicateFix::fix_lock(&input).expect("should fix");
        let name = out.get("components").get("minecraft:custom_name");
        assert_eq!(name.as_string(), Some("\"a\\\"b\\\\c\"".to_string()));
    }

    #[test]
    fn fix_component_delegates_to_fix_lock() {
        let input = Dynamic::String("key".to_string());
        assert_eq!(
            LockComponentPredicateFix::fix_component(&input),
            LockComponentPredicateFix::fix_lock(&input)
        );
    }
}
