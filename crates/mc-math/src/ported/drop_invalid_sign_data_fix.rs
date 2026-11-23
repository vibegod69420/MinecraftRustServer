use std::collections::BTreeMap;

/// Minimal self-contained dynamic value representation for datafix logic.
#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Null,
    String(String),
    Bool(bool),
    Long(i64),
    Double(f64),
    Int(i32),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

/// Minimal self-contained Dynamic wrapper mirroring DFU Dynamic operations used by this fix.
#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynamicValue,
}

impl Dynamic {
    pub fn null() -> Self {
        Self {
            value: DynamicValue::Null,
        }
    }

    pub fn string(s: impl Into<String>) -> Self {
        Self {
            value: DynamicValue::String(s.into()),
        }
    }

    pub fn bool(v: bool) -> Self {
        Self {
            value: DynamicValue::Bool(v),
        }
    }

    pub fn long(v: i64) -> Self {
        Self {
            value: DynamicValue::Long(v),
        }
    }

    pub fn int(v: i32) -> Self {
        Self {
            value: DynamicValue::Int(v),
        }
    }

    pub fn double(v: f64) -> Self {
        Self {
            value: DynamicValue::Double(v),
        }
    }

    pub fn list(items: Vec<Dynamic>) -> Self {
        Self {
            value: DynamicValue::List(items),
        }
    }

    pub fn map(entries: BTreeMap<String, Dynamic>) -> Self {
        Self {
            value: DynamicValue::Map(entries),
        }
    }

    pub fn get(&self, key: &str) -> Self {
        match &self.value {
            DynamicValue::Map(m) => m.get(key).cloned().unwrap_or_else(Dynamic::null),
            _ => Dynamic::null(),
        }
    }

