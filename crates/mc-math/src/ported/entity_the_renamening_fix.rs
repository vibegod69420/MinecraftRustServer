use std::collections::HashMap;

// JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#RENAMED_IDS
pub const RENAMED_IDS_ENTRIES: &[(&str, &str)] = &[
    ("minecraft:commandblock_minecart", "minecraft:command_block_minecart"),
    ("minecraft:ender_crystal", "minecraft:end_crystal"),
    ("minecraft:snowman", "minecraft:snow_golem"),
    ("minecraft:evocation_illager", "minecraft:evoker"),
    ("minecraft:evocation_fangs", "minecraft:evoker_fangs"),
    ("minecraft:illusion_illager", "minecraft:illusioner"),
    ("minecraft:vindication_illager", "minecraft:vindicator"),
    ("minecraft:villager_golem", "minecraft:iron_golem"),
    ("minecraft:xp_orb", "minecraft:experience_orb"),
    ("minecraft:xp_bottle", "minecraft:experience_bottle"),
    ("minecraft:eye_of_ender_signal", "minecraft:eye_of_ender"),
    ("minecraft:fireworks_rocket", "minecraft:firework_rocket"),
];

// JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#RENAMED_BLOCKS
pub const RENAMED_BLOCKS_ENTRIES: &[(&str, &str)] = &[
    ("minecraft:portal", "minecraft:nether_portal"),
    ("minecraft:oak_bark", "minecraft:oak_wood"),
    ("minecraft:spruce_bark", "minecraft:spruce_wood"),
    ("minecraft:birch_bark", "minecraft:birch_wood"),
    ("minecraft:jungle_bark", "minecraft:jungle_wood"),
    ("minecraft:acacia_bark", "minecraft:acacia_wood"),
    ("minecraft:dark_oak_bark", "minecraft:dark_oak_wood"),
    ("minecraft:stripped_oak_bark", "minecraft:stripped_oak_wood"),
    ("minecraft:stripped_spruce_bark", "minecraft:stripped_spruce_wood"),
    ("minecraft:stripped_birch_bark", "minecraft:stripped_birch_wood"),
    ("minecraft:stripped_jungle_bark", "minecraft:stripped_jungle_wood"),
    ("minecraft:stripped_acacia_bark", "minecraft:stripped_acacia_wood"),
    ("minecraft:stripped_dark_oak_bark", "minecraft:stripped_dark_oak_wood"),
    ("minecraft:mob_spawner", "minecraft:spawner"),
];

// JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#RENAMED_ITEMS
pub fn renamed_items_entries() -> Vec<(&'static str, &'static str)> {
    let mut out = RENAMED_BLOCKS_ENTRIES.to_vec();
    out.push(("minecraft:clownfish", "minecraft:tropical_fish"));
    out.push(("minecraft:chorus_fruit_popped", "minecraft:popped_chorus_fruit"));
    out.push(("minecraft:evocation_illager_spawn_egg", "minecraft:evoker_spawn_egg"));
    out.push(("minecraft:vindication_illager_spawn_egg", "minecraft:vindicator_spawn_egg"));
    out
}

// JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#RENAMED_IDS
pub fn renamed_ids() -> HashMap<String, String> {
    RENAMED_IDS_ENTRIES.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

// JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#RENAMED_BLOCKS
pub fn renamed_blocks() -> HashMap<String, String> {
    RENAMED_BLOCKS_ENTRIES.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

// JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#RENAMED_ITEMS
pub fn renamed_items() -> HashMap<String, String> {
    renamed_items_entries()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#rename
pub fn rename(input: &str) -> String {
    let mut value = input.to_string();
    const PREFIX: &str = "minecraft:bred_";
    if value.starts_with(PREFIX) {
        value = format!("minecraft:{}", &value[PREFIX.len()..]);
    }
    renamed_ids().get(&value).cloned().unwrap_or(value)
}

// JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#EntityTheRenameningFix
pub struct EntityTheRenameningFix;

impl EntityTheRenameningFix {
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityTheRenameningFix#rename
    pub fn rename(&self, input: &str) -> String {
        rename(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renamed_ids_contains_expected() {
        let map = renamed_ids();
        assert_eq!(map.get("minecraft:snowman"), Some(&"minecraft:snow_golem".to_string()));
        assert_eq!(map.get("minecraft:missing"), None);
    }

    #[test]
    fn renamed_items_includes_blocks_and_extras() {
        let map = renamed_items();
        assert_eq!(map.get("minecraft:mob_spawner"), Some(&"minecraft:spawner".to_string()));
        assert_eq!(map.get("minecraft:clownfish"), Some(&"minecraft:tropical_fish".to_string()));
    }

    #[test]
    fn rename_bred_prefix() {
        assert_eq!(rename("minecraft:bred_foo"), "minecraft:foo");
    }

    #[test]
    fn rename_known_id() {
        assert_eq!(rename("minecraft:xp_orb"), "minecraft:experience_orb");
    }

    #[test]
    fn rename_unknown_unchanged() {
        assert_eq!(rename("minecraft:stone"), "minecraft:stone");
    }
}
