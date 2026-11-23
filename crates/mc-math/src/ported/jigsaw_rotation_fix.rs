use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix ports.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Null),
            _ => Dynamic::Null,
        }
    }

    pub fn as_string_or(&self, default: &str) -> String {
        match self {
            Dynamic::String(s) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub fn remove(self, key: &str) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }
}

fn rename(facing: &str) -> &str {
    match facing {
        "down" => "down_south",
        "up" => "up_north",
        "north" => "north_up",
        "south" => "south_up",
        "west" => "west_up",
        "east" => "east_up",
        _ => facing,
    }
}

pub struct JigsawRotationFix;

impl JigsawRotationFix {
    // JAVA: net.minecraft.util.datafix.fixes.JigsawRotationFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.JigsawRotationFix#shouldFix
    pub fn should_fix(&self, name: &str) -> bool {
        name == "minecraft:jigsaw"
    }

    // JAVA: net.minecraft.util.datafix.fixes.JigsawRotationFix#fixProperties
    pub fn fix_properties(&self, _name: &str, d: Dynamic) -> Dynamic {
        let facing = d.get("facing").as_string_or("north");
        let orientation = rename(&facing);
        let orientation_dyn = d.create_string(orientation);
        d.remove("facing").set("orientation", orientation_dyn)
    }
}

impl Default for JigsawRotationFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn should_fix_only_jigsaw() {
        let fix = JigsawRotationFix::new();
        assert!(fix.should_fix("minecraft:jigsaw"));
        assert!(!fix.should_fix("minecraft:stone"));
    }

    #[test]
    fn renames_known_facings() {
        let fix = JigsawRotationFix::new();
        let cases = [
            ("down", "down_south"),
            ("up", "up_north"),
            ("north", "north_up"),
            ("south", "south_up"),
            ("west", "west_up"),
            ("east", "east_up"),
        ];
        for (facing, expected) in cases {
            let input = map(vec![("facing", Dynamic::String(facing.into()))]);
            let out = fix.fix_properties("minecraft:jigsaw", input);
            assert_eq!(out.get("facing"), Dynamic::Null);
            assert_eq!(out.get("orientation"), Dynamic::String(expected.into()));
        }
    }

    #[test]
    fn unknown_facing_preserved_as_orientation() {
        let fix = JigsawRotationFix::new();
        let input = map(vec![("facing", Dynamic::String("diagonal".into()))]);
        let out = fix.fix_properties("minecraft:jigsaw", input);
        assert_eq!(out.get("orientation"), Dynamic::String("diagonal".into()));
    }

    #[test]
    fn missing_facing_defaults_to_north() {
        let fix = JigsawRotationFix::new();
        let input = map(vec![("other", Dynamic::Int(1))]);
        let out = fix.fix_properties("minecraft:jigsaw", input);
        assert_eq!(out.get("orientation"), Dynamic::String("north_up".into()));
        assert_eq!(out.get("other"), Dynamic::Int(1));
    }

    #[test]
    fn handles_non_map_dynamic() {
        let fix = JigsawRotationFix::new();
        let input = Dynamic::Bool(true);
        let out = fix.fix_properties("minecraft:jigsaw", input.clone());
        // remove/set on non-map return the original shape; orientation set is a no-op on non-map
        assert_eq!(out, input);
    }
}
