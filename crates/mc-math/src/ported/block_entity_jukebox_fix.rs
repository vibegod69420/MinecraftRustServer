#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    Null,
    Int(i32),
    Byte(i8),
    String(String),
    Map(Vec<(String, DynamicValue)>),
}

static NULL_DYNAMIC: DynamicValue = DynamicValue::Null;

impl DynamicValue {
    pub fn get(&self, key: &str) -> &DynamicValue {
        match self {
            DynamicValue::Map(entries) => {
                for (k, v) in entries {
                    if k == key {
                        return v;
                    }
                }
                &NULL_DYNAMIC
            }
            _ => &NULL_DYNAMIC,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match self {
            DynamicValue::Int(v) => *v,
            DynamicValue::Byte(v) => *v as i32,
            _ => default,
        }
    }

    pub fn remove(&self, key: &str) -> DynamicValue {
        match self {
            DynamicValue::Map(entries) => DynamicValue::Map(
                entries.iter().filter(|(k, _)| k != key).cloned().collect(),
            ),
            _ => self.clone(),
        }
    }

    pub fn empty_map() -> DynamicValue {
        DynamicValue::Map(Vec::new())
    }

    pub fn set(&self, key: &str, value: DynamicValue) -> DynamicValue {
        match self {
            DynamicValue::Map(entries) => {
                let mut new_entries = Vec::with_capacity(entries.len() + 1);
                let mut found = false;
                for (k, v) in entries {
                    if k == key {
                        new_entries.push((k.clone(), value.clone()));
                        found = true;
                    } else {
                        new_entries.push((k.clone(), v.clone()));
                    }
                }
                if !found {
                    new_entries.push((key.to_string(), value));
                }
                DynamicValue::Map(new_entries)
            }
            _ => self.clone(),
        }
    }

    pub fn create_string(s: &str) -> DynamicValue {
        DynamicValue::String(s.to_string())
    }

    pub fn create_byte(b: i8) -> DynamicValue {
        DynamicValue::Byte(b)
    }
}

mod item_id_fix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemIdFix#getItem
    pub fn get_item(id: i32) -> String {
        match id {
            2256 => "minecraft:record_13".to_string(),
            2257 => "minecraft:record_cat".to_string(),
            2258 => "minecraft:record_blocks".to_string(),
            2259 => "minecraft:record_chirp".to_string(),
            2260 => "minecraft:record_far".to_string(),
            2261 => "minecraft:record_mall".to_string(),
            2262 => "minecraft:record_mellohi".to_string(),
            2263 => "minecraft:record_stal".to_string(),
            2264 => "minecraft:record_strad".to_string(),
            2265 => "minecraft:record_ward".to_string(),
            2266 => "minecraft:record_11".to_string(),
            2267 => "minecraft:record_wait".to_string(),
            _ => format!("minecraft:unknown_{}", id),
        }
    }
}

mod item_stack_the_flattening_fix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackTheFlatteningFix#updateItem
    pub fn update_item(name: &str, _damage: i32) -> Option<String> {
        match name {
            "minecraft:record_13" => Some("minecraft:music_disc_13".to_string()),
            "minecraft:record_cat" => Some("minecraft:music_disc_cat".to_string()),
            "minecraft:record_blocks" => Some("minecraft:music_disc_blocks".to_string()),
            "minecraft:record_chirp" => Some("minecraft:music_disc_chirp".to_string()),
            "minecraft:record_far" => Some("minecraft:music_disc_far".to_string()),
            "minecraft:record_mall" => Some("minecraft:music_disc_mall".to_string()),
            "minecraft:record_mellohi" => Some("minecraft:music_disc_mellohi".to_string()),
            "minecraft:record_stal" => Some("minecraft:music_disc_stal".to_string()),
            "minecraft:record_strad" => Some("minecraft:music_disc_strad".to_string()),
            "minecraft:record_ward" => Some("minecraft:music_disc_ward".to_string()),
            "minecraft:record_11" => Some("minecraft:music_disc_11".to_string()),
            "minecraft:record_wait" => Some("minecraft:music_disc_wait".to_string()),
            _ => None,
        }
    }
}

pub struct BlockEntityJukeboxFix;

impl BlockEntityJukeboxFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityJukeboxFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityJukeboxFix#fix
    pub fn fix(&self, input: &DynamicValue) -> DynamicValue {
        let mut remainder = input.clone();
        let record = remainder.get("Record").as_int(0);
        if record > 0 {
            remainder = remainder.remove("Record");
            let item_name =
                item_stack_the_flattening_fix::update_item(&item_id_fix::get_item(record), 0);
            if let Some(name) = item_name {
                let record_item = DynamicValue::empty_map()
                    .set("id", DynamicValue::create_string(&name))
                    .set("Count", DynamicValue::create_byte(1));
                return remainder.set("RecordItem", record_item);
            }
        }
        input.clone()
    }
}

impl Default for BlockEntityJukeboxFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jukebox(record_id: i32) -> DynamicValue {
        DynamicValue::empty_map().set("Record", DynamicValue::Int(record_id))
    }

    #[test]
    fn test_fix_valid_record_preserves_other_fields() {
        let fix = BlockEntityJukeboxFix::new();
        let input = DynamicValue::empty_map()
            .set("Record", DynamicValue::Int(2257))
            .set("x", DynamicValue::Int(10))
            .set("y", DynamicValue::Int(64));
        let output = fix.fix(&input);
        assert_eq!(output.get("Record").as_int(-1), -1);
        assert_eq!(output.get("x").as_int(0), 10);
        assert_eq!(output.get("y").as_int(0), 64);
        let record_item = output.get("RecordItem");
        assert_eq!(
            record_item.get("id"),
            &DynamicValue::String("minecraft:music_disc_cat".to_string())
        );
        assert_eq!(record_item.get("Count"), &DynamicValue::Byte(1));
    }

    #[test]
    fn test_fix_no_record() {
        let fix = BlockEntityJukeboxFix::new();
        let input = DynamicValue::empty_map().set("x", DynamicValue::Int(1));
        let output = fix.fix(&input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_fix_record_zero() {
        let fix = BlockEntityJukeboxFix::new();
        let input = make_jukebox(0);
        let output = fix.fix(&input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_fix_unknown_record_returns_original() {
        let fix = BlockEntityJukeboxFix::new();
        let input = make_jukebox(9999);
        let output = fix.fix(&input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_fix_negative_record() {
        let fix = BlockEntityJukeboxFix::new();
        let input = make_jukebox(-1);
        let output = fix.fix(&input);
        assert_eq!(output, input);
    }
}
