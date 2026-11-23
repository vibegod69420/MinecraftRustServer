use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Compound(BTreeMap<String, Dynamic>),
    IntArray(Vec<i32>),
    Long(i64),
    Int(i32),
    String(String),
    List(Vec<Dynamic>),
}

impl Dynamic {
    pub fn compound() -> Self {
        Self::Compound(BTreeMap::new())
    }

    pub fn as_map(&self) -> Option<&BTreeMap<String, Dynamic>> {
        match self {
            Self::Compound(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut BTreeMap<String, Dynamic>> {
        match self {
            Self::Compound(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_long(&self) -> Option<i64> {
        match self {
            Self::Long(v) => Some(*v),
            Self::Int(v) => Some(i64::from(*v)),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        self.as_map().and_then(|m| m.get(key))
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Dynamic) {
        if let Self::Compound(m) = self {
            m.insert(key.into(), value);
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<Dynamic> {
        match self {
            Self::Compound(m) => m.remove(key),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Schema;

// JAVA: net.minecraft.util.datafix.fixes.AbstractUUIDFix#createUUIDFromLongs
fn create_uuid_from_longs(most: i64, least: i64) -> Dynamic {
    Dynamic::IntArray(vec![
        (most >> 32) as i32,
        most as i32,
        (least >> 32) as i32,
        least as i32,
    ])
}

// JAVA: net.minecraft.util.datafix.fixes.AbstractUUIDFix#replaceUUIDLeastMost
fn replace_uuid_least_most(mut dynamic: Dynamic, old_name: &str, new_name: &str) -> Dynamic {
    let most_key = format!("{old_name}Most");
    let least_key = format!("{old_name}Least");
    let (most, least) = match dynamic.as_map() {
        Some(map) => {
            let Some(most) = map.get(&most_key).and_then(Dynamic::as_long) else {
                return dynamic;
            };
            let Some(least) = map.get(&least_key).and_then(Dynamic::as_long) else {
                return dynamic;
            };
            (most, least)
        }
        None => return dynamic,
    };
    if let Some(map) = dynamic.as_map_mut() {
        map.remove(&most_key);
        map.remove(&least_key);
        map.insert(new_name.to_string(), create_uuid_from_longs(most, least));
    }
    dynamic
}

fn update_compound_field(
    dynamic: Dynamic,
    key: &str,
    f: impl FnOnce(Dynamic) -> Dynamic,
) -> Dynamic {
    let has_key = dynamic.as_map().is_some_and(|m| m.contains_key(key));
    if !has_key {
        return dynamic;
    }
    let mut dynamic = dynamic;
    if let Some(map) = dynamic.as_map_mut() {
        if let Some(value) = map.remove(key) {
            map.insert(key.to_string(), f(value));
        }
    }
    dynamic
}

// JAVA: net.minecraft.util.datafix.fixes.EntityUUIDFix#updateEntityUUID
fn update_entity_uuid(dynamic: Dynamic) -> Dynamic {
    replace_uuid_least_most(dynamic, "UUID", "UUID")
}

// JAVA: net.minecraft.util.datafix.fixes.EntityUUIDFix#updateLivingEntity
fn update_living_entity(dynamic: Dynamic) -> Dynamic {
    update_compound_field(dynamic, "Leash", |leash| {
        replace_uuid_least_most(leash, "UUID", "UUID")
    })
}

pub struct PlayerUUIDFix {
    _schema: Schema,
}

impl PlayerUUIDFix {
    // JAVA: net.minecraft.util.datafix.fixes.PlayerUUIDFix#PlayerUUIDFix
    pub fn new(schema: Schema) -> Self {
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.PlayerUUIDFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic + use<> {
        |dynamic: Dynamic| {
            let dynamic = update_compound_field(dynamic, "RootVehicle", |rv| {
                replace_uuid_least_most(rv, "Attach", "Attach")
            });
            update_entity_uuid(update_living_entity(dynamic))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixes_player_uuid_and_root_vehicle_attach() {
        let mut root_vehicle = Dynamic::compound();
        root_vehicle.insert("AttachMost", Dynamic::Long(0x0123_4567_89AB_CDEF_i64));
        root_vehicle.insert(
            "AttachLeast",
            Dynamic::Long(0xFEDC_BA98_7654_3210_u64 as i64),
        );

        let mut player = Dynamic::compound();
        player.insert("UUIDMost", Dynamic::Long(1));
        player.insert("UUIDLeast", Dynamic::Long(2));
        player.insert("RootVehicle", root_vehicle);

        let rule = PlayerUUIDFix::new(Schema).make_rule();
        let out = rule(player);

        assert_eq!(out.get("UUID"), Some(&Dynamic::IntArray(vec![0, 1, 0, 2])));
        assert!(out.get("UUIDMost").is_none());
        assert!(out.get("UUIDLeast").is_none());

        let rv = out.get("RootVehicle").unwrap();
        assert_eq!(
            rv.get("Attach"),
            Some(&Dynamic::IntArray(vec![
                0x0123_4567,
                0x89AB_CDEF_u32 as i32,
                0xFEDC_BA98_u32 as i32,
                0x7654_3210,
            ]))
        );
        assert!(rv.get("AttachMost").is_none());
        assert!(rv.get("AttachLeast").is_none());
    }

    #[test]
    fn no_op_when_no_uuid_fields() {
        let mut player = Dynamic::compound();
        player.insert("Name", Dynamic::String("Steve".to_string()));
        let original = player.clone();
        let rule = PlayerUUIDFix::new(Schema).make_rule();
        let out = rule(player);
        assert_eq!(out, original);
    }

    #[test]
    fn root_vehicle_without_attach_preserved() {
        let mut entity = Dynamic::compound();
        entity.insert("id", Dynamic::String("minecraft:pig".to_string()));
        let mut rv = Dynamic::compound();
        rv.insert("Entity", entity);
        let rv_expected = rv.clone();

        let mut player = Dynamic::compound();
        player.insert("RootVehicle", rv);
        let rule = PlayerUUIDFix::new(Schema).make_rule();
        let out = rule(player);
        assert_eq!(out.get("RootVehicle"), Some(&rv_expected));
    }

    #[test]
    fn uuid_conversion_preserves_negative_bit_patterns() {
        let mut player = Dynamic::compound();
        player.insert("UUIDMost", Dynamic::Long(-1_i64));
        player.insert("UUIDLeast", Dynamic::Long(i64::MIN));

        let rule = PlayerUUIDFix::new(Schema).make_rule();
        let out = rule(player);

        assert_eq!(
            out.get("UUID"),
            Some(&Dynamic::IntArray(vec![-1, -1, i32::MIN, 0]))
        );
    }

    #[test]
    fn fixes_leash_uuid_in_living_entity() {
        let mut leash = Dynamic::compound();
        leash.insert("UUIDMost", Dynamic::Long(10));
        leash.insert("UUIDLeast", Dynamic::Long(20));

        let mut player = Dynamic::compound();
        player.insert("UUIDMost", Dynamic::Long(1));
        player.insert("UUIDLeast", Dynamic::Long(2));
        player.insert("Leash", leash);

        let rule = PlayerUUIDFix::new(Schema).make_rule();
        let out = rule(player);

        let leash_out = out.get("Leash").unwrap();
        assert_eq!(
            leash_out.get("UUID"),
            Some(&Dynamic::IntArray(vec![0, 10, 0, 20]))
        );
        assert!(leash_out.get("UUIDMost").is_none());
        assert!(leash_out.get("UUIDLeast").is_none());
    }

    #[test]
    fn partial_uuid_not_converted() {
        let mut player = Dynamic::compound();
        player.insert("UUIDMost", Dynamic::Long(1));
        let original = player.clone();
        let rule = PlayerUUIDFix::new(Schema).make_rule();
        let out = rule(player);
        assert_eq!(out, original);
    }
}
