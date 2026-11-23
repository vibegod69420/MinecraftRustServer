/// Minimal self-contained dynamic value for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Map(Vec<(String, Dynamic)>),
    Other,
}

impl Dynamic {
    fn as_string_result(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    fn create_string(&self, s: String) -> Self {
        Self::String(s)
    }

    fn update<F: FnOnce(Self) -> Self>(self, key: &str, f: F) -> Self {
        match self {
            Self::Map(mut entries) => {
                if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key) {
                    let old = std::mem::replace(v, Self::Other);
                    *v = f(old);
                }
                Self::Map(entries)
            }
            other => other,
        }
    }
}

/// Minimal stub for DFU TypeRewriteRule.
pub struct TypeRewriteRule {
    apply_fn: Box<dyn Fn(Dynamic) -> Dynamic>,
}

impl TypeRewriteRule {
    pub fn apply(&self, input: Dynamic) -> Dynamic {
        (self.apply_fn)(input)
    }
}

/// Minimal stub for DFU Schema.
pub struct Schema;

pub struct OptionsProgrammerArtFix;

impl OptionsProgrammerArtFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsProgrammerArtFix#<init>
    pub fn new(_schema: Schema) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsProgrammerArtFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            apply_fn: Box::new(|typed| {
                typed
                    .update("resourcePacks", Self::fix_list)
                    .update("incompatibleResourcePacks", Self::fix_list)
            }),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsProgrammerArtFix#fixList
    fn fix_list(value: Dynamic) -> Dynamic {
        if let Some(s) = value.as_string_result() {
            let replaced = s.replace("\"programer_art\"", "\"programmer_art\"");
            value.create_string(replaced)
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn fixes_programer_art_typo() {
        let input = string("[\"programer_art\"]");
        let out = OptionsProgrammerArtFix::fix_list(input);
        assert_eq!(out, string("[\"programmer_art\"]"));
    }

    #[test]
    fn leaves_correct_or_unrelated_string() {
        let input = string("[\"programmer_art\",\"vanilla\"]");
        let out = OptionsProgrammerArtFix::fix_list(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_string_passthrough() {
        let input = Dynamic::Other;
        let out = OptionsProgrammerArtFix::fix_list(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn make_rule_updates_both_pack_fields() {
        let fix = OptionsProgrammerArtFix::new(Schema);
        let input = map(vec![
            ("resourcePacks", string("[\"programer_art\",\"vanilla\"]")),
            ("incompatibleResourcePacks", string("[\"programer_art\"]")),
            ("lang", string("en_us")),
        ]);
        let out = fix.make_rule().apply(input);
        assert_eq!(
            out,
            map(vec![
                ("resourcePacks", string("[\"programmer_art\",\"vanilla\"]")),
                ("incompatibleResourcePacks", string("[\"programmer_art\"]")),
                ("lang", string("en_us")),
            ])
        );
    }

    #[test]
    fn missing_fields_left_unchanged() {
        let fix = OptionsProgrammerArtFix::new(Schema);
        let input = map(vec![("lang", string("en_us"))]);
        let out = fix.make_rule().apply(input.clone());
        assert_eq!(out, input);
    }
}
