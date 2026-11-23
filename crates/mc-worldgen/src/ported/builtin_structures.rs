use std::marker::PhantomData;
use std::sync::LazyLock;

/// Marker type for structure registry entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Structure;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_owned(),
            path: path.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey<T> {
    pub registry: Identifier,
    pub location: Identifier,
    _marker: PhantomData<T>,
}

impl<T> ResourceKey<T> {
    // JAVA: net.minecraft.resources.ResourceKey#create
    pub fn create(registry: Identifier, location: Identifier) -> Self {
        Self {
            registry,
            location,
            _marker: PhantomData,
        }
    }
}

/// Minimal registry constants needed by `BuiltinStructures`.
pub mod registries {
    use super::Identifier;

    // JAVA: net.minecraft.core.registries.Registries#STRUCTURE
    pub fn structure() -> Identifier {
        Identifier::with_default_namespace("worldgen/structure")
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructures#createKey
fn create_key(name: &str) -> ResourceKey<Structure> {
    ResourceKey::create(registries::structure(), Identifier::with_default_namespace(name))
}

pub static PILLAGER_OUTPOST: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("pillager_outpost"));
pub static MINESHAFT: LazyLock<ResourceKey<Structure>> = LazyLock::new(|| create_key("mineshaft"));
pub static MINESHAFT_MESA: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("mineshaft_mesa"));
pub static WOODLAND_MANSION: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("mansion"));
pub static JUNGLE_TEMPLE: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("jungle_pyramid"));
pub static DESERT_PYRAMID: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("desert_pyramid"));
pub static IGLOO: LazyLock<ResourceKey<Structure>> = LazyLock::new(|| create_key("igloo"));
pub static SHIPWRECK: LazyLock<ResourceKey<Structure>> = LazyLock::new(|| create_key("shipwreck"));
pub static SHIPWRECK_BEACHED: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("shipwreck_beached"));
pub static SWAMP_HUT: LazyLock<ResourceKey<Structure>> = LazyLock::new(|| create_key("swamp_hut"));
pub static STRONGHOLD: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("stronghold"));
pub static OCEAN_MONUMENT: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("monument"));
pub static OCEAN_RUIN_COLD: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ocean_ruin_cold"));
pub static OCEAN_RUIN_WARM: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ocean_ruin_warm"));
pub static FORTRESS: LazyLock<ResourceKey<Structure>> = LazyLock::new(|| create_key("fortress"));
pub static NETHER_FOSSIL: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("nether_fossil"));
pub static END_CITY: LazyLock<ResourceKey<Structure>> = LazyLock::new(|| create_key("end_city"));
pub static BURIED_TREASURE: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("buried_treasure"));
pub static BASTION_REMNANT: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("bastion_remnant"));
pub static VILLAGE_PLAINS: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("village_plains"));
pub static VILLAGE_DESERT: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("village_desert"));
pub static VILLAGE_SAVANNA: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("village_savanna"));
pub static VILLAGE_SNOWY: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("village_snowy"));
pub static VILLAGE_TAIGA: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("village_taiga"));
pub static RUINED_PORTAL_STANDARD: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ruined_portal"));
pub static RUINED_PORTAL_DESERT: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ruined_portal_desert"));
pub static RUINED_PORTAL_JUNGLE: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ruined_portal_jungle"));
pub static RUINED_PORTAL_SWAMP: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ruined_portal_swamp"));
pub static RUINED_PORTAL_MOUNTAIN: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ruined_portal_mountain"));
pub static RUINED_PORTAL_OCEAN: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ruined_portal_ocean"));
pub static RUINED_PORTAL_NETHER: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ruined_portal_nether"));
pub static ANCIENT_CITY: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("ancient_city"));
pub static TRAIL_RUINS: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("trail_ruins"));
pub static TRIAL_CHAMBERS: LazyLock<ResourceKey<Structure>> =
    LazyLock::new(|| create_key("trial_chambers"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_key_uses_default_namespace_and_structure_registry() {
        let key = PILLAGER_OUTPOST.clone();
        assert_eq!(key.location.namespace, "minecraft");
        assert_eq!(key.location.path, "pillager_outpost");
        assert_eq!(key.registry.namespace, "minecraft");
        assert_eq!(key.registry.path, "worldgen/structure");
    }

    #[test]
    fn special_name_mappings_match_java() {
        assert_eq!(WOODLAND_MANSION.location.path, "mansion");
        assert_eq!(JUNGLE_TEMPLE.location.path, "jungle_pyramid");
        assert_eq!(DESERT_PYRAMID.location.path, "desert_pyramid");
        assert_eq!(OCEAN_MONUMENT.location.path, "monument");
        assert_eq!(RUINED_PORTAL_STANDARD.location.path, "ruined_portal");
    }

    #[test]
    fn keys_are_distinct_and_stable() {
        let a = MINESHAFT.clone();
        let b = MINESHAFT_MESA.clone();
        let c = MINESHAFT.clone();
        assert_ne!(a.location.path, b.location.path);
        assert_eq!(a, c);
        assert_eq!(a.registry, b.registry);
    }

    #[test]
    fn identifier_with_default_namespace() {
        let id = Identifier::with_default_namespace("test_path");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "test_path");
    }
}
