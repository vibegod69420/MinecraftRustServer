use std::collections::HashMap;
use std::sync::OnceLock;

/// Faithful port of `net.minecraft.util.datafix.fixes.BlockEntityIdFix`.
///
/// Maps legacy block entity IDs to modern namespaced IDs.
pub struct BlockEntityIdFix {
    pub bypass: bool,
}

impl BlockEntityIdFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityIdFix#<init>
    pub fn new(bypass: bool) -> Self {
        Self { bypass }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityIdFix#makeRule
    pub fn fix_block_entity_id<'a>(&self, id: &'a str) -> &'a str {
        block_entity_id_map().get(id).copied().unwrap_or(id)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.BlockEntityIdFix#ID_MAP
pub fn block_entity_id_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            ("Airportal", "minecraft:end_portal"),
            ("Banner", "minecraft:banner"),
            ("Beacon", "minecraft:beacon"),
            ("Cauldron", "minecraft:brewing_stand"),
            ("Chest", "minecraft:chest"),
            ("Comparator", "minecraft:comparator"),
            ("Control", "minecraft:command_block"),
            ("DLDetector", "minecraft:daylight_detector"),
            ("Dropper", "minecraft:dropper"),
            ("EnchantTable", "minecraft:enchanting_table"),
            ("EndGateway", "minecraft:end_gateway"),
            ("EnderChest", "minecraft:ender_chest"),
            ("FlowerPot", "minecraft:flower_pot"),
            ("Furnace", "minecraft:furnace"),
            ("Hopper", "minecraft:hopper"),
            ("MobSpawner", "minecraft:mob_spawner"),
            ("Music", "minecraft:noteblock"),
            ("Piston", "minecraft:piston"),
            ("RecordPlayer", "minecraft:jukebox"),
            ("Sign", "minecraft:sign"),
            ("Skull", "minecraft:skull"),
            ("Structure", "minecraft:structure_block"),
            ("Trap", "minecraft:dispenser"),
        ])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_mappings() {
        let fix = BlockEntityIdFix::new(false);
        assert_eq!(fix.fix_block_entity_id("Airportal"), "minecraft:end_portal");
        assert_eq!(fix.fix_block_entity_id("Chest"), "minecraft:chest");
        assert_eq!(fix.fix_block_entity_id("Trap"), "minecraft:dispenser");
        assert_eq!(fix.fix_block_entity_id("Control"), "minecraft:command_block");
    }

    #[test]
    fn test_unknown_id_passthrough() {
        let fix = BlockEntityIdFix::new(true);
        assert_eq!(fix.fix_block_entity_id("Unknown"), "Unknown");
        assert_eq!(fix.fix_block_entity_id("minecraft:chest"), "minecraft:chest");
        assert_eq!(fix.fix_block_entity_id(""), "");
    }

    #[test]
    fn test_id_map_completeness_and_values() {
        let map = block_entity_id_map();
        assert_eq!(map.len(), 23);
        assert_eq!(map.get("Music"), Some(&"minecraft:noteblock"));
        assert_eq!(map.get("RecordPlayer"), Some(&"minecraft:jukebox"));
        assert_eq!(map.get("DLDetector"), Some(&"minecraft:daylight_detector"));
    }

    #[test]
    fn test_case_sensitive_and_edge_cases() {
        let fix = BlockEntityIdFix::new(false);
        assert_eq!(fix.fix_block_entity_id("chest"), "chest");
        assert_eq!(fix.fix_block_entity_id("CHEST"), "CHEST");
        assert_eq!(fix.fix_block_entity_id(" Chest"), " Chest");
    }
}
