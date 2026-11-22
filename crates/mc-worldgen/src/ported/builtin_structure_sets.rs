use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: &'static str,
    pub path: &'static str,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub const fn with_default_namespace(path: &'static str) -> Self {
        Self {
            namespace: "minecraft",
            path,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceKey<T: 'static> {
    pub registry: Identifier,
    pub location: Identifier,
    _marker: PhantomData<&'static T>,
}

impl<T: 'static> ResourceKey<T> {
    // JAVA: net.minecraft.resources.ResourceKey#create
    pub const fn create(registry: Identifier, location: Identifier) -> Self {
        Self {
            registry,
            location,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureSet;

pub mod registries {
    use super::Identifier;

    // JAVA: net.minecraft.core.registries.Registries#STRUCTURE_SET
    pub const STRUCTURE_SET: Identifier = Identifier::with_default_namespace("worldgen/structure_set");
}

// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#register
const fn register(name: &'static str) -> ResourceKey<StructureSet> {
    ResourceKey::create(registries::STRUCTURE_SET, Identifier::with_default_namespace(name))
}

// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#VILLAGES
pub const VILLAGES: ResourceKey<StructureSet> = register("villages");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#DESERT_PYRAMIDS
pub const DESERT_PYRAMIDS: ResourceKey<StructureSet> = register("desert_pyramids");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#IGLOOS
pub const IGLOOS: ResourceKey<StructureSet> = register("igloos");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#JUNGLE_TEMPLES
pub const JUNGLE_TEMPLES: ResourceKey<StructureSet> = register("jungle_temples");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#SWAMP_HUTS
pub const SWAMP_HUTS: ResourceKey<StructureSet> = register("swamp_huts");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#PILLAGER_OUTPOSTS
pub const PILLAGER_OUTPOSTS: ResourceKey<StructureSet> = register("pillager_outposts");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#OCEAN_MONUMENTS
pub const OCEAN_MONUMENTS: ResourceKey<StructureSet> = register("ocean_monuments");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#WOODLAND_MANSIONS
pub const WOODLAND_MANSIONS: ResourceKey<StructureSet> = register("woodland_mansions");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#BURIED_TREASURES
pub const BURIED_TREASURES: ResourceKey<StructureSet> = register("buried_treasures");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#MINESHAFTS
pub const MINESHAFTS: ResourceKey<StructureSet> = register("mineshafts");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#RUINED_PORTALS
pub const RUINED_PORTALS: ResourceKey<StructureSet> = register("ruined_portals");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#SHIPWRECKS
pub const SHIPWRECKS: ResourceKey<StructureSet> = register("shipwrecks");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#OCEAN_RUINS
pub const OCEAN_RUINS: ResourceKey<StructureSet> = register("ocean_ruins");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#NETHER_COMPLEXES
pub const NETHER_COMPLEXES: ResourceKey<StructureSet> = register("nether_complexes");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#NETHER_FOSSILS
pub const NETHER_FOSSILS: ResourceKey<StructureSet> = register("nether_fossils");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#END_CITIES
pub const END_CITIES: ResourceKey<StructureSet> = register("end_cities");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#ANCIENT_CITIES
pub const ANCIENT_CITIES: ResourceKey<StructureSet> = register("ancient_cities");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#STRONGHOLDS
pub const STRONGHOLDS: ResourceKey<StructureSet> = register("strongholds");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#TRAIL_RUINS
pub const TRAIL_RUINS: ResourceKey<StructureSet> = register("trail_ruins");
// JAVA: net.minecraft.world.level.levelgen.structure.BuiltinStructureSets#TRIAL_CHAMBERS
pub const TRIAL_CHAMBERS: ResourceKey<StructureSet> = register("trial_chambers");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_creates_correct_key() {
        let k = register("villages");
        assert_eq!(k.location.namespace, "minecraft");
        assert_eq!(k.location.path, "villages");
        assert_eq!(k.registry, registries::STRUCTURE_SET);
        assert_eq!(k.registry.path, "worldgen/structure_set");
    }

    #[test]
    fn all_keys_use_default_namespace_and_structure_set_registry() {
        let keys = [
            VILLAGES,
            DESERT_PYRAMIDS,
            IGLOOS,
            JUNGLE_TEMPLES,
            SWAMP_HUTS,
            PILLAGER_OUTPOSTS,
            OCEAN_MONUMENTS,
            WOODLAND_MANSIONS,
            BURIED_TREASURES,
            MINESHAFTS,
            RUINED_PORTALS,
            SHIPWRECKS,
            OCEAN_RUINS,
            NETHER_COMPLEXES,
            NETHER_FOSSILS,
            END_CITIES,
            ANCIENT_CITIES,
            STRONGHOLDS,
            TRAIL_RUINS,
            TRIAL_CHAMBERS,
        ];
        assert_eq!(keys.len(), 20);
        for k in &keys {
            assert_eq!(k.location.namespace, "minecraft");
            assert_eq!(k.registry, registries::STRUCTURE_SET);
        }
    }

    #[test]
    fn specific_paths_match_java() {
        assert_eq!(VILLAGES.location.path, "villages");
        assert_eq!(DESERT_PYRAMIDS.location.path, "desert_pyramids");
        assert_eq!(TRIAL_CHAMBERS.location.path, "trial_chambers");
        assert_eq!(NETHER_FOSSILS.location.path, "nether_fossils");
    }
}
