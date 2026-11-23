// JAVA: net.minecraft.util.datafix.fixes.BannerPatternFormatFix#class
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn set(mut self, key: &str, value: Dynamic) -> Self {
        if let Dynamic::Map(m) = &mut self {
            m.insert(key.to_string(), value);
        }
        self
    }

    pub fn remove(mut self, key: &str) -> Self {
        if let Dynamic::Map(m) = &mut self {
            m.remove(key);
        }
        self
    }

    pub fn rename_and_fix_field<F>(mut self, old: &str, new: &str, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        if let Dynamic::Map(m) = &mut self {
            if let Some(v) = m.remove(old) {
                m.insert(new.to_string(), f(v));
            }
        }
        self
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(x) => *x as i32,
            Dynamic::Str(s) => s.parse().unwrap_or(default),
            _ => default,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Dynamic::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn create_string(s: impl Into<String>) -> Self {
        Dynamic::Str(s.into())
    }

    pub fn create_list(items: Vec<Dynamic>) -> Self {
        Dynamic::List(items)
    }
}

pub fn dye_color_id_to_name(id: i32) -> &'static str {
    match id {
        0 => "white",
        1 => "orange",
        2 => "magenta",
        3 => "light_blue",
        4 => "yellow",
        5 => "lime",
        6 => "pink",
        7 => "gray",
        8 => "light_gray",
        9 => "cyan",
        10 => "purple",
        11 => "blue",
        12 => "brown",
        13 => "green",
        14 => "red",
        15 => "black",
        _ => "white",
    }
}

fn pattern_id_map() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("b".to_string(), "minecraft:base".to_string()),
        ("bl".to_string(), "minecraft:square_bottom_left".to_string()),
        ("br".to_string(), "minecraft:square_bottom_right".to_string()),
        ("tl".to_string(), "minecraft:square_top_left".to_string()),
        ("tr".to_string(), "minecraft:square_top_right".to_string()),
        ("bs".to_string(), "minecraft:stripe_bottom".to_string()),
        ("ts".to_string(), "minecraft:stripe_top".to_string()),
        ("ls".to_string(), "minecraft:stripe_left".to_string()),
        ("rs".to_string(), "minecraft:stripe_right".to_string()),
        ("cs".to_string(), "minecraft:stripe_center".to_string()),
        ("ms".to_string(), "minecraft:stripe_middle".to_string()),
        ("drs".to_string(), "minecraft:stripe_downright".to_string()),
        ("dls".to_string(), "minecraft:stripe_downleft".to_string()),
        ("ss".to_string(), "minecraft:small_stripes".to_string()),
        ("cr".to_string(), "minecraft:cross".to_string()),
        ("sc".to_string(), "minecraft:straight_cross".to_string()),
        ("bt".to_string(), "minecraft:triangle_bottom".to_string()),
        ("tt".to_string(), "minecraft:triangle_top".to_string()),
        ("bts".to_string(), "minecraft:triangles_bottom".to_string()),
        ("tts".to_string(), "minecraft:triangles_top".to_string()),
        ("ld".to_string(), "minecraft:diagonal_left".to_string()),
        ("rd".to_string(), "minecraft:diagonal_up_right".to_string()),
        ("lud".to_string(), "minecraft:diagonal_up_left".to_string()),
        ("rud".to_string(), "minecraft:diagonal_right".to_string()),
        ("mc".to_string(), "minecraft:circle".to_string()),
        ("mr".to_string(), "minecraft:rhombus".to_string()),
        ("vh".to_string(), "minecraft:half_vertical".to_string()),
        ("hh".to_string(), "minecraft:half_horizontal".to_string()),
        ("vhr".to_string(), "minecraft:half_vertical_right".to_string()),
        ("hhb".to_string(), "minecraft:half_horizontal_bottom".to_string()),
        ("bo".to_string(), "minecraft:border".to_string()),
        ("cbo".to_string(), "minecraft:curly_border".to_string()),
        ("gra".to_string(), "minecraft:gradient".to_string()),
        ("gru".to_string(), "minecraft:gradient_up".to_string()),
        ("bri".to_string(), "minecraft:bricks".to_string()),
        ("glb".to_string(), "minecraft:globe".to_string()),
        ("cre".to_string(), "minecraft:creeper".to_string()),
        ("sku".to_string(), "minecraft:skull".to_string()),
        ("flo".to_string(), "minecraft:flower".to_string()),
        ("moj".to_string(), "minecraft:mojang".to_string()),
        ("pig".to_string(), "minecraft:piglin".to_string()),
    ])
}

pub fn fix_tag(tag: Dynamic) -> Dynamic {
    tag.rename_and_fix_field("Patterns", "patterns", |value| {
        if let Dynamic::List(items) = value {
            Dynamic::create_list(items.into_iter().map(fix_layer).collect())
        } else {
            value
        }
    })
}

pub fn fix_layer(mut layer: Dynamic) -> Dynamic {
    layer = layer.rename_and_fix_field("Pattern", "pattern", |value| {
        if let Dynamic::Str(s) = value {
            let mapped = pattern_id_map().get(&s).cloned().unwrap_or_else(|| s.clone());
            Dynamic::create_string(mapped)
        } else {
            value
        }
    });

    let color = layer.get("Color").map(|d| d.as_int(0)).unwrap_or(0);
    layer = layer.set("color", Dynamic::create_string(dye_color_id_to_name(color)));
    layer.remove("Color")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_map_contains_known_entry() {
        assert_eq!(pattern_id_map().get("pig"), Some(&"minecraft:piglin".to_string()));
    }

    #[test]
    fn unknown_pattern_stays_same() {
        let mut layer = Dynamic::Map(BTreeMap::new());
        layer = layer.set("Pattern", Dynamic::create_string("zzz"));
        let fixed = fix_layer(layer);
        assert_eq!(fixed.get("pattern").unwrap().as_str(), Some("zzz"));
    }

    #[test]
    fn fixes_pattern_and_color() {
        let mut layer = Dynamic::Map(BTreeMap::new());
        layer = layer.set("Pattern", Dynamic::create_string("b"));
        layer = layer.set("Color", Dynamic::Int(14));
        let fixed = fix_layer(layer);
        assert_eq!(fixed.get("pattern").unwrap().as_str(), Some("minecraft:base"));
        assert_eq!(fixed.get("color").unwrap().as_str(), Some("red"));
        assert!(fixed.get("Color").is_none());
    }

    #[test]
    fn fixes_patterns_list() {
        let mut layer = Dynamic::Map(BTreeMap::new());
        layer = layer.set("Pattern", Dynamic::create_string("flo"));
        layer = layer.set("Color", Dynamic::Int(5));
        let tag = Dynamic::Map(BTreeMap::from([(
            "Patterns".to_string(),
            Dynamic::create_list(vec![layer]),
        )]));
        let fixed = fix_tag(tag);
        assert!(fixed.get("Patterns").is_none());
        let patterns = fixed.get("patterns").unwrap();
        if let Dynamic::List(items) = patterns {
            assert_eq!(items[0].get("pattern").unwrap().as_str(), Some("minecraft:flower"));
            assert_eq!(items[0].get("color").unwrap().as_str(), Some("lime"));
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn dye_color_mapping_edges() {
        assert_eq!(dye_color_id_to_name(0), "white");
        assert_eq!(dye_color_id_to_name(15), "black");
        assert_eq!(dye_color_id_to_name(999), "white");
    }
}
