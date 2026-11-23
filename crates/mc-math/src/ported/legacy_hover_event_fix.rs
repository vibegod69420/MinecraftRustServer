use std::collections::BTreeMap;

/// Minimal self-contained dynamic value used to represent legacy text-component hover events.
#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Array(Vec<DynValue>),
    Object(BTreeMap<String, DynValue>),
}

impl DynValue {
    pub fn get(&self, key: &str) -> Option<&DynValue> {
        match self {
            DynValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            DynValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[DynValue]> {
        match self {
            DynValue::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }

    pub fn rename_field(&self, old: &str, new_name: &str) -> DynValue {
        match self {
            DynValue::Object(map) => {
                let mut out = BTreeMap::new();
                for (k, v) in map {
                    let key = if k == old {
                        new_name.to_string()
                    } else {
                        k.clone()
                    };
                    out.insert(key, v.clone());
                }
                DynValue::Object(out)
            }
            _ => self.clone(),
        }
    }

    pub fn to_stable_json(&self) -> String {
        let mut out = String::new();
        self.write_stable_json(&mut out);
        out
    }

    fn write_stable_json(&self, out: &mut String) {
        match self {
            DynValue::Null => out.push_str("null"),
            DynValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            DynValue::I64(n) => out.push_str(&n.to_string()),
            DynValue::F64(n) => {
                if n.is_finite() && n.fract() == 0.0 {
                    let s = n.to_string();
                    if s.contains('.') || s.contains('e') || s.contains('E') {
                        out.push_str(&s);
                    } else {
                        out.push_str(&s);
                        out.push_str(".0");
                    }
                } else {
                    out.push_str(&n.to_string());
                }
            }
            DynValue::String(s) => write_json_string(s, out),
            DynValue::Array(arr) => {
                out.push('[');
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    v.write_stable_json(out);
                }
                out.push(']');
            }
            DynValue::Object(map) => {
                out.push('{');
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_json_string(k, out);
                    out.push(':');
                    v.write_stable_json(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if c < '\u{20}' => {
                let code = c as u32;
                out.push_str(&format!("\\u{:04x}", code));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LegacyHoverEventFix;

impl LegacyHoverEventFix {
    // JAVA: net.minecraft.util.datafix.fixes.LegacyHoverEventFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.LegacyHoverEventFix#makeRule
    pub fn make_rule(&self) -> Box<dyn Fn(&DynValue) -> DynValue> {
        self.create_fixer()
    }

    // JAVA: net.minecraft.util.datafix.fixes.LegacyHoverEventFix#createFixer
    fn create_fixer(&self) -> Box<dyn Fn(&DynValue) -> DynValue> {
        Box::new(|component| LegacyHoverEventFix.fix_component(component))
    }

    // JAVA: net.minecraft.util.datafix.fixes.LegacyHoverEventFix#fixHoverEvent
    fn fix_hover_event(&self, action: &str, hover_event: &DynValue) -> DynValue {
        if action == "show_text" {
            Self::fix_show_text_hover(hover_event)
        } else {
            Self::create_placeholder_hover(hover_event)
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.LegacyHoverEventFix#fixShowTextHover
    fn fix_show_text_hover(hover_event: &DynValue) -> DynValue {
        hover_event.rename_field("value", "contents")
    }

    // JAVA: net.minecraft.util.datafix.fixes.LegacyHoverEventFix#createPlaceholderHover
    fn create_placeholder_hover(hover_event: &DynValue) -> DynValue {
        let json_str = hover_event.to_stable_json();
        let mut contents = BTreeMap::new();
        contents.insert(
            "text".to_string(),
            DynValue::String(format!("Legacy hoverEvent: {}", json_str)),
        );
        let mut map = BTreeMap::new();
        map.insert(
            "action".to_string(),
            DynValue::String("show_text".to_string()),
        );
        map.insert("contents".to_string(), DynValue::Object(contents));
        DynValue::Object(map)
    }

    fn fix_component(&self, component: &DynValue) -> DynValue {
        match component {
            DynValue::Array(arr) => DynValue::Array(
                arr.iter()
                    .map(|c| self.fix_component(c))
                    .collect(),
            ),
            DynValue::Object(map) => {
                let mut out = BTreeMap::new();
                for (k, v) in map {
                    let fixed_v = match k.as_str() {
                        "extra" | "separator" => self.fix_component(v),
                        "hoverEvent" => self.fix_hover_event_field(v),
                        _ => v.clone(),
                    };
                    out.insert(k.clone(), fixed_v);
                }
                DynValue::Object(out)
            }
            _ => component.clone(),
        }
    }

    fn fix_hover_event_field(&self, hover_event: &DynValue) -> DynValue {
        if hover_event.get("value").is_none() {
            return hover_event.clone();
        }
        let action = hover_event
            .get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("");
        self.fix_hover_event(action, hover_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> DynValue {
        DynValue::String(v.to_string())
    }

    fn obj(entries: Vec<(&str, DynValue)>) -> DynValue {
        DynValue::Object(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn fix_show_text_renames_value_to_contents() {
        let fix = LegacyHoverEventFix::new();
        let component = obj(vec![
            ("text", s("hi")),
            (
                "hoverEvent",
                obj(vec![("action", s("show_text")), ("value", s("tooltip"))]),
            ),
        ]);
        let out = (fix.make_rule())(&component);
        let hover = out.get("hoverEvent").unwrap();
        assert_eq!(hover.get("action").and_then(DynValue::as_str), Some("show_text"));
        assert_eq!(hover.get("contents").and_then(DynValue::as_str), Some("tooltip"));
        assert!(hover.get("value").is_none());
    }

    #[test]
    fn non_show_text_creates_placeholder() {
        let fix = LegacyHoverEventFix::new();
        let component = obj(vec![(
            "hoverEvent",
            obj(vec![
                ("action", s("show_entity")),
                ("value", s("{id:\"minecraft:pig\"}")),
            ]),
        )]);
        let out = (fix.make_rule())(&component);
        let hover = out.get("hoverEvent").unwrap();
        assert_eq!(hover.get("action").and_then(DynValue::as_str), Some("show_text"));
        let text = hover
            .get("contents")
            .and_then(|c| c.get("text"))
            .and_then(DynValue::as_str)
            .unwrap();
        assert!(text.starts_with("Legacy hoverEvent: "));
        assert!(text.contains("show_entity"));
        assert!(text.contains("{id:\\\"minecraft:pig\\\"}"));
    }

    #[test]
    fn modern_hover_without_value_unchanged() {
        let fix = LegacyHoverEventFix::new();
        let hover = obj(vec![("action", s("show_text")), ("contents", s("modern"))]);
        let component = obj(vec![("hoverEvent", hover.clone())]);
        let out = (fix.make_rule())(&component);
        assert_eq!(out.get("hoverEvent").unwrap(), &hover);
    }

    #[test]
    fn fixes_nested_extra_and_separator() {
        let fix = LegacyHoverEventFix::new();
        let legacy_hover = obj(vec![("action", s("show_text")), ("value", s("tip"))]);
        let component = obj(vec![
            (
                "extra",
                DynValue::Array(vec![obj(vec![("hoverEvent", legacy_hover.clone())])]),
            ),
            ("separator", obj(vec![("hoverEvent", legacy_hover)])),
        ]);
        let out = (fix.make_rule())(&component);
        let extra_hover = out.get("extra").unwrap().as_array().unwrap()[0]
            .get("hoverEvent")
            .unwrap();
        assert_eq!(extra_hover.get("contents").and_then(DynValue::as_str), Some("tip"));
        assert!(extra_hover.get("value").is_none());
        let sep_hover = out.get("separator").unwrap().get("hoverEvent").unwrap();
        assert_eq!(sep_hover.get("contents").and_then(DynValue::as_str), Some("tip"));
    }

    #[test]
    fn stable_json_handles_edge_cases_negative_overflow_and_float_bits() {
        let mut inner = BTreeMap::new();
        inner.insert("neg".to_string(), DynValue::I64(i64::MIN));
        inner.insert("pos".to_string(), DynValue::I64(i64::MAX));
        inner.insert(
            "float_bits".to_string(),
            DynValue::F64(f64::from_bits(0xBFF0_0000_0000_0000u64)), // -1.0
        );
        inner.insert(
            "float_pos".to_string(),
            DynValue::F64(f64::from_bits(0x3FF0_0000_0000_0000u64)), // 1.0
        );
        inner.insert("esc".to_string(), DynValue::String("a\"b\\c\n\t".to_string()));
        let val = DynValue::Object(inner);
        let json = val.to_stable_json();
        assert!(json.contains(&i64::MIN.to_string()));
        assert!(json.contains(&i64::MAX.to_string()));
        assert!(json.contains("-1.0"));
        assert!(json.contains("1.0"));
        assert!(json.contains("a\\\"b\\\\c\\n\\t"));
        // BTreeMap key order: esc, float_bits, float_pos, neg, pos
        let esc_idx = json.find("\"esc\"").unwrap();
        let neg_idx = json.find("\"neg\"").unwrap();
        let pos_idx = json.find("\"pos\"").unwrap();
        assert!(esc_idx < neg_idx);
        assert!(neg_idx < pos_idx);
    }
}
