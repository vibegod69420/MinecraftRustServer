#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Null,
    Str(String),
    Map(Vec<(String, Dynamic)>),
}

static NULL_DYNAMIC: Dynamic = Dynamic::Null;

impl Dynamic {
    pub fn get(&self, key: &str) -> &Dynamic {
        match self {
            Dynamic::Map(entries) => entries
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .unwrap_or(&NULL_DYNAMIC),
            _ => &NULL_DYNAMIC,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

fn update_field(dynamic: &Dynamic, key: &str, f: &dyn Fn(&Dynamic) -> Dynamic) -> Dynamic {
    match dynamic {
        Dynamic::Map(entries) => {
            let mut found = false;
            let new_entries: Vec<(String, Dynamic)> = entries
                .iter()
                .map(|(k, v)| {
                    if k == key {
                        found = true;
                        (k.clone(), f(v))
                    } else {
                        (k.clone(), v.clone())
                    }
                })
                .collect();
            if found {
                Dynamic::Map(new_entries)
            } else {
                dynamic.clone()
            }
        }
        _ => dynamic.clone(),
    }
}

// JAVA: net.minecraft.util.datafix.LegacyComponentDataFixUtils#createTextComponentJson
pub fn create_text_component_json(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return text.to_string();
    }
    let mut escaped = String::with_capacity(text.len() + 16);
    escaped.push_str("{\"text\":\"");
    for c in text.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                escaped.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => escaped.push(c),
        }
    }
    escaped.push_str("\"}");
    escaped
}

pub struct ItemCustomNameToComponentFix;

impl ItemCustomNameToComponentFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemCustomNameToComponentFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemCustomNameToComponentFix#makeRule
    pub fn fix(&self, input: &Dynamic) -> Dynamic {
        update_field(input, "tag", &|tag| {
            update_field(tag, "display", &|display| {
                update_field(display, "Name", &|name| match name {
                    Dynamic::Str(s) => Dynamic::Str(create_text_component_json(s)),
                    _ => name.clone(),
                })
            })
        })
    }
}

impl Default for ItemCustomNameToComponentFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(name: &str) -> Dynamic {
        Dynamic::Map(vec![
            (
                "id".to_string(),
                Dynamic::Str("minecraft:diamond".to_string()),
            ),
            (
                "tag".to_string(),
                Dynamic::Map(vec![(
                    "display".to_string(),
                    Dynamic::Map(vec![(
                        "Name".to_string(),
                        Dynamic::Str(name.to_string()),
                    )]),
                )]),
            ),
        ])
    }

    #[test]
    fn test_plain_text_converted() {
        let fix = ItemCustomNameToComponentFix::new();
        let input = make_item("Hello");
        let output = fix.fix(&input);
        let name = output
            .get("tag")
            .get("display")
            .get("Name")
            .as_string()
            .unwrap();
        assert_eq!(name, "{\"text\":\"Hello\"}");
    }

    #[test]
    fn test_json_object_preserved() {
        let fix = ItemCustomNameToComponentFix::new();
        let input = make_item("{\"text\":\"Hi\",\"color\":\"red\"}");
        let output = fix.fix(&input);
        let name = output
            .get("tag")
            .get("display")
            .get("Name")
            .as_string()
            .unwrap();
        assert_eq!(name, "{\"text\":\"Hi\",\"color\":\"red\"}");
    }

    #[test]
    fn test_json_array_preserved() {
        let fix = ItemCustomNameToComponentFix::new();
        let input = make_item("[{\"text\":\"A\"}]");
        let output = fix.fix(&input);
        let name = output
            .get("tag")
            .get("display")
            .get("Name")
            .as_string()
            .unwrap();
        assert_eq!(name, "[{\"text\":\"A\"}]");
    }

    #[test]
    fn test_missing_fields_unchanged() {
        let fix = ItemCustomNameToComponentFix::new();
        let input = Dynamic::Map(vec![(
            "id".to_string(),
            Dynamic::Str("minecraft:stone".to_string()),
        )]);
        let output = fix.fix(&input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_escaped_characters() {
        let fix = ItemCustomNameToComponentFix::new();
        let input = make_item("Say \"hi\"\n\\path");
        let output = fix.fix(&input);
        let name = output
            .get("tag")
            .get("display")
            .get("Name")
            .as_string()
            .unwrap();
        assert_eq!(name, "{\"text\":\"Say \\\"hi\\\"\\n\\\\path\"}");
    }

    #[test]
    fn test_missing_display_unchanged() {
        let fix = ItemCustomNameToComponentFix::new();
        let input = Dynamic::Map(vec![(
            "tag".to_string(),
            Dynamic::Map(vec![("Damage".to_string(), Dynamic::Str("0".to_string()))]),
        )]);
        let output = fix.fix(&input);
        assert_eq!(output, input);
    }
}
