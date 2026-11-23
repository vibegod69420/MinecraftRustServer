use std::collections::HashMap;
use std::sync::OnceLock;

pub struct RenamedCoralFansFix;

impl RenamedCoralFansFix {
    // JAVA: net.minecraft.util.datafix.fixes.RenamedCoralFansFix#RENAMED_IDS
    pub fn renamed_ids() -> &'static HashMap<String, String> {
        static RENAMED_IDS: OnceLock<HashMap<String, String>> = OnceLock::new();
        RENAMED_IDS.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert(
                "minecraft:tube_coral_fan".to_string(),
                "minecraft:tube_coral_wall_fan".to_string(),
            );
            m.insert(
                "minecraft:brain_coral_fan".to_string(),
                "minecraft:brain_coral_wall_fan".to_string(),
            );
            m.insert(
                "minecraft:bubble_coral_fan".to_string(),
                "minecraft:bubble_coral_wall_fan".to_string(),
            );
            m.insert(
                "minecraft:fire_coral_fan".to_string(),
                "minecraft:fire_coral_wall_fan".to_string(),
            );
            m.insert(
                "minecraft:horn_coral_fan".to_string(),
                "minecraft:horn_coral_wall_fan".to_string(),
            );
            m
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_all_expected_mappings() {
        let r = RenamedCoralFansFix::renamed_ids();
        assert_eq!(r.len(), 5);
        assert_eq!(
            r.get("minecraft:tube_coral_fan").map(String::as_str),
            Some("minecraft:tube_coral_wall_fan")
        );
        assert_eq!(
            r.get("minecraft:brain_coral_fan").map(String::as_str),
            Some("minecraft:brain_coral_wall_fan")
        );
        assert_eq!(
            r.get("minecraft:bubble_coral_fan").map(String::as_str),
            Some("minecraft:bubble_coral_wall_fan")
        );
        assert_eq!(
            r.get("minecraft:fire_coral_fan").map(String::as_str),
            Some("minecraft:fire_coral_wall_fan")
        );
        assert_eq!(
            r.get("minecraft:horn_coral_fan").map(String::as_str),
            Some("minecraft:horn_coral_wall_fan")
        );
    }

    #[test]
    fn missing_and_edge_case_keys_return_none() {
        let r = RenamedCoralFansFix::renamed_ids();
        assert_eq!(r.get("minecraft:dead_brain_coral_fan"), None);
        assert_eq!(r.get("minecraft:tube_coral_wall_fan"), None);
        assert_eq!(r.get(""), None);
        assert_eq!(r.get("minecraft:TUBE_CORAL_FAN"), None);
    }

    #[test]
    fn renamed_ids_is_stable_singleton() {
        let a = RenamedCoralFansFix::renamed_ids() as *const HashMap<String, String>;
        let b = RenamedCoralFansFix::renamed_ids() as *const HashMap<String, String>;
        assert_eq!(a, b);
    }
}
