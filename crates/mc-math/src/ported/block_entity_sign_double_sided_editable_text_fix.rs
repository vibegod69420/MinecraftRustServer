use std::collections::BTreeMap;

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
];
pub const FILTERED_CORRECT: &str = "_filtered_correct";
const DEFAULT_COLOR: &str = "black";

#[derive(Clone, Copy, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Bool(bool),
    Int(i32),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key).cloned(),
            _ => None,
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.insert(key.to_string(), value);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => {
                let mut out = m.clone();
                out.remove(key);
                Dynamic::Map(out)
            }
            _ => self.clone(),
        }
    }

    pub fn create_boolean(&self, v: bool) -> Dynamic {
        Dynamic::Bool(v)
    }

    pub fn create_string(&self, v: &str) -> Dynamic {
        Dynamic::String(v.to_string())
    }

    pub fn create_list<I: IntoIterator<Item = Dynamic>>(&self, items: I) -> Dynamic {
        Dynamic::List(items.into_iter().collect())
    }

    pub fn empty_map(&self) -> Dynamic {
        Dynamic::Map(BTreeMap::new())
    }
}

fn create_empty_component(_d: &Dynamic) -> Dynamic {
    Dynamic::String(String::new())
}

#[derive(Clone, Debug, Default)]
pub struct BlockEntitySignDoubleSidedEditableTextFix {
    pub name: String,
    pub entity_name: String,
}

