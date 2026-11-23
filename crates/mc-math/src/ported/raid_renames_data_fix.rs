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

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Dynamic::Int(n) => Some(*n),
            Dynamic::Float(f) => Some(*f as i64),
            _ => None,
        }
    }
}

fn rename_field(obj: &mut BTreeMap<String, Dynamic>, old: &str, new: &str) {
    if let Some(value) = obj.remove(old) {
        obj.insert(new.to_string(), value);
    }
}

fn fix_inline_block_pos(raid: &mut Dynamic) {
    // JAVA: net.minecraft.util.datafix.ExtraDataFixUtils#fixInlineBlockPos
    if let Dynamic::Map(obj) = raid {
        let has_all = obj.contains_key("CX") && obj.contains_key("CY") && obj.contains_key("CZ");
        if has_all {
            let x = obj.get("CX").and_then(Dynamic::as_i64).unwrap_or(0);
            let y = obj.get("CY").and_then(Dynamic::as_i64).unwrap_or(0);
            let z = obj.get("CZ").and_then(Dynamic::as_i64).unwrap_or(0);

            let packed = ((x & 0x3FFFFFF) << 38)
                | ((z & 0x3FFFFFF) << 12)
                | (y & 0xFFF);

            obj.insert("center".to_string(), Dynamic::Int(packed));
            obj.remove("CX");
            obj.remove("CY");
            obj.remove("CZ");
        }
    }
}

pub fn fix_raid(mut raid: Dynamic) -> Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.RaidRenamesDataFix#fixRaid
    fix_inline_block_pos(&mut raid);

    if let Dynamic::Map(obj) = &mut raid {
        rename_field(obj, "Id", "id");
        rename_field(obj, "Started", "started");
        rename_field(obj, "Active", "active");
        rename_field(obj, "TicksActive", "ticks_active");
        rename_field(obj, "BadOmenLevel", "raid_omen_level");
        rename_field(obj, "GroupsSpawned", "groups_spawned");
        rename_field(obj, "PreRaidTicks", "cooldown_ticks");
        rename_field(obj, "PostRaidTicks", "post_raid_ticks");
        rename_field(obj, "TotalHealth", "total_health");
        rename_field(obj, "NumGroups", "group_count");
        rename_field(obj, "Status", "status");
        rename_field(obj, "HeroesOfTheVillage", "heroes_of_the_village");
    }

    raid
}

pub fn fix_data(mut data: Dynamic) -> Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.RaidRenamesDataFix#fix
    if let Dynamic::Map(obj) = &mut data {
        if let Some(raids) = obj.remove("Raids") {
            let new_raids = match raids {
                Dynamic::List(items) => Dynamic::List(items.into_iter().map(fix_raid).collect()),
                other => other,
            };
            obj.insert("raids".to_string(), new_raids);
        }

        rename_field(obj, "Tick", "tick");
        rename_field(obj, "NextAvailableID", "next_id");
    }

    data
}

pub fn raid_renames_data_fix(mut value: Dynamic) -> Dynamic {
    // JAVA: net.minecraft.util.datafix.fixes.RaidRenamesDataFix#makeRule
    if let Dynamic::Map(obj) = &mut value {
        if let Some(data) = obj.remove("data") {
            obj.insert("data".to_string(), fix_data(data));
        }
    }
    value
}

pub fn fix(value: Dynamic) -> Dynamic {
    raid_renames_data_fix(value)
}

pub fn apply(value: Dynamic) -> Dynamic {
    raid_renames_data_fix(value)
}

pub fn apply_fix(value: Dynamic) -> Dynamic {
    raid_renames_data_fix(value)
}

pub fn fix_in_place(value: &mut Dynamic) {
    let old = std::mem::replace(value, Dynamic::Null);
    *value = raid_renames_data_fix(old);
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
    fn renames_top_level_fields() {
        let input = map(vec![
            ("Tick", Dynamic::Int(5)),
            ("NextAvailableID", Dynamic::Int(7)),
            ("Raids", Dynamic::List(vec![])),
        ]);
        let wrapped = map(vec![("data", input)]);
        let out = raid_renames_data_fix(wrapped);
        let data = out.get("data").unwrap();
        assert!(data.get("Tick").is_none());
        assert!(data.get("NextAvailableID").is_none());
        assert_eq!(data.get("tick"), Some(&Dynamic::Int(5)));
        assert_eq!(data.get("next_id"), Some(&Dynamic::Int(7)));
        assert!(data.get("raids").is_some());
    }

    #[test]
    fn fix_raid_renames_and_creates_center() {
        let raid = map(vec![
            ("CX", Dynamic::Int(1)),
            ("CY", Dynamic::Int(2)),
            ("CZ", Dynamic::Int(3)),
            ("Id", Dynamic::Int(9)),
            ("Status", Dynamic::Str("ONGOING".to_string())),
        ]);
        let out = fix_raid(raid);
        assert!(out.get("CX").is_none());
        assert!(out.get("CY").is_none());
        assert!(out.get("CZ").is_none());
        assert!(out.get("center").is_some());
        assert_eq!(out.get("id"), Some(&Dynamic::Int(9)));
        assert_eq!(out.get("status"), Some(&Dynamic::Str("ONGOING".to_string())));
    }

    #[test]
    fn missing_fields_do_not_panic() {
        let raid = map(vec![("Id", Dynamic::Int(1))]);
        let out = fix_raid(raid);
        assert_eq!(out.get("id"), Some(&Dynamic::Int(1)));
        assert!(out.get("center").is_none());
    }

    #[test]
    fn fix_data_handles_missing_raids() {
        let data = map(vec![("Tick", Dynamic::Int(1))]);
        let out = fix_data(data);
        assert_eq!(out.get("tick"), Some(&Dynamic::Int(1)));
        assert!(out.get("raids").is_none());
    }

    #[test]
    fn in_place_update_works() {
        let mut value = map(vec![("data", map(vec![("Tick", Dynamic::Int(2))]))]);
        fix_in_place(&mut value);
        assert_eq!(
            value.get("data").and_then(|d| d.get("tick")),
            Some(&Dynamic::Int(2))
        );
    }
}
