use std::collections::HashMap;
use std::sync::LazyLock;

pub static RENAMES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("minecraft:badlands_plateau", "minecraft:badlands"),
        ("minecraft:bamboo_jungle_hills", "minecraft:bamboo_jungle"),
        ("minecraft:birch_forest_hills", "minecraft:birch_forest"),
        ("minecraft:dark_forest_hills", "minecraft:dark_forest"),
        ("minecraft:desert_hills", "minecraft:desert"),
        ("minecraft:desert_lakes", "minecraft:desert"),
        (
            "minecraft:giant_spruce_taiga_hills",
            "minecraft:old_growth_spruce_taiga",
        ),
        (
            "minecraft:giant_spruce_taiga",
            "minecraft:old_growth_spruce_taiga",
        ),
        (
            "minecraft:giant_tree_taiga_hills",
            "minecraft:old_growth_pine_taiga",
        ),
        (
            "minecraft:giant_tree_taiga",
            "minecraft:old_growth_pine_taiga",
        ),
        (
            "minecraft:gravelly_mountains",
            "minecraft:windswept_gravelly_hills",
        ),
        ("minecraft:jungle_edge", "minecraft:sparse_jungle"),
        ("minecraft:jungle_hills", "minecraft:jungle"),
        ("minecraft:modified_badlands_plateau", "minecraft:badlands"),
        (
            "minecraft:modified_gravelly_mountains",
            "minecraft:windswept_gravelly_hills",
        ),
        ("minecraft:modified_jungle_edge", "minecraft:sparse_jungle"),
        ("minecraft:modified_jungle", "minecraft:jungle"),
        (
            "minecraft:modified_wooded_badlands_plateau",
            "minecraft:wooded_badlands",
        ),
        ("minecraft:mountain_edge", "minecraft:windswept_hills"),
        ("minecraft:mountains", "minecraft:windswept_hills"),
        ("minecraft:mushroom_field_shore", "minecraft:mushroom_fields"),
        ("minecraft:shattered_savanna", "minecraft:windswept_savanna"),
        (
            "minecraft:shattered_savanna_plateau",
            "minecraft:windswept_savanna",
        ),
        ("minecraft:snowy_mountains", "minecraft:snowy_plains"),
        ("minecraft:snowy_taiga_hills", "minecraft:snowy_taiga"),
        ("minecraft:snowy_taiga_mountains", "minecraft:snowy_taiga"),
        ("minecraft:snowy_tundra", "minecraft:snowy_plains"),
        ("minecraft:stone_shore", "minecraft:stony_shore"),
        ("minecraft:swamp_hills", "minecraft:swamp"),
        ("minecraft:taiga_hills", "minecraft:taiga"),
        ("minecraft:taiga_mountains", "minecraft:taiga"),
        (
            "minecraft:tall_birch_forest",
            "minecraft:old_growth_birch_forest",
        ),
        (
            "minecraft:tall_birch_hills",
            "minecraft:old_growth_birch_forest",
        ),
        (
            "minecraft:wooded_badlands_plateau",
            "minecraft:wooded_badlands",
        ),
        ("minecraft:wooded_hills", "minecraft:forest"),
        ("minecraft:wooded_mountains", "minecraft:windswept_forest"),
        ("minecraft:lofty_peaks", "minecraft:jagged_peaks"),
        ("minecraft:snowcapped_peaks", "minecraft:frozen_peaks"),
    ])
});

pub struct CavesAndCliffsRenames;

impl CavesAndCliffsRenames {
    // JAVA: net.minecraft.util.datafix.fixes.CavesAndCliffsRenames#<init>
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_known_entries() {
        assert_eq!(
            RENAMES.get("minecraft:badlands_plateau"),
            Some(&"minecraft:badlands")
        );
        assert_eq!(
            RENAMES.get("minecraft:stone_shore"),
            Some(&"minecraft:stony_shore")
        );
        assert_eq!(
            RENAMES.get("minecraft:snowcapped_peaks"),
            Some(&"minecraft:frozen_peaks")
        );
    }

    #[test]
    fn renames_missing_key_returns_none() {
        assert_eq!(RENAMES.get("minecraft:plains"), None);
        assert_eq!(RENAMES.get(""), None);
        assert_eq!(RENAMES.get("badlands_plateau"), None);
    }

    #[test]
    fn renames_count_and_namespaced() {
        assert_eq!(RENAMES.len(), 38);
        for (k, v) in RENAMES.iter() {
            assert!(k.starts_with("minecraft:"), "key not namespaced: {k}");
            assert!(v.starts_with("minecraft:"), "value not namespaced: {v}");
        }
    }

    #[test]
    fn renames_collapse_targets() {
        assert_eq!(
            RENAMES.get("minecraft:giant_spruce_taiga"),
            RENAMES.get("minecraft:giant_spruce_taiga_hills")
        );
        assert_eq!(
            RENAMES.get("minecraft:snowy_tundra"),
            Some(&"minecraft:snowy_plains")
        );
        assert_eq!(
            RENAMES.get("minecraft:snowy_mountains"),
            Some(&"minecraft:snowy_plains")
        );
    }
}
