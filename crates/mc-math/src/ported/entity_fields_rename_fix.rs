use std::collections::BTreeMap;

/// Minimal abstraction over Mojang DFU `Dynamic<?>` used by this fix.
pub trait DynamicLike: Sized {
    /// Renames a field in a map-like dynamic.
    fn rename_field(self, old_key: &str, new_key: &str) -> Self;
}

/// Minimal abstraction over Mojang DFU `Typed<?>` used by this fix.
pub trait TypedLike: Sized {
    type Dyn: DynamicLike;

    /// Applies a function to the remainder dynamic, corresponding to
    /// `Typed.update(DSL.remainderFinder(), function)`.
    fn update_remainder(self, f: impl FnOnce(Self::Dyn) -> Self::Dyn) -> Self;
}

/// A simple self-contained dynamic value for testing and local use.
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    Map(BTreeMap<String, DynamicValue>),
    String(String),
    Long(i64),
    Double(f64),
    Bool(bool),
    List(Vec<DynamicValue>),
}

impl DynamicLike for DynamicValue {
    fn rename_field(self, old_key: &str, new_key: &str) -> Self {
        match self {
            DynamicValue::Map(mut map) => {
                if let Some(value) = map.remove(old_key) {
                    map.insert(new_key.to_string(), value);
                }
                DynamicValue::Map(map)
            }
            other => other,
        }
    }
}

impl DynamicValue {
    pub fn map(entries: impl IntoIterator<Item = (impl Into<String>, DynamicValue)>) -> Self {
        DynamicValue::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect(),
        )
    }

    pub fn get_field(&self, key: &str) -> Option<&DynamicValue> {
        match self {
            DynamicValue::Map(map) => map.get(key),
            _ => None,
        }
    }
}

/// Port of `net.minecraft.util.datafix.fixes.EntityFieldsRenameFix`.
///
/// The original Java class extends `NamedEntityFix`; this self-contained port
/// preserves the field-renaming behavior. Renames are stored as an ordered list
/// to preserve the original iteration/application order.
#[derive(Debug, Clone)]
pub struct EntityFieldsRenameFix {
    pub renames: Vec<(String, String)>,
}

impl EntityFieldsRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityFieldsRenameFix#<init>
    pub fn new(renames: Vec<(String, String)>) -> Self {
        Self { renames }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityFieldsRenameFix#fixTag
    pub fn fix_tag<D: DynamicLike>(&self, mut dynamic: D) -> D {
        for (old_key, new_key) in &self.renames {
            dynamic = dynamic.rename_field(old_key, new_key);
        }
        dynamic
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityFieldsRenameFix#fix
    pub fn fix<T: TypedLike>(&self, typed: T) -> T {
        typed.update_remainder(|d| self.fix_tag(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestTyped {
        remainder: DynamicValue,
    }

    impl TypedLike for TestTyped {
        type Dyn = DynamicValue;

        fn update_remainder(self, f: impl FnOnce(Self::Dyn) -> Self::Dyn) -> Self {
            TestTyped {
                remainder: f(self.remainder),
            }
        }
    }

    fn str_val(s: &str) -> DynamicValue {
        DynamicValue::String(s.to_string())
    }

    #[test]
    fn fix_tag_renames_fields() {
        let fix = EntityFieldsRenameFix::new(vec![
            ("oldA".to_string(), "newA".to_string()),
            ("oldB".to_string(), "newB".to_string()),
        ]);
        let input = DynamicValue::map([
            ("oldA", DynamicValue::Long(1)),
            ("oldB", str_val("x")),
            ("keep", DynamicValue::Bool(true)),
        ]);

        let out = fix.fix_tag(input);
        assert_eq!(out.get_field("newA"), Some(&DynamicValue::Long(1)));
        assert_eq!(out.get_field("newB"), Some(&str_val("x")));
        assert_eq!(out.get_field("keep"), Some(&DynamicValue::Bool(true)));
        assert_eq!(out.get_field("oldA"), None);
        assert_eq!(out.get_field("oldB"), None);
    }

    #[test]
    fn fix_tag_missing_field_is_noop() {
        let fix = EntityFieldsRenameFix::new(vec![("missing".to_string(), "renamed".to_string())]);
        let input = DynamicValue::map([("present", DynamicValue::Long(42))]);
        let expected = input.clone();

        let out = fix.fix_tag(input);
        assert_eq!(out, expected);
    }

    #[test]
    fn fix_tag_applies_renames_in_order() {
        // Sequential application: a->b overwrites existing b, then b->c moves it.
        let fix = EntityFieldsRenameFix::new(vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ]);
        let input = DynamicValue::map([
            ("a", DynamicValue::Long(1)),
            ("b", DynamicValue::Long(2)),
        ]);

        let out = fix.fix_tag(input);
        assert_eq!(out.get_field("c"), Some(&DynamicValue::Long(1)));
        assert_eq!(out.get_field("a"), None);
        assert_eq!(out.get_field("b"), None);
    }

    #[test]
    fn fix_tag_empty_renames_returns_same_value() {
        let fix = EntityFieldsRenameFix::new(vec![]);
        let input = DynamicValue::map([("k", DynamicValue::Double(1.5))]);
        let expected = input.clone();

        let out = fix.fix_tag(input);
        assert_eq!(out, expected);
    }

    #[test]
    fn fix_tag_non_map_dynamic_unchanged() {
        let fix = EntityFieldsRenameFix::new(vec![("a".to_string(), "b".to_string())]);
        let input = DynamicValue::Long(7);
        let out = fix.fix_tag(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn fix_updates_typed_remainder() {
        let fix = EntityFieldsRenameFix::new(vec![("old".to_string(), "new".to_string())]);
        let typed = TestTyped {
            remainder: DynamicValue::map([("old", str_val("value"))]),
        };

        let out = fix.fix(typed);
        assert_eq!(out.remainder.get_field("new"), Some(&str_val("value")));
        assert_eq!(out.remainder.get_field("old"), None);
    }
}
