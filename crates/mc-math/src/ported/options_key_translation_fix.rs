use std::collections::BTreeMap;

/// Minimal dynamic value representation for datafix option-map transformations.
/// Self-contained stand-in for Mojang DFU `Dynamic<?>` used by this fix.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Map(BTreeMap<String, Dynamic>),
    Other,
}

impl Dynamic {
    fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_owned(),
        }
    }

    fn create_string(&self, value: &str) -> Dynamic {
        Dynamic::String(value.to_owned())
    }

    fn create_map(&self, entries: BTreeMap<String, Dynamic>) -> Dynamic {
        Dynamic::Map(entries)
    }
}

/// Port of `net.minecraft.util.datafix.fixes.OptionsKeyTranslationFix`.
///
/// In Java this extends DFU `DataFix` and rewrites typed OPTIONS data.
/// This self-contained port preserves the key/value translation behavior.
#[derive(Clone, Debug, Default)]
pub struct OptionsKeyTranslationFix {
    _changes_type: bool,
}

impl OptionsKeyTranslationFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsKeyTranslationFix#<init>
    pub fn new(_schema: (), changes_type: bool) -> Self {
        Self {
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsKeyTranslationFix#makeRule
    pub fn apply(&self, dynamic: Dynamic) -> Dynamic {
        Self::fix_options(dynamic)
    }

    fn fix_options(dynamic: Dynamic) -> Dynamic {
        let Some(map) = (match &dynamic {
            Dynamic::Map(m) => Some(m.clone()),
            _ => None,
        }) else {
            return dynamic;
        };

        let mut out = BTreeMap::new();
        for (key, value) in map {
            if key.starts_with("key_") {
                let value_str = value.as_string("");
                if !value_str.starts_with("key.mouse") && !value_str.starts_with("scancode.") {
                    // "key.".length() == 4
                    let suffix = if value_str.len() >= 4 {
                        &value_str[4..]
                    } else {
                        ""
                    };
                    let translated = dynamic.create_string(&format!("key.keyboard.{}", suffix));
                    out.insert(key, translated);
                    continue;
                }
            }
            out.insert(key, value);
        }

        dynamic.create_map(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> Dynamic {
        Dynamic::String(v.to_owned())
    }

    #[test]
    fn translates_keyboard_key_binding() {
        let mut map = BTreeMap::new();
        map.insert("key_forward".to_owned(), s("key.w"));
        let fix = OptionsKeyTranslationFix::new((), true);
        let out = fix.apply(Dynamic::Map(map));
        let Dynamic::Map(out_map) = out else {
            panic!("expected map");
        };
        assert_eq!(
            out_map.get("key_forward"),
            Some(&s("key.keyboard.w"))
        );
    }

    #[test]
    fn leaves_mouse_and_scancode_bindings_alone() {
        let mut map = BTreeMap::new();
        map.insert("key_attack".to_owned(), s("key.mouse.left"));
        map.insert("key_drop".to_owned(), s("scancode.16"));
        let fix = OptionsKeyTranslationFix::new((), false);
        let out = fix.apply(Dynamic::Map(map.clone()));
        assert_eq!(out, Dynamic::Map(map));
    }

    #[test]
    fn leaves_non_key_entries_alone() {
        let mut map = BTreeMap::new();
        map.insert("fov".to_owned(), s("90"));
        map.insert("renderDistance".to_owned(), s("12"));
        let fix = OptionsKeyTranslationFix::new((), true);
        let out = fix.apply(Dynamic::Map(map.clone()));
        assert_eq!(out, Dynamic::Map(map));
    }

    #[test]
    fn non_map_input_is_returned_unchanged() {
        let fix = OptionsKeyTranslationFix::new((), true);
        let input = Dynamic::String("not a map".to_owned());
        assert_eq!(fix.apply(input.clone()), input);
        let other = Dynamic::Other;
        assert_eq!(fix.apply(other.clone()), other);
    }

    #[test]
    fn handles_short_value_and_mixed_entries() {
        let mut map = BTreeMap::new();
        map.insert("key_a".to_owned(), s("key."));
        map.insert("key_b".to_owned(), s("key"));
        map.insert("key_c".to_owned(), s("key.keyboard.already"));
        map.insert("not_key".to_owned(), s("key.x"));
        let fix = OptionsKeyTranslationFix::new((), true);
        let Dynamic::Map(out) = fix.apply(Dynamic::Map(map)) else {
            panic!("expected map");
        };
        assert_eq!(out.get("key_a"), Some(&s("key.keyboard.")));
        assert_eq!(out.get("key_b"), Some(&s("key.keyboard.")));
        assert_eq!(out.get("key_c"), Some(&s("key.keyboard.keyboard.already")));
        assert_eq!(out.get("not_key"), Some(&s("key.x")));
    }
}
