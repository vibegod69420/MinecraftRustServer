// === FILE: primary_level_data.rs ===
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::NbtTag;

/// Minimal NBT compound wrapper for faithful PrimaryLevelData serialization/deserialization.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompoundTag {
    pub entries: Vec<(String, NbtTag)>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_tag(self) -> NbtTag {
        NbtTag::Compound(self.entries)
    }

    pub fn from_tag(tag: &NbtTag) -> Option<Self> {
        match tag {
            NbtTag::Compound(entries) => Some(Self {
                entries: entries.clone(),
            }),
            _ => None,
        }
    }

    pub fn put(&mut self, key: impl Into<String>, value: NbtTag) {
        let key = key.into();
        if let Some((_, v)) = self.entries.iter_mut().find(|(k, _)| k == &key) {
            *v = value;
        } else {
            self.entries.push((key, value));
        }
    }

    pub fn put_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.put(key, NbtTag::String(value.into()));
    }

    pub fn put_int(&mut self, key: impl Into<String>, value: i32) {
        self.put(key, NbtTag::Int(value));
    }

    pub fn put_long(&mut self, key: impl Into<String>, value: i64) {
        self.put(key, NbtTag::Long(value));
    }

    pub fn put_byte(&mut self, key: impl Into<String>, value: i8) {
        self.put(key, NbtTag::Byte(value));
    }

    pub fn put_boolean(&mut self, key: impl Into<String>, value: bool) {
        self.put_byte(key, if value { 1 } else { 0 });
    }

    pub fn get(&self, key: &str) -> Option<&NbtTag> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_compound(&self, key: &str) -> Option<CompoundTag> {
        self.get(key).and_then(Self::from_tag)
    }

    pub fn get_int_or(&self, key: &str, default: i32) -> i32 {
        match self.get(key) {
            Some(NbtTag::Int(v)) => *v,
            Some(NbtTag::Byte(v)) => i32::from(*v),
            Some(NbtTag::Short(v)) => i32::from(*v),
            Some(NbtTag::Long(v)) => *v as i32,
            _ => default,
        }
    }

    pub fn get_long_or(&self, key: &str, default: i64) -> i64 {
        match self.get(key) {
            Some(NbtTag::Long(v)) => *v,
            Some(NbtTag::Int(v)) => i64::from(*v),
            Some(NbtTag::Short(v)) => i64::from(*v),
            Some(NbtTag::Byte(v)) => i64::from(*v),
            _ => default,
        }
    }

    pub fn get_boolean_or(&self, key: &str, default: bool) -> bool {
        match self.get(key) {
            Some(NbtTag::Byte(v)) => *v != 0,
            Some(NbtTag::Short(v)) => *v != 0,
            Some(NbtTag::Int(v)) => *v != 0,
            Some(NbtTag::Long(v)) => *v != 0,
            _ => default,
        }
    }

    pub fn get_string_list(&self, key: &str) -> Vec<String> {
        match self.get(key) {
            Some(NbtTag::List { items, .. }) => items
                .iter()
                .filter_map(|t| match t {
                    NbtTag::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn get_string_set(&self, key: &str) -> HashSet<String> {
        self.get_string_list(key).into_iter().collect()
    }

    pub fn get_uuid(&self, key: &str) -> Option<Uuid> {
        match self.get(key) {
            Some(NbtTag::IntArray(arr)) if arr.len() == 4 => {
                let most = ((arr[0] as u32 as u64) << 32) | (arr[1] as u32 as u64);
                let least = ((arr[2] as u32 as u64) << 32) | (arr[3] as u32 as u64);
                Some(Uuid {
                    most: most as i64,
                    least: least as i64,
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most: i64,
    pub least: i64,
}

impl Uuid {
    pub fn encode_tag(&self) -> NbtTag {
        let most = self.most as u64;
        let least = self.least as u64;
        NbtTag::IntArray(vec![
            (most >> 32) as i32,
            most as i32,
            (least >> 32) as i32,
            least as i32,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameType {
    pub fn get_id(self) -> i32 {
        match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }

    pub fn by_id(id: i32) -> Self {
        match id {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            _ => Self::Survival,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn get_id(self) -> i32 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }

    pub fn by_id(id: i32) -> Self {
        match id & 3 {
            0 => Self::Peaceful,
            1 => Self::Easy,
            3 => Self::Hard,
            _ => Self::Normal,
        }
    }
}

// JAVA: net.minecraft.world.level.storage.PrimaryLevelData$SpecialWorldProperty
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialWorldProperty {
    None,
    Flat,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RegistryAccess;

#[derive(Debug, Default)]
pub struct CrashReportCategory;

pub trait LevelHeightAccessor {}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorldOptions {
    pub tag: Option<CompoundTag>,
}

impl WorldOptions {
    pub fn encode_tag(&self) -> Option<NbtTag> {
        self.tag.clone().map(CompoundTag::into_tag)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RespawnData {
    pub tag: Option<CompoundTag>,
}

impl RespawnData {
    pub fn encode_tag(&self) -> NbtTag {
        self.tag
            .clone()
            .map(CompoundTag::into_tag)
            .unwrap_or_else(|| CompoundTag::new().into_tag())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorldBorderSettings {
    pub tag: CompoundTag,
}

impl WorldBorderSettings {
    pub fn encode_tag(&self) -> NbtTag {
        self.tag.clone().into_tag()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EndDragonFightData {
    pub tag: Option<CompoundTag>,
}

impl EndDragonFightData {
    pub fn encode_tag(&self) -> NbtTag {
        self.tag
            .clone()
            .map(CompoundTag::into_tag)
            .unwrap_or_else(|| CompoundTag::new().into_tag())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TimerQueue {
    pub data: Option<NbtTag>,
}

impl TimerQueue {
    pub fn store(&self) -> NbtTag {
        self.data.clone().unwrap_or_else(|| NbtTag::List {
            elem_type: 10,
            items: Vec::new(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GameRules {
    pub tag: Option<CompoundTag>,
}

impl GameRules {
    pub fn encode_tag(&self) -> NbtTag {
        self.tag
            .clone()
            .map(CompoundTag::into_tag)
            .unwrap_or_else(|| CompoundTag::new().into_tag())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorldDataConfiguration {
    pub tag: Option<CompoundTag>,
}

impl WorldDataConfiguration {
    pub fn store_into(&self, tag: &mut CompoundTag) {
        if let Some(ref c) = self.tag {
            for (k, v) in &c.entries {
                tag.put(k.clone(), v.clone());
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lifecycle {
    #[default]
    Stable,
    Experimental,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelSettings {
    pub level_name: String,
    pub game_type: GameType,
    pub hardcore: bool,
    pub difficulty: Difficulty,
    pub allow_commands: bool,
    pub game_rules: GameRules,
    pub data_configuration: WorldDataConfiguration,
}

impl LevelSettings {
    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn game_type(&self) -> GameType {
        self.game_type
    }

    pub fn hardcore(&self) -> bool {
        self.hardcore
    }

    pub fn allow_commands(&self) -> bool {
        self.allow_commands
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn game_rules(&self) -> &GameRules {
        &self.game_rules
    }

    pub fn level_name(&self) -> &str {
        &self.level_name
    }

    pub fn get_data_configuration(&self) -> &WorldDataConfiguration {
        &self.data_configuration
    }

    pub fn with_game_type(&self, game_type: GameType) -> Self {
        let mut c = self.clone();
        c.game_type = game_type;
        c
    }

    pub fn with_difficulty(&self, difficulty: Difficulty) -> Self {
        let mut c = self.clone();
        c.difficulty = difficulty;
        c
    }

    pub fn with_data_configuration(&self, data_configuration: WorldDataConfiguration) -> Self {
        let mut c = self.clone();
        c.data_configuration = data_configuration;
        c
    }
}

struct SharedVersion {
    name: &'static str,
    id: i32,
    stable: bool,
    series: &'static str,
    data_version: i32,
}

struct SharedConstants;
impl SharedConstants {
    fn get_current_version() -> SharedVersion {
        SharedVersion {
            name: "1.21.11",
            id: 4325,
            stable: true,
            series: "main",
            data_version: 4325,
        }
    }
}

struct NbtUtils;
impl NbtUtils {
    fn add_current_data_version(tag: &mut CompoundTag) {
        let current = SharedConstants::get_current_version();
        tag.put_int("DataVersion", current.data_version);
    }
}

struct Util;
impl Util {
    fn get_epoch_millis() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }
}

struct LevelVersion;
impl LevelVersion {
    fn parse(tag: &CompoundTag) -> i32 {
        tag.get_int_or("version", 19133)
    }
}

#[derive(Debug, Clone)]
pub struct PrimaryLevelData {
    settings: LevelSettings,
    world_options: WorldOptions,
    special_world_property: SpecialWorldProperty,
    world_gen_settings_lifecycle: Lifecycle,
    respawn_data: RespawnData,
    game_time: i64,
    day_time: i64,
    loaded_player_tag: Option<CompoundTag>,
    version: i32,
    clear_weather_time: i32,
    raining: bool,
    rain_time: i32,
    thundering: bool,
    thunder_time: i32,
    initialized: bool,
    difficulty_locked: bool,
    legacy_world_border_settings: Option<WorldBorderSettings>,
    end_dragon_fight_data: EndDragonFightData,
    custom_boss_events: Option<CompoundTag>,
    wandering_trader_spawn_delay: i32,
    wandering_trader_spawn_chance: i32,
    wandering_trader_id: Option<Uuid>,
    known_server_brands: Vec<String>,
    was_modded: bool,
    removed_feature_flags: HashSet<String>,
    scheduled_events: TimerQueue,
}

impl PrimaryLevelData {
    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#<init>
    #[allow(clippy::too_many_arguments)]
    fn new_full(
        loaded_player_tag: Option<CompoundTag>,
        was_modded: bool,
        respawn_data: RespawnData,
        game_time: i64,
        day_time: i64,
        version: i32,
        clear_weather_time: i32,
        rain_time: i32,
        raining: bool,
        thunder_time: i32,
        thundering: bool,
        initialized: bool,
        difficulty_locked: bool,
        legacy_world_border_settings: Option<WorldBorderSettings>,
        wandering_trader_spawn_delay: i32,
        wandering_trader_spawn_chance: i32,
        wandering_trader_id: Option<Uuid>,
        known_server_brands: Vec<String>,
        removed_feature_flags: HashSet<String>,
        scheduled_events: TimerQueue,
        custom_boss_events: Option<CompoundTag>,
        end_dragon_fight_data: EndDragonFightData,
        settings: LevelSettings,
        world_options: WorldOptions,
        special_world_property: SpecialWorldProperty,
        world_gen_settings_lifecycle: Lifecycle,
    ) -> Self {
        Self {
            was_modded,
            respawn_data,
            game_time,
            day_time,
            version,
            clear_weather_time,
            rain_time,
            raining,
            thunder_time,
            thundering,
            initialized,
            difficulty_locked,
            legacy_world_border_settings,
            wandering_trader_spawn_delay,
            wandering_trader_spawn_chance,
            wandering_trader_id,
            known_server_brands,
            removed_feature_flags,
            loaded_player_tag,
            scheduled_events,
            custom_boss_events,
            end_dragon_fight_data,
            settings,
            world_options,
            special_world_property,
            world_gen_settings_lifecycle,
        }
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#<init>
    pub fn new(
        settings: LevelSettings,
        world_options: WorldOptions,
        special_world_property: SpecialWorldProperty,
        world_gen_settings_lifecycle: Lifecycle,
    ) -> Self {
        Self::new_full(
            None,
            false,
            RespawnData::default(),
            0i64,
            0i64,
            19133,
            0,
            0,
            false,
            0,
            false,
            false,
            false,
            None,
            0,
            0,
            None,
            Vec::new(),
            HashSet::new(),
            TimerQueue::default(),
            None,
            EndDragonFightData::default(),
            settings.copy(),
            world_options,
            special_world_property,
            world_gen_settings_lifecycle,
        )
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#parse
    pub fn parse(
        tag: &CompoundTag,
        settings: LevelSettings,
        special_world_property: SpecialWorldProperty,
        world_options: WorldOptions,
        world_gen_settings_lifecycle: Lifecycle,
    ) -> Self {
        let game_time = tag.get_long_or("Time", 0i64);
        let mut known_server_brands = Vec::new();
        for s in tag.get_string_list("ServerBrands") {
            if !known_server_brands.contains(&s) {
                known_server_brands.push(s);
            }
        }

        Self::new_full(
            tag.get_compound("Player"),
            tag.get_boolean_or("WasModded", false),
            tag.get_compound("spawn")
                .map(|c| RespawnData { tag: Some(c) })
                .unwrap_or_default(),
            game_time,
            tag.get_long_or("DayTime", game_time),
            LevelVersion::parse(tag),
            tag.get_int_or("clearWeatherTime", 0),
            tag.get_int_or("rainTime", 0),
            tag.get_boolean_or("raining", false),
            tag.get_int_or("thunderTime", 0),
            tag.get_boolean_or("thundering", false),
            tag.get_boolean_or("initialized", true),
            tag.get_boolean_or("DifficultyLocked", false),
            tag.get_compound("world_border")
                .map(|c| WorldBorderSettings { tag: c }),
            tag.get_int_or("WanderingTraderSpawnDelay", 0),
            tag.get_int_or("WanderingTraderSpawnChance", 0),
            tag.get_uuid("WanderingTraderId"),
            known_server_brands,
            tag.get_string_set("removed_features"),
            TimerQueue {
                data: tag.get("ScheduledEvents").cloned(),
            },
            Some(tag.get_compound("CustomBossEvents").unwrap_or_default()),
            tag.get_compound("DragonFight")
                .map(|c| EndDragonFightData { tag: Some(c) })
                .unwrap_or_default(),
            settings,
            world_options,
            special_world_property,
            world_gen_settings_lifecycle,
        )
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#createTag
    pub fn create_tag(
        &self,
        registry_access: &RegistryAccess,
        player_tag: Option<CompoundTag>,
    ) -> CompoundTag {
        let player = player_tag.or_else(|| self.loaded_player_tag.clone());
        let mut tag = CompoundTag::new();
        self.set_tag_data(registry_access, &mut tag, player.as_ref());
        tag
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setTagData
    fn set_tag_data(
        &self,
        _registry_access: &RegistryAccess,
        tag: &mut CompoundTag,
        player_tag: Option<&CompoundTag>,
    ) {
        tag.put(
            "ServerBrands",
            Self::string_collection_to_tag(&self.known_server_brands),
        );
        tag.put_boolean("WasModded", self.was_modded);
        if !self.removed_feature_flags.is_empty() {
            let flags: Vec<String> = self.removed_feature_flags.iter().cloned().collect();
            tag.put(
                "removed_features",
                Self::string_collection_to_tag(&flags),
            );
        }

        let mut version_tag = CompoundTag::new();
        let current = SharedConstants::get_current_version();
        version_tag.put_string("Name", current.name);
        version_tag.put_int("Id", current.id);
        version_tag.put_boolean("Snapshot", !current.stable);
        version_tag.put_string("Series", current.series);
        tag.put("Version", version_tag.into_tag());
        NbtUtils::add_current_data_version(tag);

        if let Some(wgs) = self.world_options.encode_tag() {
            tag.put("WorldGenSettings", wgs);
        }

        tag.put_int("GameType", self.settings.game_type.get_id());
        tag.put("spawn", self.respawn_data.encode_tag());
        tag.put_long("Time", self.game_time);
        tag.put_long("DayTime", self.day_time);
        tag.put_long("LastPlayed", Util::get_epoch_millis());
        tag.put_string("LevelName", self.settings.level_name.clone());
        tag.put_int("version", 19133);
        tag.put_int("clearWeatherTime", self.clear_weather_time);
        tag.put_int("rainTime", self.rain_time);
        tag.put_boolean("raining", self.raining);
        tag.put_int("thunderTime", self.thunder_time);
        tag.put_boolean("thundering", self.thundering);
        tag.put_boolean("hardcore", self.settings.hardcore);
        tag.put_boolean("allowCommands", self.settings.allow_commands);
        tag.put_boolean("initialized", self.initialized);
        if let Some(ref border) = self.legacy_world_border_settings {
            tag.put("world_border", border.encode_tag());
        }
        tag.put_byte("Difficulty", self.settings.difficulty.get_id() as i8);
        tag.put_boolean("DifficultyLocked", self.difficulty_locked);
        tag.put("game_rules", self.settings.game_rules.encode_tag());
        tag.put("DragonFight", self.end_dragon_fight_data.encode_tag());
        if let Some(p) = player_tag {
            tag.put("Player", p.clone().into_tag());
        }
        self.settings.data_configuration.store_into(tag);
        if let Some(ref cbe) = self.custom_boss_events {
            tag.put("CustomBossEvents", cbe.clone().into_tag());
        }
        tag.put("ScheduledEvents", self.scheduled_events.store());
        tag.put_int("WanderingTraderSpawnDelay", self.wandering_trader_spawn_delay);
        tag.put_int("WanderingTraderSpawnChance", self.wandering_trader_spawn_chance);
        if let Some(ref id) = self.wandering_trader_id {
            tag.put("WanderingTraderId", id.encode_tag());
        }
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#stringCollectionToTag
    fn string_collection_to_tag(values: &[String]) -> NbtTag {
        let items = values.iter().cloned().map(NbtTag::String).collect();
        NbtTag::List {
            elem_type: 8,
            items,
        }
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getRespawnData
    pub fn get_respawn_data(&self) -> &RespawnData {
        &self.respawn_data
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getGameTime
    pub fn get_game_time(&self) -> i64 {
        self.game_time
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getDayTime
    pub fn get_day_time(&self) -> i64 {
        self.day_time
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getLoadedPlayerTag
    pub fn get_loaded_player_tag(&self) -> Option<&CompoundTag> {
        self.loaded_player_tag.as_ref()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setGameTime
    pub fn set_game_time(&mut self, game_time: i64) {
        self.game_time = game_time;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setDayTime
    pub fn set_day_time(&mut self, day_time: i64) {
        self.day_time = day_time;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setSpawn
    pub fn set_spawn(&mut self, respawn_data: RespawnData) {
        self.respawn_data = respawn_data;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getLevelName
    pub fn get_level_name(&self) -> &str {
        self.settings.level_name()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getVersion
    pub fn get_version(&self) -> i32 {
        self.version
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getClearWeatherTime
    pub fn get_clear_weather_time(&self) -> i32 {
        self.clear_weather_time
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setClearWeatherTime
    pub fn set_clear_weather_time(&mut self, clear_weather_time: i32) {
        self.clear_weather_time = clear_weather_time;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#isThundering
    pub fn is_thundering(&self) -> bool {
        self.thundering
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setThundering
    pub fn set_thundering(&mut self, thundering: bool) {
        self.thundering = thundering;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getThunderTime
    pub fn get_thunder_time(&self) -> i32 {
        self.thunder_time
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setThunderTime
    pub fn set_thunder_time(&mut self, thunder_time: i32) {
        self.thunder_time = thunder_time;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#isRaining
    pub fn is_raining(&self) -> bool {
        self.raining
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setRaining
    pub fn set_raining(&mut self, raining: bool) {
        self.raining = raining;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getRainTime
    pub fn get_rain_time(&self) -> i32 {
        self.rain_time
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setRainTime
    pub fn set_rain_time(&mut self, rain_time: i32) {
        self.rain_time = rain_time;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getGameType
    pub fn get_game_type(&self) -> GameType {
        self.settings.game_type()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setGameType
    pub fn set_game_type(&mut self, game_type: GameType) {
        self.settings = self.settings.with_game_type(game_type);
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#isHardcore
    pub fn is_hardcore(&self) -> bool {
        self.settings.hardcore()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#isAllowCommands
    pub fn is_allow_commands(&self) -> bool {
        self.settings.allow_commands()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#isInitialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setInitialized
    pub fn set_initialized(&mut self, initialized: bool) {
        self.initialized = initialized;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getGameRules
    pub fn get_game_rules(&self) -> &GameRules {
        self.settings.game_rules()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getLegacyWorldBorderSettings
    pub fn get_legacy_world_border_settings(&self) -> Option<&WorldBorderSettings> {
        self.legacy_world_border_settings.as_ref()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setLegacyWorldBorderSettings
    pub fn set_legacy_world_border_settings(&mut self, settings: Option<WorldBorderSettings>) {
        self.legacy_world_border_settings = settings;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getDifficulty
    pub fn get_difficulty(&self) -> Difficulty {
        self.settings.difficulty()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setDifficulty
    pub fn set_difficulty(&mut self, difficulty: Difficulty) {
        self.settings = self.settings.with_difficulty(difficulty);
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#isDifficultyLocked
    pub fn is_difficulty_locked(&self) -> bool {
        self.difficulty_locked
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setDifficultyLocked
    pub fn set_difficulty_locked(&mut self, difficulty_locked: bool) {
        self.difficulty_locked = difficulty_locked;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getScheduledEvents
    pub fn get_scheduled_events(&self) -> &TimerQueue {
        &self.scheduled_events
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#fillCrashReportCategory
    pub fn fill_crash_report_category(
        &self,
        _category: &mut CrashReportCategory,
        _height_accessor: &dyn LevelHeightAccessor,
    ) {
        // ServerLevelData.super.fillCrashReportCategory($$0, $$1);
        // WorldData.super.fillCrashReportCategory($$0);
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#worldGenOptions
    pub fn world_gen_options(&self) -> &WorldOptions {
        &self.world_options
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#isFlatWorld
    pub fn is_flat_world(&self) -> bool {
        self.special_world_property == SpecialWorldProperty::Flat
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#isDebugWorld
    pub fn is_debug_world(&self) -> bool {
        self.special_world_property == SpecialWorldProperty::Debug
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#worldGenSettingsLifecycle
    pub fn world_gen_settings_lifecycle(&self) -> Lifecycle {
        self.world_gen_settings_lifecycle
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#endDragonFightData
    pub fn end_dragon_fight_data(&self) -> &EndDragonFightData {
        &self.end_dragon_fight_data
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setEndDragonFightData
    pub fn set_end_dragon_fight_data(&mut self, data: EndDragonFightData) {
        self.end_dragon_fight_data = data;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getDataConfiguration
    pub fn get_data_configuration(&self) -> &WorldDataConfiguration {
        self.settings.get_data_configuration()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setDataConfiguration
    pub fn set_data_configuration(&mut self, data_configuration: WorldDataConfiguration) {
        self.settings = self.settings.with_data_configuration(data_configuration);
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getCustomBossEvents
    pub fn get_custom_boss_events(&self) -> Option<&CompoundTag> {
        self.custom_boss_events.as_ref()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setCustomBossEvents
    pub fn set_custom_boss_events(&mut self, custom_boss_events: Option<CompoundTag>) {
        self.custom_boss_events = custom_boss_events;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getWanderingTraderSpawnDelay
    pub fn get_wandering_trader_spawn_delay(&self) -> i32 {
        self.wandering_trader_spawn_delay
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setWanderingTraderSpawnDelay
    pub fn set_wandering_trader_spawn_delay(&mut self, delay: i32) {
        self.wandering_trader_spawn_delay = delay;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getWanderingTraderSpawnChance
    pub fn get_wandering_trader_spawn_chance(&self) -> i32 {
        self.wandering_trader_spawn_chance
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setWanderingTraderSpawnChance
    pub fn set_wandering_trader_spawn_chance(&mut self, chance: i32) {
        self.wandering_trader_spawn_chance = chance;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getWanderingTraderId
    pub fn get_wandering_trader_id(&self) -> Option<Uuid> {
        self.wandering_trader_id
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setWanderingTraderId
    pub fn set_wandering_trader_id(&mut self, id: Uuid) {
        self.wandering_trader_id = Some(id);
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#setModdedInfo
    pub fn set_modded_info(&mut self, brand: String, modded: bool) {
        if !self.known_server_brands.contains(&brand) {
            self.known_server_brands.push(brand);
        }
        self.was_modded |= modded;
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#wasModded
    pub fn was_modded(&self) -> bool {
        self.was_modded
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getKnownServerBrands
    pub fn get_known_server_brands(&self) -> Vec<String> {
        self.known_server_brands.clone()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getRemovedFeatureFlags
    pub fn get_removed_feature_flags(&self) -> HashSet<String> {
        self.removed_feature_flags.clone()
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#overworldData
    pub fn overworld_data(&self) -> &Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.PrimaryLevelData#getLevelSettings
    pub fn get_level_settings(&self) -> LevelSettings {
        self.settings.copy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_settings() -> LevelSettings {
        LevelSettings {
            level_name: "world".to_string(),
            game_type: GameType::Survival,
            hardcore: false,
            difficulty: Difficulty::Normal,
            allow_commands: false,
            game_rules: GameRules::default(),
            data_configuration: WorldDataConfiguration::default(),
        }
    }

    #[test]
    fn test_default_and_special_world_property() {
        let data = PrimaryLevelData::new(
            test_settings(),
            WorldOptions::default(),
            SpecialWorldProperty::Flat,
            Lifecycle::Stable,
        );
        assert_eq!(data.get_game_time(), 0);
        assert_eq!(data.get_day_time(), 0);
        assert_eq!(data.get_version(), 19133);
        assert!(!data.is_raining());
        assert!(!data.is_thundering());
        assert!(!data.is_initialized());
        assert!(!data.is_difficulty_locked());
        assert!(data.is_flat_world());
        assert!(!data.is_debug_world());
        assert_eq!(data.get_level_name(), "world");
    }

    #[test]
    fn test_negative_and_extreme_integers() {
        let mut data = PrimaryLevelData::new(
            test_settings(),
            WorldOptions::default(),
            SpecialWorldProperty::None,
            Lifecycle::Stable,
        );
        data.set_game_time(i64::MIN);
        data.set_day_time(i64::MAX);
        data.set_rain_time(i32::MIN);
        data.set_thunder_time(i32::MAX);
        data.set_clear_weather_time(-1);
        data.set_wandering_trader_spawn_delay(i32::MIN);
        data.set_wandering_trader_spawn_chance(-999);

        assert_eq!(data.get_game_time(), i64::MIN);
        assert_eq!(data.get_day_time(), i64::MAX);
        assert_eq!(data.get_rain_time(), i32::MIN);
        assert_eq!(data.get_thunder_time(), i32::MAX);
        assert_eq!(data.get_clear_weather_time(), -1);
        assert_eq!(data.get_wandering_trader_spawn_delay(), i32::MIN);
        assert_eq!(data.get_wandering_trader_spawn_chance(), -999);
    }

    #[test]
    fn test_modded_info_and_string_collection_tag() {
        let mut data = PrimaryLevelData::new(
            test_settings(),
            WorldOptions::default(),
            SpecialWorldProperty::None,
            Lifecycle::Stable,
        );
        assert!(!data.was_modded());
        data.set_modded_info("forge".to_string(), false);
        data.set_modded_info("fabric".to_string(), true);
        data.set_modded_info("forge".to_string(), false);

        assert!(data.was_modded());
        let brands = data.get_known_server_brands();
        assert_eq!(brands, vec!["forge".to_string(), "fabric".to_string()]);

        let tag = PrimaryLevelData::string_collection_to_tag(&brands);
        match tag {
            NbtTag::List { elem_type, items } => {
                assert_eq!(elem_type, 8);
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], NbtTag::String("forge".to_string()));
                assert_eq!(items[1], NbtTag::String("fabric".to_string()));
            }
            _ => panic!("expected string list tag"),
        }
    }

    #[test]
    fn test_create_tag_and_parse_defaults() {
        let mut data = PrimaryLevelData::new(
            test_settings(),
            WorldOptions::default(),
            SpecialWorldProperty::Debug,
            Lifecycle::Experimental,
        );
        data.set_game_time(12345);
        data.set_day_time(-67890);
        data.set_raining(true);
        data.set_thunder_time(42);
        data.set_difficulty(Difficulty::Hard);
        data.set_difficulty_locked(true);

        let tag = data.create_tag(&RegistryAccess, None);
        assert_eq!(tag.get_long_or("Time", 0), 12345);
        assert_eq!(tag.get_long_or("DayTime", 0), -67890);
        assert!(tag.get_boolean_or("raining", false));
        assert_eq!(tag.get_int_or("thunderTime", 0), 42);
        assert_eq!(tag.get_int_or("version", 0), 19133);
        assert!(tag.get_boolean_or("DifficultyLocked", false));
        match tag.get("Difficulty") {
            Some(NbtTag::Byte(b)) => assert_eq!(*b, 3i8),
            other => panic!("expected difficulty byte, got {:?}", other),
        }

        let parsed = PrimaryLevelData::parse(
            &CompoundTag::new(),
            test_settings(),
            SpecialWorldProperty::None,
            WorldOptions::default(),
            Lifecycle::Stable,
        );
        assert_eq!(parsed.get_game_time(), 0);
        assert_eq!(parsed.get_day_time(), 0);
        assert!(parsed.is_initialized());
        assert_eq!(parsed.get_version(), 19133);
    }

    #[test]
    fn test_float_bits_and_uuid_round_trip() {
        let mut tag = CompoundTag::new();
        tag.put("f_nan", NbtTag::Float(f32::NAN));
        tag.put("f_neg_zero", NbtTag::Float(-0.0f32));
        tag.put("d_nan", NbtTag::Double(f64::NAN));
        tag.put("d_neg_zero", NbtTag::Double(-0.0f64));

        let f_nan = match tag.get("f_nan") {
            Some(NbtTag::Float(v)) => *v,
            _ => panic!("expected float"),
        };
        assert_eq!(f_nan.to_bits(), f32::NAN.to_bits());

        let f_neg_zero = match tag.get("f_neg_zero") {
            Some(NbtTag::Float(v)) => *v,
            _ => panic!("expected float"),
        };
        assert_eq!(f_neg_zero.to_bits(), (-0.0f32).to_bits());
        assert_ne!(f_neg_zero.to_bits(), 0.0f32.to_bits());

        let d_nan = match tag.get("d_nan") {
            Some(NbtTag::Double(v)) => *v,
            _ => panic!("expected double"),
        };
        assert_eq!(d_nan.to_bits(), f64::NAN.to_bits());

        let uuid = Uuid {
            most: i64::MIN,
            least: -1i64,
        };
        tag.put("WanderingTraderId", uuid.encode_tag());
        let decoded = tag.get_uuid("WanderingTraderId").expect("uuid");
        assert_eq!(decoded, uuid);
    }
}
