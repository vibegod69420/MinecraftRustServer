use std::collections::BTreeMap;

/// Minimal dynamic value for datafix porting.
#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Null,
    Int(i32),
    Str(String),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> &Dynamic {
        static NULL: Dynamic = Dynamic::Null;
        match self {
            Dynamic::Map(m) => m.get(key).unwrap_or(&NULL),
            _ => &NULL,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            _ => default,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            _ => {
                let mut m = BTreeMap::new();
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
        }
    }
}

/// Simplified item stack representation for the fix.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackData {
    pub id: Option<String>,
    pub damage: i32,
    pub tag: Option<Dynamic>,
}

/// Port of `net.minecraft.util.datafix.fixes.ItemStackMapIdFix`.
pub struct ItemStackMapIdFix;

impl ItemStackMapIdFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackMapIdFix#<init>
    pub fn new(_changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackMapIdFix#makeRule
    pub fn apply(&self, stack: &ItemStackData) -> ItemStackData {
        let is_filled_map = match &stack.id {
            Some(id) => id == "minecraft:filled_map",
            None => false,
        };
        if !is_filled_map {
            return stack.clone();
        }
        let tag = stack.tag.clone().unwrap_or_else(Dynamic::empty_map);
        let tag = tag.set("map", Dynamic::Int(stack.damage));
        ItemStackData {
            id: stack.id.clone(),
            damage: stack.damage,
            tag: Some(tag),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn filled_map_sets_map_from_damage() {
        let fix = ItemStackMapIdFix::new(false);
        let stack = ItemStackData {
            id: Some("minecraft:filled_map".to_string()),
            damage: 42,
            tag: None,
        };
        let out = fix.apply(&stack);
        assert_eq!(out.tag.as_ref().unwrap().get("map").as_int(-1), 42);
    }

    #[test]
    fn filled_map_preserves_existing_tag_fields() {
        let fix = ItemStackMapIdFix::new(true);
        let mut tag_m = BTreeMap::new();
        tag_m.insert("display".to_string(), Dynamic::Str("name".to_string()));
        let stack = ItemStackData {
            id: Some("minecraft:filled_map".to_string()),
            damage: 7,
            tag: Some(Dynamic::Map(tag_m)),
        };
        let out = fix.apply(&stack);
        let tag = out.tag.unwrap();
        assert_eq!(tag.get("map").as_int(-1), 7);
        match tag.get("display") {
            Dynamic::Str(s) => assert_eq!(s, "name"),
            _ => panic!("display field lost"),
        }
    }

    #[test]
    fn non_filled_map_unchanged() {
        let fix = ItemStackMapIdFix::new(false);
        let stack = ItemStackData {
            id: Some("minecraft:stone".to_string()),
            damage: 5,
            tag: None,
        };
        let out = fix.apply(&stack);
        assert_eq!(out, stack);
    }

    #[test]
    fn missing_id_unchanged() {
        let fix = ItemStackMapIdFix::new(false);
        let stack = ItemStackData {
            id: None,
            damage: 3,
            tag: None,
        };
        let out = fix.apply(&stack);
        assert_eq!(out, stack);
    }

    #[test]
    fn negative_and_extreme_damage() {
        let fix = ItemStackMapIdFix::new(false);
        for dmg in [0, -1, i32::MIN, i32::MAX] {
            let stack = ItemStackData {
                id: Some("minecraft:filled_map".to_string()),
                damage: dmg,
                tag: None,
            };
            let out = fix.apply(&stack);
            assert_eq!(out.tag.unwrap().get("map").as_int(999), dmg);
        }
    }
}
