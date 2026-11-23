use std::collections::BTreeMap;
use std::sync::LazyLock;

// JAVA: net.minecraft.util.datafix.fixes.BiomeFix#BIOMES
pub static BIOMES: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let mut biomes = BTreeMap::new();
    biomes.insert("minecraft:extreme_hills".to_string(), "minecraft:mountains".to_string());
    biomes.insert("minecraft:swampland".to_string(), "minecraft:swamp".to_string());
    biomes.insert("minecraft:hell".to_string(), "minecraft:nether_wastes".to_string());
    biomes.insert("minecraft:sky".to_string(), "minecraft:the_end".to_string());
    biomes.insert("minecraft:ice_flats".to_string(), "minecraft:snowy_tundra".to_string());
    biomes.insert("minecraft:ice_mountains".to_string(), "minecraft:snowy_mountains".to_string());
    biomes.insert("minecraft:mushroom_island".to_string(), "minecraft:mushroom_fields".to_string());
    biomes.insert("minecraft:mushroom_island_shore".to_string(), "minecraft:mushroom_field_shore".to_string());
    biomes.insert("minecraft:beaches".to_string(), "minecraft:beach".to_string());
    biomes.insert("minecraft:forest_hills".to_string(), "minecraft:wooded_hills".to_string());
    biomes.insert("minecraft:smaller_extreme_hills".to_string(), "minecraft:mountain_edge".to_string());
    biomes.insert("minecraft:stone_beach".to_string(), "minecraft:stone_shore".to_string());
    biomes.insert("minecraft:cold_beach".to_string(), "minecraft:snowy_beach".to_string());
    biomes.insert("minecraft:roofed_forest".to_string(), "minecraft:dark_forest".to_string());
    biomes.insert("minecraft:taiga_cold".to_string(), "minecraft:snowy_taiga".to_string());
    biomes.insert("minecraft:taiga_cold_hills".to_string(), "minecraft:snowy_taiga_hills".to_string());
    biomes.insert("minecraft:redwood_taiga".to_string(), "minecraft:giant_tree_taiga".to_string());
    biomes.insert("minecraft:redwood_taiga_hills".to_string(), "minecraft:giant_tree_taiga_hills".to_string());
    biomes.insert("minecraft:extreme_hills_with_trees".to_string(), "minecraft:wooded_mountains".to_string());
    biomes.insert("minecraft:savanna_rock".to_string(), "minecraft:savanna_plateau".to_string());
    biomes.insert("minecraft:mesa".to_string(), "minecraft:badlands".to_string());
    biomes.insert("minecraft:mesa_rock".to_string(), "minecraft:wooded_badlands_plateau".to_string());
    biomes.insert("minecraft:mesa_clear_rock".to_string(), "minecraft:badlands_plateau".to_string());
    biomes.insert("minecraft:sky_island_low".to_string(), "minecraft:small_end_islands".to_string());
    biomes.insert("minecraft:sky_island_medium".to_string(), "minecraft:end_midlands".to_string());
    biomes.insert("minecraft:sky_island_high".to_string(), "minecraft:end_highlands".to_string());
    biomes.insert("minecraft:sky_island_barren".to_string(), "minecraft:end_barrens".to_string());
    biomes.insert("minecraft:void".to_string(), "minecraft:the_void".to_string());
    biomes.insert("minecraft:mutated_plains".to_string(), "minecraft:sunflower_plains".to_string());
    biomes.insert("minecraft:mutated_desert".to_string(), "minecraft:desert_lakes".to_string());
    biomes.insert("minecraft:mutated_extreme_hills".to_string(), "minecraft:gravelly_mountains".to_string());
    biomes.insert("minecraft:mutated_forest".to_string(), "minecraft:flower_forest".to_string());
    biomes.insert("minecraft:mutated_taiga".to_string(), "minecraft:taiga_mountains".to_string());
    biomes.insert("minecraft:mutated_swampland".to_string(), "minecraft:swamp_hills".to_string());
    biomes.insert("minecraft:mutated_ice_flats".to_string(), "minecraft:ice_spikes".to_string());
    biomes.insert("minecraft:mutated_jungle".to_string(), "minecraft:modified_jungle".to_string());
    biomes.insert("minecraft:mutated_jungle_edge".to_string(), "minecraft:modified_jungle_edge".to_string());
    biomes.insert("minecraft:mutated_birch_forest".to_string(), "minecraft:tall_birch_forest".to_string());
    biomes.insert("minecraft:mutated_birch_forest_hills".to_string(), "minecraft:tall_birch_hills".to_string());
    biomes.insert("minecraft:mutated_roofed_forest".to_string(), "minecraft:dark_forest_hills".to_string());
    biomes.insert("minecraft:mutated_taiga_cold".to_string(), "minecraft:snowy_taiga_mountains".to_string());
    biomes.insert("minecraft:mutated_redwood_taiga".to_string(), "minecraft:giant_spruce_taiga".to_string());
    biomes.insert("minecraft:mutated_redwood_taiga_hills".to_string(), "minecraft:giant_spruce_taiga_hills".to_string());
    biomes.insert("minecraft:mutated_extreme_hills_with_trees".to_string(), "minecraft:modified_gravelly_mountains".to_string());
    biomes.insert("minecraft:mutated_savanna".to_string(), "minecraft:shattered_savanna".to_string());
    biomes.insert("minecraft:mutated_savanna_rock".to_string(), "minecraft:shattered_savanna_plateau".to_string());
    biomes.insert("minecraft:mutated_mesa".to_string(), "minecraft:eroded_badlands".to_string());
    biomes.insert("minecraft:mutated_mesa_rock".to_string(), "minecraft:modified_wooded_badlands_plateau".to_string());
    biomes.insert("minecraft:mutated_mesa_clear_rock".to_string(), "minecraft:modified_badlands_plateau".to_string());
    biomes.insert("minecraft:warm_deep_ocean".to_string(), "minecraft:deep_warm_ocean".to_string());
    biomes.insert("minecraft:lukewarm_deep_ocean".to_string(), "minecraft:deep_lukewarm_ocean".to_string());
    biomes.insert("minecraft:cold_deep_ocean".to_string(), "minecraft:deep_cold_ocean".to_string());
    biomes.insert("minecraft:frozen_deep_ocean".to_string(), "minecraft:deep_frozen_ocean".to_string());
    biomes
});

// JAVA: net.minecraft.util.datafix.fixes.BiomeFix#BIOMES
pub fn biomes() -> &'static BTreeMap<String, String> {
    &BIOMES
}

// JAVA: net.minecraft.util.datafix.fixes.BiomeFix#BIOMES
pub fn lookup(name: &str) -> Option<&'static str> {
    BIOMES.get(name).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_biome_is_mapped() {
        assert_eq!(lookup("minecraft:hell"), Some("minecraft:nether_wastes"));
        assert_eq!(lookup("minecraft:void"), Some("minecraft:the_void"));
    }

    #[test]
    fn unknown_biome_is_not_mapped() {
        assert_eq!(lookup("minecraft:plains"), None);
        assert_eq!(lookup(""), None);
    }

    #[test]
    fn biome_map_has_expected_size() {
        assert_eq!(BIOMES.len(), 53);
    }

    #[test]
    fn all_keys_and_values_are_namespaced() {
        for (key, value) in BIOMES.iter() {
            assert!(key.starts_with("minecraft:"));
            assert!(value.starts_with("minecraft:"));
            assert_ne!(key, value);
        }
    }
}
