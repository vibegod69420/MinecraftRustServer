use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn string(s: impl Into<String>) -> Self {
        Dynamic::String(s.into())
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                if let Some(child) = m.remove(key) {
                    m.insert(key.to_string(), f(child));
                    Dynamic::Map(m)
                } else {
                    Dynamic::Map(m)
                }
            }
            other => other,
        }
    }

    pub fn update_list<F: FnMut(Dynamic) -> Dynamic>(self, mut f: F) -> Self {
        match self {
            Dynamic::List(items) => Dynamic::List(items.into_iter().map(&mut f).collect()),
            other => other,
        }
    }
}

// Minimal self-contained stub for LegacyComponentDataFixUtils.createTextComponentJson.
// Wraps plain text as a JSON text component string.
// JAVA: net.minecraft.util.datafix.LegacyComponentDataFixUtils#createTextComponentJson
pub fn create_text_component_json(text: String) -> String {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"text\":\"{escaped}\"}}")
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct ItemLoreFix {
    pub schema: Schema,
}

impl ItemLoreFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemLoreFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemLoreFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic + '_ {
        move |data| {
            data.update("tag", |tag| {
                tag.update("display", |display| {
                    display.update("Lore", |lore| {
                        lore.update_list(|item| match item {
                            Dynamic::String(s) => {
                                Dynamic::String(create_text_component_json(s))
                            }
                            other => other,
                        })
                    })
                })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    fn lore_strings(d: &Dynamic) -> Vec<String> {
        match d {
            Dynamic::Map(m) => match m.get("tag") {
                Some(Dynamic::Map(tag)) => match tag.get("display") {
                    Some(Dynamic::Map(disp)) => match disp.get("Lore") {
                        Some(Dynamic::List(items)) => items
                            .iter()
                            .map(|i| match i {
                                Dynamic::String(s) => s.clone(),
                                other => panic!("not string: {other:?}"),
                            })
                            .collect(),
                        _ => panic!("missing Lore"),
                    },
                    _ => panic!("missing display"),
                },
                _ => panic!("missing tag"),
            },
            _ => panic!("not a map"),
        }
    }

    #[test]
    fn converts_lore_strings() {
        let fix = ItemLoreFix::new(Schema);
        let input = map(vec![(
            "tag",
            map(vec![(
                "display",
                map(vec![(
                    "Lore",
                    Dynamic::List(vec![
                        Dynamic::String("Hello".into()),
                        Dynamic::String("World".into()),
                    ]),
                )]),
            )]),
        )]);
        let out = fix.make_rule()(input);
        assert_eq!(
            lore_strings(&out),
            vec![
                "{\"text\":\"Hello\"}".to_string(),
                "{\"text\":\"World\"}".to_string()
            ]
        );
    }

    #[test]
    fn escapes_quotes_and_backslashes() {
        let s = create_text_component_json("a\"b\\c".to_string());
        assert_eq!(s, "{\"text\":\"a\\\"b\\\\c\"}");
    }

    #[test]
    fn missing_lore_is_noop() {
        let fix = ItemLoreFix::new(Schema);
        let input = map(vec![(
            "tag",
            map(vec![(
                "display",
                map(vec![("Name", Dynamic::string("x"))]),
            )]),
        )]);
        let out = fix.make_rule()(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_tag_is_noop() {
        let fix = ItemLoreFix::new(Schema);
        let input = map(vec![("id", Dynamic::string("minecraft:stone"))]);
        let out = fix.make_rule()(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_string_lore_elements_preserved() {
        let fix = ItemLoreFix::new(Schema);
        let input = map(vec![(
            "tag",
            map(vec![(
                "display",
                map(vec![(
                    "Lore",
                    Dynamic::List(vec![
                        Dynamic::Int(42),
                        Dynamic::String("ok".into()),
                        Dynamic::Bool(false),
                    ]),
                )]),
            )]),
        )]);
        let out = fix.make_rule()(input);
        match &out {
            Dynamic::Map(m) => match m.get("tag") {
                Some(Dynamic::Map(tag)) => match tag.get("display") {
                    Some(Dynamic::Map(disp)) => match disp.get("Lore") {
                        Some(Dynamic::List(items)) => {
                            assert_eq!(items[0], Dynamic::Int(42));
                            assert_eq!(items[1], Dynamic::String("{\"text\":\"ok\"}".into()));
                            assert_eq!(items[2], Dynamic::Bool(false));
                        }
                        _ => panic!("bad Lore"),
                    },
                    _ => panic!("bad display"),
                },
                _ => panic!("bad tag"),
            },
            _ => panic!("bad root"),
        }
    }
}
