use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    Seq(Vec<Dynamic>),
    Str(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn remove(self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }
}

// JAVA: net.minecraft.util.datafix.LegacyComponentDataFixUtils#createPlainTextComponent
fn create_plain_text_component(text: &str) -> Dynamic {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    Dynamic::Str(format!("{{\"text\":\"{}\"}}", escaped))
}

pub struct BlockEntityCustomNameToComponentFix;

impl BlockEntityCustomNameToComponentFix {
    const NAMEABLE_BLOCK_ENTITIES: &[&str] = &[
        "minecraft:beacon",
        "minecraft:banner",
        "minecraft:brewing_stand",
        "minecraft:chest",
        "minecraft:trapped_chest",
        "minecraft:dispenser",
        "minecraft:dropper",
        "minecraft:enchanting_table",
        "minecraft:furnace",
        "minecraft:hopper",
        "minecraft:shulker_box",
    ];

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityCustomNameToComponentFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    fn is_nameable(id: &str) -> bool {
        Self::NAMEABLE_BLOCK_ENTITIES.contains(&id)
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityCustomNameToComponentFix#makeRule
    pub fn fix(&self, block_entity: Dynamic) -> Dynamic {
        let id = block_entity
            .get("id")
            .and_then(|d| d.as_string())
            .map(|s| s.to_string());
        match id {
            Some(ref id) if !Self::is_nameable(id) => block_entity,
            _ => Self::fix_tag_custom_name(block_entity),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityCustomNameToComponentFix#fixTagCustomName
    pub fn fix_tag_custom_name(d: Dynamic) -> Dynamic {
        let custom_name = d
            .get("CustomName")
            .and_then(|v| v.as_string())
            .unwrap_or("");
        if custom_name.is_empty() {
            d.remove("CustomName")
        } else {
            let component = create_plain_text_component(custom_name);
            d.set("CustomName", component)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    fn str_val(s: &str) -> Dynamic {
        Dynamic::Str(s.to_string())
    }

    #[test]
    fn nameable_block_entity_custom_name_converted() {
        let fix = BlockEntityCustomNameToComponentFix::new(());
        let input = map(vec![
            ("id", str_val("minecraft:chest")),
            ("CustomName", str_val("My Chest")),
        ]);
        let out = fix.fix(input);
        assert_eq!(
            out.get("CustomName"),
            Some(&str_val("{\"text\":\"My Chest\"}"))
        );
    }

    #[test]
    fn non_nameable_block_entity_unchanged() {
        let fix = BlockEntityCustomNameToComponentFix::new(());
        let input = map(vec![
            ("id", str_val("minecraft:sign")),
            ("CustomName", str_val("Hello")),
        ]);
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn empty_custom_name_removed() {
        let fix = BlockEntityCustomNameToComponentFix::new(());
        let input = map(vec![
            ("id", str_val("minecraft:chest")),
            ("CustomName", str_val("")),
        ]);
        let out = fix.fix(input);
        assert_eq!(out.get("CustomName"), None);
    }

    #[test]
    fn missing_id_applies_fix() {
        let fix = BlockEntityCustomNameToComponentFix::new(());
        let input = map(vec![("CustomName", str_val("Name"))]);
        let out = fix.fix(input);
        assert_eq!(
            out.get("CustomName"),
            Some(&str_val("{\"text\":\"Name\"}"))
        );
    }

    #[test]
    fn fix_tag_custom_name_escapes_special_chars() {
        let input = map(vec![("CustomName", str_val("A \"quote\" \\ slash"))]);
        let out = BlockEntityCustomNameToComponentFix::fix_tag_custom_name(input);
        assert_eq!(
            out.get("CustomName"),
            Some(&str_val("{\"text\":\"A \\\"quote\\\" \\\\ slash\"}"))
        );
    }
}
