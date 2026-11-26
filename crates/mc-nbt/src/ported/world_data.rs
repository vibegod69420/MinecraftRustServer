//! Faithful port of `net.minecraft.world.level.storage.WorldData`.
//!
//! Many dependent Minecraft types are not yet ported in this crate. This module
//! keeps the interface shape and all directly expressible Java semantics
//! (constants, default methods, crash-report formatting, and storage version
//! naming) while using local placeholder types for unported dependencies.

use std::fmt;

/// JAVA: net.minecraft.world.level.storage.WorldData#ANVIL_VERSION_ID
pub const ANVIL_VERSION_ID: i32 = 19133;
/// JAVA: net.minecraft.world.level.storage.WorldData#MCREGION_VERSION_ID
pub const MCREGION_VERSION_ID: i32 = 19132;

pub type CompoundTag = crate::NbtTag;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrashReportCategory {
    details: Vec<(String, String)>,
}

impl CrashReportCategory {
    // JAVA: net.minecraft.CrashReportCategory#setDetail
    pub fn set_detail(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.details.push((name.into(), value.into()));
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.details
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldDataConfiguration {
    pub enabled_features: FeatureFlagSet,
}

impl WorldDataConfiguration {
    // JAVA: net.minecraft.world.level.WorldDataConfiguration#enabledFeatures
    pub fn enabled_features(&self) -> FeatureFlagSet {
        self.enabled_features.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FeatureFlagSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameType {
    #[default]
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Difficulty {
    Peaceful,
    Easy,
    #[default]
    Normal,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LevelSettings;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerLevelData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameRules;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lifecycle {
    #[default]
    Stable,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegistryAccess;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EndDragonFightData;

// JAVA: net.minecraft.world.level.storage.WorldData
pub trait WorldData {
    // JAVA: net.minecraft.world.level.storage.WorldData#getDataConfiguration
    fn get_data_configuration(&self) -> WorldDataConfiguration;

    // JAVA: net.minecraft.world.level.storage.WorldData#setDataConfiguration
    fn set_data_configuration(&mut self, configuration: WorldDataConfiguration);

    // JAVA: net.minecraft.world.level.storage.WorldData#wasModded
    fn was_modded(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.WorldData#getKnownServerBrands
    fn get_known_server_brands(&self) -> std::collections::BTreeSet<String>;

    // JAVA: net.minecraft.world.level.storage.WorldData#getRemovedFeatureFlags
    fn get_removed_feature_flags(&self) -> std::collections::BTreeSet<String>;

    // JAVA: net.minecraft.world.level.storage.WorldData#setModdedInfo
    fn set_modded_info(&mut self, brand: String, modded: bool);

    // JAVA: net.minecraft.world.level.storage.WorldData#fillCrashReportCategory
    fn fill_crash_report_category(&self, category: &mut CrashReportCategory) {
        let brands = self.get_known_server_brands();
        let removed = self.get_removed_feature_flags();
        let was_modded = self.was_modded();
        let version = self.get_version();
        let storage_name = self.get_storage_version_name(version);
        category.set_detail("Known server brands", join_comma(&brands));
        category.set_detail("Removed feature flags", join_comma(&removed));
        category.set_detail("Level was modded", was_modded.to_string());
        category.set_detail(
            "Level storage version",
            format_storage_version(version, &storage_name),
        );
    }

    // JAVA: net.minecraft.world.level.storage.WorldData#getStorageVersionName
    fn get_storage_version_name(&self, version: i32) -> String {
        get_storage_version_name(version)
    }

    // JAVA: net.minecraft.world.level.storage.WorldData#getCustomBossEvents
    fn get_custom_boss_events(&self) -> Option<CompoundTag>;

    // JAVA: net.minecraft.world.level.storage.WorldData#setCustomBossEvents
    fn set_custom_boss_events(&mut self, tag: Option<CompoundTag>);

    // JAVA: net.minecraft.world.level.storage.WorldData#overworldData
    fn overworld_data(&self) -> &ServerLevelData;

    // JAVA: net.minecraft.world.level.storage.WorldData#overworldData (mutable access helper)
    fn overworld_data_mut(&mut self) -> &mut ServerLevelData;

    // JAVA: net.minecraft.world.level.storage.WorldData#getLevelSettings
    fn get_level_settings(&self) -> LevelSettings;

    // JAVA: net.minecraft.world.level.storage.WorldData#createTag
    fn create_tag(
        &self,
        registry_access: &RegistryAccess,
        host_player_nbt: Option<CompoundTag>,
    ) -> CompoundTag;

    // JAVA: net.minecraft.world.level.storage.WorldData#isHardcore
    fn is_hardcore(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.WorldData#getVersion
    fn get_version(&self) -> i32;

    // JAVA: net.minecraft.world.level.storage.WorldData#getLevelName
    fn get_level_name(&self) -> String;

    // JAVA: net.minecraft.world.level.storage.WorldData#getGameType
    fn get_game_type(&self) -> GameType;

    // JAVA: net.minecraft.world.level.storage.WorldData#setGameType
    fn set_game_type(&mut self, game_type: GameType);

    // JAVA: net.minecraft.world.level.storage.WorldData#isAllowCommands
    fn is_allow_commands(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.WorldData#getDifficulty
    fn get_difficulty(&self) -> Difficulty;

    // JAVA: net.minecraft.world.level.storage.WorldData#setDifficulty
    fn set_difficulty(&mut self, difficulty: Difficulty);

    // JAVA: net.minecraft.world.level.storage.WorldData#isDifficultyLocked
    fn is_difficulty_locked(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.WorldData#setDifficultyLocked
    fn set_difficulty_locked(&mut self, locked: bool);

    // JAVA: net.minecraft.world.level.storage.WorldData#getGameRules
    fn get_game_rules(&self) -> &GameRules;

    // JAVA: net.minecraft.world.level.storage.WorldData#getGameRules (mutable access helper)
    fn get_game_rules_mut(&mut self) -> &mut GameRules;

    // JAVA: net.minecraft.world.level.storage.WorldData#getLoadedPlayerTag
    fn get_loaded_player_tag(&self) -> Option<CompoundTag>;

    // JAVA: net.minecraft.world.level.storage.WorldData#endDragonFightData
    fn end_dragon_fight_data(&self) -> EndDragonFightData;

    // JAVA: net.minecraft.world.level.storage.WorldData#setEndDragonFightData
    fn set_end_dragon_fight_data(&mut self, data: EndDragonFightData);

    // JAVA: net.minecraft.world.level.storage.WorldData#worldGenOptions
    fn world_gen_options(&self) -> WorldOptions;

    // JAVA: net.minecraft.world.level.storage.WorldData#isFlatWorld
    fn is_flat_world(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.WorldData#isDebugWorld
    fn is_debug_world(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.WorldData#worldGenSettingsLifecycle
    fn world_gen_settings_lifecycle(&self) -> Lifecycle;

    // JAVA: net.minecraft.world.level.storage.WorldData#enabledFeatures
    fn enabled_features(&self) -> FeatureFlagSet {
        self.get_data_configuration().enabled_features()
    }
}

// JAVA: net.minecraft.world.level.storage.WorldData#getStorageVersionName
pub fn get_storage_version_name(version: i32) -> String {
    match version {
        19132 => "McRegion".to_string(),
        19133 => "Anvil".to_string(),
        _ => "Unknown?".to_string(),
    }
}

fn join_comma(values: &std::collections::BTreeSet<String>) -> String {
    let mut out = String::new();
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(v);
    }
    out
}

fn format_storage_version(version: i32, name: &str) -> String {
    // Java: String.format(Locale.ROOT, "0x%05X - %s", version, name)
    let mut hex = format!("{:X}", version as u32);
    if hex.len() < 5 {
        let mut padded = String::with_capacity(5);
        for _ in 0..(5 - hex.len()) {
            padded.push('0');
        }
        padded.push_str(&hex);
        hex = padded;
    }
    fmt::format(format_args!("0x{} - {}", hex, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct MockWorldData {
        config: WorldDataConfiguration,
        modded: bool,
        brands: BTreeSet<String>,
        removed: BTreeSet<String>,
        custom_boss: Option<CompoundTag>,
        overworld: ServerLevelData,
        game_rules: GameRules,
        version: i32,
    }

    impl WorldData for MockWorldData {
        fn get_data_configuration(&self) -> WorldDataConfiguration {
            self.config.clone()
        }
        fn set_data_configuration(&mut self, configuration: WorldDataConfiguration) {
            self.config = configuration;
        }
        fn was_modded(&self) -> bool {
            self.modded
        }
        fn get_known_server_brands(&self) -> BTreeSet<String> {
            self.brands.clone()
        }
        fn get_removed_feature_flags(&self) -> BTreeSet<String> {
            self.removed.clone()
        }
        fn set_modded_info(&mut self, brand: String, modded: bool) {
            self.brands.insert(brand);
            self.modded |= modded;
        }
        fn get_custom_boss_events(&self) -> Option<CompoundTag> {
            self.custom_boss.clone()
        }
        fn set_custom_boss_events(&mut self, tag: Option<CompoundTag>) {
            self.custom_boss = tag;
        }
        fn overworld_data(&self) -> &ServerLevelData {
            &self.overworld
        }
        fn overworld_data_mut(&mut self) -> &mut ServerLevelData {
            &mut self.overworld
        }
        fn get_level_settings(&self) -> LevelSettings {
            LevelSettings
        }
        fn create_tag(
            &self,
            _registry_access: &RegistryAccess,
            host_player_nbt: Option<CompoundTag>,
        ) -> CompoundTag {
            host_player_nbt.unwrap_or(CompoundTag::Compound(Vec::new()))
        }
        fn is_hardcore(&self) -> bool {
            false
        }
        fn get_version(&self) -> i32 {
            self.version
        }
        fn get_level_name(&self) -> String {
            "world".to_string()
        }
        fn get_game_type(&self) -> GameType {
            GameType::Survival
        }
        fn set_game_type(&mut self, _game_type: GameType) {}
        fn is_allow_commands(&self) -> bool {
            false
        }
        fn get_difficulty(&self) -> Difficulty {
            Difficulty::Normal
        }
        fn set_difficulty(&mut self, _difficulty: Difficulty) {}
        fn is_difficulty_locked(&self) -> bool {
            false
        }
        fn set_difficulty_locked(&mut self, _locked: bool) {}
        fn get_game_rules(&self) -> &GameRules {
            &self.game_rules
        }
        fn get_game_rules_mut(&mut self) -> &mut GameRules {
            &mut self.game_rules
        }
        fn get_loaded_player_tag(&self) -> Option<CompoundTag> {
            None
        }
        fn end_dragon_fight_data(&self) -> EndDragonFightData {
            EndDragonFightData
        }
        fn set_end_dragon_fight_data(&mut self, _data: EndDragonFightData) {}
        fn world_gen_options(&self) -> WorldOptions {
            WorldOptions
        }
        fn is_flat_world(&self) -> bool {
            false
        }
        fn is_debug_world(&self) -> bool {
            false
        }
        fn world_gen_settings_lifecycle(&self) -> Lifecycle {
            Lifecycle::Stable
        }
    }

    #[test]
    fn storage_version_name_constants_and_default() {
        assert_eq!(ANVIL_VERSION_ID, 19133);
        assert_eq!(MCREGION_VERSION_ID, 19132);
        let data = MockWorldData::default();
        assert_eq!(data.get_storage_version_name(19133), "Anvil");
        assert_eq!(data.get_storage_version_name(19132), "McRegion");
        assert_eq!(data.get_storage_version_name(0), "Unknown?");
        assert_eq!(get_storage_version_name(-1), "Unknown?");
    }

    #[test]
    fn fill_crash_report_category_formats_like_java() {
        let mut data = MockWorldData {
            version: ANVIL_VERSION_ID,
            modded: true,
            ..Default::default()
        };
        data.brands.insert("vanilla".to_string());
        data.brands.insert("forge".to_string());
        data.removed.insert("minecraft:old".to_string());

        let mut cat = CrashReportCategory::default();
        data.fill_crash_report_category(&mut cat);

        assert_eq!(cat.get("Known server brands"), Some("forge, vanilla"));
        assert_eq!(cat.get("Removed feature flags"), Some("minecraft:old"));
        assert_eq!(cat.get("Level was modded"), Some("true"));
        assert_eq!(
            cat.get("Level storage version"),
            Some("0x04ABD - Anvil")
        );
    }

    #[test]
    fn fill_crash_report_category_negative_and_unknown_version() {
        let data = MockWorldData {
            version: -1,
            ..Default::default()
        };
        let mut cat = CrashReportCategory::default();
        data.fill_crash_report_category(&mut cat);
        assert_eq!(
            cat.get("Level storage version"),
            Some("0xFFFFFFFF - Unknown?")
        );
    }

    #[test]
    fn enabled_features_default_uses_data_configuration() {
        let data = MockWorldData::default();
        let _ = data.enabled_features();
        assert_eq!(data.enabled_features(), FeatureFlagSet);
    }

    #[test]
    fn custom_boss_events_roundtrip_and_create_tag() {
        let mut data = MockWorldData::default();
        assert!(data.get_custom_boss_events().is_none());
        let tag = CompoundTag::Compound(vec![("id".to_string(), CompoundTag::String("boss".to_string()))]);
        data.set_custom_boss_events(Some(tag.clone()));
        assert_eq!(data.get_custom_boss_events(), Some(tag.clone()));
        let created = data.create_tag(&RegistryAccess, Some(tag.clone()));
        assert_eq!(created, tag);
        let empty = data.create_tag(&RegistryAccess, None);
        assert_eq!(empty, CompoundTag::Compound(Vec::new()));
    }
}
