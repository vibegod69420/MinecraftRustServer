use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
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

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Dynamic> {
        match self {
            Dynamic::Map(m) => m.get_mut(key),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Dynamic::Int(n) => Some(*n),
            Dynamic::Float(f) => Some(*f as i64),
            _ => None,
        }
    }
}

fn update_field(value: &mut Dynamic, field: &str, f: fn(Dynamic) -> Dynamic) {
    if let Dynamic::Map(obj) = value {
        if let Some(old) = obj.remove(field) {
            obj.insert(field.to_string(), f(old));
        }
    }
}

pub fn fix_dimension_id(value: Dynamic) -> Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.LegacyDimensionIdFix#fixDimensionId
    match value.as_i64() {
        Some(-1) => Dynamic::Str("minecraft:the_nether".to_string()),
        Some(1) => Dynamic::Str("minecraft:the_end".to_string()),
        Some(_) => Dynamic::Str("minecraft:overworld".to_string()),
        None => value,
    }
}

pub fn fix_player(mut value: Dynamic) -> Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.LegacyDimensionIdFix#fixPlayer
    update_field(&mut value, "Dimension", fix_dimension_id);
    value
}

pub fn fix_map(mut value: Dynamic) -> Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.LegacyDimensionIdFix#fixMap
    update_field(&mut value, "dimension", fix_dimension_id);
    value
}

pub fn legacy_dimension_id_fix(mut value: Dynamic) -> Dynamic {
    update_field(&mut value, "Dimension", fix_dimension_id);
    update_field(&mut value, "dimension", fix_dimension_id);
    value
}

pub fn fix(value: Dynamic) -> Dynamic {
    legacy_dimension_id_fix(value)
}

pub fn apply(value: Dynamic) -> Dynamic {
    legacy_dimension_id_fix(value)
}

pub fn apply_fix(value: Dynamic) -> Dynamic {
    legacy_dimension_id_fix(value)
}

pub fn fix_in_place(value: &mut Dynamic) {
    let old = std::mem::replace(value, Dynamic::Null);
    *value = legacy_dimension_id_fix(old);
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
    fn player_minus_one_becomes_nether() {
        let input = map(vec![("Dimension", Dynamic::Int(-1))]);
        let out = fix_player(input);
        assert_eq!(
            out.get("Dimension"),
            Some(&Dynamic::Str("minecraft:the_nether".to_string()))
        );
    }

    #[test]
    fn player_one_becomes_end() {
        let input = map(vec![("Dimension", Dynamic::Int(1))]);
        let out = fix(input);
        assert_eq!(
            out.get("Dimension"),
            Some(&Dynamic::Str("minecraft:the_end".to_string()))
        );
    }

    #[test]
    fn player_other_becomes_overworld() {
        let input = map(vec![("Dimension", Dynamic::Int(0))]);
        let out = legacy_dimension_id_fix(input);
        assert_eq!(
            out.get("Dimension"),
            Some(&Dynamic::Str("minecraft:overworld".to_string()))
        );
    }

    #[test]
    fn map_dimension_is_fixed() {
        let input = map(vec![("dimension", Dynamic::Float(1.0))]);
        let out = fix_map(input);
        assert_eq!(
            out.get("dimension"),
            Some(&Dynamic::Str("minecraft:the_end".to_string()))
        );
    }

    #[test]
    fn non_number_is_unchanged() {
        let input = map(vec![("Dimension", Dynamic::Str("minecraft:overworld".to_string()))]);
        let out = fix_player(input);
        assert_eq!(
            out.get("Dimension"),
            Some(&Dynamic::Str("minecraft:overworld".to_string()))
        );
    }

    #[test]
    fn both_player_and_map_fields_can_be_fixed() {
        let input = map(vec![
            ("Dimension", Dynamic::Int(-1)),
            ("dimension", Dynamic::Int(1)),
        ]);
        let out = apply_fix(input);
        assert_eq!(
            out.get("Dimension"),
            Some(&Dynamic::Str("minecraft:the_nether".to_string()))
        );
        assert_eq!(
            out.get("dimension"),
            Some(&Dynamic::Str("minecraft:the_end".to_string()))
        );
    }
}
