use std::collections::BTreeMap;

/// Minimal self-contained dynamic value representation for datafix porting.
///
/// Models the subset of DFU `Dynamic` operations needed by `ItemSpawnEggFix`.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Short(i16),
    String(String),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    /// Creates an empty compound/map dynamic value.
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    /// Creates a short dynamic value.
    pub fn create_short(value: i16) -> Self {
        Dynamic::Short(value)
    }

    /// Creates a string dynamic value.
    pub fn create_string(value: impl Into<String>) -> Self {
        Dynamic::String(value.into())
    }

    /// Returns the map contents if this is a compound/map.
    pub fn as_map(&self) -> Option<&BTreeMap<String, Dynamic>> {
        match self {
            Dynamic::Map(map) => Some(map),
            _ => None,
        }
    }

    /// Returns a mutable reference to map contents if this is a compound/map.
    pub fn as_map_mut(&mut self) -> Option<&mut BTreeMap<String, Dynamic>> {
        match self {
            Dynamic::Map(map) => Some(map),
            _ => None,
        }
    }

    /// Gets a child field by key if this is a compound/map.
    pub fn get(&self, key: &str) -> Option<&Dynamic> {
        self.as_map().and_then(|map| map.get(key))
    }

    /// Gets a child field by key, returning an empty map if missing or not a map.
    pub fn get_or_empty_map(&self, key: &str) -> Dynamic {
        match self.get(key) {
            Some(Dynamic::Map(_)) => self.get(key).cloned().unwrap_or_else(Dynamic::empty_map),
            _ => Dynamic::empty_map(),
        }
    }

    /// Reads this value as a short, returning `default` if it is not a short.
    pub fn as_short(&self, default: i16) -> i16 {
        match self {
            Dynamic::Short(value) => *value,
            _ => default,
        }
    }

    /// Reads this value as a string, if it is a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Dynamic::String(value) => Some(value),
            _ => None,
        }
    }

    /// Sets a child field if this is a compound/map.
    pub fn set(&mut self, key: &str, value: Dynamic) {
        if let Dynamic::Map(map) = self {
            map.insert(key.to_string(), value);
        }
    }
}

/// Legacy spawn-egg entity ID table indexed by `damage & 0xFF`.
///
/// JAVA: net.minecraft.util.datafix.fixes.ItemSpawnEggFix#ID_TO_ENTITY
const ID_TO_ENTITY: [Option<&str>; 256] = {
    let mut table: [Option<&str>; 256] = [None; 256];
    table[1] = Some("Item");
    table[2] = Some("XPOrb");
    table[7] = Some("ThrownEgg");
    table[8] = Some("LeashKnot");
    table[9] = Some("Painting");
    table[10] = Some("Arrow");
    table[11] = Some("Snowball");
    table[12] = Some("Fireball");
    table[13] = Some("SmallFireball");
    table[14] = Some("ThrownEnderpearl");
    table[15] = Some("EyeOfEnderSignal");
    table[16] = Some("ThrownPotion");
    table[17] = Some("ThrownExpBottle");
    table[18] = Some("ItemFrame");
    table[19] = Some("WitherSkull");
    table[20] = Some("PrimedTnt");
    table[21] = Some("FallingSand");
    table[22] = Some("FireworksRocketEntity");
    table[23] = Some("TippedArrow");
    table[24] = Some("SpectralArrow");
    table[25] = Some("ShulkerBullet");
    table[26] = Some("DragonFireball");
    table[30] = Some("ArmorStand");
    table[41] = Some("Boat");
    table[42] = Some("MinecartRideable");
    table[43] = Some("MinecartChest");
    table[44] = Some("MinecartFurnace");
    table[45] = Some("MinecartTNT");
    table[46] = Some("MinecartHopper");
    table[47] = Some("MinecartSpawner");
    table[40] = Some("MinecartCommandBlock");
    table[50] = Some("Creeper");
    table[51] = Some("Skeleton");
    table[52] = Some("Spider");
    table[53] = Some("Giant");
    table[54] = Some("Zombie");
    table[55] = Some("Slime");
    table[56] = Some("Ghast");
    table[57] = Some("PigZombie");
    table[58] = Some("Enderman");
    table[59] = Some("CaveSpider");
    table[60] = Some("Silverfish");
    table[61] = Some("Blaze");
    table[62] = Some("LavaSlime");
    table[63] = Some("EnderDragon");
    table[64] = Some("WitherBoss");
    table[65] = Some("Bat");
    table[66] = Some("Witch");
    table[67] = Some("Endermite");
    table[68] = Some("Guardian");
    table[69] = Some("Shulker");
    table[90] = Some("Pig");
    table[91] = Some("Sheep");
    table[92] = Some("Cow");
    table[93] = Some("Chicken");
    table[94] = Some("Squid");
    table[95] = Some("Wolf");
    table[96] = Some("MushroomCow");
    table[97] = Some("SnowMan");
    table[98] = Some("Ozelot");
    table[99] = Some("VillagerGolem");
    table[100] = Some("EntityHorse");
    table[101] = Some("Rabbit");
    table[120] = Some("Villager");
    table[200] = Some("EnderCrystal");
    table
};

/// Port of `net.minecraft.util.datafix.fixes.ItemSpawnEggFix`.
///
/// The DFU schema/type machinery is not ported; this applies the same item-stack
/// transformation to a self-contained dynamic representation.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItemSpawnEggFix;

