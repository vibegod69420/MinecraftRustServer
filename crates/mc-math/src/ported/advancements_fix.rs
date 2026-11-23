use std::ops::Deref;

// JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#RENAMES
pub const RENAMES_ENTRIES: &[(&str, &str)] = &[
    ("minecraft:recipes/brewing/speckled_melon", "minecraft:recipes/brewing/glistering_melon_slice"),
    ("minecraft:recipes/building_blocks/black_stained_hardened_clay", "minecraft:recipes/building_blocks/black_terracotta"),
    ("minecraft:recipes/building_blocks/blue_stained_hardened_clay", "minecraft:recipes/building_blocks/blue_terracotta"),
    ("minecraft:recipes/building_blocks/brown_stained_hardened_clay", "minecraft:recipes/building_blocks/brown_terracotta"),
    ("minecraft:recipes/building_blocks/cyan_stained_hardened_clay", "minecraft:recipes/building_blocks/cyan_terracotta"),
    ("minecraft:recipes/building_blocks/gray_stained_hardened_clay", "minecraft:recipes/building_blocks/gray_terracotta"),
    ("minecraft:recipes/building_blocks/green_stained_hardened_clay", "minecraft:recipes/building_blocks/green_terracotta"),
    ("minecraft:recipes/building_blocks/light_blue_stained_hardened_clay", "minecraft:recipes/building_blocks/light_blue_terracotta"),
    ("minecraft:recipes/building_blocks/light_gray_stained_hardened_clay", "minecraft:recipes/building_blocks/light_gray_terracotta"),
    ("minecraft:recipes/building_blocks/lime_stained_hardened_clay", "minecraft:recipes/building_blocks/lime_terracotta"),
    ("minecraft:recipes/building_blocks/magenta_stained_hardened_clay", "minecraft:recipes/building_blocks/magenta_terracotta"),
    ("minecraft:recipes/building_blocks/orange_stained_hardened_clay", "minecraft:recipes/building_blocks/orange_terracotta"),
    ("minecraft:recipes/building_blocks/pink_stained_hardened_clay", "minecraft:recipes/building_blocks/pink_terracotta"),
    ("minecraft:recipes/building_blocks/purple_stained_hardened_clay", "minecraft:recipes/building_blocks/purple_terracotta"),
    ("minecraft:recipes/building_blocks/red_stained_hardened_clay", "minecraft:recipes/building_blocks/red_terracotta"),
    ("minecraft:recipes/building_blocks/white_stained_hardened_clay", "minecraft:recipes/building_blocks/white_terracotta"),
    ("minecraft:recipes/building_blocks/yellow_stained_hardened_clay", "minecraft:recipes/building_blocks/yellow_terracotta"),
    ("minecraft:recipes/building_blocks/acacia_wooden_slab", "minecraft:recipes/building_blocks/acacia_slab"),
    ("minecraft:recipes/building_blocks/birch_wooden_slab", "minecraft:recipes/building_blocks/birch_slab"),
    ("minecraft:recipes/building_blocks/dark_oak_wooden_slab", "minecraft:recipes/building_blocks/dark_oak_slab"),
    ("minecraft:recipes/building_blocks/jungle_wooden_slab", "minecraft:recipes/building_blocks/jungle_slab"),
    ("minecraft:recipes/building_blocks/oak_wooden_slab", "minecraft:recipes/building_blocks/oak_slab"),
    ("minecraft:recipes/building_blocks/spruce_wooden_slab", "minecraft:recipes/building_blocks/spruce_slab"),
    ("minecraft:recipes/building_blocks/brick_block", "minecraft:recipes/building_blocks/bricks"),
    ("minecraft:recipes/building_blocks/chiseled_stonebrick", "minecraft:recipes/building_blocks/chiseled_stone_bricks"),
    ("minecraft:recipes/building_blocks/end_bricks", "minecraft:recipes/building_blocks/end_stone_bricks"),
    ("minecraft:recipes/building_blocks/lit_pumpkin", "minecraft:recipes/building_blocks/jack_o_lantern"),
    ("minecraft:recipes/building_blocks/magma", "minecraft:recipes/building_blocks/magma_block"),
    ("minecraft:recipes/building_blocks/melon_block", "minecraft:recipes/building_blocks/melon"),
    ("minecraft:recipes/building_blocks/mossy_stonebrick", "minecraft:recipes/building_blocks/mossy_stone_bricks"),
    ("minecraft:recipes/building_blocks/nether_brick", "minecraft:recipes/building_blocks/nether_bricks"),
    ("minecraft:recipes/building_blocks/pillar_quartz_block", "minecraft:recipes/building_blocks/quartz_pillar"),
    ("minecraft:recipes/building_blocks/red_nether_brick", "minecraft:recipes/building_blocks/red_nether_bricks"),
    ("minecraft:recipes/building_blocks/snow", "minecraft:recipes/building_blocks/snow_block"),
    ("minecraft:recipes/building_blocks/smooth_red_sandstone", "minecraft:recipes/building_blocks/cut_red_sandstone"),
    ("minecraft:recipes/building_blocks/smooth_sandstone", "minecraft:recipes/building_blocks/cut_sandstone"),
    ("minecraft:recipes/building_blocks/stonebrick", "minecraft:recipes/building_blocks/stone_bricks"),
    ("minecraft:recipes/building_blocks/stone_stairs", "minecraft:recipes/building_blocks/cobblestone_stairs"),
    ("minecraft:recipes/building_blocks/string_to_wool", "minecraft:recipes/building_blocks/white_wool_from_string"),
    ("minecraft:recipes/decorations/fence", "minecraft:recipes/decorations/oak_fence"),
    ("minecraft:recipes/decorations/purple_shulker_box", "minecraft:recipes/decorations/shulker_box"),
    ("minecraft:recipes/decorations/slime", "minecraft:recipes/decorations/slime_block"),
    ("minecraft:recipes/decorations/snow_layer", "minecraft:recipes/decorations/snow"),
    ("minecraft:recipes/misc/bone_meal_from_block", "minecraft:recipes/misc/bone_meal_from_bone_block"),
    ("minecraft:recipes/misc/bone_meal_from_bone", "minecraft:recipes/misc/bone_meal"),
    ("minecraft:recipes/misc/gold_ingot_from_block", "minecraft:recipes/misc/gold_ingot_from_gold_block"),
    ("minecraft:recipes/misc/iron_ingot_from_block", "minecraft:recipes/misc/iron_ingot_from_iron_block"),
    ("minecraft:recipes/redstone/fence_gate", "minecraft:recipes/redstone/oak_fence_gate"),
    ("minecraft:recipes/redstone/noteblock", "minecraft:recipes/redstone/note_block"),
    ("minecraft:recipes/redstone/trapdoor", "minecraft:recipes/redstone/oak_trapdoor"),
    ("minecraft:recipes/redstone/wooden_button", "minecraft:recipes/redstone/oak_button"),
    ("minecraft:recipes/redstone/wooden_door", "minecraft:recipes/redstone/oak_door"),
    ("minecraft:recipes/redstone/wooden_pressure_plate", "minecraft:recipes/redstone/oak_pressure_plate"),
    ("minecraft:recipes/transportation/boat", "minecraft:recipes/transportation/oak_boat"),
    ("minecraft:recipes/transportation/golden_rail", "minecraft:recipes/transportation/powered_rail"),
];

// JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#getOrDefault
pub struct RenameMap;

impl RenameMap {
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#getOrDefault
    pub fn get(&self, key: &str) -> Option<&'static str> {
        for (from, to) in RENAMES_ENTRIES {
            if *from == key {
                return Some(*to);
            }
        }
        None
    }

    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#getOrDefault
    pub fn get_or_default<'a>(&self, key: &'a str) -> &'a str {
        rename_advancement(key)
    }

    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#containsKey
    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    // JAVA: java.util.Map#size
    pub fn len(&self) -> usize {
        RENAMES_ENTRIES.len()
    }

    // JAVA: java.util.Map#isEmpty
    pub fn is_empty(&self) -> bool {
        RENAMES_ENTRIES.is_empty()
    }

    // JAVA: java.util.Map#entrySet
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static str)> {
        RENAMES_ENTRIES.iter().copied()
    }
}

impl Deref for RenameMap {
    type Target = [(&'static str, &'static str)];

    fn deref(&self) -> &Self::Target {
        RENAMES_ENTRIES
    }
}

// JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#RENAMES
pub static RENAMES: RenameMap = RenameMap;

// JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#rename
pub fn rename_advancement<'a>(advancement_name: &'a str) -> &'a str {
    for (from, to) in RENAMES_ENTRIES {
        if *from == advancement_name {
            return *to;
        }
    }
    advancement_name
}

// JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#rename
pub fn rename_advancement_owned(advancement_name: &str) -> String {
    rename_advancement(advancement_name).to_string()
}

// JAVA: net.minecraft.util.datafix.fixes.AdvancementsRenameFix
pub struct AdvancementsRenameFix;

impl AdvancementsRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsRenameFix#AdvancementsRenameFix
    pub fn new<S, F>(_schema: S, _changes_type: bool, _name: impl Into<String>, _renamer: F) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsRenameFix#getName
    pub fn name(&self) -> &'static str {
        "AdvancementsRenameFix"
    }

    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsRenameFix#rename
    pub fn rename<'a>(&self, advancement_name: &'a str) -> &'a str {
        rename_advancement(advancement_name)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#AdvancementsFix
pub struct AdvancementsFix;

impl AdvancementsFix {
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#AdvancementsFix
    pub fn new<S>(_schema: S, _changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#getName
    pub fn name(&self) -> &'static str {
        "AdvancementsFix"
    }

    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#rename
    pub fn rename<'a>(&self, advancement_name: &'a str) -> &'a str {
        rename_advancement(advancement_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#rename
    fn renames_known_advancement() {
        assert_eq!(
            rename_advancement("minecraft:recipes/brewing/speckled_melon"),
            "minecraft:recipes/brewing/glistering_melon_slice"
        );
    }

    #[test]
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#rename
    fn unknown_advancement_is_unchanged() {
        let input = "minecraft:story/root";
        assert_eq!(rename_advancement(input), input);
    }

    #[test]
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#rename
    fn rename_preserves_input_lifetime() {
        let input = String::from("minecraft:recipes/redstone/noteblock");
        let output = rename_advancement(&input);
        assert_eq!(output, "minecraft:recipes/redstone/note_block");
    }

    #[test]
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#RENAMES
    fn rename_map_contains_expected_entries() {
        assert!(RENAMES.contains_key("minecraft:recipes/transportation/boat"));
        assert_eq!(RENAMES.get("minecraft:recipes/transportation/boat"), Some("minecraft:recipes/transportation/oak_boat"));
        assert_eq!(RENAMES.len(), 55);
    }

    #[test]
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#AdvancementsFix
    fn fix_object_delegates_rename() {
        let fix = AdvancementsFix::new((), true);
        assert_eq!(
            fix.rename("minecraft:recipes/building_blocks/brick_block"),
            "minecraft:recipes/building_blocks/bricks"
        );
    }

    #[test]
    // JAVA: net.minecraft.util.datafix.fixes.AdvancementsFix#rename
    fn all_rename_targets_are_distinct_from_sources() {
        for (from, to) in RENAMES_ENTRIES {
            assert_ne!(*from, *to);
        }
    }
}
