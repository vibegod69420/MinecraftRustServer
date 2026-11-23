use std::collections::BTreeMap;

/// Line field names on a sign block entity, in order.
pub const LINE_FIELDS: [&str; 4] = ["Text1", "Text2", "Text3", "Text4"];

/// Port of `net.minecraft.util.datafix.fixes.SignTextStrictJsonFix`.
///
/// Rewrites legacy/lenient sign text component strings into strict JSON form
/// for the four sign line fields.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignTextStrictJsonFix;

impl SignTextStrictJsonFix {
    // JAVA: net.minecraft.util.datafix.fixes.SignTextStrictJsonFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.SignTextStrictJsonFix#fix
    pub fn fix(&self, mut data: BTreeMap<String, String>) -> BTreeMap<String, String> {
        for field in LINE_FIELDS {
            if let Some(value) = data.get_mut(field) {
                *value = rewrite_from_lenient(value);
            }
        }
        data
    }
}

// JAVA: net.minecraft.util.datafix.LegacyComponentDataFixUtils#rewriteFromLenient
pub fn rewrite_from_lenient(json: &str) -> String {
    let trimmed = json.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return empty_text_component();
    }
    if trimmed == "true" {
        return make_text_component("true");
    }
    if trimmed == "false" {
        return make_text_component("false");
    }
    if let Some(s) = parse_json_string_literal(trimmed) {
        return if s.is_empty() {
            empty_text_component()
        } else {
            make_text_component(&s)
        };
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return trimmed.to_string();
    }
    if is_json_number(trimmed) {
        return make_text_component(trimmed);
    }
    make_text_component(trimmed)
}

fn empty_text_component() -> String {
    "{\"text\":\"\"}".to_string()
}

fn make_text_component(text: &str) -> String {
    format!("{{\"text\":{}}}", escape_json_string(text))
}

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\u{0020}' => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn parse_json_string_literal(s: &str) -> Option<String> {
    let mut chars = s.chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut out = String::new();
    loop {
        let c = chars.next()?;
        match c {
            '"' => {
                return if chars.as_str().is_empty() {
                    Some(out)
                } else {
                    None
                };
            }
            '\\' => {
                let e = chars.next()?;
                match e {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'b' => out.push('\u{0008}'),
                    'f' => out.push('\u{000C}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            hex.push(chars.next()?);
                        }
                        let cp = u32::from_str_radix(&hex, 16).ok()?;
                        let ch = char::from_u32(cp)?;
                        out.push(ch);
                    }
                    _ => return None,
                }
            }
            c if c < '\u{0020}' => return None,
            c => out.push(c),
        }
    }
}

fn is_json_number(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0usize;
    if bytes[i] == b'-' {
        i += 1;
        if i >= bytes.len() {
            return false;
        }
    }
    let mut has_digit = false;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        has_digit = true;
        i += 1;
    }
    if !has_digit {
        return false;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let mut frac_digit = false;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            frac_digit = true;
            i += 1;
        }
        if !frac_digit {
            return false;
        }
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        let mut exp_digit = false;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            exp_digit = true;
            i += 1;
        }
        if !exp_digit {
            return false;
        }
    }
    i == bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_plain_text_and_empty() {
        assert_eq!(rewrite_from_lenient(""), "{\"text\":\"\"}");
        assert_eq!(rewrite_from_lenient("null"), "{\"text\":\"\"}");
        assert_eq!(rewrite_from_lenient("Hello"), "{\"text\":\"Hello\"}");
        assert_eq!(rewrite_from_lenient("  spaced  "), "{\"text\":\"spaced\"}");
    }

    #[test]
    fn rewrite_json_primitives() {
        assert_eq!(rewrite_from_lenient("\"hello\""), "{\"text\":\"hello\"}");
        assert_eq!(rewrite_from_lenient("\"\""), "{\"text\":\"\"}");
        assert_eq!(rewrite_from_lenient("true"), "{\"text\":\"true\"}");
        assert_eq!(rewrite_from_lenient("false"), "{\"text\":\"false\"}");
        assert_eq!(rewrite_from_lenient("42"), "{\"text\":\"42\"}");
        assert_eq!(rewrite_from_lenient("-3.5e2"), "{\"text\":\"-3.5e2\"}");
    }

    #[test]
    fn rewrite_json_object_or_array_passthrough() {
        let obj = "{\"text\":\"Hi\",\"color\":\"red\"}";
        let arr = "[{\"text\":\"A\"},{\"text\":\"B\"}]";
        assert_eq!(rewrite_from_lenient(obj), obj);
        assert_eq!(rewrite_from_lenient(arr), arr);
    }

    #[test]
    fn rewrite_escapes_special_characters() {
        assert_eq!(
            rewrite_from_lenient("line1\nline2\t\"q\"\\"),
            "{\"text\":\"line1\\nline2\\t\\\"q\\\"\\\\\"}"
        );
        assert_eq!(
            rewrite_from_lenient("\"a\\n\\u0041\""),
            "{\"text\":\"a\\nA\"}"
        );
    }

    #[test]
    fn fix_updates_only_line_fields() {
        let fix = SignTextStrictJsonFix::new();
        let mut data = BTreeMap::new();
        data.insert("Text1".to_string(), "Hello".to_string());
        data.insert("Text2".to_string(), "\"World\"".to_string());
        data.insert("Text3".to_string(), "{\"text\":\"!\"}".to_string());
        data.insert("Text4".to_string(), "".to_string());
        data.insert("Color".to_string(), "black".to_string());

        let fixed = fix.fix(data);
        assert_eq!(fixed.get("Text1").unwrap(), "{\"text\":\"Hello\"}");
        assert_eq!(fixed.get("Text2").unwrap(), "{\"text\":\"World\"}");
        assert_eq!(fixed.get("Text3").unwrap(), "{\"text\":\"!\"}");
        assert_eq!(fixed.get("Text4").unwrap(), "{\"text\":\"\"}");
        assert_eq!(fixed.get("Color").unwrap(), "black");
    }

    #[test]
    fn fix_handles_missing_line_fields() {
        let fix = SignTextStrictJsonFix::new();
        let mut data = BTreeMap::new();
        data.insert("Text2".to_string(), "only".to_string());
        let fixed = fix.fix(data);
        assert_eq!(fixed.len(), 1);
        assert_eq!(fixed.get("Text2").unwrap(), "{\"text\":\"only\"}");
        assert!(!fixed.contains_key("Text1"));
    }
}
