/// Minimal dynamic value representation for world spawn datafix transformations.
/// Self-contained stand-in for Mojang DFU `Dynamic<?>` used by this fix.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    String(String),
    Int(i32),
    Float(f32),
    IntList(Vec<i32>),
    Map(Vec<(String, Dynamic)>),
}

impl Dynamic {
    fn empty_map(&self) -> Dynamic {
        Dynamic::Map(Vec::new())
    }

    fn create_string(&self, value: &str) -> Dynamic {
        Dynamic::String(value.to_owned())
    }

    fn create_int_list(&self, values: Vec<i32>) -> Dynamic {
        Dynamic::IntList(values)
    }

    fn create_float(&self, value: f32) -> Dynamic {
        Dynamic::Float(value)
    }

    fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn as_int(&self, default: i32) -> i32 {
        match self {
            Dynamic::Int(v) => *v,
            _ => default,
        }
    }

    fn as_float(&self, default: f32) -> f32 {
        match self {
            Dynamic::Float(v) => *v,
            _ => default,
        }
    }

    fn set(self, key: &str, value: Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut entries) => {
                if let Some((_, existing)) = entries.iter_mut().find(|(k, _)| k == key) {
                    *existing = value;
                } else {
                    entries.push((key.to_owned(), value));
                }
                Dynamic::Map(entries)
            }
            other => other,
        }
    }

    fn remove(self, key: &str) -> Dynamic {
        match self {
            Dynamic::Map(mut entries) => {
                entries.retain(|(k, _)| k != key);
                Dynamic::Map(entries)
            }
            other => other,
        }
    }
}

/// Port of `net.minecraft.util.datafix.fixes.WorldSpawnDataFix`.
#[derive(Clone, Debug, Default)]
pub struct WorldSpawnDataFix;

impl WorldSpawnDataFix {
    // JAVA: net.minecraft.util.datafix.fixes.WorldSpawnDataFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.WorldSpawnDataFix#makeRule
    pub fn apply(&self, dynamic: Dynamic) -> Dynamic {
        Self::fix_level(dynamic)
    }

    fn fix_level(dynamic: Dynamic) -> Dynamic {
        let spawn_x = dynamic.get("SpawnX").map(|v| v.as_int(0)).unwrap_or(0);
        let spawn_y = dynamic.get("SpawnY").map(|v| v.as_int(0)).unwrap_or(0);
        let spawn_z = dynamic.get("SpawnZ").map(|v| v.as_int(0)).unwrap_or(0);
        let spawn_angle = dynamic
            .get("SpawnAngle")
            .map(|v| v.as_float(0.0_f32))
            .unwrap_or(0.0_f32);

        let spawn = dynamic
            .empty_map()
            .set("dimension", dynamic.create_string("minecraft:overworld"))
            .set("pos", dynamic.create_int_list(vec![spawn_x, spawn_y, spawn_z]))
            .set("yaw", dynamic.create_float(spawn_angle))
            .set("pitch", dynamic.create_float(0.0_f32));

        dynamic
            .remove("SpawnX")
            .remove("SpawnY")
            .remove("SpawnZ")
            .remove("SpawnAngle")
            .set("spawn", spawn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_map<'a>(d: &'a Dynamic) -> &'a [(String, Dynamic)] {
        match d {
            Dynamic::Map(m) => m,
            _ => panic!("expected map"),
        }
    }

    fn get_key<'a>(d: &'a Dynamic, key: &str) -> &'a Dynamic {
        d.get(key).unwrap_or_else(|| panic!("missing key {key}"))
    }

    #[test]
    fn moves_spawn_fields_into_spawn_object() {
        let level = Dynamic::Map(vec![
            ("SpawnX".to_owned(), Dynamic::Int(10)),
            ("SpawnY".to_owned(), Dynamic::Int(64)),
            ("SpawnZ".to_owned(), Dynamic::Int(-20)),
            ("SpawnAngle".to_owned(), Dynamic::Float(90.0_f32)),
            ("LevelName".to_owned(), Dynamic::String("world".to_owned())),
        ]);
        let fixed = WorldSpawnDataFix::new(()).apply(level);
        assert!(fixed.get("SpawnX").is_none());
        assert!(fixed.get("SpawnY").is_none());
        assert!(fixed.get("SpawnZ").is_none());
        assert!(fixed.get("SpawnAngle").is_none());
        assert_eq!(get_key(&fixed, "LevelName"), &Dynamic::String("world".to_owned()));

        let spawn = get_key(&fixed, "spawn");
        assert_eq!(
            get_key(spawn, "dimension"),
            &Dynamic::String("minecraft:overworld".to_owned())
        );
        assert_eq!(
            get_key(spawn, "pos"),
            &Dynamic::IntList(vec![10, 64, -20])
        );
        assert_eq!(get_key(spawn, "yaw"), &Dynamic::Float(90.0_f32));
        assert_eq!(get_key(spawn, "pitch"), &Dynamic::Float(0.0_f32));
    }

    #[test]
    fn defaults_missing_spawn_fields() {
        let level = Dynamic::Map(vec![("LevelName".to_owned(), Dynamic::String("w".to_owned()))]);
        let fixed = WorldSpawnDataFix::new(()).apply(level);
        let spawn = get_key(&fixed, "spawn");
        assert_eq!(get_key(spawn, "pos"), &Dynamic::IntList(vec![0, 0, 0]));
        assert_eq!(get_key(spawn, "yaw"), &Dynamic::Float(0.0_f32));
        assert_eq!(get_key(spawn, "pitch"), &Dynamic::Float(0.0_f32));
    }

    #[test]
    fn negative_coordinates_and_angle_preserved() {
        let level = Dynamic::Map(vec![
            ("SpawnX".to_owned(), Dynamic::Int(-1_000_000)),
            ("SpawnY".to_owned(), Dynamic::Int(-64)),
            ("SpawnZ".to_owned(), Dynamic::Int(2_147_483_647_i32)),
            ("SpawnAngle".to_owned(), Dynamic::Float(-180.5_f32)),
        ]);
        let fixed = WorldSpawnDataFix::new(()).apply(level);
        let spawn = get_key(&fixed, "spawn");
        assert_eq!(
            get_key(spawn, "pos"),
            &Dynamic::IntList(vec![-1_000_000, -64, 2_147_483_647_i32])
        );
        assert_eq!(get_key(spawn, "yaw"), &Dynamic::Float(-180.5_f32));
    }

    #[test]
    fn overwrites_existing_spawn_key() {
        let level = Dynamic::Map(vec![
            ("SpawnX".to_owned(), Dynamic::Int(1)),
            ("spawn".to_owned(), Dynamic::String("old".to_owned())),
        ]);
        let fixed = WorldSpawnDataFix::new(()).apply(level);
        let spawn = get_key(&fixed, "spawn");
        assert_eq!(
            get_key(spawn, "dimension"),
            &Dynamic::String("minecraft:overworld".to_owned())
        );
    }

    #[test]
    fn spawn_map_has_expected_keys() {
        let level = Dynamic::Map(vec![]);
        let fixed = WorldSpawnDataFix::new(()).apply(level);
        let spawn_entries = get_map(get_key(&fixed, "spawn"));
        let keys: Vec<&str> = spawn_entries.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["dimension", "pos", "yaw", "pitch"]);
    }
}
