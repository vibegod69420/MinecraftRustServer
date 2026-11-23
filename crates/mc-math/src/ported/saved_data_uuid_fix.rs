use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix transformations.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Long(i64),
    Int(i32),
    IntArray(Vec<i32>),
}

impl Dynamic {
    fn as_map(&self) -> Option<&BTreeMap<String, Dynamic>> {
        match self {
            Dynamic::Map(m) => Some(m),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Dynamic::Long(v) => Some(*v),
            Dynamic::Int(v) => Some(*v as i64),
            _ => None,
        }
    }

    fn get(&self, key: &str) -> Option<&Dynamic> {
        self.as_map().and_then(|m| m.get(key))
    }
}

fn update_field(value: Dynamic, key: &str, f: impl FnOnce(Dynamic) -> Dynamic) -> Dynamic {
    match value {
        Dynamic::Map(mut map) => {
            if let Some(child) = map.remove(key) {
                map.insert(key.to_string(), f(child));
            }
            Dynamic::Map(map)
        }
        other => other,
    }
}

fn map_list(value: Dynamic, mut f: impl FnMut(Dynamic) -> Dynamic) -> Dynamic {
    match value {
        Dynamic::List(list) => Dynamic::List(list.into_iter().map(|v| f(v)).collect()),
        other => other,
    }
}

// JAVA: net.minecraft.util.datafix.fixes.AbstractUUIDFix#createUUIDFromLongs
pub fn create_uuid_from_longs(value: &Dynamic, most_key: &str, least_key: &str) -> Option<Dynamic> {
    let most = value.get(most_key)?.as_i64()?;
    let least = value.get(least_key)?.as_i64()?;
    let mut map = value.as_map()?.clone();
    map.remove(most_key);
    map.remove(least_key);

    let most_u = most as u64;
    let least_u = least as u64;
    let uuid = Dynamic::IntArray(vec![
        (most_u >> 32) as i32,
        most as i32,
        (least_u >> 32) as i32,
        least as i32,
    ]);
    map.insert("UUID".to_string(), uuid);
    Some(Dynamic::Map(map))
}

/// Port of `net.minecraft.util.datafix.fixes.SavedDataUUIDFix`.
///
/// DFU schema/type infrastructure is not ported; this module preserves the
/// faithful raid HeroesOfTheVillage UUID conversion logic.
pub struct SavedDataUuidFix;

impl SavedDataUuidFix {
    // JAVA: net.minecraft.util.datafix.fixes.SavedDataUUIDFix#SavedDataUUIDFix
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.SavedDataUUIDFix#makeRule
    pub fn make_rule(&self, value: Dynamic) -> Dynamic {
        update_field(value, "data", |data| {
            update_field(data, "Raids", |raids| {
                map_list(raids, |raid| {
                    update_field(raid, "HeroesOfTheVillage", |heroes| {
                        map_list(heroes, |hero| {
                            create_uuid_from_longs(&hero, "UUIDMost", "UUIDLeast").unwrap_or_else(
                                || {
                                    // Java: LOGGER.warn("HeroesOfTheVillage contained invalid UUIDs.");
                                    hero
                                },
                            )
                        })
                    })
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{create_uuid_from_longs, Dynamic, SavedDataUuidFix};
    use std::collections::BTreeMap;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    #[test]
    fn converts_heroes_uuid_most_least_to_int_array() {
        let fix = SavedDataUuidFix::new();
        let input = map(vec![(
            "data",
            map(vec![(
                "Raids",
                Dynamic::List(vec![map(vec![(
                    "HeroesOfTheVillage",
                    Dynamic::List(vec![map(vec![
                        ("UUIDMost", Dynamic::Long(0x0123_4567_89AB_CDEFu64 as i64)),
                        ("UUIDLeast", Dynamic::Long(0x0FED_CBA9_8765_4321u64 as i64)),
                    ])]),
                )])]),
            )]),
        )]);

        let out = fix.make_rule(input);
        let raids = out.get("data").and_then(|d| d.get("Raids")).unwrap();
        let Dynamic::List(raids) = raids else { panic!("expected raids list") };
        let hero = raids[0].get("HeroesOfTheVillage").unwrap();
        let Dynamic::List(heroes) = hero else { panic!("expected heroes list") };
        let Dynamic::Map(hero_map) = &heroes[0] else { panic!("expected hero map") };
        assert!(!hero_map.contains_key("UUIDMost"));
        assert!(!hero_map.contains_key("UUIDLeast"));
        assert_eq!(
            hero_map.get("UUID"),
            Some(&Dynamic::IntArray(vec![
                0x0123_4567u32 as i32,
                0x89AB_CDEFu32 as i32,
                0x0FED_CBA9u32 as i32,
                0x8765_4321u32 as i32,
            ]))
        );
    }

    #[test]
    fn invalid_uuid_is_preserved() {
        let fix = SavedDataUuidFix::new();
        let invalid = map(vec![("UUIDMost", Dynamic::Long(1))]); // missing UUIDLeast
        let input = map(vec![(
            "data",
            map(vec![(
                "Raids",
                Dynamic::List(vec![map(vec![(
                    "HeroesOfTheVillage",
                    Dynamic::List(vec![invalid.clone()]),
                )])]),
            )]),
        )]);
        let out = fix.make_rule(input);
        let hero = &match out.get("data").unwrap().get("Raids").unwrap() {
            Dynamic::List(r) => match r[0].get("HeroesOfTheVillage").unwrap() {
                Dynamic::List(h) => h[0].clone(),
                _ => panic!("expected list"),
            },
            _ => panic!("expected list"),
        };
        assert_eq!(hero, &invalid);
    }

    #[test]
    fn handles_negative_uuid_components() {
        let value = map(vec![
            ("UUIDMost", Dynamic::Long(-1)),
            ("UUIDLeast", Dynamic::Long(i64::MIN)),
        ]);
        let converted = create_uuid_from_longs(&value, "UUIDMost", "UUIDLeast").unwrap();
        let Dynamic::Map(m) = converted else { panic!("expected map") };
        assert_eq!(
            m.get("UUID"),
            Some(&Dynamic::IntArray(vec![-1, -1, i32::MIN, 0]))
        );
    }

    #[test]
    fn missing_data_path_is_noop() {
        let fix = SavedDataUuidFix::new();
        let input = map(vec![("other", Dynamic::Int(1))]);
        let out = fix.make_rule(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn empty_raids_and_heroes_are_preserved() {
        let fix = SavedDataUuidFix::new();
        let input = map(vec![(
            "data",
            map(vec![(
                "Raids",
                Dynamic::List(vec![map(vec![(
                    "HeroesOfTheVillage",
                    Dynamic::List(vec![]),
                )])]),
            )]),
        )]);
        let out = fix.make_rule(input.clone());
        assert_eq!(out, input);
    }
}
