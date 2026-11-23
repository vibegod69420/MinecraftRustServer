// JAVA: net.minecraft.util.datafix.fixes.References

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeReference {
    name: &'static str,
}

impl TypeReference {
    // JAVA: net.minecraft.util.datafix.fixes.References#reference (typeName)
    pub const fn type_name(&self) -> &'static str {
        self.name
    }
}

impl fmt::Display for TypeReference {
    // JAVA: net.minecraft.util.datafix.fixes.References#reference (toString)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.name)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.References#reference
pub const fn reference(name: &'static str) -> TypeReference {
    TypeReference { name }
}

pub struct References;

impl References {
    pub const LEVEL: TypeReference = reference("level");
    pub const LIGHTWEIGHT_LEVEL: TypeReference = reference("lightweight_level");
    pub const PLAYER: TypeReference = reference("player");
    pub const CHUNK: TypeReference = reference("chunk");
    pub const HOTBAR: TypeReference = reference("hotbar");
    pub const OPTIONS: TypeReference = reference("options");
    pub const STRUCTURE: TypeReference = reference("structure");
    pub const STATS: TypeReference = reference("stats");
    pub const SAVED_DATA_COMMAND_STORAGE: TypeReference = reference("saved_data/command_storage");
    pub const SAVED_DATA_TICKETS: TypeReference = reference("saved_data/tickets");
    pub const SAVED_DATA_MAP_DATA: TypeReference = reference("saved_data/map_data");
    pub const SAVED_DATA_MAP_INDEX: TypeReference = reference("saved_data/idcounts");
    pub const SAVED_DATA_RAIDS: TypeReference = reference("saved_data/raids");
    pub const SAVED_DATA_RANDOM_SEQUENCES: TypeReference = reference("saved_data/random_sequences");
    pub const SAVED_DATA_SCOREBOARD: TypeReference = reference("saved_data/scoreboard");
    pub const SAVED_DATA_STOPWATCHES: TypeReference = reference("saved_data/stopwatches");
    pub const SAVED_DATA_STRUCTURE_FEATURE_INDICES: TypeReference =
        reference("saved_data/structure_feature_indices");
    pub const SAVED_DATA_WORLD_BORDER: TypeReference = reference("saved_data/world_border");
    pub const ADVANCEMENTS: TypeReference = reference("advancements");
    pub const POI_CHUNK: TypeReference = reference("poi_chunk");
    pub const ENTITY_CHUNK: TypeReference = reference("entity_chunk");
    pub const DEBUG_PROFILE: TypeReference = reference("debug_profile");
    pub const BLOCK_ENTITY: TypeReference = reference("block_entity");
    pub const ITEM_STACK: TypeReference = reference("item_stack");
    pub const BLOCK_STATE: TypeReference = reference("block_state");
    pub const FLAT_BLOCK_STATE: TypeReference = reference("flat_block_state");
    pub const DATA_COMPONENTS: TypeReference = reference("data_components");
    pub const VILLAGER_TRADE: TypeReference = reference("villager_trade");
    pub const PARTICLE: TypeReference = reference("particle");
    pub const TEXT_COMPONENT: TypeReference = reference("text_component");
    pub const ENTITY_EQUIPMENT: TypeReference = reference("entity_equipment");
    pub const ENTITY_NAME: TypeReference = reference("entity_name");
    pub const ENTITY_TREE: TypeReference = reference("entity_tree");
    pub const ENTITY: TypeReference = reference("entity");
    pub const BLOCK_NAME: TypeReference = reference("block_name");
    pub const ITEM_NAME: TypeReference = reference("item_name");
    pub const GAME_EVENT_NAME: TypeReference = reference("game_event_name");
    pub const UNTAGGED_SPAWNER: TypeReference = reference("untagged_spawner");
    pub const STRUCTURE_FEATURE: TypeReference = reference("structure_feature");
    pub const OBJECTIVE: TypeReference = reference("objective");
    pub const TEAM: TypeReference = reference("team");
    pub const RECIPE: TypeReference = reference("recipe");
    pub const BIOME: TypeReference = reference("biome");
    pub const MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST: TypeReference =
        reference("multi_noise_biome_source_parameter_list");
    pub const WORLD_GEN_SETTINGS: TypeReference = reference("world_gen_settings");
}

pub const LEVEL: TypeReference = reference("level");
pub const LIGHTWEIGHT_LEVEL: TypeReference = reference("lightweight_level");
pub const PLAYER: TypeReference = reference("player");
pub const CHUNK: TypeReference = reference("chunk");
pub const HOTBAR: TypeReference = reference("hotbar");
pub const OPTIONS: TypeReference = reference("options");
pub const STRUCTURE: TypeReference = reference("structure");
pub const STATS: TypeReference = reference("stats");
pub const SAVED_DATA_COMMAND_STORAGE: TypeReference = reference("saved_data/command_storage");
pub const SAVED_DATA_TICKETS: TypeReference = reference("saved_data/tickets");
pub const SAVED_DATA_MAP_DATA: TypeReference = reference("saved_data/map_data");
pub const SAVED_DATA_MAP_INDEX: TypeReference = reference("saved_data/idcounts");
pub const SAVED_DATA_RAIDS: TypeReference = reference("saved_data/raids");
pub const SAVED_DATA_RANDOM_SEQUENCES: TypeReference = reference("saved_data/random_sequences");
pub const SAVED_DATA_SCOREBOARD: TypeReference = reference("saved_data/scoreboard");
pub const SAVED_DATA_STOPWATCHES: TypeReference = reference("saved_data/stopwatches");
pub const SAVED_DATA_STRUCTURE_FEATURE_INDICES: TypeReference =
    reference("saved_data/structure_feature_indices");
pub const SAVED_DATA_WORLD_BORDER: TypeReference = reference("saved_data/world_border");
pub const ADVANCEMENTS: TypeReference = reference("advancements");
pub const POI_CHUNK: TypeReference = reference("poi_chunk");
pub const ENTITY_CHUNK: TypeReference = reference("entity_chunk");
pub const DEBUG_PROFILE: TypeReference = reference("debug_profile");
pub const BLOCK_ENTITY: TypeReference = reference("block_entity");
pub const ITEM_STACK: TypeReference = reference("item_stack");
pub const BLOCK_STATE: TypeReference = reference("block_state");
pub const FLAT_BLOCK_STATE: TypeReference = reference("flat_block_state");
pub const DATA_COMPONENTS: TypeReference = reference("data_components");
pub const VILLAGER_TRADE: TypeReference = reference("villager_trade");
pub const PARTICLE: TypeReference = reference("particle");
pub const TEXT_COMPONENT: TypeReference = reference("text_component");
pub const ENTITY_EQUIPMENT: TypeReference = reference("entity_equipment");
pub const ENTITY_NAME: TypeReference = reference("entity_name");
pub const ENTITY_TREE: TypeReference = reference("entity_tree");
pub const ENTITY: TypeReference = reference("entity");
pub const BLOCK_NAME: TypeReference = reference("block_name");
pub const ITEM_NAME: TypeReference = reference("item_name");
pub const GAME_EVENT_NAME: TypeReference = reference("game_event_name");
pub const UNTAGGED_SPAWNER: TypeReference = reference("untagged_spawner");
pub const STRUCTURE_FEATURE: TypeReference = reference("structure_feature");
pub const OBJECTIVE: TypeReference = reference("objective");
pub const TEAM: TypeReference = reference("team");
pub const RECIPE: TypeReference = reference("recipe");
pub const BIOME: TypeReference = reference("biome");
pub const MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST: TypeReference =
    reference("multi_noise_biome_source_parameter_list");
pub const WORLD_GEN_SETTINGS: TypeReference = reference("world_gen_settings");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_name_matches_java() {
        assert_eq!(LEVEL.type_name(), "level");
        assert_eq!(SAVED_DATA_MAP_INDEX.type_name(), "saved_data/idcounts");
    }

    #[test]
    fn display_matches_java() {
        assert_eq!(LEVEL.to_string(), "@level");
        assert_eq!(WORLD_GEN_SETTINGS.to_string(), "@world_gen_settings");
    }

    #[test]
    fn reference_equality() {
        assert_eq!(reference("level"), LEVEL);
        assert_ne!(LEVEL, PLAYER);
    }

    #[test]
    fn associated_constants_match_module_constants() {
        assert_eq!(References::LEVEL, LEVEL);
        assert_eq!(References::BIOME.type_name(), "biome");
    }

    #[test]
    fn all_names_are_non_empty() {
        assert!(!ENTITY.type_name().is_empty());
        assert!(ENTITY.to_string().starts_with('@'));
    }
}
