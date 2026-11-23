use std::collections::HashMap;
use std::sync::LazyLock;

// JAVA: net.minecraft.util.datafix.fixes.RecipesFix#RECIPES
pub static RECIPES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("minecraft:acacia_wooden_slab", "minecraft:acacia_slab"),
        ("minecraft:birch_wooden_slab", "minecraft:birch_slab"),
        (
            "minecraft:black_stained_hardened_clay",
            "minecraft:black_terracotta",
        ),
        (
            "minecraft:blue_stained_hardened_clay",
            "minecraft:blue_terracotta",
        ),
        ("minecraft:boat", "minecraft:oak_boat"),
        (
            "minecraft:bone_meal_from_block",
            "minecraft:bone_meal_from_bone_block",
        ),
        ("minecraft:bone_meal_from_bone", "minecraft:bone_meal"),
        ("minecraft:brick_block", "minecraft:bricks"),
        (
            "minecraft:brown_stained_hardened_clay",
            "minecraft:brown_terracotta",
        ),
        (
            "minecraft:chiseled_stonebrick",
            "minecraft:chiseled_stone_bricks",
        ),
        (
            "minecraft:cyan_stained_hardened_clay",
            "minecraft:cyan_terracotta",
        ),
        (
            "minecraft:dark_oak_wooden_slab",
            "minecraft:dark_oak_slab",
        ),
        ("minecraft:end_bricks", "minecraft:end_stone_bricks"),
        ("minecraft:fence_gate", "minecraft:oak_fence_gate"),
        ("minecraft:fence", "minecraft:oak_fence"),
        ("minecraft:golden_rail", "minecraft:powered_rail"),
        (
            "minecraft:gold_ingot_from_block",
            "minecraft:gold_ingot_from_gold_block",
        ),
        (
            "minecraft:gray_stained_hardened_clay",
            "minecraft:gray_terracotta",
        ),
        (
            "minecraft:green_stained_hardened_clay",
            "minecraft:green_terracotta",
        ),
        (
            "minecraft:iron_ingot_from_block",
            "minecraft:iron_ingot_from_iron_block",
        ),
        ("minecraft:jungle_wooden_slab", "minecraft:jungle_slab"),
        (
            "minecraft:light_blue_stained_hardened_clay",
            "minecraft:light_blue_terracotta",
        ),
        (
            "minecraft:light_gray_stained_hardened_clay",
            "minecraft:light_gray_terracotta",
        ),
        (
            "minecraft:lime_stained_hardened_clay",
            "minecraft:lime_terracotta",
        ),
        ("minecraft:lit_pumpkin", "minecraft:jack_o_lantern"),
        (
            "minecraft:magenta_stained_hardened_clay",
            "minecraft:magenta_terracotta",
        ),
        ("minecraft:magma", "minecraft:magma_block"),
        ("minecraft:melon_block", "minecraft:melon"),
        (
            "minecraft:mossy_stonebrick",
            "minecraft:mossy_stone_bricks",
        ),
        ("minecraft:noteblock", "minecraft:note_block"),
        ("minecraft:oak_wooden_slab", "minecraft:oak_slab"),
        (
            "minecraft:orange_stained_hardened_clay",
            "minecraft:orange_terracotta",
        ),
        ("minecraft:pillar_quartz_block", "minecraft:quartz_pillar"),
        (
            "minecraft:pink_stained_hardened_clay",
            "minecraft:pink_terracotta",
        ),
        ("minecraft:purple_shulker_box", "minecraft:shulker_box"),
        (
            "minecraft:purple_stained_hardened_clay",
            "minecraft:purple_terracotta",
        ),
        ("minecraft:red_nether_brick", "minecraft:red_nether_bricks"),
        (
            "minecraft:red_stained_hardened_clay",
            "minecraft:red_terracotta",
        ),
        ("minecraft:slime", "minecraft:slime_block"),
        (
            "minecraft:smooth_red_sandstone",
            "minecraft:cut_red_sandstone",
        ),
        ("minecraft:smooth_sandstone", "minecraft:cut_sandstone"),
        ("minecraft:snow_layer", "minecraft:snow"),
        ("minecraft:snow", "minecraft:snow_block"),
        (
            "minecraft:speckled_melon",
            "minecraft:glistering_melon_slice",
        ),
        ("minecraft:spruce_wooden_slab", "minecraft:spruce_slab"),
        ("minecraft:stonebrick", "minecraft:stone_bricks"),
        ("minecraft:stone_stairs", "minecraft:cobblestone_stairs"),
        (
            "minecraft:string_to_wool",
            "minecraft:white_wool_from_string",
        ),
        ("minecraft:trapdoor", "minecraft:oak_trapdoor"),
        (
            "minecraft:white_stained_hardened_clay",
            "minecraft:white_terracotta",
        ),
        ("minecraft:wooden_button", "minecraft:oak_button"),
        ("minecraft:wooden_door", "minecraft:oak_door"),
        (
            "minecraft:wooden_pressure_plate",
            "minecraft:oak_pressure_plate",
        ),
        (
            "minecraft:yellow_stained_hardened_clay",
            "minecraft:yellow_terracotta",
        ),
    ])
});

#[cfg(test)]
mod tests {
    use super::RECIPES;

    #[test]
    fn recipes_size_matches_java() {
        assert_eq!(RECIPES.len(), 54);
    }

    #[test]
    fn known_mappings_match_java() {
        assert_eq!(RECIPES.get("minecraft:boat"), Some(&"minecraft:oak_boat"));
        assert_eq!(
            RECIPES.get("minecraft:lit_pumpkin"),
            Some(&"minecraft:jack_o_lantern")
        );
        assert_eq!(
            RECIPES.get("minecraft:stone_stairs"),
            Some(&"minecraft:cobblestone_stairs")
        );
        assert_eq!(
            RECIPES.get("minecraft:purple_shulker_box"),
            Some(&"minecraft:shulker_box")
        );
    }

    #[test]
    fn missing_key_returns_none() {
        assert_eq!(RECIPES.get("minecraft:does_not_exist"), None);
        assert_eq!(RECIPES.get(""), None);
        assert_eq!(RECIPES.get("minecraft:oak_boat"), None);
    }

    #[test]
    fn snow_and_snow_layer_are_distinct_mappings() {
        assert_eq!(RECIPES.get("minecraft:snow_layer"), Some(&"minecraft:snow"));
        assert_eq!(
            RECIPES.get("minecraft:snow"),
            Some(&"minecraft:snow_block")
        );
    }

    #[test]
    fn all_entries_are_namespaced_and_non_empty() {
        for (key, value) in RECIPES.iter() {
            assert!(key.starts_with("minecraft:"), "key not namespaced: {key}");
            assert!(
                value.starts_with("minecraft:"),
                "value not namespaced: {value}"
            );
            assert!(!key["minecraft:".len()..].is_empty(), "empty key name");
            assert!(
                !value["minecraft:".len()..].is_empty(),
                "empty value name"
            );
            assert_ne!(key, value, "identity mapping: {key}");
        }
    }
}
