use std::collections::HashMap;
use std::sync::OnceLock;

/// Port of `net.minecraft.util.datafix.fixes.RenamedCoralFix`.
///
/// DataFixerUpper schema/type infrastructure is not ported; this module
/// preserves the faithful rename mapping as self-contained data.
pub struct RenamedCoralFix;

impl RenamedCoralFix {
    // JAVA: net.minecraft.util.datafix.fixes.RenamedCoralFix#RENAMED_IDS
    pub fn renamed_ids() -> &'static HashMap<&'static str, &'static str> {
        static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
        MAP.get_or_init(|| {
            HashMap::from([
                ("minecraft:blue_coral", "minecraft:tube_coral_block"),
                ("minecraft:pink_coral", "minecraft:brain_coral_block"),
                ("minecraft:purple_coral", "minecraft:bubble_coral_block"),
                ("minecraft:red_coral", "minecraft:fire_coral_block"),
                ("minecraft:yellow_coral", "minecraft:horn_coral_block"),
                ("minecraft:blue_coral_plant", "minecraft:tube_coral"),
                ("minecraft:pink_coral_plant", "minecraft:brain_coral"),
                ("minecraft:purple_coral_plant", "minecraft:bubble_coral"),
                ("minecraft:red_coral_plant", "minecraft:fire_coral"),
                ("minecraft:yellow_coral_plant", "minecraft:horn_coral"),
                ("minecraft:blue_coral_fan", "minecraft:tube_coral_fan"),
                ("minecraft:pink_coral_fan", "minecraft:brain_coral_fan"),
                ("minecraft:purple_coral_fan", "minecraft:bubble_coral_fan"),
                ("minecraft:red_coral_fan", "minecraft:fire_coral_fan"),
                ("minecraft:yellow_coral_fan", "minecraft:horn_coral_fan"),
                ("minecraft:blue_dead_coral", "minecraft:dead_tube_coral"),
                ("minecraft:pink_dead_coral", "minecraft:dead_brain_coral"),
                ("minecraft:purple_dead_coral", "minecraft:dead_bubble_coral"),
                ("minecraft:red_dead_coral", "minecraft:dead_fire_coral"),
                ("minecraft:yellow_dead_coral", "minecraft:dead_horn_coral"),
            ])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RenamedCoralFix;

    #[test]
    fn contains_all_20_mappings() {
        assert_eq!(RenamedCoralFix::renamed_ids().len(), 20);
    }

    #[test]
    fn maps_coral_blocks() {
        let ids = RenamedCoralFix::renamed_ids();
        assert_eq!(ids.get("minecraft:blue_coral"), Some(&"minecraft:tube_coral_block"));
        assert_eq!(ids.get("minecraft:red_coral"), Some(&"minecraft:fire_coral_block"));
        assert_eq!(ids.get("minecraft:yellow_coral"), Some(&"minecraft:horn_coral_block"));
    }

    #[test]
    fn maps_plants_fans_and_dead_variants() {
        let ids = RenamedCoralFix::renamed_ids();
        assert_eq!(ids.get("minecraft:pink_coral_plant"), Some(&"minecraft:brain_coral"));
        assert_eq!(ids.get("minecraft:purple_coral_fan"), Some(&"minecraft:bubble_coral_fan"));
        assert_eq!(ids.get("minecraft:yellow_dead_coral"), Some(&"minecraft:dead_horn_coral"));
    }

    #[test]
    fn unknown_id_is_absent() {
        let ids = RenamedCoralFix::renamed_ids();
        assert_eq!(ids.get("minecraft:tube_coral_block"), None);
        assert_eq!(ids.get("minecraft:unknown"), None);
    }
}
