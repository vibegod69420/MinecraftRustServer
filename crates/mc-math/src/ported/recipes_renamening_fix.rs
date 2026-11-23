use std::collections::HashMap;
use std::sync::OnceLock;

pub struct RecipesRenameningFix;

impl RecipesRenameningFix {
    // JAVA: net.minecraft.util.datafix.fixes.RecipesRenameningFix#RECIPES
    pub fn recipes() -> &'static HashMap<String, String> {
        static RECIPES: OnceLock<HashMap<String, String>> = OnceLock::new();
        RECIPES.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert(
                "minecraft:acacia_bark".to_string(),
                "minecraft:acacia_wood".to_string(),
            );
            m.insert(
                "minecraft:birch_bark".to_string(),
                "minecraft:birch_wood".to_string(),
            );
            m.insert(
                "minecraft:dark_oak_bark".to_string(),
                "minecraft:dark_oak_wood".to_string(),
            );
            m.insert(
                "minecraft:jungle_bark".to_string(),
                "minecraft:jungle_wood".to_string(),
            );
            m.insert(
                "minecraft:oak_bark".to_string(),
                "minecraft:oak_wood".to_string(),
            );
            m.insert(
                "minecraft:spruce_bark".to_string(),
                "minecraft:spruce_wood".to_string(),
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
        let r = RecipesRenameningFix::recipes();
        assert_eq!(r.len(), 6);
        assert_eq!(
            r.get("minecraft:acacia_bark").map(String::as_str),
            Some("minecraft:acacia_wood")
        );
        assert_eq!(
            r.get("minecraft:birch_bark").map(String::as_str),
            Some("minecraft:birch_wood")
        );
        assert_eq!(
            r.get("minecraft:dark_oak_bark").map(String::as_str),
            Some("minecraft:dark_oak_wood")
        );
        assert_eq!(
            r.get("minecraft:jungle_bark").map(String::as_str),
            Some("minecraft:jungle_wood")
        );
        assert_eq!(
            r.get("minecraft:oak_bark").map(String::as_str),
            Some("minecraft:oak_wood")
        );
        assert_eq!(
            r.get("minecraft:spruce_bark").map(String::as_str),
            Some("minecraft:spruce_wood")
        );
    }

    #[test]
    fn missing_and_edge_case_keys_return_none() {
        let r = RecipesRenameningFix::recipes();
        assert_eq!(r.get("minecraft:stone"), None);
        assert_eq!(r.get("minecraft:acacia_wood"), None);
        assert_eq!(r.get(""), None);
        assert_eq!(r.get("minecraft:ACACIA_BARK"), None);
    }

    #[test]
    fn recipes_is_stable_singleton() {
        let a = RecipesRenameningFix::recipes() as *const HashMap<String, String>;
        let b = RecipesRenameningFix::recipes() as *const HashMap<String, String>;
        assert_eq!(a, b);
    }
}
