use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn null() -> Self {
        Self::Null
    }

    pub fn int(value: i64) -> Self {
        Self::Int(value)
    }

    pub fn float(value: f64) -> Self {
        Self::Float(value)
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn list(values: Vec<Dynamic>) -> Self {
        Self::List(values)
    }

    pub fn map(values: BTreeMap<String, Dynamic>) -> Self {
        Self::Map(values)
    }

    pub fn get(&self, key: &str) -> Self {
        match self {
            Self::Map(map) => map.get(key).cloned().unwrap_or(Self::Null),
            _ => Self::Null,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Self::Int(value) => *value as i32,
            Self::Float(value) => *value as i32,
            Self::String(value) => value.parse::<i32>().unwrap_or(default),
            Self::Bool(value) => {
                if *value {
                    1
                } else {
                    0
                }
            }
            _ => default,
        }
    }

    pub fn asInt(&self, default: i32) -> i32 {
        self.as_int(default)
    }

    pub fn as_stream_opt(&self) -> Option<Vec<Dynamic>> {
        match self {
            Self::List(values) => Some(values.clone()),
            _ => None,
        }
    }

    pub fn asStreamOpt(&self) -> Option<Vec<Dynamic>> {
        self.as_stream_opt()
    }

    pub fn set(&self, key: &str, value: Self) -> Self {
        match self {
            Self::Map(map) => {
                let mut map = map.clone();
                map.insert(key.to_string(), value);
                Self::Map(map)
            }
            _ => {
                let mut map = BTreeMap::new();
                map.insert(key.to_string(), value);
                Self::Map(map)
            }
        }
    }

    pub fn remove(&self, key: &str) -> Self {
        match self {
            Self::Map(map) => {
                let mut map = map.clone();
                map.remove(key);
                Self::Map(map)
            }
            _ => self.clone(),
        }
    }

    pub fn update<F>(&self, key: &str, f: F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        match self {
            Self::Map(map) => {
                if map.contains_key(key) {
                    let mut map = map.clone();
                    let current = map.get(key).cloned().unwrap_or(Self::Null);
                    map.insert(key.to_string(), f(current));
                    Self::Map(map)
                } else {
                    self.clone()
                }
            }
            _ => self.clone(),
        }
    }

    pub fn create_string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn createString(value: impl Into<String>) -> Self {
        Self::create_string(value)
    }

    pub fn create_string_on(&self, value: impl Into<String>) -> Self {
        Self::create_string(value)
    }

    pub fn create_list(values: Vec<Dynamic>) -> Self {
        Self::List(values)
    }

    pub fn createList(values: Vec<Dynamic>) -> Self {
        Self::create_list(values)
    }

    pub fn create_list_on(&self, values: Vec<Dynamic>) -> Self {
        Self::create_list(values)
    }
}

// Free-function aliases, matching both Rust and Java-style call sites.
pub fn create_string(value: impl Into<String>) -> Dynamic {
    Dynamic::create_string(value)
}

pub fn createString(value: impl Into<String>) -> Dynamic {
    Dynamic::create_string(value)
}

pub fn create_list(values: Vec<Dynamic>) -> Dynamic {
    Dynamic::create_list(values)
}

pub fn createList(values: Vec<Dynamic>) -> Dynamic {
    Dynamic::create_list(values)
}

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub id: i32,
}

#[derive(Clone, Debug)]
pub struct ItemStackEnchantmentNamesFix {
    input_schema: Schema,
    changes_type: bool,
}

