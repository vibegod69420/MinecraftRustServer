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
    pub fn map(entries: impl IntoIterator<Item = (String, Dynamic)>) -> Self {
        Dynamic::Map(entries.into_iter().collect())
    }

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

/// Minimal self-contained typed wrapper for datafix ports.
#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

/// Stand-in for `ItemStackComponentizationFix.fixProfile`.
/// Kept self-contained; the real Java method converts legacy skull owner formats.
fn item_stack_componentization_fix_profile(d: Dynamic) -> Dynamic {
    d
}

pub struct PlayerHeadBlockProfileFix;

impl PlayerHeadBlockProfileFix {
    // JAVA: net.minecraft.util.datafix.fixes.PlayerHeadBlockProfileFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.PlayerHeadBlockProfileFix#fix
    pub fn fix_typed(&self, typed: Typed) -> Typed {
        Typed {
            value: Self::fix_dynamic(typed.value),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.PlayerHeadBlockProfileFix#fix
    fn fix_dynamic(d: Dynamic) -> Dynamic {
        let skull_owner = d.get("SkullOwner").result();
        let extra_type = d.get("ExtraType").result();
        let profile_source = skull_owner.or(extra_type);
        let Some(profile_source) = profile_source else {
            return d;
        };

        let d = d.remove("SkullOwner").remove("ExtraType");
        d.set(
            "profile",
            item_stack_componentization_fix_profile(profile_source),
        )
    }
}

impl Default for PlayerHeadBlockProfileFix {
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
    fn moves_skull_owner_to_profile() {
        let input = map(vec![
            ("SkullOwner", Dynamic::String("Alice".into())),
            ("id", Dynamic::String("minecraft:skull".into())),
        ]);
        let out = PlayerHeadBlockProfileFix::new().fix_typed(Typed { value: input });
        assert_eq!(out.value.get("SkullOwner").result(), None);
        assert_eq!(out.value.get("ExtraType").result(), None);
        assert_eq!(
            out.value.get("profile").result(),
            Some(Dynamic::String("Alice".into()))
        );
        assert_eq!(
            out.value.get("id").result(),
            Some(Dynamic::String("minecraft:skull".into()))
        );
    }

    #[test]
    fn falls_back_to_extra_type() {
        let input = map(vec![("ExtraType", Dynamic::String("Bob".into()))]);
        let out = PlayerHeadBlockProfileFix::new().fix_typed(Typed { value: input });
        assert_eq!(
            out.value.get("profile").result(),
            Some(Dynamic::String("Bob".into()))
        );
        assert_eq!(out.value.get("ExtraType").result(), None);
    }

    #[test]
    fn skull_owner_takes_precedence_over_extra_type() {
        let input = map(vec![
            ("SkullOwner", Dynamic::String("Alice".into())),
            ("ExtraType", Dynamic::String("Bob".into())),
        ]);
        let out = PlayerHeadBlockProfileFix::new().fix_typed(Typed { value: input });
        assert_eq!(
            out.value.get("profile").result(),
            Some(Dynamic::String("Alice".into()))
        );
        assert_eq!(out.value.get("SkullOwner").result(), None);
        assert_eq!(out.value.get("ExtraType").result(), None);
    }

    #[test]
    fn no_profile_source_returns_unchanged() {
        let input = map(vec![("id", Dynamic::String("minecraft:skull".into()))]);
        let out = PlayerHeadBlockProfileFix::new().fix_typed(Typed {
            value: input.clone(),
        });
        assert_eq!(out.value, input);
    }

    #[test]
    fn handles_non_map_dynamic() {
        let input = Dynamic::String("not-a-map".into());
        let out = PlayerHeadBlockProfileFix::new().fix_typed(Typed {
            value: input.clone(),
        });
        assert_eq!(out.value, input);
    }
}
