// JAVA: net.minecraft.util.datafix.fixes.EntityProjectileOwnerFix#class
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn new_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        match self {
            Dynamic::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_long(&self, default: i64) -> i64 {
        match self {
            Dynamic::Int(value) => *value,
            Dynamic::Bool(value) => {
                if *value {
                    1
                } else {
                    0
                }
            }
            _ => default,
        }
    }

    pub fn remove(mut self, key: &str) -> Self {
        if let Dynamic::Map(map) = &mut self {
            map.remove(key);
        }
        self
    }

    pub fn set(mut self, key: &str, value: Dynamic) -> Self {
        if let Dynamic::Map(map) = &mut self {
            map.insert(key.to_string(), value);
        }
        self
    }

    pub fn create_int_list(values: impl IntoIterator<Item = i32>) -> Self {
        Dynamic::List(values.into_iter().map(|value| Dynamic::Int(value as i64)).collect())
    }
}

pub fn create_uuid_array(most: i64, least: i64) -> [i32; 4] {
    [
        (most >> 32) as i32,
        most as i32,
        (least >> 32) as i32,
        least as i32,
    ]
}

pub fn set_uuid(value: Dynamic, most: i64, least: i64) -> Dynamic {
    if most != 0 && least != 0 {
        value.set("OwnerUUID", Dynamic::create_int_list(create_uuid_array(most, least)))
    } else {
        value
    }
}

pub fn update_owner_arrow(value: Dynamic) -> Dynamic {
    let most = value
        .get("OwnerUUIDMost")
        .map(|dynamic| dynamic.as_long(0))
        .unwrap_or(0);
    let least = value
        .get("OwnerUUIDLeast")
        .map(|dynamic| dynamic.as_long(0))
        .unwrap_or(0);

    let value = set_uuid(value, most, least);
    value.remove("OwnerUUIDMost").remove("OwnerUUIDLeast")
}

pub fn update_owner_llama_spit(value: Dynamic) -> Dynamic {
    let owner = value.get("Owner").cloned().unwrap_or(Dynamic::Null);
    let most = owner
        .get("OwnerUUIDMost")
        .map(|dynamic| dynamic.as_long(0))
        .unwrap_or(0);
    let least = owner
        .get("OwnerUUIDLeast")
        .map(|dynamic| dynamic.as_long(0))
        .unwrap_or(0);

    let value = set_uuid(value, most, least);
    value.remove("Owner")
}

pub fn update_owner_throwable(value: Dynamic) -> Dynamic {
    let owner = value.get("owner").cloned().unwrap_or(Dynamic::Null);
    let most = owner.get("M").map(|dynamic| dynamic.as_long(0)).unwrap_or(0);
    let least = owner.get("L").map(|dynamic| dynamic.as_long(0)).unwrap_or(0);

    let value = set_uuid(value, most, least);
    value.remove("owner")
}

pub struct EntityProjectileOwnerFix;

impl EntityProjectileOwnerFix {
    pub fn new() -> Self {
        Self
    }

    pub fn update_owner_arrow(&self, value: Dynamic) -> Dynamic {
        update_owner_arrow(value)
    }

    pub fn update_owner_llama_spit(&self, value: Dynamic) -> Dynamic {
        update_owner_llama_spit(value)
    }

    pub fn update_owner_throwable(&self, value: Dynamic) -> Dynamic {
        update_owner_throwable(value)
    }

    pub fn set_uuid(&self, value: Dynamic, most: i64, least: i64) -> Dynamic {
        set_uuid(value, most, least)
    }

    pub fn create_uuid_array(&self, most: i64, least: i64) -> [i32; 4] {
        create_uuid_array(most, least)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_uuid_array() {
        assert_eq!(
            create_uuid_array(0x1234_5678_9abc_def0, 0x0fed_cba9_8765_4321),
            [
                0x1234_5678,
                0x9abc_def0u32 as i32,
                0x0fed_cba9,
                0x8765_4321u32 as i32,
            ]
        );
    }

    #[test]
    fn set_uuid_requires_both_parts() {
        let value = Dynamic::new_map();
        assert_eq!(set_uuid(value.clone(), 0, 2), value);
        assert!(set_uuid(value, 1, 2).get("OwnerUUID").is_some());
    }

    #[test]
    fn updates_arrow_fields() {
        let value = Dynamic::new_map()
            .set("OwnerUUIDMost", Dynamic::Int(5))
            .set("OwnerUUIDLeast", Dynamic::Int(6));

        let updated = update_owner_arrow(value);
        assert!(updated.get("OwnerUUID").is_some());
        assert!(updated.get("OwnerUUIDMost").is_none());
        assert!(updated.get("OwnerUUIDLeast").is_none());
    }

    #[test]
    fn updates_throwable_fields() {
        let owner = Dynamic::new_map()
            .set("M", Dynamic::Int(7))
            .set("L", Dynamic::Int(8));
        let value = Dynamic::new_map().set("owner", owner);

        let updated = update_owner_throwable(value);
        assert!(updated.get("OwnerUUID").is_some());
        assert!(updated.get("owner").is_none());
    }

    #[test]
    fn updates_llama_spit_fields() {
        let owner = Dynamic::new_map()
            .set("OwnerUUIDMost", Dynamic::Int(9))
            .set("OwnerUUIDLeast", Dynamic::Int(10));
        let value = Dynamic::new_map().set("Owner", owner);

        let updated = update_owner_llama_spit(value);
        assert!(updated.get("OwnerUUID").is_some());
        assert!(updated.get("Owner").is_none());
    }

    #[test]
    fn zero_uuid_is_not_written() {
        let value = Dynamic::new_map()
            .set("OwnerUUIDMost", Dynamic::Int(0))
            .set("OwnerUUIDLeast", Dynamic::Int(6));

        let updated = update_owner_arrow(value);
        assert!(updated.get("OwnerUUID").is_none());
    }
}
