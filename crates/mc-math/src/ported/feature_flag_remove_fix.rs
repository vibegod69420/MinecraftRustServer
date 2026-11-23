use std::collections::HashSet;

/// Minimal self-contained dynamic value representation for datafix porting.
/// Models the subset of DFU `Dynamic<T>` operations used by feature flag fixes.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    List(Vec<Dynamic>),
    Map(Vec<(String, Dynamic)>),
    Empty,
}

impl Dynamic {
    /// Creates an empty/missing dynamic.
    pub fn empty() -> Self {
        Dynamic::Empty
    }

    /// Creates a string dynamic. Mirrors `Dynamic#createString`.
    pub fn create_string(&self, value: String) -> Self {
        Dynamic::String(value)
    }

    /// Creates a list dynamic. Mirrors `Dynamic#createList`.
    pub fn create_list(&self, values: Vec<Dynamic>) -> Self {
        Dynamic::List(values)
    }

    /// Gets a field by key. Returns `Dynamic::Empty` if missing or not a map.
    /// Mirrors `Dynamic#get`.
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(entries) => {
                for (k, v) in entries {
                    if k == key {
                        return v.clone();
                    }
                }
                Dynamic::Empty
            }
            _ => Dynamic::Empty,
        }
    }

    /// Returns the list elements, or an empty vec if not a list.
    /// Mirrors `Dynamic#asStream()` defaulting to empty on error.
    pub fn as_stream(&self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(items) => items.clone(),
            _ => Vec::new(),
        }
    }

    /// Returns the list elements if this is a list, otherwise `None`.
    /// Mirrors `Dynamic#asStreamOpt().result()`.
    pub fn as_stream_opt(&self) -> Option<Vec<Dynamic>> {
        match self {
            Dynamic::List(items) => Some(items.clone()),
            _ => None,
        }
    }

    /// Returns the string value if this is a string, otherwise `None`.
    /// Mirrors `Dynamic#asString().result()`.
    pub fn as_string(&self) -> Option<String> {
        match self {
            Dynamic::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Updates a field if present in a map. If the field is missing, returns self unchanged.
    /// Mirrors `Dynamic#update` (present-field semantics).
    pub fn update<F>(&self, key: &str, function: F) -> Dynamic
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(entries) => {
                let mut found = false;
                for (k, _) in entries {
                    if k == key {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return self.clone();
                }
                let current = self.get(key);
                let updated = function(current);
                self.set(key, updated)
            }
            _ => self.clone(),
        }
    }

    /// Sets a field in a map, adding or replacing it. Returns self unchanged if not a map.
    /// Mirrors `Dynamic#set`.
    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(entries) => {
                let mut new_entries: Vec<(String, Dynamic)> = Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((k.clone(), value.clone()));
                        replaced = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !replaced {
                    new_entries.push((key.to_string(), value));
                }
                Dynamic::Map(new_entries)
            }
            _ => self.clone(),
        }
    }
}

pub struct FeatureFlagRemoveFix {
    pub name: String,
    pub flags_to_remove: HashSet<String>,
}

