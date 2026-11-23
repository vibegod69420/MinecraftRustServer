use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema;

impl Schema {
    pub fn get_type(&self, reference: &str) -> String {
        reference.to_string()
    }
}

pub struct References;

impl References {
    pub const OPTIONS: &'static str = "options";
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dynamic {
    String(String),
    Map(BTreeMap<String, Dynamic>),
    Missing,
}

impl Dynamic {
    pub fn as_string(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn rename_and_fix_field<F>(self, old_name: &str, new_name: &str, fix: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        match self {
            Dynamic::Map(mut map) => {
                if let Some(old_value) = map.remove(old_name) {
                    let new_value = fix(old_value);
                    map.insert(new_name.to_string(), new_value);
                }
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemainderFinder;

pub struct Dsl;

impl Dsl {
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Typed {
    remainder: Dynamic,
}

impl Typed {
    pub fn new(remainder: Dynamic) -> Self {
        Self { remainder }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, _finder: RemainderFinder, f: F) -> Self {
        Self {
            remainder: f(self.remainder),
        }
    }

    pub fn remainder(&self) -> &Dynamic {
        &self.remainder
    }
}

pub struct TypeRewriteRule {
    pub name: String,
    apply_fn: Box<dyn Fn(Typed) -> Typed>,
}

impl TypeRewriteRule {
    pub fn apply(&self, typed: Typed) -> Typed {
        (self.apply_fn)(typed)
    }
}

pub struct OptionsMusicToastFix {
    pub output_schema: Schema,
    pub changes_type: bool,
}

impl OptionsMusicToastFix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsMusicToastFix#OptionsMusicToastFix
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            output_schema: schema,
            changes_type,
        }
    }

    fn get_input_schema(&self) -> &Schema {
        &self.output_schema
    }

    fn fix_type_everywhere_typed<F>(&self, name: &str, _input_type: &str, f: F) -> TypeRewriteRule
    where
        F: Fn(Typed) -> Typed + 'static,
    {
        TypeRewriteRule {
            name: name.to_string(),
            apply_fn: Box::new(f),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsMusicToastFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let input_type = self.get_input_schema().get_type(References::OPTIONS);
        self.fix_type_everywhere_typed("OptionsMusicToastFix", &input_type, |typed| {
            typed.update(Dsl::remainder_finder(), |d| {
                d.rename_and_fix_field("showNowPlayingToast", "musicToast", |old| {
                    let s = old.as_string("false");
                    let new_val = if s == "false" { "never" } else { "pause_and_toast" };
                    Dynamic::String(new_val.to_string())
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options_dynamic(show: Option<&str>) -> Dynamic {
        let mut map = BTreeMap::new();
        if let Some(value) = show {
            map.insert(
                "showNowPlayingToast".to_string(),
                Dynamic::String(value.to_string()),
            );
        }
        Dynamic::Map(map)
    }

    fn apply_fix(show: Option<&str>) -> Dynamic {
        let fix = OptionsMusicToastFix::new(Schema, true);
        let rule = fix.make_rule();
        let out = rule.apply(Typed::new(options_dynamic(show)));
        out.remainder().clone()
    }

    #[test]
    fn false_becomes_never() {
        let d = apply_fix(Some("false"));
        assert_eq!(
            d,
            Dynamic::Map(BTreeMap::from([(
                "musicToast".to_string(),
                Dynamic::String("never".to_string())
            )]))
        );
    }

    #[test]
    fn true_becomes_pause_and_toast() {
        let d = apply_fix(Some("true"));
        assert_eq!(
            d,
            Dynamic::Map(BTreeMap::from([(
                "musicToast".to_string(),
                Dynamic::String("pause_and_toast".to_string())
            )]))
        );
    }

    #[test]
    fn missing_field_leaves_dynamic_unchanged() {
        let d = apply_fix(None);
        assert_eq!(d, Dynamic::Map(BTreeMap::new()));
    }

    #[test]
    fn non_false_string_becomes_pause_and_toast() {
        let d = apply_fix(Some("maybe"));
        assert_eq!(
            d,
            Dynamic::Map(BTreeMap::from([(
                "musicToast".to_string(),
                Dynamic::String("pause_and_toast".to_string())
            )]))
        );
    }

    #[test]
    fn non_string_and_numeric_like_values_follow_default_semantics() {
        let mut map = BTreeMap::new();
        map.insert(
            "showNowPlayingToast".to_string(),
            Dynamic::Map(BTreeMap::new()),
        );
        let fix = OptionsMusicToastFix::new(Schema, true);
        let out = fix.make_rule().apply(Typed::new(Dynamic::Map(map)));
        assert_eq!(
            out.remainder(),
            &Dynamic::Map(BTreeMap::from([(
                "musicToast".to_string(),
                Dynamic::String("never".to_string())
            )]))
        );

        let d_neg = apply_fix(Some("-1.0"));
        assert_eq!(
            d_neg,
            Dynamic::Map(BTreeMap::from([(
                "musicToast".to_string(),
                Dynamic::String("pause_and_toast".to_string())
            )]))
        );
    }
}