impl BlockEntitySignDoubleSidedEditableTextFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntitySignDoubleSidedEditableTextFix#<init>
    pub fn new(_schema: Schema, name: &str, entity_name: &str) -> Self {
        Self {
            name: name.to_string(),
            entity_name: entity_name.to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntitySignDoubleSidedEditableTextFix#fix
    pub fn fix(&self, d: Dynamic) -> Dynamic {
        let mut out = d
            .set("front_text", fix_front_text_tag(&d))
            .set("back_text", create_default_text(&d))
            .set("is_waxed", d.create_boolean(false))
            .set(FILTERED_CORRECT, d.create_boolean(true));
        for key in FIELDS_TO_DROP {
            out = out.remove(key);
        }
        out
    }
}

// JAVA: net.minecraft.util.datafix.fixes.BlockEntitySignDoubleSidedEditableTextFix#fixFrontTextTag
fn fix_front_text_tag(d: &Dynamic) -> Dynamic {
    let empty = create_empty_component(d);
    let messages: Vec<Dynamic> = get_lines(d, "Text")
        .into_iter()
        .map(|opt| opt.unwrap_or_else(|| empty.clone()))
        .collect();
    let mut front = d
        .empty_map()
        .set("messages", d.create_list(messages.clone()))
        .set(
            "color",
            d.get("Color")
                .unwrap_or_else(|| d.create_string(DEFAULT_COLOR)),
        )
        .set(
            "has_glowing_text",
            d.get("GlowingText")
                .unwrap_or_else(|| d.create_boolean(false)),
        );
    let filtered: Vec<Option<Dynamic>> = get_lines(d, "FilteredText");
    if filtered.iter().any(|o| o.is_some()) {
        let filtered_messages: Vec<Dynamic> = filtered
            .into_iter()
            .enumerate()
            .map(|(i, opt)| opt.unwrap_or_else(|| messages[i].clone()))
            .collect();
        front = front.set("filtered_messages", d.create_list(filtered_messages));
    }
    front
}

// JAVA: net.minecraft.util.datafix.fixes.BlockEntitySignDoubleSidedEditableTextFix#getLines
fn get_lines(d: &Dynamic, prefix: &str) -> Vec<Option<Dynamic>> {
    vec![
        d.get(&format!("{prefix}1")),
        d.get(&format!("{prefix}2")),
        d.get(&format!("{prefix}3")),
        d.get(&format!("{prefix}4")),
    ]
}

// JAVA: net.minecraft.util.datafix.fixes.BlockEntitySignDoubleSidedEditableTextFix#createDefaultText
fn create_default_text(d: &Dynamic) -> Dynamic {
    d.empty_map()
        .set("messages", create_empty_lines(d))
        .set("color", d.create_string(DEFAULT_COLOR))
        .set("has_glowing_text", d.create_boolean(false))
}

// JAVA: net.minecraft.util.datafix.fixes.BlockEntitySignDoubleSidedEditableTextFix#createEmptyLines
fn create_empty_lines(d: &Dynamic) -> Dynamic {
    let empty = create_empty_component(d);
    d.create_list(vec![empty.clone(), empty.clone(), empty.clone(), empty])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn fix_moves_front_text_and_sets_defaults() {
        let fix =
            BlockEntitySignDoubleSidedEditableTextFix::new(Schema, "SignFix", "minecraft:sign");
        let input = map(vec![
            ("Text1", Dynamic::String("\"a\"".to_string())),
            ("Text2", Dynamic::String("\"b\"".to_string())),
            ("Text3", Dynamic::String("\"c\"".to_string())),
            ("Text4", Dynamic::String("\"d\"".to_string())),
            ("Color", Dynamic::String("red".to_string())),
            ("GlowingText", Dynamic::Bool(true)),
            ("keep_neg", Dynamic::Int(-7)),
        ]);
        let out = fix.fix(input);
        let front = out.get("front_text").unwrap();
        assert_eq!(
            front.get("messages"),
            Some(Dynamic::List(vec![
                Dynamic::String("\"a\"".to_string()),
                Dynamic::String("\"b\"".to_string()),
                Dynamic::String("\"c\"".to_string()),
                Dynamic::String("\"d\"".to_string()),
            ]))
        );
        assert_eq!(front.get("color"), Some(Dynamic::String("red".to_string())));
        assert_eq!(front.get("has_glowing_text"), Some(Dynamic::Bool(true)));
        let back = out.get("back_text").unwrap();
        assert_eq!(back.get("color"), Some(Dynamic::String("black".to_string())));
        assert_eq!(back.get("has_glowing_text"), Some(Dynamic::Bool(false)));
        assert_eq!(out.get("is_waxed"), Some(Dynamic::Bool(false)));
        assert_eq!(out.get("_filtered_correct"), Some(Dynamic::Bool(true)));
        assert_eq!(out.get("keep_neg"), Some(Dynamic::Int(-7)));
    }

    #[test]
    fn fix_includes_filtered_messages_with_fallback() {
        let fix =
            BlockEntitySignDoubleSidedEditableTextFix::new(Schema, "SignFix", "minecraft:sign");
        let input = map(vec![
            ("Text1", Dynamic::String("a".to_string())),
            ("Text2", Dynamic::String("b".to_string())),
            ("Text3", Dynamic::String("c".to_string())),
            ("Text4", Dynamic::String("d".to_string())),
            ("FilteredText1", Dynamic::String("fa".to_string())),
            ("FilteredText3", Dynamic::String("fc".to_string())),
        ]);
        let out = fix.fix(input);
        let front = out.get("front_text").unwrap();
        assert_eq!(
            front.get("filtered_messages"),
            Some(Dynamic::List(vec![
                Dynamic::String("fa".to_string()),
                Dynamic::String("b".to_string()),
                Dynamic::String("fc".to_string()),
                Dynamic::String("d".to_string()),
            ]))
        );
    }

    #[test]
    fn fix_drops_legacy_fields_and_keeps_others() {
        let fix =
            BlockEntitySignDoubleSidedEditableTextFix::new(Schema, "SignFix", "minecraft:sign");
        let input = map(vec![
            ("Text1", Dynamic::String("x".to_string())),
            ("Color", Dynamic::String("blue".to_string())),
            ("GlowingText", Dynamic::Bool(true)),
            ("keep", Dynamic::Int(i32::MIN)),
        ]);
        let out = fix.fix(input);
        for key in FIELDS_TO_DROP {
            assert!(out.get(key).is_none(), "field {key} should be removed");
        }
        assert_eq!(out.get("keep"), Some(Dynamic::Int(i32::MIN)));
    }

    #[test]
    fn fix_defaults_for_missing_fields_and_empty_lines() {
        let fix =
            BlockEntitySignDoubleSidedEditableTextFix::new(Schema, "SignFix", "minecraft:sign");
        let out = fix.fix(map(vec![]));
        let front = out.get("front_text").unwrap();
        assert_eq!(front.get("color"), Some(Dynamic::String("black".to_string())));
        assert_eq!(front.get("has_glowing_text"), Some(Dynamic::Bool(false)));
        assert_eq!(
            front.get("messages"),
            Some(Dynamic::List(vec![
                Dynamic::String(String::new()),
                Dynamic::String(String::new()),
                Dynamic::String(String::new()),
                Dynamic::String(String::new()),
            ]))
        );
        assert!(front.get("filtered_messages").is_none());
        let back = out.get("back_text").unwrap();
        assert_eq!(
            back.get("messages"),
            Some(Dynamic::List(vec![
                Dynamic::String(String::new()),
                Dynamic::String(String::new()),
                Dynamic::String(String::new()),
                Dynamic::String(String::new()),
            ]))
        );
    }
}
