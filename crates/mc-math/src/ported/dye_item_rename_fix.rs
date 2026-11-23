use std::collections::HashMap;
use std::sync::LazyLock;

pub struct DyeItemRenameFix;

// JAVA: net.minecraft.util.datafix.fixes.DyeItemRenameFix#RENAMED_IDS
pub static RENAMED_IDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    HashMap::from([
        (
            "minecraft:cactus_green".to_string(),
            "minecraft:green_dye".to_string(),
        ),
        (
            "minecraft:rose_red".to_string(),
            "minecraft:red_dye".to_string(),
        ),
        (
            "minecraft:dandelion_yellow".to_string(),
            "minecraft:yellow_dye".to_string(),
        ),
    ])
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_all_three_mappings() {
        let ids = &*RENAMED_IDS;
        assert_eq!(ids.len(), 3);
        assert_eq!(
            ids.get("minecraft:cactus_green").map(String::as_str),
            Some("minecraft:green_dye")
        );
        assert_eq!(
            ids.get("minecraft:rose_red").map(String::as_str),
            Some("minecraft:red_dye")
        );
        assert_eq!(
            ids.get("minecraft:dandelion_yellow").map(String::as_str),
            Some("minecraft:yellow_dye")
        );
    }

    #[test]
    fn unknown_keys_return_none() {
        let ids = &*RENAMED_IDS;
        assert_eq!(ids.get("minecraft:green_dye"), None);
        assert_eq!(ids.get("minecraft:bone_meal"), None);
        assert_eq!(ids.get(""), None);
    }

    #[test]
    fn keys_are_case_sensitive() {
        let ids = &*RENAMED_IDS;
        assert_eq!(ids.get("minecraft:Cactus_Green"), None);
        assert_eq!(ids.get("MINECRAFT:CACTUS_GREEN"), None);
    }

    #[test]
    fn values_are_exact_expected_strings() {
        let ids = &*RENAMED_IDS;
        for (k, v) in ids.iter() {
            assert!(k.starts_with("minecraft:"));
            assert!(v.starts_with("minecraft:"));
            assert!(v.ends_with("_dye"));
        }
    }
}