impl ItemStackEnchantmentNamesFix {
    pub fn new(input_schema: Schema, changes_type: bool) -> Self {
        Self {
            input_schema,
            changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackEnchantmentNamesFix#fixTag
    pub fn fix_tag(&self, tag: Dynamic) -> Dynamic {
        fix_tag(tag)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackEnchantmentNamesFix#fixTag
    pub fn fixTag(&self, tag: Dynamic) -> Dynamic {
        fix_tag(tag)
    }
}

pub const MAP_ENTRIES: &[(i32, &str)] = &[
    (0, "minecraft:protection"),
    (1, "minecraft:fire_protection"),
    (2, "minecraft:feather_falling"),
    (3, "minecraft:blast_protection"),
    (4, "minecraft:projectile_protection"),
    (5, "minecraft:respiration"),
    (6, "minecraft:aqua_affinity"),
    (7, "minecraft:thorns"),
    (8, "minecraft:depth_strider"),
    (9, "minecraft:frost_walker"),
    (10, "minecraft:binding_curse"),
    (16, "minecraft:sharpness"),
    (17, "minecraft:smite"),
    (18, "minecraft:bane_of_arthropods"),
    (19, "minecraft:knockback"),
    (20, "minecraft:fire_aspect"),
    (21, "minecraft:looting"),
    (22, "minecraft:sweeping"),
    (32, "minecraft:efficiency"),
    (33, "minecraft:silk_touch"),
    (34, "minecraft:unbreaking"),
    (35, "minecraft:fortune"),
    (48, "minecraft:power"),
    (49, "minecraft:punch"),
    (50, "minecraft:flame"),
    (51, "minecraft:infinity"),
    (61, "minecraft:luck_of_the_sea"),
    (62, "minecraft:lure"),
    (65, "minecraft:loyalty"),
    (66, "minecraft:impaling"),
    (67, "minecraft:riptide"),
    (68, "minecraft:channeling"),
    (70, "minecraft:mending"),
    (71, "minecraft:vanishing_curse"),
];

pub fn enchantment_name(id: i32) -> Option<&'static str> {
    MAP_ENTRIES
        .iter()
        .find(|(key, _)| *key == id)
        .map(|(_, value)| *value)
}

// JAVA: net.minecraft.util.datafix.fixes.ItemStackEnchantmentNamesFix#fixTag
pub fn fix_tag(tag: Dynamic) -> Dynamic {
    let mut tag = tag;

    let ench = tag.get("ench").as_stream_opt().map(|stream| {
        stream
            .into_iter()
            .map(|entry| {
                let id = entry.get("id").as_int(0);
                let name = enchantment_name(id).unwrap_or("null");
                entry.set("id", Dynamic::create_string(name))
            })
            .collect::<Vec<_>>()
    });

    if let Some(new_enchantments) = ench {
        tag = tag.remove("ench");
        tag = tag.set("Enchantments", Dynamic::create_list(new_enchantments));
    }

    tag.update("StoredEnchantments", |stored| {
        if let Some(stream) = stored.as_stream_opt() {
            let new_stored = stream
                .into_iter()
                .map(|entry| {
                    let id = entry.get("id").as_int(0);
                    let name = enchantment_name(id).unwrap_or("null");
                    entry.set("id", Dynamic::create_string(name))
                })
                .collect::<Vec<_>>();
            Dynamic::create_list(new_stored)
        } else {
            stored
        }
    })
}

// JAVA: net.minecraft.util.datafix.fixes.ItemStackEnchantmentNamesFix#fixTag
pub fn fixTag(tag: Dynamic) -> Dynamic {
    fix_tag(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ench(id: i32) -> Dynamic {
        let mut map = BTreeMap::new();
        map.insert("id".to_string(), Dynamic::Int(id as i64));
        Dynamic::Map(map)
    }

    #[test]
    fn converts_ench_list() {
        let mut tag_map = BTreeMap::new();
        tag_map.insert(
            "ench".to_string(),
            Dynamic::List(vec![ench(0), ench(16)]),
        );
        let tag = Dynamic::Map(tag_map);

        let fixed = fix_tag(tag);
        assert_eq!(fixed.get("ench"), Dynamic::Null);

        let enchantments = fixed.get("Enchantments");
        let list = enchantments.as_stream_opt().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].get("id"), Dynamic::String("minecraft:protection".to_string()));
        assert_eq!(list[1].get("id"), Dynamic::String("minecraft:sharpness".to_string()));
    }

    #[test]
    fn converts_stored_enchantments() {
        let mut tag_map = BTreeMap::new();
        tag_map.insert(
            "StoredEnchantments".to_string(),
            Dynamic::List(vec![ench(70), ench(999)]),
        );
        let tag = Dynamic::Map(tag_map);

        let fixed = fix_tag(tag);
        let stored = fixed.get("StoredEnchantments").as_stream_opt().unwrap();
        assert_eq!(stored[0].get("id"), Dynamic::String("minecraft:mending".to_string()));
        assert_eq!(stored[1].get("id"), Dynamic::String("null".to_string()));
    }

    #[test]
    fn leaves_absent_fields_absent() {
        let tag = Dynamic::Map(BTreeMap::new());
        let fixed = fix_tag(tag.clone());
        assert_eq!(fixed, tag);
        assert_eq!(fixed.get("Enchantments"), Dynamic::Null);
        assert_eq!(fixed.get("StoredEnchantments"), Dynamic::Null);
    }

    #[test]
    fn as_int_handles_negative_and_strings() {
        assert_eq!(Dynamic::String("-7".to_string()).as_int(0), -7);
        assert_eq!(Dynamic::Float(-3.9).as_int(0), -3);
        assert_eq!(Dynamic::Null.as_int(42), 42);
    }

    #[test]
    fn unknown_id_becomes_null_string() {
        let mut tag_map = BTreeMap::new();
        tag_map.insert("ench".to_string(), Dynamic::List(vec![ench(12345)]));
        let fixed = fix_tag(Dynamic::Map(tag_map));
        let list = fixed.get("Enchantments").as_stream_opt().unwrap();
        assert_eq!(list[0].get("id"), Dynamic::String("null".to_string()));
    }
}
