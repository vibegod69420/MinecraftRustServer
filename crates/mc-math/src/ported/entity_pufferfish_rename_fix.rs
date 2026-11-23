use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Debug, Clone)]
pub struct EntityPufferfishRenameFix {
    pub name: &'static str,
    pub schema: Schema,
    pub changes_type: bool,
}

fn renamed_ids_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([(
            "minecraft:puffer_fish_spawn_egg",
            "minecraft:pufferfish_spawn_egg",
        )])
    })
}

/// Port of `EntityPufferfishRenameFix.RENAMED_IDS`.
pub fn renamed_ids() -> &'static HashMap<&'static str, &'static str> {
    renamed_ids_map()
}

impl EntityPufferfishRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityPufferfishRenameFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            name: "EntityPufferfishRenameFix",
            schema,
            changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityPufferfishRenameFix#rename
    pub fn rename(&self, id: &str) -> String {
        if id == "minecraft:puffer_fish" {
            "minecraft:pufferfish".to_string()
        } else {
            id.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_pufferfish_entity() {
        let fix = EntityPufferfishRenameFix::new(Schema, true);
        assert_eq!(fix.rename("minecraft:puffer_fish"), "minecraft:pufferfish");
    }

    #[test]
    fn rename_passthrough_unchanged() {
        let fix = EntityPufferfishRenameFix::new(Schema, false);
        assert_eq!(fix.rename("minecraft:cod"), "minecraft:cod");
        assert_eq!(fix.rename(""), "");
        assert_eq!(fix.rename("minecraft:pufferfish"), "minecraft:pufferfish");
    }

    #[test]
    fn renamed_ids_contains_spawn_egg_mapping() {
        let ids = renamed_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(
            ids.get("minecraft:puffer_fish_spawn_egg"),
            Some(&"minecraft:pufferfish_spawn_egg")
        );
    }

    #[test]
    fn constructor_sets_name_and_changes_type() {
        let fix = EntityPufferfishRenameFix::new(Schema, true);
        assert_eq!(fix.name, "EntityPufferfishRenameFix");
        assert!(fix.changes_type);

        let fix2 = EntityPufferfishRenameFix::new(Schema, false);
        assert!(!fix2.changes_type);
    }
}