    pub fn as_stream_opt(&self) -> Option<Vec<Dynamic>> {
        match &self.value {
            DynamicValue::List(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn as_boolean(&self, default: bool) -> bool {
        match &self.value {
            DynamicValue::Bool(b) => *b,
            _ => default,
        }
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(&self, key: &str, f: F) -> Self {
        match &self.value {
            DynamicValue::Map(m) => {
                let mut new_m = m.clone();
                if let Some(v) = new_m.get(key).cloned() {
                    new_m.insert(key.to_string(), f(v));
                }
                Dynamic::map(new_m)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Self {
        match &self.value {
            DynamicValue::Map(m) => {
                let mut new_m = m.clone();
                new_m.remove(key);
                Dynamic::map(new_m)
            }
            _ => self.clone(),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Self {
        match &self.value {
            DynamicValue::Map(m) => {
                let mut new_m = m.clone();
                new_m.insert(key.to_string(), value);
                Dynamic::map(new_m)
            }
            _ => self.clone(),
        }
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Self {
        Dynamic::list(items)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.BlockEntitySignDoubleSidedEditableTextFix#FIELDS_TO_DROP
pub const FIELDS_TO_DROP: &[&str] = &[
    "Text1",
    "Text2",
    "Text3",
    "Text4",
    "FilteredText1",
    "FilteredText2",
    "FilteredText3",
    "FilteredText4",
    "Color",
    "GlowingText",
    "is_waxed",
];

// JAVA: net.minecraft.util.datafix.LegacyComponentDataFixUtils#createEmptyComponent
fn create_empty_component() -> Dynamic {
    Dynamic::string("")
}

pub struct DropInvalidSignDataFix {
    pub entity_name: String,
}

impl DropInvalidSignDataFix {
    // JAVA: net.minecraft.util.datafix.fixes.DropInvalidSignDataFix#<init>
    pub fn new(entity_name: String) -> Self {
        Self { entity_name }
    }

    // JAVA: net.minecraft.util.datafix.fixes.DropInvalidSignDataFix#fixText
    fn fix_text(d: Dynamic) -> Dynamic {
        let Some(filtered) = d.get("filtered_messages").as_stream_opt() else {
            return d;
        };
        let empty_component = create_empty_component();
        let messages = d.get("messages").as_stream_opt().unwrap_or_default();
        let fixed: Vec<Dynamic> = filtered
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                let msg = if i < messages.len() {
                    messages[i].clone()
                } else {
                    empty_component.clone()
                };
                if f == empty_component {
                    msg
                } else {
                    f
                }
            })
            .collect();
        if fixed == messages {
            d.remove("filtered_messages")
        } else {
            let new_list = d.create_list(fixed);
            d.set("filtered_messages", new_list)
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.DropInvalidSignDataFix#fix
    pub fn fix(&self, mut d: Dynamic) -> Dynamic {
        d = d.update("front_text", Self::fix_text);
        d = d.update("back_text", Self::fix_text);
        for field in FIELDS_TO_DROP {
            d = d.remove(field);
        }
        d
    }

    // JAVA: net.minecraft.util.datafix.fixes.DropInvalidSignDataFix#makeRule
    pub fn make_rule_description(&self) -> String {
        format!("DropInvalidSignDataFix for {}", self.entity_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dmap(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::map(m)
    }

    #[test]
    fn fix_text_missing_filtered_returns_unchanged() {
        let input = dmap(vec![(
            "messages",
            Dynamic::list(vec![Dynamic::string("a"), Dynamic::string("b")]),
        )]);
        let out = DropInvalidSignDataFix::fix_text(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn fix_text_replaces_empty_filtered_with_message() {
        let input = dmap(vec![
            (
                "messages",
                Dynamic::list(vec![Dynamic::string("hello"), Dynamic::string("world")]),
            ),
            (
                "filtered_messages",
                Dynamic::list(vec![Dynamic::string(""), Dynamic::string("filtered")]),
            ),
        ]);
        let out = DropInvalidSignDataFix::fix_text(input);
        let filtered = out.get("filtered_messages").as_stream_opt().unwrap();
        assert_eq!(filtered[0], Dynamic::string("hello"));
        assert_eq!(filtered[1], Dynamic::string("filtered"));
    }

    #[test]
    fn fix_text_removes_filtered_when_equal_to_messages() {
        let msgs = Dynamic::list(vec![Dynamic::string("a"), Dynamic::string("b")]);
        let input = dmap(vec![
            ("messages", msgs.clone()),
            (
                "filtered_messages",
                Dynamic::list(vec![Dynamic::string(""), Dynamic::string("")]),
            ),
        ]);
        let out = DropInvalidSignDataFix::fix_text(input);
        assert!(out.get("filtered_messages").as_stream_opt().is_none());
    }

    #[test]
    fn fix_text_handles_short_messages_list_and_negative_int_fields() {
        let input = dmap(vec![
            ("messages", Dynamic::list(vec![Dynamic::string("only")])),
            (
                "filtered_messages",
                Dynamic::list(vec![Dynamic::string(""), Dynamic::string("")]),
            ),
            ("extra", Dynamic::int(-1)),
        ]);
        let out = DropInvalidSignDataFix::fix_text(input);
        let filtered = out.get("filtered_messages").as_stream_opt().unwrap();
        assert_eq!(filtered[0], Dynamic::string("only"));
        assert_eq!(filtered[1], Dynamic::string(""));
        assert_eq!(out.get("extra"), Dynamic::int(-1));
    }

    #[test]
    fn fix_removes_dropped_fields_and_updates_front_back_text() {
        let fix = DropInvalidSignDataFix::new("minecraft:sign".to_string());
        let text = dmap(vec![
            ("messages", Dynamic::list(vec![Dynamic::string("m")])),
            (
                "filtered_messages",
                Dynamic::list(vec![Dynamic::string("")]),
            ),
        ]);
        let input = dmap(vec![
            ("front_text", text.clone()),
            ("back_text", text),
            ("Text1", Dynamic::string("old")),
            ("Color", Dynamic::string("black")),
            ("is_waxed", Dynamic::bool(false)),
            ("keep_me", Dynamic::long(i64::MIN)),
        ]);
        let out = fix.fix(input);
        for field in FIELDS_TO_DROP {
            assert_eq!(out.get(field), Dynamic::null(), "field {field} should be removed");
        }
        assert!(out.get("front_text").get("filtered_messages").as_stream_opt().is_none());
        assert!(out.get("back_text").get("filtered_messages").as_stream_opt().is_none());
        assert_eq!(out.get("keep_me"), Dynamic::long(i64::MIN));
    }

    #[test]
    fn fix_preserves_float_bits_and_make_rule_description() {
        let fix = DropInvalidSignDataFix::new("sign".to_string());
        let bits = f64::from_bits(0x7FF8_0000_0000_0001u64); // NaN payload
        let input = dmap(vec![
            ("front_text", dmap(vec![("v", Dynamic::double(bits))])),
            ("back_text", dmap(vec![])),
        ]);
        let out = fix.fix(input);
        let v = out.get("front_text").get("v");
        match v.value {
            DynamicValue::Double(d) => assert_eq!(d.to_bits(), bits.to_bits()),
            _ => panic!("expected double"),
        }
        assert_eq!(fix.make_rule_description(), "DropInvalidSignDataFix for sign");
    }
}
