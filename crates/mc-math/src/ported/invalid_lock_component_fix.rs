use std::collections::BTreeMap;

// JAVA: net.minecraft.util.datafix.fixes.InvalidLockComponentFix#INVALID_LOCK_CUSTOM_NAME
const INVALID_LOCK_CUSTOM_NAME: Option<&str> = Some("\"\"");

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Int(i32),
    Long(i64),
    Byte(i8),
    Short(i16),
    Float(f32),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    fn as_string_result(&self) -> Option<&str> {
        match self {
            Dynamic::String(s) => Some(s),
            _ => None,
        }
    }
}

pub struct InvalidLockComponentFix;

impl InvalidLockComponentFix {
    // JAVA: net.minecraft.util.datafix.fixes.InvalidLockComponentFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.InvalidLockComponentFix#fixComponent
    pub fn fix_component(&self, value: Dynamic) -> Option<Dynamic> {
        Self::fix_lock(value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.InvalidLockComponentFix#fixLock
    pub fn fix_lock(value: Dynamic) -> Option<Dynamic> {
        if is_broken_lock(&value) {
            None
        } else {
            Some(value)
        }
    }
}

impl Default for InvalidLockComponentFix {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.util.datafix.fixes.InvalidLockComponentFix#isBrokenLock
fn is_broken_lock(value: &Dynamic) -> bool {
    is_map_with_one_field(value, "components", |components| {
        is_map_with_one_field(components, "minecraft:custom_name", |name| {
            name.as_string_result() == INVALID_LOCK_CUSTOM_NAME
        })
    })
}

// JAVA: net.minecraft.util.datafix.fixes.InvalidLockComponentFix#isMapWithOneField
fn is_map_with_one_field<F: FnOnce(&Dynamic) -> bool>(
    value: &Dynamic,
    key: &str,
    predicate: F,
) -> bool {
    match value {
        Dynamic::Map(m) if m.len() == 1 => m.get(key).is_some_and(predicate),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn broken_lock_returns_none() {
        let lock = map(vec![(
            "components",
            map(vec![(
                "minecraft:custom_name",
                Dynamic::String("\"\"".to_string()),
            )]),
        )]);
        assert_eq!(InvalidLockComponentFix::fix_lock(lock), None);
    }

    #[test]
    fn valid_lock_with_different_name_preserved() {
        let lock = map(vec![(
            "components",
            map(vec![(
                "minecraft:custom_name",
                Dynamic::String("\"Key\"".to_string()),
            )]),
        )]);
        let out = InvalidLockComponentFix::new().fix_component(lock.clone());
        assert_eq!(out, Some(lock));
    }

    #[test]
    fn extra_component_field_not_broken() {
        let lock = map(vec![(
            "components",
            map(vec![
                ("minecraft:custom_name", Dynamic::String("\"\"".to_string())),
                ("minecraft:lore", Dynamic::List(vec![])),
            ]),
        )]);
        let out = InvalidLockComponentFix::fix_lock(lock.clone());
        assert_eq!(out, Some(lock));
    }

    #[test]
    fn extra_top_level_field_not_broken() {
        let lock = map(vec![
            (
                "components",
                map(vec![(
                    "minecraft:custom_name",
                    Dynamic::String("\"\"".to_string()),
                )]),
            ),
            ("count", Dynamic::Int(1)),
        ]);
        let out = InvalidLockComponentFix::fix_lock(lock.clone());
        assert_eq!(out, Some(lock));
    }

    #[test]
    fn non_string_custom_name_not_broken() {
        let lock = map(vec![(
            "components",
            map(vec![("minecraft:custom_name", Dynamic::Int(0))]),
        )]);
        let out = InvalidLockComponentFix::fix_lock(lock.clone());
        assert_eq!(out, Some(lock));
    }
}
