use std::sync::LazyLock;

/// Minimal self-contained Identifier for registry keys.
// JAVA: net.minecraft.resources.Identifier (minimal support for Registries)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    /// Creates a minecraft-namespaced identifier.
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }

    /// Creates an identifier with an explicit namespace and path.
    pub fn new(namespace: &str, path: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            path: path.to_string(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Minimal self-contained ResourceKey for registry references.
// JAVA: net.minecraft.resources.ResourceKey (minimal support for Registries)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    registry_name: Identifier,
    location: Identifier,
}

impl ResourceKey {
    /// Creates a resource key in the given registry.
    pub fn create(registry: &ResourceKey, location: Identifier) -> Self {
        Self {
            registry_name: registry.location.clone(),
            location,
        }
    }

    /// Creates a registry key under the root registry.
    pub fn create_registry_key(location: Identifier) -> Self {
        Self {
            registry_name: Identifier::with_default_namespace("root"),
            location,
        }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.location
    }

    pub fn registry_name(&self) -> &Identifier {
        &self.registry_name
    }
}

// JAVA: net.minecraft.core.registries.Registries#ROOT_REGISTRY_NAME
pub static ROOT_REGISTRY_NAME: LazyLock<Identifier> =
    LazyLock::new(|| Identifier::with_default_namespace("root"));

// JAVA: net.minecraft.core.registries.Registries#createRegistryKey
fn create_registry_key(path: &str) -> ResourceKey {
    ResourceKey::create_registry_key(Identifier::with_default_namespace(path))
}

macro_rules! registry_keys {
    ($($name:ident = $path:literal),* $(,)?) => {
        $(
            pub static $name: LazyLock<ResourceKey> = LazyLock::new(|| create_registry_key($path));
        )*
    };
}

registry_keys! {
    ACTIVITY = "activity",
    ATTRIBUTE = "attribute",
    BIOME_SOURCE = "worldgen/biome_source",
    BLOCK_ENTITY_TYPE = "block_entity_type",
    BLOCK_PREDICATE_TYPE = "block_predicate_type",
    BLOCK_STATE_PROVIDER_TYPE = "worldgen/block_state_provider_type",
    BLOCK_TYPE = "block_type",
    BLOCK = "block",
    CARVER = "worldgen/carver",
    CHUNK_GENERATOR = "worldgen/chunk_generator",
    CHUNK_STATUS = "chunk_status",
    COMMAND_ARGUMENT_TYPE = "command_argument_type",
    CONSUME_EFFECT_TYPE = "consume_effect_type",
    CREATIVE_MODE_TAB = "creative_mode_tab",
    CUSTOM_STAT = "custom_stat",
    DATA_COMPONENT_PREDICATE_TYPE = "data_component_predicate_type",
    DATA_COMPONENT_TYPE = "data_component_type",
    GAME_RULE = "game_rule",
    DEBUG_SUBSCRIPTION = "debug_subscription",
    DECORATED_POT_PATTERN = "decorated_pot_pattern",
    DENSITY_FUNCTION_TYPE = "worldgen/density_function_type",
    DIALOG_BODY_TYPE = "dialog_body_type",
    DIALOG_TYPE = "dialog_type",
    ENCHANTMENT_EFFECT_COMPONENT_TYPE = "enchantment_effect_component_type",
    ENCHANTMENT_ENTITY_EFFECT_TYPE = "enchantment_entity_effect_type",
    ENCHANTMENT_LEVEL_BASED_VALUE_TYPE = "enchantment_level_based_value_type",
    ENCHANTMENT_LOCATION_BASED_EFFECT_TYPE = "enchantment_location_based_effect_type",
    ENCHANTMENT_PROVIDER_TYPE = "enchantment_provider_type",
    ENCHANTMENT_VALUE_EFFECT_TYPE = "enchantment_value_effect_type",
    ENTITY_SUB_PREDICATE_TYPE = "entity_sub_predicate_type",
    ENTITY_TYPE = "entity_type",
    ENVIRONMENT_ATTRIBUTE = "environment_attribute",
    ATTRIBUTE_TYPE = "attribute_type",
    FEATURE_SIZE_TYPE = "worldgen/feature_size_type",
    FEATURE = "worldgen/feature",
    FLOAT_PROVIDER_TYPE = "float_provider_type",
    FLUID = "fluid",
    FOLIAGE_PLACER_TYPE = "worldgen/foliage_placer_type",
    GAME_EVENT = "game_event",
    HEIGHT_PROVIDER_TYPE = "height_provider_type",
    INPUT_CONTROL_TYPE = "input_control_type",
    INT_PROVIDER_TYPE = "int_provider_type",
    ITEM = "item",
    SLOT_SOURCE_TYPE = "slot_source_type",
    LOOT_CONDITION_TYPE = "loot_condition_type",
    LOOT_FUNCTION_TYPE = "loot_function_type",
    LOOT_NBT_PROVIDER_TYPE = "loot_nbt_provider_type",
    LOOT_NUMBER_PROVIDER_TYPE = "loot_number_provider_type",
    LOOT_POOL_ENTRY_TYPE = "loot_pool_entry_type",
    LOOT_SCORE_PROVIDER_TYPE = "loot_score_provider_type",
    MAP_DECORATION_TYPE = "map_decoration_type",
    MATERIAL_CONDITION = "worldgen/material_condition",
    MATERIAL_RULE = "worldgen/material_rule",
    MEMORY_MODULE_TYPE = "memory_module_type",
    MENU = "menu",
    MOB_EFFECT = "mob_effect",
    NUMBER_FORMAT_TYPE = "number_format_type",
    PARTICLE_TYPE = "particle_type",
    PLACEMENT_MODIFIER_TYPE = "worldgen/placement_modifier_type",
    POINT_OF_INTEREST_TYPE = "point_of_interest_type",
    POOL_ALIAS_BINDING = "worldgen/pool_alias_binding",
    POSITION_SOURCE_TYPE = "position_source_type",
    POS_RULE_TEST = "pos_rule_test",
    POTION = "potion",
    RECIPE_BOOK_CATEGORY = "recipe_book_category",
    RECIPE_DISPLAY = "recipe_display",
    RECIPE_SERIALIZER = "recipe_serializer",
    RECIPE_TYPE = "recipe_type",
    ROOT_PLACER_TYPE = "worldgen/root_placer_type",
    RULE_BLOCK_ENTITY_MODIFIER = "rule_block_entity_modifier",
    RULE_TEST = "rule_test",
    SENSOR_TYPE = "sensor_type",
    SLOT_DISPLAY = "slot_display",
    SOUND_EVENT = "sound_event",
    SPAWN_CONDITION_TYPE = "spawn_condition_type",
    STAT_TYPE = "stat_type",
    STRUCTURE_PIECE = "worldgen/structure_piece",
    STRUCTURE_PLACEMENT = "worldgen/structure_placement",
    STRUCTURE_POOL_ELEMENT = "worldgen/structure_pool_element",
    STRUCTURE_PROCESSOR = "worldgen/structure_processor",
    STRUCTURE_TYPE = "worldgen/structure_type",
    DIALOG_ACTION_TYPE = "dialog_action_type",
    TEST_ENVIRONMENT_DEFINITION_TYPE = "test_environment_definition_type",
    TEST_FUNCTION = "test_function",
    TEST_INSTANCE_TYPE = "test_instance_type",
    TICKET_TYPE = "ticket_type",
    TREE_DECORATOR_TYPE = "worldgen/tree_decorator_type",
    TRUNK_PLACER_TYPE = "worldgen/trunk_placer_type",
    VILLAGER_PROFESSION = "villager_profession",
    VILLAGER_TYPE = "villager_type",
    INCOMING_RPC_METHOD = "incoming_rpc_methods",
    OUTGOING_RPC_METHOD = "outgoing_rpc_methods",
    PERMISSION_TYPE = "permission_type",
    PERMISSION_CHECK_TYPE = "permission_check_type",
    BANNER_PATTERN = "banner_pattern",
    BIOME = "worldgen/biome",
    CAT_VARIANT = "cat_variant",
    CHAT_TYPE = "chat_type",
    CHICKEN_VARIANT = "chicken_variant",
    ZOMBIE_NAUTILUS_VARIANT = "zombie_nautilus_variant",
    CONFIGURED_CARVER = "worldgen/configured_carver",
    CONFIGURED_FEATURE = "worldgen/configured_feature",
    COW_VARIANT = "cow_variant",
    DAMAGE_TYPE = "damage_type",
    DENSITY_FUNCTION = "worldgen/density_function",
    DIALOG = "dialog",
    DIMENSION_TYPE = "dimension_type",
    ENCHANTMENT_PROVIDER = "enchantment_provider",
    ENCHANTMENT = "enchantment",
    FLAT_LEVEL_GENERATOR_PRESET = "worldgen/flat_level_generator_preset",
    FROG_VARIANT = "frog_variant",
    INSTRUMENT = "instrument",
    JUKEBOX_SONG = "jukebox_song",
    MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST = "worldgen/multi_noise_biome_source_parameter_list",
    NOISE_SETTINGS = "worldgen/noise_settings",
    NOISE = "worldgen/noise",
    PAINTING_VARIANT = "painting_variant",
    PIG_VARIANT = "pig_variant",
    PLACED_FEATURE = "worldgen/placed_feature",
    PROCESSOR_LIST = "worldgen/processor_list",
    STRUCTURE_SET = "worldgen/structure_set",
    STRUCTURE = "worldgen/structure",
    TEMPLATE_POOL = "worldgen/template_pool",
    TEST_ENVIRONMENT = "test_environment",
    TEST_INSTANCE = "test_instance",
    TIMELINE = "timeline",
    TRIAL_SPAWNER_CONFIG = "trial_spawner",
    TRIGGER_TYPE = "trigger_type",
    TRIM_MATERIAL = "trim_material",
    TRIM_PATTERN = "trim_pattern",
    WOLF_VARIANT = "wolf_variant",
    WOLF_SOUND_VARIANT = "wolf_sound_variant",
    WORLD_PRESET = "worldgen/world_preset",
    DIMENSION = "dimension",
    LEVEL_STEM = "dimension",
    LOOT_TABLE = "loot_table",
    ITEM_MODIFIER = "item_modifier",
    PREDICATE = "predicate",
    ADVANCEMENT = "advancement",
    RECIPE = "recipe",
}

// JAVA: net.minecraft.core.registries.Registries#levelStemToLevel
pub fn level_stem_to_level(key: &ResourceKey) -> ResourceKey {
    ResourceKey::create(&DIMENSION, key.identifier().clone())
}

// JAVA: net.minecraft.core.registries.Registries#levelToLevelStem
pub fn level_to_level_stem(key: &ResourceKey) -> ResourceKey {
    ResourceKey::create(&LEVEL_STEM, key.identifier().clone())
}

// JAVA: net.minecraft.core.registries.Registries#elementsDirPath
pub fn elements_dir_path(key: &ResourceKey) -> String {
    key.identifier().path().to_string()
}

// JAVA: net.minecraft.core.registries.Registries#tagsDirPath
pub fn tags_dir_path(key: &ResourceKey) -> String {
    format!("tags/{}", key.identifier().path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_registry_name() {
        assert_eq!(ROOT_REGISTRY_NAME.namespace(), "minecraft");
        assert_eq!(ROOT_REGISTRY_NAME.path(), "root");
    }

    #[test]
    fn registry_key_structure() {
        let key = &*ACTIVITY;
        assert_eq!(key.registry_name(), &*ROOT_REGISTRY_NAME);
        assert_eq!(key.identifier().namespace(), "minecraft");
        assert_eq!(key.identifier().path(), "activity");
    }

    #[test]
    fn elements_and_tags_dir_paths_edge_cases() {
        let key = create_registry_key("worldgen/biome");
        assert_eq!(elements_dir_path(&key), "worldgen/biome");
        assert_eq!(tags_dir_path(&key), "tags/worldgen/biome");
        let empty = create_registry_key("");
        assert_eq!(elements_dir_path(&empty), "");
        assert_eq!(tags_dir_path(&empty), "tags/");
    }

    #[test]
    fn level_stem_and_dimension_conversion() {
        let overworld =
            ResourceKey::create(&LEVEL_STEM, Identifier::with_default_namespace("overworld"));
        let level = level_stem_to_level(&overworld);
        assert_eq!(level.identifier().path(), "overworld");
        assert_eq!(level.registry_name(), DIMENSION.identifier());
        let back = level_to_level_stem(&level);
        assert_eq!(back.registry_name(), LEVEL_STEM.identifier());
        assert_eq!(back.identifier(), overworld.identifier());
        // DIMENSION and LEVEL_STEM share the same registry path "dimension"
        assert_eq!(DIMENSION.identifier().path(), "dimension");
        assert_eq!(LEVEL_STEM.identifier().path(), "dimension");
    }

    #[test]
    fn resource_key_create_preserves_location() {
        let reg = create_registry_key("custom_registry");
        let entry = ResourceKey::create(&reg, Identifier::with_default_namespace("my/entry"));
        assert_eq!(entry.registry_name(), reg.identifier());
        assert_eq!(entry.identifier().path(), "my/entry");
        assert_eq!(entry.identifier().namespace(), "minecraft");
    }

    #[test]
    fn path_edge_cases_negative_and_large_numbers() {
        let key = create_registry_key("-2147483648");
        assert_eq!(key.identifier().path(), "-2147483648");
        assert_eq!(elements_dir_path(&key), "-2147483648");
        let key2 = create_registry_key("18446744073709551615");
        assert_eq!(tags_dir_path(&key2), "tags/18446744073709551615");
    }
}
