/// Minimal dynamic value representation for igloo metadata removal datafix.
/// Self-contained stand-in for Mojang DFU `Dynamic<?>` used by this fix.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    List(Vec<Dynamic>),
    Map(Vec<(String, Dynamic)>),
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

    fn create_list(&self, values: Vec<Dynamic>) -> Dynamic {
        Dynamic::List(values)
    }

    fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(entries) => entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or(Dynamic::Other),
            _ => Dynamic::Other,
        }
    }

    fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut entries) => {
                if let Some((_, existing)) = entries.iter_mut().find(|(k, _)| k == key) {
                    *existing = value;
                } else {
                    entries.push((key.to_owned(), value));
                }
                Dynamic::Map(entries)
            }
            other => other,
        }
    }

    fn remove(self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(mut entries) => {
                entries.retain(|(k, _)| k != key);
                Dynamic::Map(entries)
            }
            other => other,
        }
    }

    fn update(self, key: &str, f: impl FnOnce(Dynamic) -> Dynamic) -> Dynamic {
        let current = self.get(key);
        let updated = f(current);
        self.set(key, updated)
    }

    fn as_stream_opt(&self) -> Option<Vec<Dynamic>> {
        match self {
            Dynamic::List(values) => Some(values.clone()),
            _ => None,
        }
    }
}

/// Port of `net.minecraft.util.datafix.fixes.IglooMetadataRemovalFix`.
#[derive(Clone, Debug, Default)]
pub struct IglooMetadataRemovalFix {
    _changes_type: bool,
}

impl IglooMetadataRemovalFix {
    // JAVA: net.minecraft.util.datafix.fixes.IglooMetadataRemovalFix#<init>
    pub fn new(_schema: (), changes_type: bool) -> Self {
        Self {
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.IglooMetadataRemovalFix#makeRule
    pub fn apply(&self, dynamic: Dynamic) -> Dynamic {
        Self::fix_tag(dynamic)
    }

    // JAVA: net.minecraft.util.datafix.fixes.IglooMetadataRemovalFix#fixTag
    fn fix_tag(dynamic: Dynamic) -> Dynamic {
        let is_igloo = dynamic
            .get("Children")
            .as_stream_opt()
            .map(|children| children.iter().all(Self::is_igloo_piece))
            .unwrap_or(false);

        if is_igloo {
            let id = dynamic.create_string("Igloo");
            dynamic.set("id", id).remove("Children")
        } else {
            dynamic.update("Children", Self::remove_igloo_pieces)
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.IglooMetadataRemovalFix#removeIglooPieces
    fn remove_igloo_pieces(dynamic: Dynamic) -> Dynamic {
        match dynamic.as_stream_opt() {
            Some(children) => {
                let filtered: Vec<Dynamic> = children
                    .into_iter()
                    .filter(|child| !Self::is_igloo_piece(child))
                    .collect();
                dynamic.create_list(filtered)
            }
            None => dynamic,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.IglooMetadataRemovalFix#isIglooPiece
    fn is_igloo_piece(dynamic: &Dynamic) -> bool {
        dynamic.get("id").as_string("") == "Iglu"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(entries.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }

    fn igloo_piece() -> Dynamic {
        map(vec![("id", Dynamic::String("Iglu".to_owned()))])
    }

    fn other_piece(id: &str) -> Dynamic {
        map(vec![("id", Dynamic::String(id.to_owned()))])
    }

    #[test]
    fn pure_igloo_children_collapses_to_igloo_id() {
        let structure = map(vec![
            ("id", Dynamic::String("Something".to_owned())),
            ("Children", Dynamic::List(vec![igloo_piece(), igloo_piece()])),
        ]);
        let fixed = IglooMetadataRemovalFix::new((), true).apply(structure);
        assert_eq!(fixed.get("id"), Dynamic::String("Igloo".to_owned()));
        assert_eq!(fixed.get("Children"), Dynamic::Other);
    }

    #[test]
    fn mixed_children_only_removes_igloo_pieces() {
        let structure = map(vec![
            ("id", Dynamic::String("Village".to_owned())),
            (
                "Children",
                Dynamic::List(vec![igloo_piece(), other_piece("House"), igloo_piece()]),
            ),
        ]);
        let fixed = IglooMetadataRemovalFix::new((), false).apply(structure);
        assert_eq!(fixed.get("id"), Dynamic::String("Village".to_owned()));
        let children = fixed.get("Children").as_stream_opt().expect("children list");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].get("id"), Dynamic::String("House".to_owned()));
    }

    #[test]
    fn no_children_or_non_list_children_defaults_to_remove_path() {
        let no_children = map(vec![("id", Dynamic::String("X".to_owned()))]);
        let fixed = IglooMetadataRemovalFix::new((), true).apply(no_children.clone());
        // No Children -> is_igloo false (unwrap_or(false)), update Children with remove_igloo_pieces on Other returns Other
        assert_eq!(fixed.get("id"), Dynamic::String("X".to_owned()));
        assert_eq!(fixed.get("Children"), Dynamic::Other);

        let bad_children = map(vec![
            ("id", Dynamic::String("Y".to_owned())),
            ("Children", Dynamic::String("oops".to_owned())),
        ]);
        let fixed2 = IglooMetadataRemovalFix::new((), true).apply(bad_children);
        assert_eq!(fixed2.get("Children"), Dynamic::String("oops".to_owned()));
    }

    #[test]
    fn is_igloo_piece_checks_exact_id() {
        assert!(IglooMetadataRemovalFix::is_igloo_piece(&igloo_piece()));
        assert!(!IglooMetadataRemovalFix::is_igloo_piece(&other_piece("Igloo")));
        assert!(!IglooMetadataRemovalFix::is_igloo_piece(&other_piece("Iglu2")));
        assert!(!IglooMetadataRemovalFix::is_igloo_piece(&Dynamic::Other));
    }

    #[test]
    fn empty_children_list_collapses_to_igloo() {
        // Java: allMatch on empty stream returns true
        let structure = map(vec![
            ("id", Dynamic::String("Old".to_owned())),
            ("Children", Dynamic::List(vec![])),
        ]);
        let fixed = IglooMetadataRemovalFix::new((), true).apply(structure);
        assert_eq!(fixed.get("id"), Dynamic::String("Igloo".to_owned()));
        assert_eq!(fixed.get("Children"), Dynamic::Other);
    }
}
