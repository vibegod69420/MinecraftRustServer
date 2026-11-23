use std::collections::HashMap;

/// Faithful port of `net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackSpawnEggFix {
    item_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub id: String,
    pub entity_id: Option<String>,
}

impl ItemStackSpawnEggFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix#<init>
    pub fn new(item_type: impl Into<String>) -> Self {
        Self {
            item_type: item_type.into(),
        }
    }

    pub fn item_type(&self) -> &str {
        &self.item_type
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix#makeRule (item-stack transformation portion)
    pub fn fix_item_stack(&self, stack: &mut ItemStack) -> bool {
        if stack.id != self.item_type {
            return false;
        }

        if let Some(entity_id) = &stack.entity_id {
            stack.id = spawn_egg_for(entity_id).to_string();
            true
        } else {
            false
        }
    }

    pub fn apply(&self, stack: &mut ItemStack) -> bool {
        self.fix_item_stack(stack)
    }

    pub fn fix(&self, stack: &mut ItemStack) -> bool {
        self.fix_item_stack(stack)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix#MAP
pub fn spawn_egg_for(entity_id: &str) -> &'static str {
    match entity_id {
        "minecraft:bat" => "minecraft:bat_spawn_egg",
        "minecraft:blaze" => "minecraft:blaze_spawn_egg",
        "minecraft:cave_spider" => "minecraft:cave_spider_spawn_egg",
        "minecraft:chicken" => "minecraft:chicken_spawn_egg",
        "minecraft:cow" => "minecraft:cow_spawn_egg",
        "minecraft:creeper" => "minecraft:creeper_spawn_egg",
        "minecraft:donkey" => "minecraft:donkey_spawn_egg",
        "minecraft:elder_guardian" => "minecraft:elder_guardian_spawn_egg",
        "minecraft:ender_dragon" => "minecraft:ender_dragon_spawn_egg",
        "minecraft:enderman" => "minecraft:enderman_spawn_egg",
        "minecraft:endermite" => "minecraft:endermite_spawn_egg",
        "minecraft:evocation_illager" => "minecraft:evocation_illager_spawn_egg",
        "minecraft:ghast" => "minecraft:ghast_spawn_egg",
        "minecraft:guardian" => "minecraft:guardian_spawn_egg",
        "minecraft:horse" => "minecraft:horse_spawn_egg",
        "minecraft:husk" => "minecraft:husk_spawn_egg",
        "minecraft:iron_golem" => "minecraft:iron_golem_spawn_egg",
        "minecraft:llama" => "minecraft:llama_spawn_egg",
        "minecraft:magma_cube" => "minecraft:magma_cube_spawn_egg",
        "minecraft:mooshroom" => "minecraft:mooshroom_spawn_egg",
        "minecraft:mule" => "minecraft:mule_spawn_egg",
        "minecraft:ocelot" => "minecraft:ocelot_spawn_egg",
        "minecraft:pufferfish" => "minecraft:pufferfish_spawn_egg",
        "minecraft:parrot" => "minecraft:parrot_spawn_egg",
        "minecraft:pig" => "minecraft:pig_spawn_egg",
        "minecraft:polar_bear" => "minecraft:polar_bear_spawn_egg",
        "minecraft:rabbit" => "minecraft:rabbit_spawn_egg",
        "minecraft:sheep" => "minecraft:sheep_spawn_egg",
        "minecraft:shulker" => "minecraft:shulker_spawn_egg",
        "minecraft:silverfish" => "minecraft:silverfish_spawn_egg",
        "minecraft:skeleton" => "minecraft:skeleton_spawn_egg",
        "minecraft:skeleton_horse" => "minecraft:skeleton_horse_spawn_egg",
        "minecraft:slime" => "minecraft:slime_spawn_egg",
        "minecraft:snow_golem" => "minecraft:snow_golem_spawn_egg",
        "minecraft:spider" => "minecraft:spider_spawn_egg",
        "minecraft:squid" => "minecraft:squid_spawn_egg",
        "minecraft:stray" => "minecraft:stray_spawn_egg",
        "minecraft:turtle" => "minecraft:turtle_spawn_egg",
        "minecraft:vex" => "minecraft:vex_spawn_egg",
        "minecraft:villager" => "minecraft:villager_spawn_egg",
        "minecraft:vindication_illager" => "minecraft:vindication_illager_spawn_egg",
        "minecraft:witch" => "minecraft:witch_spawn_egg",
        "minecraft:wither" => "minecraft:wither_spawn_egg",
        "minecraft:wither_skeleton" => "minecraft:wither_skeleton_spawn_egg",
        "minecraft:wolf" => "minecraft:wolf_spawn_egg",
        "minecraft:zombie" => "minecraft:zombie_spawn_egg",
        "minecraft:zombie_horse" => "minecraft:zombie_horse_spawn_egg",
        "minecraft:zombie_pigman" => "minecraft:zombie_pigman_spawn_egg",
        "minecraft:zombie_villager" => "minecraft:zombie_villager_spawn_egg",
        _ => "minecraft:pig_spawn_egg",
    }
}

pub fn fix_item_ids(item_type: &str, item_id: &str, entity_id: Option<&str>) -> Option<String> {
    if item_id == item_type {
        if let Some(entity_id) = entity_id {
            return Some(spawn_egg_for(entity_id).to_string());
        }
    }
    None
}

pub fn fix_item_stack(item_type: &str, stack: &mut ItemStack) -> bool {
    ItemStackSpawnEggFix::new(item_type).fix_item_stack(stack)
}

pub fn spawn_egg_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for entity in [
        "minecraft:bat",
        "minecraft:blaze",
        "minecraft:cave_spider",
        "minecraft:chicken",
        "minecraft:cow",
        "minecraft:creeper",
        "minecraft:donkey",
        "minecraft:elder_guardian",
        "minecraft:ender_dragon",
        "minecraft:enderman",
        "minecraft:endermite",
        "minecraft:evocation_illager",
        "minecraft:ghast",
        "minecraft:guardian",
        "minecraft:horse",
        "minecraft:husk",
        "minecraft:iron_golem",
        "minecraft:llama",
        "minecraft:magma_cube",
        "minecraft:mooshroom",
        "minecraft:mule",
        "minecraft:ocelot",
        "minecraft:pufferfish",
        "minecraft:parrot",
        "minecraft:pig",
        "minecraft:polar_bear",
        "minecraft:rabbit",
        "minecraft:sheep",
        "minecraft:shulker",
        "minecraft:silverfish",
        "minecraft:skeleton",
        "minecraft:skeleton_horse",
        "minecraft:slime",
        "minecraft:snow_golem",
        "minecraft:spider",
        "minecraft:squid",
        "minecraft:stray",
        "minecraft:turtle",
        "minecraft:vex",
        "minecraft:villager",
        "minecraft:vindication_illager",
        "minecraft:witch",
        "minecraft:wither",
        "minecraft:wither_skeleton",
        "minecraft:wolf",
        "minecraft:zombie",
        "minecraft:zombie_horse",
        "minecraft:zombie_pigman",
        "minecraft:zombie_villager",
    ] {
        map.insert(entity.to_string(), spawn_egg_for(entity).to_string());
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_entity() {
        assert_eq!(spawn_egg_for("minecraft:blaze"), "minecraft:blaze_spawn_egg");
    }

    #[test]
    fn defaults_to_pig_spawn_egg() {
        assert_eq!(spawn_egg_for("minecraft:unknown"), "minecraft:pig_spawn_egg");
    }

    #[test]
    fn transforms_matching_item_type() {
        let fix = ItemStackSpawnEggFix::new("minecraft:spawn_egg");
        let mut stack = ItemStack {
            id: "minecraft:spawn_egg".to_string(),
            entity_id: Some("minecraft:cow".to_string()),
        };
        assert!(fix.fix_item_stack(&mut stack));
        assert_eq!(stack.id, "minecraft:cow_spawn_egg");
    }

    #[test]
    fn does_not_transform_wrong_item_type() {
        let fix = ItemStackSpawnEggFix::new("minecraft:spawn_egg");
        let mut stack = ItemStack {
            id: "minecraft:other".to_string(),
            entity_id: Some("minecraft:cow".to_string()),
        };
        assert!(!fix.fix_item_stack(&mut stack));
        assert_eq!(stack.id, "minecraft:other");
    }

    #[test]
    fn does_not_transform_without_entity_id() {
        let fix = ItemStackSpawnEggFix::new("minecraft:spawn_egg");
        let mut stack = ItemStack {
            id: "minecraft:spawn_egg".to_string(),
            entity_id: None,
        };
        assert!(!fix.fix_item_stack(&mut stack));
        assert_eq!(stack.id, "minecraft:spawn_egg");
    }

    #[test]
    fn fix_item_ids_matches_java_default() {
        assert_eq!(
            fix_item_ids("minecraft:spawn_egg", "minecraft:spawn_egg", Some("minecraft:nope")),
            Some("minecraft:pig_spawn_egg".to_string())
        );
    }
}
