use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Debug, Clone)]
pub struct EntityRavagerRenameFix {
    pub name: &'static str,
    pub schema: Schema,
    pub changes_type: bool,
}

fn renamed_ids_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([(
            "minecraft:illager_beast_spawn_egg",
            "minecraft:ravager_spawn_egg",
        )])
    })
}

/// Port of `EntityRavagerRenameFix.RENAMED_IDS`.
pub fn renamed_ids() -> &'static HashMap<&'static str, &'static str> {
    renamed_ids_map()
}

impl EntityRavagerRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityRavagerRenameFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            name: "EntityRavagerRenameFix",
            schema,
            changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityRavagerRenameFix#rename
    pub fn rename(&self, id: &str) -> String {
        if id == "minecraft:illager_beast" {
            "minecraft:ravager".to_string()
        } else {
            id.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_ravager_entity() {
        let fix = EntityRavagerRenameFix::new(Schema, true);
        assert_eq!(fix.rename("minecraft:illager_beast"), "minecraft:ravager");
    }

    #[test]
    fn rename_passthrough_unchanged() {
        let fix = EntityRavagerRenameFix::new(Schema, false);
        assert_eq!(fix.rename("minecraft:zombie"), "minecraft:zombie");
        assert_eq!(fix.rename(""), "");
        assert_eq!(fix.rename("minecraft:ravager"), "minecraft:ravager");
    }

    #[test]
    fn renamed_ids_contains_spawn_egg_mapping() {
        let ids = renamed_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(
            ids.get("minecraft:illager_beast_spawn_egg"),
            Some(&"minecraft:ravager_spawn_egg")
        );
    }

    #[test]
    fn constructor_sets_name_and_changes_type() {
        let fix = EntityRavagerRenameFix::new(Schema, true);
        assert_eq!(fix.name, "EntityRavagerRenameFix");
        assert!(fix.changes_type);

        let fix2 = EntityRavagerRenameFix::new(Schema, false);
        assert!(!fix2.changes_type);
    }
}
