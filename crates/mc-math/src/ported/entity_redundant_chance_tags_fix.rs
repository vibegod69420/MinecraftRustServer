use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Float(f32),
    Double(f64),
    Int(i32),
    Long(i64),
    String(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    fn as_f32(&self) -> Option<f32> {
        match self {
            Dynamic::Float(v) => Some(*v),
            Dynamic::Double(v) => Some(*v as f32),
            _ => None,
        }
    }

    fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    fn as_map(&self) -> Option<&BTreeMap<String, Dynamic>> {
        match self {
            Dynamic::Map(m) => Some(m),
            _ => None,
        }
    }

    fn remove(self, key: &str) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
        }
    }
}

pub struct EntityRedundantChanceTagsFix;

impl EntityRedundantChanceTagsFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityRedundantChanceTagsFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityRedundantChanceTagsFix#makeRule
    pub fn apply(&self, value: Dynamic) -> Dynamic {
        let mut value = value;
        if is_zero_list(value.get("HandDropChances"), 2) {
            value = value.remove("HandDropChances");
        }
        if is_zero_list(value.get("ArmorDropChances"), 4) {
            value = value.remove("ArmorDropChances");
        }
        value
    }
}

impl Default for EntityRedundantChanceTagsFix {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.util.datafix.fixes.EntityRedundantChanceTagsFix#isZeroList
fn is_zero_list(value: Option<&Dynamic>, expected_size: usize) -> bool {
    let Some(Dynamic::List(list)) = value else {
        return false;
    };
    list.len() == expected_size && list.iter().all(|v| v.as_f32() == Some(0.0f32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn removes_zero_hand_and_armor_drop_chances() {
        let input = map(vec![
            (
                "HandDropChances",
                Dynamic::List(vec![Dynamic::Float(0.0), Dynamic::Float(0.0)]),
            ),
            (
                "ArmorDropChances",
                Dynamic::List(vec![
                    Dynamic::Float(0.0),
                    Dynamic::Float(0.0),
                    Dynamic::Float(0.0),
                    Dynamic::Float(0.0),
                ]),
            ),
            ("id", Dynamic::String("minecraft:zombie".to_string())),
        ]);
        let out = EntityRedundantChanceTagsFix::new().apply(input);
        let m = out.as_map().unwrap();
        assert!(!m.contains_key("HandDropChances"));
        assert!(!m.contains_key("ArmorDropChances"));
        assert_eq!(
            m.get("id"),
            Some(&Dynamic::String("minecraft:zombie".to_string()))
        );
    }

    #[test]
    fn keeps_nonzero_or_wrong_size_lists() {
        let input = map(vec![
            (
                "HandDropChances",
                Dynamic::List(vec![Dynamic::Float(0.0), Dynamic::Float(0.1)]),
            ),
            (
                "ArmorDropChances",
                Dynamic::List(vec![Dynamic::Float(0.0), Dynamic::Float(0.0)]),
            ),
        ]);
        let out = EntityRedundantChanceTagsFix::new().apply(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_fields_unchanged() {
        let input = map(vec![("id", Dynamic::String("minecraft:cow".to_string()))]);
        let out = EntityRedundantChanceTagsFix::new().apply(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn negative_zero_treated_as_zero() {
        let input = map(vec![(
            "HandDropChances",
            Dynamic::List(vec![Dynamic::Float(-0.0f32), Dynamic::Float(0.0f32)]),
        )]);
        let out = EntityRedundantChanceTagsFix::new().apply(input);
        assert!(out.as_map().unwrap().get("HandDropChances").is_none());
    }

    #[test]
    fn non_float_elements_prevent_removal() {
        let input = map(vec![(
            "HandDropChances",
            Dynamic::List(vec![Dynamic::Int(0), Dynamic::Float(0.0)]),
        )]);
        let out = EntityRedundantChanceTagsFix::new().apply(input.clone());
        assert_eq!(out, input);
    }
}
