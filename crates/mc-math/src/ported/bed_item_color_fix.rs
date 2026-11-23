use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    String(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Byte(v) => i32::from(*v),
            Dynamic::Short(v) => i32::from(*v),
            Dynamic::Int(v) => *v,
            Dynamic::Long(v) => *v as i32,
            _ => default,
        }
    }

    fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }
}

pub struct BedItemColorFix;

impl BedItemColorFix {
    // JAVA: net.minecraft.util.datafix.fixes.BedItemColorFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.BedItemColorFix#makeRule
    pub fn apply(&self, value: Dynamic) -> Dynamic {
        let is_bed = value
            .get("id")
            .and_then(|id| id.as_string())
            .is_some_and(|id| id == "minecraft:bed");
        if is_bed {
            let damage = value.get("Damage").map_or(0, |d| d.as_int(0));
            if damage == 0 {
                return value.set("Damage", Dynamic::Short(14));
            }
        }
        value
    }
}

impl Default for BedItemColorFix {
    fn default() -> Self {
        Self::new()
    }
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
    fn bed_with_zero_damage_gets_red_color() {
        let input = map(vec![
            ("id", Dynamic::String("minecraft:bed".to_string())),
            ("Damage", Dynamic::Short(0)),
        ]);
        let out = BedItemColorFix::new().apply(input);
        assert_eq!(out.get("Damage"), Some(&Dynamic::Short(14)));
    }

    #[test]
    fn bed_missing_damage_defaults_to_zero_and_is_fixed() {
        let input = map(vec![("id", Dynamic::String("minecraft:bed".to_string()))]);
        let out = BedItemColorFix::new().apply(input);
        assert_eq!(out.get("Damage"), Some(&Dynamic::Short(14)));
    }

    #[test]
    fn bed_with_nonzero_damage_unchanged() {
        let input = map(vec![
            ("id", Dynamic::String("minecraft:bed".to_string())),
            ("Damage", Dynamic::Int(5)),
        ]);
        let out = BedItemColorFix::new().apply(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn non_bed_item_unchanged() {
        let input = map(vec![
            ("id", Dynamic::String("minecraft:stone".to_string())),
            ("Damage", Dynamic::Int(0)),
        ]);
        let out = BedItemColorFix::new().apply(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn bed_with_negative_damage_unchanged() {
        let input = map(vec![
            ("id", Dynamic::String("minecraft:bed".to_string())),
            ("Damage", Dynamic::Int(-1)),
        ]);
        let out = BedItemColorFix::new().apply(input.clone());
        assert_eq!(out, input);
    }
}
