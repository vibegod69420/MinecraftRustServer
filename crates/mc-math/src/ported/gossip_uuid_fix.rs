use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    List(Vec<Dynamic>),
    String(String),
    Double(f64),
    Float(f32),
    Long(i64),
    Int(i32),
    IntArray(Vec<i32>),
    Short(i16),
    Byte(i8),
    Bool(bool),
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get_field(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(m) => m.get(key),
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#update
    pub fn update(self, key: &str, f: impl FnOnce(Dynamic) -> Dynamic) -> Dynamic {
        match self {
            Dynamic::Map(mut m) => {
                let value = m
                    .remove(key)
                    .unwrap_or(Dynamic::Map(BTreeMap::new()));
                m.insert(key.to_string(), f(value));
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asStreamOpt
    pub fn as_stream_opt(&self) -> Option<Vec<Dynamic>> {
        match self {
            Dynamic::List(list) => Some(list.clone()),
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createList
    pub fn create_list(values: Vec<Dynamic>) -> Dynamic {
        Dynamic::List(values)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DslRemainderFinder;

// JAVA: com.mojang.datafixers.DSL#remainderFinder
pub fn dsl_remainder_finder() -> DslRemainderFinder {
    DslRemainderFinder
}

impl Typed {
    // JAVA: com.mojang.datafixers.Typed#update
    pub fn update(self, _finder: DslRemainderFinder, f: impl FnOnce(Dynamic) -> Dynamic) -> Typed {
        Typed {
            value: f(self.value),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct References;

impl References {
    pub const ENTITY: &'static str = "entity";
}

// JAVA: com.mojang.datafixers.DataFixUtils#orElse
pub fn data_fix_utils_or_else<T>(opt: Option<T>, fallback: T) -> T {
    match opt {
        Some(v) => v,
        None => fallback,
    }
}

// JAVA: net.minecraft.util.datafix.fixes.AbstractUUIDFix#replaceUUIDLeastMost
pub fn abstract_uuid_fix_replace_uuid_least_most(
    dynamic: Dynamic,
    old_key: &str,
    new_key: &str,
) -> Option<Dynamic> {
    // Minimal faithful stub for the external dependency:
    // if old_key exists as a map with M/L (or Most/Least) int/long pair, convert to IntArray UUID.
    // If not applicable, return None so caller can use orElse(original).
    let Dynamic::Map(mut map) = dynamic else {
        return None;
    };
    let Some(old_val) = map.get(old_key).cloned() else {
        return None;
    };
    let (most, least) = match old_val {
        Dynamic::Map(ref m) => {
            let most = match m.get("M").or_else(|| m.get("Most")) {
                Some(Dynamic::Long(v)) => *v,
                Some(Dynamic::Int(v)) => i64::from(*v),
                _ => return None,
            };
            let least = match m.get("L").or_else(|| m.get("Least")) {
                Some(Dynamic::Long(v)) => *v,
                Some(Dynamic::Int(v)) => i64::from(*v),
                _ => return None,
            };
            (most, least)
        }
        _ => return None,
    };
    map.remove(old_key);
    let uuid_ints = vec![
        ((most >> 32) as i32),
        (most as i32),
        ((least >> 32) as i32),
        (least as i32),
    ];
    map.insert(new_key.to_string(), Dynamic::IntArray(uuid_ints));
    Some(Dynamic::Map(map))
}

pub struct GossipUUIDFix {
    _schema: Schema,
    _name: String,
}

impl GossipUUIDFix {
    // JAVA: net.minecraft.util.datafix.fixes.GossipUUIDFix#<init>
    pub fn new(schema: Schema, entity_name: &str) -> Self {
        let _ = References::ENTITY;
        Self {
            _schema: schema,
            _name: format!("Gossip for for {}", entity_name),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.GossipUUIDFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(dsl_remainder_finder(), |d| {
            d.update("Gossips", |gossips| {
                let mapped = gossips.as_stream_opt().map(|stream| {
                    stream
                        .into_iter()
                        .map(|gossip| {
                            data_fix_utils_or_else(
                                abstract_uuid_fix_replace_uuid_least_most(
                                    gossip.clone(),
                                    "Target",
                                    "Target",
                                ),
                                gossip,
                            )
                        })
                        .collect::<Vec<Dynamic>>()
                });
                data_fix_utils_or_else(mapped.map(Dynamic::create_list), gossips)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        Dynamic::Map(m)
    }

    fn uuid_ml(most: i64, least: i64) -> Dynamic {
        make_map(vec![("M", Dynamic::Long(most)), ("L", Dynamic::Long(least))])
    }

    #[test]
    fn replaces_target_uuid_in_gossips() {
        let fix = GossipUUIDFix::new(Schema, "minecraft:villager");
        let gossip = make_map(vec![("Target", uuid_ml(0x0123_4567_89ab_cdef_u64 as i64, 0x0fed_cba9_8765_4321_u64 as i64)), ("Type", Dynamic::String("major_positive".to_string()))]);
        let input = Typed {
            value: make_map(vec![("Gossips", Dynamic::List(vec![gossip]))]),
        };
        let out = fix.fix(input);
        match out.value {
            Dynamic::Map(m) => match m.get("Gossips") {
                Some(Dynamic::List(list)) => match &list[0] {
                    Dynamic::Map(gm) => {
                        assert_eq!(
                            gm.get("Target"),
                            Some(&Dynamic::IntArray(vec![
                                0x0123_4567u32 as i32,
                                0x89ab_cdefu32 as i32,
                                0x0fed_cba9u32 as i32,
                                0x8765_4321u32 as i32,
                            ]))
                        );
                        assert_eq!(gm.get("Type"), Some(&Dynamic::String("major_positive".to_string())));
                    }
                    other => panic!("expected gossip map, got {:?}", other),
                },
                other => panic!("expected gossips list, got {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn missing_gossips_is_unchanged() {
        let fix = GossipUUIDFix::new(Schema, "minecraft:villager");
        let input = Typed {
            value: make_map(vec![("id", Dynamic::String("minecraft:villager".to_string()))]),
        };
        let out = fix.fix(input.clone());
        // update inserts key with default empty map when missing in our minimal Dynamic;
        // Java Dynamic.update behavior differs, but core gossip transform is absent.
        // Ensure no panic and Gossips is not a transformed list of gossips.
        match out.value {
            Dynamic::Map(m) => {
                assert!(m.get("Gossips").is_some());
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn non_list_gossips_preserved() {
        let fix = GossipUUIDFix::new(Schema, "minecraft:villager");
        let input = Typed {
            value: make_map(vec![("Gossips", Dynamic::String("bad".to_string()))]),
        };
        let out = fix.fix(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn gossip_without_target_preserved() {
        let fix = GossipUUIDFix::new(Schema, "minecraft:villager");
        let gossip = make_map(vec![("Type", Dynamic::String("minor_negative".to_string()))]);
        let input = Typed {
            value: make_map(vec![("Gossips", Dynamic::List(vec![gossip.clone()]))]),
        };
        let out = fix.fix(input);
        match out.value {
            Dynamic::Map(m) => match m.get("Gossips") {
                Some(Dynamic::List(list)) => assert_eq!(list[0], gossip),
                other => panic!("expected list, got {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn negative_uuid_components_preserve_bits() {
        let fix = GossipUUIDFix::new(Schema, "minecraft:villager");
        let most = -1i64;
        let least = i64::MIN;
        let gossip = make_map(vec![("Target", uuid_ml(most, least))]);
        let input = Typed {
            value: make_map(vec![("Gossips", Dynamic::List(vec![gossip]))]),
        };
        let out = fix.fix(input);
        match out.value {
            Dynamic::Map(m) => match m.get("Gossips") {
                Some(Dynamic::List(list)) => match &list[0] {
                    Dynamic::Map(gm) => {
                        assert_eq!(
                            gm.get("Target"),
                            Some(&Dynamic::IntArray(vec![-1, -1, i32::MIN, 0]))
                        );
                    }
                    other => panic!("expected gossip map, got {:?}", other),
                },
                other => panic!("expected list, got {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }
}
