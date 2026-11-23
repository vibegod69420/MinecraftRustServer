use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Self::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> Option<Self> {
        match self {
            Self::Map(map) => map.get(key).cloned(),
            _ => None,
        }
    }

    pub fn set(mut self, key: &str, value: Self) -> Self {
        if let Self::Map(map) = &mut self {
            map.insert(key.to_string(), value);
        }
        self
    }

    pub fn remove(mut self, key: &str) -> Self {
        if let Self::Map(map) = &mut self {
            map.remove(key);
        }
        self
    }

    pub fn rename_field(self, from: &str, to: &str) -> Self {
        if let Some(value) = self.get(from) {
            self.remove(from).set(to, value)
        } else {
            self
        }
    }

    pub fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if let Some(value) = self.get(key) {
            self.set(key, f(value))
        } else {
            self
        }
    }

    pub fn get_or_empty_map(&self, key: &str) -> Self {
        self.get(key).unwrap_or_else(Self::empty_map)
    }

    pub fn as_boolean(&self, default: bool) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Int(v) => *v != 0,
            Self::String(v) => v == "true",
            _ => default,
        }
    }

    pub fn as_string(&self, default: &str) -> String {
        match self {
            Self::String(v) => v.clone(),
            _ => default.to_string(),
        }
    }

    pub fn create_string(value: &str) -> Self {
        Self::String(value.to_string())
    }

    pub fn create_int(value: i32) -> Self {
        Self::Int(value)
    }

    pub fn create_float(value: f32) -> Self {
        Self::Float(value)
    }
}

pub const ENTITIES_WITH_SADDLE_ITEM: &[&str] = &[
    "minecraft:horse",
    "minecraft:skeleton_horse",
    "minecraft:zombie_horse",
    "minecraft:donkey",
    "minecraft:mule",
    "minecraft:camel",
    "minecraft:llama",
    "minecraft:trader_llama",
];

pub const ENTITIES_WITH_SADDLE_FLAG: &[&str] = &["minecraft:pig", "minecraft:strider"];

pub const SADDLE_FLAG: &str = "Saddle";
pub const NEW_SADDLE: &str = "saddle";

pub struct SaddleEquipmentSlotFix;

impl SaddleEquipmentSlotFix {
    pub fn new() -> Self {
        Self
    }

    pub fn fix_entity(&self, entity_type: &str, entity: Dynamic) -> Dynamic {
        if ENTITIES_WITH_SADDLE_ITEM.contains(&entity_type) {
            fix_entity_with_saddle_item(entity)
        } else if ENTITIES_WITH_SADDLE_FLAG.contains(&entity_type) {
            fix_entity_with_saddle_flag(entity)
        } else {
            entity
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.SaddleEquipmentSlotFix#fixEntityWithSaddleItem
pub fn fix_entity_with_saddle_item(entity: Dynamic) -> Dynamic {
    if entity.get("SaddleItem").is_none() {
        entity
    } else {
        fix_drop_chances(entity.rename_field("SaddleItem", NEW_SADDLE))
    }
}

// JAVA: net.minecraft.util.datafix.fixes.SaddleEquipmentSlotFix#fixEntityWithSaddleFlag
pub fn fix_entity_with_saddle_flag(entity: Dynamic) -> Dynamic {
    let saddle = entity
        .get(SADDLE_FLAG)
        .map(|value| value.as_boolean(false))
        .unwrap_or(false);
    let entity = entity.remove(SADDLE_FLAG);

    if !saddle {
        return entity;
    }

    let saddle_item = Dynamic::empty_map()
        .set("id", Dynamic::create_string("minecraft:saddle"))
        .set("count", Dynamic::create_int(1));

    fix_drop_chances(entity.set(NEW_SADDLE, saddle_item))
}

// JAVA: net.minecraft.util.datafix.fixes.SaddleEquipmentSlotFix#fixDropChances
pub fn fix_drop_chances(entity: Dynamic) -> Dynamic {
    let drop_chances = entity
        .get_or_empty_map("drop_chances")
        .set("saddle", Dynamic::create_float(2.0));
    entity.set("drop_chances", drop_chances)
}

pub fn is_entity_with_saddle_item(entity_type: &str) -> bool {
    ENTITIES_WITH_SADDLE_ITEM.contains(&entity_type)
}

pub fn is_entity_with_saddle_flag(entity_type: &str) -> bool {
    ENTITIES_WITH_SADDLE_FLAG.contains(&entity_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saddle_item_is_renamed_and_drop_chance_added() {
        let entity = Dynamic::empty_map()
            .set("SaddleItem", Dynamic::empty_map().set("id", Dynamic::create_string("minecraft:saddle")))
            .set("drop_chances", Dynamic::empty_map().set("mainhand", Dynamic::create_float(1.0)));

        let fixed = fix_entity_with_saddle_item(entity);
        assert!(fixed.get("SaddleItem").is_none());
        assert!(fixed.get("saddle").is_some());
        let drop = fixed.get("drop_chances").unwrap();
        assert_eq!(drop.get("saddle"), Some(Dynamic::Float(2.0)));
        assert_eq!(drop.get("mainhand"), Some(Dynamic::Float(1.0)));
    }

    #[test]
    fn saddle_item_absent_leaves_entity_unchanged() {
        let entity = Dynamic::empty_map().set("id", Dynamic::create_string("minecraft:horse"));
        assert_eq!(fix_entity_with_saddle_item(entity.clone()), entity);
    }

    #[test]
    fn saddle_flag_true_creates_saddle_item() {
        let entity = Dynamic::empty_map().set("Saddle", Dynamic::Bool(true));
        let fixed = fix_entity_with_saddle_flag(entity);
        assert!(fixed.get("Saddle").is_none());
        let saddle = fixed.get("saddle").unwrap();
        assert_eq!(saddle.get("id"), Some(Dynamic::create_string("minecraft:saddle")));
        assert_eq!(saddle.get("count"), Some(Dynamic::Int(1)));
        assert_eq!(
            fixed.get("drop_chances").unwrap().get("saddle"),
            Some(Dynamic::Float(2.0))
        );
    }

    #[test]
    fn saddle_flag_false_removes_flag_without_saddle() {
        let entity = Dynamic::empty_map().set("Saddle", Dynamic::Bool(false));
        let fixed = fix_entity_with_saddle_flag(entity);
        assert!(fixed.get("Saddle").is_none());
        assert!(fixed.get("saddle").is_none());
        assert!(fixed.get("drop_chances").is_none());
    }

    #[test]
    fn drop_chances_float_bits_are_preserved() {
        let entity = Dynamic::empty_map();
        let fixed = fix_drop_chances(entity);
        match fixed.get("drop_chances").unwrap().get("saddle").unwrap() {
            Dynamic::Float(value) => assert_eq!(value.to_bits(), 2.0f32.to_bits()),
            other => panic!("expected float, got {other:?}"),
        }
    }

    #[test]
    fn entity_dispatch_uses_sets() {
        let fix = SaddleEquipmentSlotFix::new();
        assert!(is_entity_with_saddle_item("minecraft:camel"));
        assert!(is_entity_with_saddle_flag("minecraft:strider"));
        let entity = Dynamic::empty_map().set("Saddle", Dynamic::Bool(true));
        assert!(fix.fix_entity("minecraft:strider", entity).get("saddle").is_some());
    }
}
