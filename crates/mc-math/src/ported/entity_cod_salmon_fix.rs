use std::collections::HashMap;
use std::sync::LazyLock;

pub static RENAMED_IDS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("minecraft:salmon_mob", "minecraft:salmon"),
        ("minecraft:cod_mob", "minecraft:cod"),
    ])
});

pub static RENAMED_EGG_IDS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("minecraft:salmon_mob_spawn_egg", "minecraft:salmon_spawn_egg"),
        ("minecraft:cod_mob_spawn_egg", "minecraft:cod_spawn_egg"),
    ])
});

pub struct EntityCodSalmonFix {
    pub name: &'static str,
}

impl EntityCodSalmonFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityCodSalmonFix#<init>
    pub fn new(_schema: (), _changes_type: bool) -> Self {
        Self {
            name: "EntityCodSalmonFix",
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityCodSalmonFix#rename
    pub fn rename(&self, id: &str) -> String {
        RENAMED_IDS.get(id).copied().unwrap_or(id).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_cod_and_salmon_mobs() {
        let fix = EntityCodSalmonFix::new((), true);
        assert_eq!(fix.rename("minecraft:salmon_mob"), "minecraft:salmon");
        assert_eq!(fix.rename("minecraft:cod_mob"), "minecraft:cod");
    }

    #[test]
    fn unknown_id_passes_through() {
        let fix = EntityCodSalmonFix::new((), false);
        assert_eq!(fix.rename("minecraft:pig"), "minecraft:pig");
        assert_eq!(fix.rename(""), "");
    }

    #[test]
    fn egg_ids_map_contains_expected_entries() {
        assert_eq!(
            RENAMED_EGG_IDS.get("minecraft:salmon_mob_spawn_egg"),
            Some(&"minecraft:salmon_spawn_egg")
        );
        assert_eq!(
            RENAMED_EGG_IDS.get("minecraft:cod_mob_spawn_egg"),
            Some(&"minecraft:cod_spawn_egg")
        );
        assert_eq!(RENAMED_EGG_IDS.len(), 2);
    }

    #[test]
    fn renamed_ids_map_contains_expected_entries() {
        assert_eq!(RENAMED_IDS.len(), 2);
        assert_eq!(RENAMED_IDS.get("minecraft:salmon_mob"), Some(&"minecraft:salmon"));
        assert_eq!(RENAMED_IDS.get("minecraft:cod_mob"), Some(&"minecraft:cod"));
    }

    #[test]
    fn does_not_rename_egg_ids_via_rename_method() {
        let fix = EntityCodSalmonFix::new((), true);
        // rename() only uses RENAMED_IDS, matching Java behavior
        assert_eq!(
            fix.rename("minecraft:salmon_mob_spawn_egg"),
            "minecraft:salmon_mob_spawn_egg"
        );
    }
}