impl ItemSpawnEggFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemSpawnEggFix#ItemSpawnEggFix
    pub fn new(_changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemSpawnEggFix#makeRule
    pub fn make_rule(&self, mut item_stack: Dynamic) -> Dynamic {
        let item_id = item_stack.get("id").and_then(Dynamic::as_string);
        if item_id != Some("minecraft:spawn_egg") {
            return item_stack;
        }

        let damage: i16 = item_stack
            .get("Damage")
            .map_or(0_i16, |d| d.as_short(0_i16));

        // Java: $$8 & 0xFF (short promoted to int, low byte selected)
        let entity_index = ((damage as i32) & 0xFF) as usize;
        let mapped_entity_id = ID_TO_ENTITY[entity_index];

        let current_entity_id = item_stack
            .get("tag")
            .and_then(|tag| tag.get("EntityTag"))
            .and_then(|entity_tag| entity_tag.get("id"))
            .and_then(Dynamic::as_string)
            .map(str::to_owned);

        if let Some(mapped_entity_id) = mapped_entity_id {
            let should_set_entity_id = match current_entity_id.as_deref() {
                Some(current) => current != mapped_entity_id,
                None => true,
            };

            if should_set_entity_id {
                let mut tag = item_stack.get_or_empty_map("tag");
                let mut entity_tag = tag.get_or_empty_map("EntityTag");
                entity_tag.set("id", Dynamic::create_string(mapped_entity_id));
                tag.set("EntityTag", entity_tag);
                item_stack.set("tag", tag);
            }
        }

        if damage != 0_i16 {
            item_stack.set("Damage", Dynamic::create_short(0_i16));
        }

        item_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn compound(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut map = BTreeMap::new();
        for (key, value) in entries {
            map.insert(key.to_string(), value);
        }
        Dynamic::Map(map)
    }

    fn spawn_egg(damage: i16) -> Dynamic {
        compound(vec![
            ("id", Dynamic::create_string("minecraft:spawn_egg")),
            ("Damage", Dynamic::create_short(damage)),
        ])
    }

    #[test]
    fn maps_spawn_egg_damage_to_entity_tag_and_resets_damage() {
        let fix = ItemSpawnEggFix::new(true);
        let input = spawn_egg(90); // Pig

        let out = fix.make_rule(input);

        assert_eq!(out.get("Damage"), Some(&Dynamic::Short(0)));
        let entity_id = out
            .get("tag")
            .and_then(|tag| tag.get("EntityTag"))
            .and_then(|entity_tag| entity_tag.get("id"));
        assert_eq!(entity_id, Some(&Dynamic::create_string("Pig")));
    }

    #[test]
    fn negative_damage_uses_low_byte_mask() {
        let fix = ItemSpawnEggFix::new(true);
        // -166i16 == 0xFF5A, low byte 90 -> Pig
        let input = spawn_egg(-166);

        let out = fix.make_rule(input);

        assert_eq!(out.get("Damage"), Some(&Dynamic::Short(0)));
        let entity_id = out
            .get("tag")
            .and_then(|tag| tag.get("EntityTag"))
            .and_then(|entity_tag| entity_tag.get("id"));
        assert_eq!(entity_id, Some(&Dynamic::create_string("Pig")));
    }

    #[test]
    fn preserves_existing_entity_tag_fields_when_id_matches() {
        let fix = ItemSpawnEggFix::new(true);
        let input = compound(vec![
            ("id", Dynamic::create_string("minecraft:spawn_egg")),
            ("Damage", Dynamic::create_short(90)),
            (
                "tag",
                compound(vec![(
                    "EntityTag",
                    compound(vec![
                        ("id", Dynamic::create_string("Pig")),
                        ("CustomName", Dynamic::create_string("Porky")),
                    ]),
                )]),
            ),
        ]);

        let out = fix.make_rule(input);
        let entity_tag = out
            .get("tag")
            .and_then(|tag| tag.get("EntityTag"))
            .expect("EntityTag should exist");

        assert_eq!(out.get("Damage"), Some(&Dynamic::Short(0)));
        assert_eq!(entity_tag.get("id"), Some(&Dynamic::create_string("Pig")));
        assert_eq!(
            entity_tag.get("CustomName"),
            Some(&Dynamic::create_string("Porky"))
        );
    }

    #[test]
    fn unmapped_damage_resets_damage_without_creating_entity_tag() {
        let fix = ItemSpawnEggFix::new(true);
        let input = spawn_egg(3); // ID_TO_ENTITY[3] == None

        let out = fix.make_rule(input);

        assert_eq!(out.get("Damage"), Some(&Dynamic::Short(0)));
        assert!(out.get("tag").is_none());
    }

    #[test]
    fn non_spawn_egg_or_zero_damage_without_mapping_is_unchanged() {
        let fix = ItemSpawnEggFix::new(true);

        let non_spawn_egg = compound(vec![
            ("id", Dynamic::create_string("minecraft:stone")),
            ("Damage", Dynamic::create_short(90)),
        ]);
        assert_eq!(fix.make_rule(non_spawn_egg.clone()), non_spawn_egg);

        let zero_damage_spawn_egg = spawn_egg(0); // ID_TO_ENTITY[0] == None
        assert_eq!(
            fix.make_rule(zero_damage_spawn_egg.clone()),
            zero_damage_spawn_egg
        );
    }
}
