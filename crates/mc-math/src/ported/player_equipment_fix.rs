use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix logic.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Missing,
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn str(s: &str) -> Self {
        Dynamic::String(s.to_string())
    }

    pub fn object(entries: &[(&str, Dynamic)]) -> Self {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        Dynamic::Map(m)
    }

    pub fn get(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(m) => m.get(key).cloned().unwrap_or(Dynamic::Missing),
            _ => Dynamic::Missing,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Dynamic::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_int_or(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v as i32,
            _ => default,
        }
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, Dynamic::Missing)
    }

    pub fn as_list(&self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(v) => v.clone(),
            _ => Vec::new(),
        }
    }

    pub fn as_map(&self) -> BTreeMap<String, Dynamic> {
        match self {
            Dynamic::Map(m) => m.clone(),
            _ => BTreeMap::new(),
        }
    }

    pub fn as_stream(&self) -> Vec<Dynamic> {
        match self {
            Dynamic::List(v) => v.clone(),
            _ => Vec::new(),
        }
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Dynamic {
        Dynamic::List(items)
    }

    pub fn create_map(&self, entries: Vec<(Dynamic, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            if let Dynamic::String(s) = k {
                m.insert(s, v);
            }
        }
        Dynamic::Map(m)
    }

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                let old = m.remove(key).unwrap_or(Dynamic::Missing);
                let new_val = f(old);
                if !matches!(new_val, Dynamic::Missing) {
                    m.insert(key.to_string(), new_val);
                }
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

    pub fn remove(self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.PlayerEquipmentFix#SLOT_TRANSLATIONS
fn slot_translation(slot: i32) -> Option<&'static str> {
    match slot {
        100 => Some("feet"),
        101 => Some("legs"),
        102 => Some("chest"),
        103 => Some("head"),
        -106 => Some("offhand"),
        _ => None,
    }
}

pub struct PlayerEquipmentFix;

impl PlayerEquipmentFix {
    // JAVA: net.minecraft.util.datafix.fixes.PlayerEquipmentFix#PlayerEquipmentFix
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.PlayerEquipmentFix#makeRule
    pub fn apply(&self, player: Dynamic) -> Dynamic {
        let mut equipment: Vec<(Dynamic, Dynamic)> = Vec::new();
        let player = player.update("Inventory", |inv| {
            let items = inv.as_stream();
            let kept: Vec<Dynamic> = items
                .into_iter()
                .filter(|item| {
                    let slot = item.get("Slot").as_int_or(-1);
                    if let Some(name) = slot_translation(slot) {
                        equipment.push((inv.create_string(name), item.clone().remove("Slot")));
                        false
                    } else {
                        true
                    }
                })
                .collect();
            inv.create_list(kept)
        });
        let eq_map = player.create_map(equipment);
        player.set("equipment", eq_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(slot: i32, id: &str) -> Dynamic {
        Dynamic::object(&[
            ("Slot", Dynamic::Int(slot as i64)),
            ("id", Dynamic::str(id)),
        ])
    }

    #[test]
    fn moves_equipment_slots() {
        let fix = PlayerEquipmentFix::new(());
        let player = Dynamic::object(&[(
            "Inventory",
            Dynamic::List(vec![
                item(100, "iron_boots"),
                item(101, "iron_leggings"),
                item(102, "iron_chestplate"),
                item(103, "iron_helmet"),
                item(-106, "shield"),
                item(0, "stone"),
            ]),
        )]);
        let out = fix.apply(player);
        let inv = out.get("Inventory").as_list();
        assert_eq!(inv.len(), 1);
        assert_eq!(inv[0].get("id").as_string().as_deref(), Some("stone"));
        assert_eq!(inv[0].get("Slot").as_int_or(-999), 0);

        let eq = out.get("equipment");
        assert_eq!(
            eq.get("feet").get("id").as_string().as_deref(),
            Some("iron_boots")
        );
        assert!(eq.get("feet").get("Slot").is_missing());
        assert_eq!(
            eq.get("legs").get("id").as_string().as_deref(),
            Some("iron_leggings")
        );
        assert_eq!(
            eq.get("chest").get("id").as_string().as_deref(),
            Some("iron_chestplate")
        );
        assert_eq!(
            eq.get("head").get("id").as_string().as_deref(),
            Some("iron_helmet")
        );
        assert_eq!(
            eq.get("offhand").get("id").as_string().as_deref(),
            Some("shield")
        );
    }

    #[test]
    fn no_equipment_slots_gives_empty_equipment() {
        let fix = PlayerEquipmentFix::new(());
        let player = Dynamic::object(&[(
            "Inventory",
            Dynamic::List(vec![item(0, "stone"), item(1, "dirt")]),
        )]);
        let out = fix.apply(player);
        assert_eq!(out.get("Inventory").as_list().len(), 2);
        assert!(out.get("equipment").as_map().is_empty());
    }

    #[test]
    fn missing_slot_defaults_to_minus_one() {
        let fix = PlayerEquipmentFix::new(());
        let player = Dynamic::object(&[(
            "Inventory",
            Dynamic::List(vec![Dynamic::object(&[("id", Dynamic::str("stone"))])]),
        )]);
        let out = fix.apply(player);
        assert_eq!(out.get("Inventory").as_list().len(), 1);
        assert!(out.get("equipment").as_map().is_empty());
    }

    #[test]
    fn negative_slot_offhand() {
        let fix = PlayerEquipmentFix::new(());
        let player = Dynamic::object(&[(
            "Inventory",
            Dynamic::List(vec![item(-106, "totem")]),
        )]);
        let out = fix.apply(player);
        assert!(out.get("Inventory").as_list().is_empty());
        assert_eq!(
            out.get("equipment")
                .get("offhand")
                .get("id")
                .as_string()
                .as_deref(),
            Some("totem")
        );
    }

    #[test]
    fn duplicate_equipment_slot_last_wins() {
        let fix = PlayerEquipmentFix::new(());
        let player = Dynamic::object(&[(
            "Inventory",
            Dynamic::List(vec![item(103, "leather_helmet"), item(103, "iron_helmet")]),
        )]);
        let out = fix.apply(player);
        assert!(out.get("Inventory").as_list().is_empty());
        assert_eq!(
            out.get("equipment")
                .get("head")
                .get("id")
                .as_string()
                .as_deref(),
            Some("iron_helmet")
        );
    }
}