impl FeatureFlagRemoveFix {
    // JAVA: net.minecraft.util.datafix.fixes.FeatureFlagRemoveFix#<init>
    pub fn new(name: String, flags_to_remove: HashSet<String>) -> Self {
        Self {
            name,
            flags_to_remove,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.FeatureFlagRemoveFix#makeRule
    pub fn make_rule(&self, dynamic: Dynamic) -> Dynamic {
        self.fix_tag(&dynamic)
    }

    // JAVA: net.minecraft.util.datafix.fixes.FeatureFlagRemoveFix#fixTag
    pub fn fix_tag(&self, dynamic: &Dynamic) -> Dynamic {
        let mut removed_features: Vec<Dynamic> = dynamic.get("removed_features").as_stream();

        let updated = dynamic.update("enabled_features", |enabled| {
            match enabled.as_stream_opt() {
                Some(stream) => {
                    let mut filtered: Vec<Dynamic> = Vec::with_capacity(stream.len());
                    for elem in stream {
                        match elem.as_string() {
                            None => {
                                filtered.push(elem);
                            }
                            Some(s) => {
                                let should_remove = self.flags_to_remove.contains(&s);
                                if should_remove {
                                    removed_features.push(dynamic.create_string(s));
                                } else {
                                    filtered.push(elem);
                                }
                            }
                        }
                    }
                    dynamic.create_list(filtered)
                }
                None => enabled,
            }
        });

        if !removed_features.is_empty() {
            updated.set("removed_features", dynamic.create_list(removed_features))
        } else {
            updated
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_dyn(s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    fn list_dyn(items: Vec<Dynamic>) -> Dynamic {
        Dynamic::List(items)
    }

    fn map_dyn(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn fix_with_flags(flags: &[&str]) -> FeatureFlagRemoveFix {
        FeatureFlagRemoveFix::new(
            "test_fix".to_string(),
            flags.iter().map(|s| s.to_string()).collect(),
        )
    }

    #[test]
    fn removes_flag_from_enabled_and_adds_to_removed() {
        let fix = fix_with_flags(&["minecraft:experiment_a"]);
        let input = map_dyn(vec![(
            "enabled_features",
            list_dyn(vec![
                str_dyn("minecraft:vanilla"),
                str_dyn("minecraft:experiment_a"),
            ]),
        )]);

        let result = fix.fix_tag(&input);

        let enabled = result.get("enabled_features").as_stream();
        assert_eq!(enabled, vec![str_dyn("minecraft:vanilla")]);

        let removed = result.get("removed_features").as_stream();
        assert_eq!(removed, vec![str_dyn("minecraft:experiment_a")]);
    }

    #[test]
    fn preserves_existing_removed_features() {
        let fix = fix_with_flags(&["minecraft:exp_b"]);
        let input = map_dyn(vec![
            (
                "enabled_features",
                list_dyn(vec![
                    str_dyn("minecraft:exp_b"),
                    str_dyn("minecraft:vanilla"),
                ]),
            ),
            (
                "removed_features",
                list_dyn(vec![str_dyn("minecraft:old_removed")]),
            ),
        ]);

        let result = fix.fix_tag(&input);

        let enabled = result.get("enabled_features").as_stream();
        assert_eq!(enabled, vec![str_dyn("minecraft:vanilla")]);

        let removed = result.get("removed_features").as_stream();
        assert_eq!(
            removed,
            vec![
                str_dyn("minecraft:old_removed"),
                str_dyn("minecraft:exp_b"),
            ]
        );
    }

    #[test]
    fn no_change_when_flag_not_present() {
        let fix = fix_with_flags(&["minecraft:missing"]);
        let input = map_dyn(vec![(
            "enabled_features",
            list_dyn(vec![str_dyn("minecraft:vanilla")]),
        )]);

        let result = fix.fix_tag(&input);

        let enabled = result.get("enabled_features").as_stream();
        assert_eq!(enabled, vec![str_dyn("minecraft:vanilla")]);
        assert_eq!(result.get("removed_features"), Dynamic::Empty);
    }

    #[test]
    fn keeps_non_string_entries_in_enabled_features() {
        let fix = fix_with_flags(&["minecraft:exp"]);
        let non_string = Dynamic::Map(vec![("k".to_string(), str_dyn("v"))]);
        let input = map_dyn(vec![(
            "enabled_features",
            list_dyn(vec![non_string.clone(), str_dyn("minecraft:exp")]),
        )]);

        let result = fix.fix_tag(&input);

        let enabled = result.get("enabled_features").as_stream();
        assert_eq!(enabled, vec![non_string]);
        let removed = result.get("removed_features").as_stream();
        assert_eq!(removed, vec![str_dyn("minecraft:exp")]);
    }

    #[test]
    fn missing_enabled_features_leaves_dynamic_unchanged() {
        let fix = fix_with_flags(&["minecraft:exp"]);
        let input = map_dyn(vec![("other", str_dyn("value"))]);

        let result = fix.fix_tag(&input);
        assert_eq!(result, input);
    }

    #[test]
    fn make_rule_applies_fix_tag() {
        let fix = fix_with_flags(&["minecraft:exp"]);
        let input = map_dyn(vec![(
            "enabled_features",
            list_dyn(vec![str_dyn("minecraft:exp"), str_dyn("minecraft:vanilla")]),
        )]);

        let via_rule = fix.make_rule(input.clone());
        let via_tag = fix.fix_tag(&input);
        assert_eq!(via_rule, via_tag);
    }
}
