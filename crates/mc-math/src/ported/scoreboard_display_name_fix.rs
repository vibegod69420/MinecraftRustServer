#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeReference(pub String);

#[derive(Clone, Debug, Default)]
pub struct TypeRewriteRule;

pub mod legacy_component_data_fix_utils {
    // JAVA: net.minecraft.util.datafix.LegacyComponentDataFixUtils#createTextComponentJson
    pub fn create_text_component_json(input: String) -> String {
        if input.starts_with('{') || input.starts_with('[') || input == "null" {
            return input;
        }
        // Minimal faithful escaping for plain text component JSON.
        let mut escaped = String::with_capacity(input.len() + 2);
        for ch in input.chars() {
            match ch {
                '"' => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                c if c.is_control() => {
                    for u in c.encode_utf16(&mut [0u16; 2]) {
                        escaped.push_str(&format!("\\u{:04x}", u));
                    }
                }
                c => escaped.push(c),
            }
        }
        format!("{{\"text\":\"{}\"}}", escaped)
    }
}

pub struct ScoreboardDisplayNameFix {
    name: String,
    type_ref: TypeReference,
    _schema: Schema,
}

impl ScoreboardDisplayNameFix {
    // JAVA: net.minecraft.util.datafix.fixes.ScoreboardDisplayNameFix#<init>
    pub fn new(schema: Schema, name: String, type_ref: TypeReference) -> Self {
        Self {
            name,
            type_ref,
            _schema: schema,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_ref(&self) -> &TypeReference {
        &self.type_ref
    }

    // JAVA: net.minecraft.util.datafix.fixes.ScoreboardDisplayNameFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        TypeRewriteRule
    }

    /// Applies the core text-component conversion used by the fix rule.
    pub fn fix_display_name(display_name: String) -> String {
        legacy_component_data_fix_utils::create_text_component_json(display_name)
    }
}

#[cfg(test)]
mod tests {
    use super::legacy_component_data_fix_utils::create_text_component_json;
    use super::*;

    #[test]
    fn constructor_stores_fields() {
        let fix = ScoreboardDisplayNameFix::new(
            Schema,
            "ScoreboardDisplayNameFix".to_string(),
            TypeReference("OBJECTIVE".to_string()),
        );
        assert_eq!(fix.name(), "ScoreboardDisplayNameFix");
        assert_eq!(fix.type_ref(), &TypeReference("OBJECTIVE".to_string()));
    }

    #[test]
    fn plain_text_is_wrapped_as_json_component() {
        assert_eq!(
            create_text_component_json("Hello".to_string()),
            "{\"text\":\"Hello\"}"
        );
    }

    #[test]
    fn json_object_is_preserved() {
        let input = "{\"text\":\"Hello\",\"color\":\"red\"}".to_string();
        assert_eq!(create_text_component_json(input.clone()), input);
    }

    #[test]
    fn json_array_and_null_are_preserved() {
        let arr = "[{\"text\":\"a\"}]".to_string();
        assert_eq!(create_text_component_json(arr.clone()), arr);
        assert_eq!(
            create_text_component_json("null".to_string()),
            "null".to_string()
        );
    }

    #[test]
    fn special_characters_are_escaped() {
        let input = "a\"b\\c\n\t".to_string();
        assert_eq!(
            create_text_component_json(input),
            "{\"text\":\"a\\\"b\\\\c\\n\\t\"}"
        );
    }

    #[test]
    fn fix_display_name_delegates_to_legacy_component_utils() {
        assert_eq!(
            ScoreboardDisplayNameFix::fix_display_name("Score".to_string()),
            "{\"text\":\"Score\"}"
        );
    }
}
