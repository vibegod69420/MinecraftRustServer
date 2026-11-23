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

    pub fn is_missing(&self) -> bool {
        matches!(self, Dynamic::Missing)
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::String(s.to_string())
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

pub struct TrialSpawnerConfigFix;

impl TrialSpawnerConfigFix {
    // JAVA: net.minecraft.util.datafix.fixes.TrialSpawnerConfigFix#TrialSpawnerConfigFix
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.TrialSpawnerConfigFix#fix
    pub fn fix(&self, dynamic: Dynamic) -> Dynamic {
        move_to_config_tag(dynamic)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.TrialSpawnerConfigFix#moveToConfigTag
fn move_to_config_tag(mut dynamic: Dynamic) -> Dynamic {
    let keys = [
        "spawn_range",
        "total_mobs",
        "simultaneous_mobs",
        "total_mobs_added_per_player",
        "simultaneous_mobs_added_per_player",
        "ticks_between_spawn",
        "spawn_potentials",
        "loot_tables_to_eject",
        "items_to_drop_when_ominous",
    ];
    let mut config_entries: Vec<(Dynamic, Dynamic)> = Vec::new();
    for key in keys {
        let val = dynamic.get(key);
        if !val.is_missing() {
            config_entries.push((dynamic.create_string(key), val));
            dynamic = dynamic.remove(key);
        }
    }
    if config_entries.is_empty() {
        dynamic
    } else {
        let config_map = dynamic.create_map(config_entries);
        dynamic.set("normal_config", config_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_config_keys_into_normal_config() {
        let fix = TrialSpawnerConfigFix::new(());
        let d = Dynamic::object(&[
            ("spawn_range", Dynamic::Int(4)),
            ("total_mobs", Dynamic::Int(6)),
            ("ticks_between_spawn", Dynamic::Int(20)),
            ("other", Dynamic::str("keep")),
        ]);
        let out = fix.fix(d);
        assert!(out.get("spawn_range").is_missing());
        assert!(out.get("total_mobs").is_missing());
        assert!(out.get("ticks_between_spawn").is_missing());
        assert_eq!(out.get("other").as_string().as_deref(), Some("keep"));
        let cfg = out.get("normal_config");
        assert_eq!(cfg.get("spawn_range"), Dynamic::Int(4));
        assert_eq!(cfg.get("total_mobs"), Dynamic::Int(6));
        assert_eq!(cfg.get("ticks_between_spawn"), Dynamic::Int(20));
    }

    #[test]
    fn no_config_keys_unchanged() {
        let fix = TrialSpawnerConfigFix::new(());
        let d = Dynamic::object(&[("id", Dynamic::str("minecraft:trial_spawner"))]);
        let out = fix.fix(d.clone());
        assert_eq!(out, d);
        assert!(out.get("normal_config").is_missing());
    }

    #[test]
    fn moves_all_known_keys() {
        let fix = TrialSpawnerConfigFix::new(());
        let d = Dynamic::object(&[
            ("spawn_range", Dynamic::Int(1)),
            ("total_mobs", Dynamic::Int(2)),
            ("simultaneous_mobs", Dynamic::Int(3)),
            ("total_mobs_added_per_player", Dynamic::Int(4)),
            ("simultaneous_mobs_added_per_player", Dynamic::Int(5)),
            ("ticks_between_spawn", Dynamic::Int(6)),
            ("spawn_potentials", Dynamic::List(vec![])),
            ("loot_tables_to_eject", Dynamic::List(vec![])),
            ("items_to_drop_when_ominous", Dynamic::List(vec![])),
        ]);
        let out = fix.fix(d);
        let cfg = out.get("normal_config");
        assert_eq!(cfg.get("spawn_range"), Dynamic::Int(1));
        assert_eq!(cfg.get("simultaneous_mobs"), Dynamic::Int(3));
        assert_eq!(cfg.get("items_to_drop_when_ominous"), Dynamic::List(vec![]));
        assert!(out.get("spawn_range").is_missing());
        assert!(out.get("loot_tables_to_eject").is_missing());
    }

    #[test]
    fn overwrites_existing_normal_config_when_keys_moved() {
        let fix = TrialSpawnerConfigFix::new(());
        let d = Dynamic::object(&[
            ("spawn_range", Dynamic::Int(4)),
            ("normal_config", Dynamic::object(&[("old", Dynamic::Int(1))])),
        ]);
        let out = fix.fix(d);
        let cfg = out.get("normal_config");
        assert_eq!(cfg.get("spawn_range"), Dynamic::Int(4));
        assert!(cfg.get("old").is_missing());
    }

    #[test]
    fn preserves_unrelated_fields() {
        let fix = TrialSpawnerConfigFix::new(());
        let d = Dynamic::object(&[
            ("spawn_range", Dynamic::Int(4)),
            ("custom", Dynamic::str("x")),
            ("id", Dynamic::str("minecraft:trial_spawner")),
        ]);
        let out = fix.fix(d);
        assert_eq!(out.get("custom").as_string().as_deref(), Some("x"));
        assert_eq!(
            out.get("id").as_string().as_deref(),
            Some("minecraft:trial_spawner")
        );
        assert_eq!(out.get("normal_config").get("spawn_range"), Dynamic::Int(4));
    }
}
