/// Minimal self-contained dynamic value model for datafix porting.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i64),
    Bool(bool),
    Map(Vec<(String, Dynamic)>),
}

pub struct OptionalDynamic(Option<Dynamic>);

pub struct OptionalString(Option<String>);

impl OptionalString {
    pub fn result(self) -> Option<String> {
        self.0
    }
}

impl OptionalDynamic {
    pub fn result(self) -> Option<Dynamic> {
        self.0
    }

    pub fn as_string(&self) -> OptionalString {
        match &self.0 {
            Some(Dynamic::String(s)) => OptionalString(Some(s.clone())),
            _ => OptionalString(None),
        }
    }
}

impl Dynamic {
    pub fn get(&self, key: &str) -> OptionalDynamic {
        match self {
            Dynamic::Map(entries) => OptionalDynamic(
                entries
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone()),
            ),
            _ => OptionalDynamic(None),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(entries) => {
                let mut new_entries = Vec::with_capacity(entries.len() + 1);
                let mut replaced = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((key.to_string(), value.clone()));
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

    pub fn create_string(&self, value: String) -> Self {
        Dynamic::String(value)
    }
}

/// Placeholder for DFU `Schema`.
#[derive(Clone, Debug, Default)]
pub struct Schema;

/// Placeholder for DFU `TypeRewriteRule`.
#[derive(Clone, Debug, Default)]
pub struct TypeRewriteRule {
    pub name: String,
}

pub struct OptionsLowerCaseLanguageFix {
    _schema: Schema,
    _changes_type: bool,
}

impl OptionsLowerCaseLanguageFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsLowerCaseLanguageFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsLowerCaseLanguageFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule {
            name: "OptionsLowerCaseLanguageFix".to_string(),
        }
    }

    /// Applies the core language-lowercasing operation modeled by this fix's rule.
    pub fn apply_to_options(&self, options: Dynamic) -> Dynamic {
        let lang = options.get("lang").as_string().result();
        match lang {
            Some(value) => options.set("lang", options.create_string(value.to_lowercase())),
            None => options,
        }
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
    fn lowercases_lang_field() {
        let fix = OptionsLowerCaseLanguageFix::new(Schema, true);
        let input = map(vec![
            ("lang", Dynamic::String("EN_US".to_string())),
            ("fov", Dynamic::Int(90)),
        ]);
        let output = fix.apply_to_options(input);
        let expected = map(vec![
            ("lang", Dynamic::String("en_us".to_string())),
            ("fov", Dynamic::Int(90)),
        ]);
        assert_eq!(output, expected);
        assert_eq!(fix.make_rule().name, "OptionsLowerCaseLanguageFix");
    }

    #[test]
    fn missing_lang_is_unchanged() {
        let fix = OptionsLowerCaseLanguageFix::new(Schema, true);
        let input = map(vec![("fov", Dynamic::Int(70))]);
        let output = fix.apply_to_options(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn non_string_lang_is_unchanged() {
        let fix = OptionsLowerCaseLanguageFix::new(Schema, true);
        let input = map(vec![("lang", Dynamic::Int(123))]);
        let output = fix.apply_to_options(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn already_lowercase_lang_remains_equivalent() {
        let fix = OptionsLowerCaseLanguageFix::new(Schema, false);
        let input = map(vec![("lang", Dynamic::String("en_us".to_string()))]);
        let output = fix.apply_to_options(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn lowercases_unicode_using_root_like_behavior() {
        let fix = OptionsLowerCaseLanguageFix::new(Schema, true);
        let input = map(vec![("lang", Dynamic::String("Iİ".to_string()))]);
        let output = fix.apply_to_options(input);
        let expected = map(vec![("lang", Dynamic::String("ii̇".to_string()))]);
        assert_eq!(output, expected);
    }
}
