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

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Dynamic::Str(s) => Some(s),
            _ => None,
        }
    }
}

pub fn chunk_status_fix2(mut value: Dynamic) -> Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkStatusFix2#makeRule
    let status = value
        .get("Level")
        .and_then(|level| level.get("Status"))
        .and_then(Dynamic::as_str)
        .unwrap_or("empty")
        .to_string();

    let new_status = match status.as_str() {
        "structure_references" => "empty",
        "biomes" => "empty",
        "base" => "surface",
        "carved" => "carvers",
        "liquid_carved" => "liquid_carvers",
        "decorated" => "features",
        "lighted" => "light",
        "mobs_spawned" => "spawn",
        "finalized" => "heightmaps",
        "fullchunk" => "full",
        _ => "empty",
    }
    .to_string();

    if status != new_status {
        if let Some(Dynamic::Map(level)) = value.get_mut("Level") {
            level.insert("Status".to_string(), Dynamic::Str(new_status));
        }
    }

    value
}

pub fn fix(value: Dynamic) -> Dynamic {
    chunk_status_fix2(value)
}

pub fn apply(value: Dynamic) -> Dynamic {
    chunk_status_fix2(value)
}

pub fn apply_fix(value: Dynamic) -> Dynamic {
    chunk_status_fix2(value)
}

pub fn fix_in_place(value: &mut Dynamic) {
    let old = std::mem::replace(value, Dynamic::Null);
    *value = chunk_status_fix2(old);
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
    fn renames_decorated_to_features() {
        let input = map(vec![("Level", map(vec![("Status", Dynamic::Str("decorated".to_string()))]))]);
        let out = chunk_status_fix2(input);
        assert_eq!(
            out.get("Level").and_then(|l| l.get("Status")),
            Some(&Dynamic::Str("features".to_string()))
        );
    }

    #[test]
    fn renames_fullchunk_to_full() {
        let input = map(vec![("Level", map(vec![("Status", Dynamic::Str("fullchunk".to_string()))]))]);
        let out = fix(input);
        assert_eq!(
            out.get("Level").and_then(|l| l.get("Status")),
            Some(&Dynamic::Str("full".to_string()))
        );
    }

    #[test]
    fn unknown_status_becomes_empty() {
        let input = map(vec![("Level", map(vec![("Status", Dynamic::Str("totally_unknown".to_string()))]))]);
        let out = apply(input);
        assert_eq!(
            out.get("Level").and_then(|l| l.get("Status")),
            Some(&Dynamic::Str("empty".to_string()))
        );
    }

    #[test]
    fn empty_status_stays_empty() {
        let input = map(vec![("Level", map(vec![("Status", Dynamic::Str("empty".to_string()))]))]);
        let out = chunk_status_fix2(input);
        assert_eq!(
            out.get("Level").and_then(|l| l.get("Status")),
            Some(&Dynamic::Str("empty".to_string()))
        );
    }

    #[test]
    fn missing_level_does_not_panic() {
        let input = map(vec![("Status", Dynamic::Str("decorated".to_string()))]);
        let out = apply_fix(input);
        assert!(out.get("Level").is_none());
    }
}
