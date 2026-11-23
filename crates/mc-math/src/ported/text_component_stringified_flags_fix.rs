/// Faithful port of `net.minecraft.util.datafix.fixes.TextComponentStringifiedFlagsFix`.
///
/// The Java fix converts stringified boolean style flags in text components to
/// actual booleans. This self-contained port applies the same field conversion
/// to a simple dynamic structure.
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    String(String),
    Bool(bool),
    Compound(Vec<(String, DynamicValue)>),
    Other,
}

impl DynamicValue {
    // JAVA: com.mojang.serialization.Dynamic#asString
    fn as_string(&self) -> Option<&str> {
        match self {
            DynamicValue::String(s) => Some(s),
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createBoolean
    fn create_boolean(value: bool) -> DynamicValue {
        DynamicValue::Bool(value)
    }

    // JAVA: com.mojang.serialization.Dynamic#update
    fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(DynamicValue) -> DynamicValue,
    {
        match self {
            DynamicValue::Compound(mut entries) => {
                if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key) {
                    let old = std::mem::replace(v, DynamicValue::Other);
                    *v = f(old);
                }
                DynamicValue::Compound(entries)
            }
            other => other,
        }
    }
}

pub struct TextComponentStringifiedFlagsFix;

impl TextComponentStringifiedFlagsFix {
    // JAVA: net.minecraft.util.datafix.fixes.TextComponentStringifiedFlagsFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.TextComponentStringifiedFlagsFix#stringToBool
    fn string_to_bool(value: DynamicValue) -> DynamicValue {
        let Some(s) = value.as_string() else {
            return value;
        };
        // Java Boolean.parseBoolean returns false for any non-"true" (case-insensitive) string.
        let parsed = s.eq_ignore_ascii_case("true");
        DynamicValue::create_boolean(parsed)
    }

    // JAVA: net.minecraft.util.datafix.fixes.TextComponentStringifiedFlagsFix#makeRule (behavioral equivalent)
    pub fn fix_component(&self, component: DynamicValue) -> DynamicValue {
        component
            .update("bold", Self::string_to_bool)
            .update("italic", Self::string_to_bool)
            .update("underlined", Self::string_to_bool)
            .update("strikethrough", Self::string_to_bool)
            .update("obfuscated", Self::string_to_bool)
    }
}

impl Default for TextComponentStringifiedFlagsFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(entries: Vec<(&str, DynamicValue)>) -> DynamicValue {
        DynamicValue::Compound(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn string(s: &str) -> DynamicValue {
        DynamicValue::String(s.to_string())
    }

    #[test]
    fn converts_stringified_flags_to_booleans() {
        let fix = TextComponentStringifiedFlagsFix::new();
        let component = compound(vec![
            ("bold", string("true")),
            ("italic", string("false")),
            ("underlined", string("TRUE")),
            ("strikethrough", string("not-bool")),
            ("obfuscated", string("false")),
        ]);
        let fixed = fix.fix_component(component);
        let DynamicValue::Compound(entries) = fixed else { panic!("expected compound") };
        let get = |k: &str| entries.iter().find(|(key, _)| key == k).unwrap().1.clone();
        assert_eq!(get("bold"), DynamicValue::Bool(true));
        assert_eq!(get("italic"), DynamicValue::Bool(false));
        assert_eq!(get("underlined"), DynamicValue::Bool(true));
        assert_eq!(get("strikethrough"), DynamicValue::Bool(false));
        assert_eq!(get("obfuscated"), DynamicValue::Bool(false));
    }

    #[test]
    fn leaves_non_string_flags_unchanged() {
        let fix = TextComponentStringifiedFlagsFix::new();
        let component = compound(vec![
            ("bold", DynamicValue::Bool(true)),
            ("italic", DynamicValue::Other),
        ]);
        let fixed = fix.fix_component(component.clone());
        assert_eq!(fixed, component);
    }

    #[test]
    fn missing_fields_are_ignored() {
        let fix = TextComponentStringifiedFlagsFix::new();
        let component = compound(vec![("text", string("hello"))]);
        let fixed = fix.fix_component(component.clone());
        assert_eq!(fixed, component);
    }

    #[test]
    fn non_compound_input_is_returned_as_is() {
        let fix = TextComponentStringifiedFlagsFix::new();
        let input = string("plain");
        assert_eq!(fix.fix_component(input.clone()), input);
    }

    #[test]
    fn default_constructor_matches_new() {
        let a = TextComponentStringifiedFlagsFix::new().fix_component(compound(vec![("bold", string("true"))]));
        let b = TextComponentStringifiedFlagsFix::default().fix_component(compound(vec![("bold", string("true"))]));
        assert_eq!(a, b);
    }
}
