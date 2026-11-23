use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    Int(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Null,
}

pub struct HorseBodyArmorItemFix {
    pub previous_body_armor_tag: String,
    pub clear_armor_items: bool,
}

impl HorseBodyArmorItemFix {
    // JAVA: net.minecraft.util.datafix.fixes.HorseBodyArmorItemFix#HorseBodyArmorItemFix
    pub fn new(previous_body_armor_tag: String, clear_armor_items: bool) -> Self {
        Self {
            previous_body_armor_tag,
            clear_armor_items,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.HorseBodyArmorItemFix#fix
    pub fn fix(&self, entity: Dynamic) -> Dynamic {
        let Dynamic::Map(mut map) = entity else {
            return entity;
        };

        let Some(old_armor) = map.remove(&self.previous_body_armor_tag) else {
            return Dynamic::Map(map);
        };

        if self.clear_armor_items {
            if let Some(armor_items) = map.remove("ArmorItems") {
                let fixed = Self::replace_at_index(armor_items, 2, |_| Dynamic::Map(BTreeMap::new()));
                map.insert("ArmorItems".to_string(), fixed);
            }
            if let Some(drop_chances) = map.remove("ArmorDropChances") {
                let fixed = Self::replace_at_index(drop_chances, 2, |_| Dynamic::Float(0.085_f32));
                map.insert("ArmorDropChances".to_string(), fixed);
            }
        }

        map.insert("body_armor_item".to_string(), old_armor);
        map.insert(
            "body_armor_drop_chance".to_string(),
            Dynamic::Float(2.0_f32),
        );
        Dynamic::Map(map)
    }

    fn replace_at_index(
        value: Dynamic,
        index: i64,
        replacement: impl Fn(&Dynamic) -> Dynamic,
    ) -> Dynamic {
        match value {
            Dynamic::List(items) => Dynamic::List(
                items
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| {
                        if (i as i64) == index {
                            replacement(&v)
                        } else {
                            v
                        }
                    })
                    .collect(),
            ),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_map() -> Dynamic {
        Dynamic::Map(BTreeMap::new())
    }

    #[test]
    fn moves_armor_tag_and_sets_drop_chance() {
        let fix = HorseBodyArmorItemFix::new("SaddleItem".to_string(), false);
        let mut armor = BTreeMap::new();
        armor.insert(
            "id".to_string(),
            Dynamic::String("minecraft:diamond_horse_armor".to_string()),
        );
        let mut entity = BTreeMap::new();
        entity.insert("SaddleItem".to_string(), Dynamic::Map(armor.clone()));
        let result = fix.fix(Dynamic::Map(entity));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        assert!(!m.contains_key("SaddleItem"));
        assert_eq!(m.get("body_armor_item"), Some(&Dynamic::Map(armor)));
        assert_eq!(
            m.get("body_armor_drop_chance"),
            Some(&Dynamic::Float(2.0_f32))
        );
    }

    #[test]
    fn clear_armor_items_replaces_index_2() {
        let fix = HorseBodyArmorItemFix::new("ArmorItem".to_string(), true);
        let mut entity = BTreeMap::new();
        entity.insert("ArmorItem".to_string(), Dynamic::String("old".to_string()));
        entity.insert(
            "ArmorItems".to_string(),
            Dynamic::List(vec![
                Dynamic::Int(1),
                Dynamic::Int(2),
                Dynamic::Int(3),
                Dynamic::Int(4),
            ]),
        );
        entity.insert(
            "ArmorDropChances".to_string(),
            Dynamic::List(vec![
                Dynamic::Float(0.1_f32),
                Dynamic::Float(0.2_f32),
                Dynamic::Float(0.3_f32),
                Dynamic::Float(0.4_f32),
            ]),
        );
        let result = fix.fix(Dynamic::Map(entity));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        let Some(Dynamic::List(items)) = m.get("ArmorItems") else { panic!("ArmorItems") };
        assert_eq!(items[0], Dynamic::Int(1));
        assert_eq!(items[2], empty_map());
        assert_eq!(items[3], Dynamic::Int(4));
        let Some(Dynamic::List(chances)) = m.get("ArmorDropChances") else { panic!("ArmorDropChances") };
        let Dynamic::Float(v) = chances[2] else { panic!("expected float") };
        assert_eq!(v.to_bits(), 0.085_f32.to_bits());
        assert_eq!(chances[0], Dynamic::Float(0.1_f32));
    }

    #[test]
    fn missing_old_tag_unchanged() {
        let fix = HorseBodyArmorItemFix::new("ArmorItem".to_string(), true);
        let mut entity = BTreeMap::new();
        entity.insert("id".to_string(), Dynamic::String("horse".to_string()));
        let original = Dynamic::Map(entity);
        assert_eq!(fix.fix(original.clone()), original);
    }

    #[test]
    fn non_map_input_unchanged() {
        let fix = HorseBodyArmorItemFix::new("ArmorItem".to_string(), false);
        let input = Dynamic::Int(-7);
        assert_eq!(fix.fix(input.clone()), input);
    }

    #[test]
    fn clear_armor_items_false_preserves_armor_lists() {
        let fix = HorseBodyArmorItemFix::new("ArmorItem".to_string(), false);
        let mut entity = BTreeMap::new();
        entity.insert("ArmorItem".to_string(), Dynamic::String("x".to_string()));
        let armor_items = Dynamic::List(vec![Dynamic::Int(1), Dynamic::Int(2), Dynamic::Int(3)]);
        entity.insert("ArmorItems".to_string(), armor_items.clone());
        let result = fix.fix(Dynamic::Map(entity));
        let Dynamic::Map(m) = result else { panic!("expected map") };
        assert_eq!(m.get("ArmorItems"), Some(&armor_items));
    }
}
