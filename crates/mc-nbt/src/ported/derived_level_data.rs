use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RespawnData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldBorderSettings;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameRules;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MinecraftServer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerQueue<T> {
    _marker: PhantomData<T>,
}

impl<T> TimerQueue<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for TimerQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait LevelHeightAccessor {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrashReportCategory {
    pub details: Vec<(String, String)>,
}

impl CrashReportCategory {
    pub fn new() -> Self {
        Self {
            details: Vec::new(),
        }
    }

    pub fn set_detail(&mut self, key: &str, value: impl fmt::Display) {
        self.details.push((key.to_string(), value.to_string()));
    }
}

pub trait WorldData {
    fn get_level_name(&self) -> String;
    fn get_game_type(&self) -> GameType;
    fn is_hardcore(&self) -> bool;
    fn is_allow_commands(&self) -> bool;
    fn get_game_rules(&self) -> GameRules;
    fn get_difficulty(&self) -> Difficulty;
    fn is_difficulty_locked(&self) -> bool;
}

pub trait ServerLevelData {
    fn get_respawn_data(&self) -> RespawnData;
    fn get_game_time(&self) -> i64;
    fn get_day_time(&self) -> i64;
    fn get_level_name(&self) -> String;
    fn get_clear_weather_time(&self) -> i32;
    fn set_clear_weather_time(&mut self, value: i32);
    fn is_thundering(&self) -> bool;
    fn get_thunder_time(&self) -> i32;
    fn is_raining(&self) -> bool;
    fn get_rain_time(&self) -> i32;
    fn get_game_type(&self) -> GameType;
    fn set_game_time(&mut self, value: i64);
    fn set_day_time(&mut self, value: i64);
    fn set_spawn(&mut self, spawn: RespawnData);
    fn set_thundering(&mut self, value: bool);
    fn set_thunder_time(&mut self, value: i32);
    fn set_raining(&mut self, value: bool);
    fn set_rain_time(&mut self, value: i32);
    fn set_game_type(&mut self, game_type: GameType);
    fn is_hardcore(&self) -> bool;
    fn is_allow_commands(&self) -> bool;
    fn is_initialized(&self) -> bool;
    fn set_initialized(&mut self, value: bool);
    fn get_game_rules(&self) -> GameRules;
    fn get_legacy_world_border_settings(&self) -> Option<WorldBorderSettings>;
    fn set_legacy_world_border_settings(&mut self, settings: Option<WorldBorderSettings>);
    fn get_difficulty(&self) -> Difficulty;
    fn is_difficulty_locked(&self) -> bool;
    fn get_scheduled_events(&self) -> TimerQueue<MinecraftServer>;
    fn get_wandering_trader_spawn_delay(&self) -> i32;
    fn set_wandering_trader_spawn_delay(&mut self, value: i32);
    fn get_wandering_trader_spawn_chance(&self) -> i32;
    fn set_wandering_trader_spawn_chance(&mut self, value: i32);
    fn get_wandering_trader_id(&self) -> Option<Uuid>;
    fn set_wandering_trader_id(&mut self, id: Option<Uuid>);
    fn fill_crash_report_category(
        &self,
        category: &mut CrashReportCategory,
        level: &dyn LevelHeightAccessor,
    );
}

#[derive(Debug)]
pub struct DerivedLevelData<W, S> {
    world_data: W,
    wrapped: S,
}

impl<W: WorldData, S: ServerLevelData> DerivedLevelData<W, S> {
    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#<init>
    pub fn new(world_data: W, wrapped: S) -> Self {
        Self {
            world_data,
            wrapped,
        }
    }
}

impl<W: WorldData, S: ServerLevelData> ServerLevelData for DerivedLevelData<W, S> {
    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getRespawnData
    fn get_respawn_data(&self) -> RespawnData {
        self.wrapped.get_respawn_data()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getGameTime
    fn get_game_time(&self) -> i64 {
        self.wrapped.get_game_time()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getDayTime
    fn get_day_time(&self) -> i64 {
        self.wrapped.get_day_time()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getLevelName
    fn get_level_name(&self) -> String {
        self.world_data.get_level_name()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getClearWeatherTime
    fn get_clear_weather_time(&self) -> i32 {
        self.wrapped.get_clear_weather_time()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setClearWeatherTime
    fn set_clear_weather_time(&mut self, _value: i32) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#isThundering
    fn is_thundering(&self) -> bool {
        self.wrapped.is_thundering()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getThunderTime
    fn get_thunder_time(&self) -> i32 {
        self.wrapped.get_thunder_time()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#isRaining
    fn is_raining(&self) -> bool {
        self.wrapped.is_raining()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getRainTime
    fn get_rain_time(&self) -> i32 {
        self.wrapped.get_rain_time()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getGameType
    fn get_game_type(&self) -> GameType {
        self.world_data.get_game_type()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setGameTime
    fn set_game_time(&mut self, _value: i64) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setDayTime
    fn set_day_time(&mut self, _value: i64) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setSpawn
    fn set_spawn(&mut self, spawn: RespawnData) {
        self.wrapped.set_spawn(spawn);
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setThundering
    fn set_thundering(&mut self, _value: bool) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setThunderTime
    fn set_thunder_time(&mut self, _value: i32) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setRaining
    fn set_raining(&mut self, _value: bool) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setRainTime
    fn set_rain_time(&mut self, _value: i32) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setGameType
    fn set_game_type(&mut self, _game_type: GameType) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#isHardcore
    fn is_hardcore(&self) -> bool {
        self.world_data.is_hardcore()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#isAllowCommands
    fn is_allow_commands(&self) -> bool {
        self.world_data.is_allow_commands()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#isInitialized
    fn is_initialized(&self) -> bool {
        self.wrapped.is_initialized()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setInitialized
    fn set_initialized(&mut self, _value: bool) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getGameRules
    fn get_game_rules(&self) -> GameRules {
        self.world_data.get_game_rules()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getLegacyWorldBorderSettings
    fn get_legacy_world_border_settings(&self) -> Option<WorldBorderSettings> {
        self.wrapped.get_legacy_world_border_settings()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setLegacyWorldBorderSettings
    fn set_legacy_world_border_settings(&mut self, _settings: Option<WorldBorderSettings>) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getDifficulty
    fn get_difficulty(&self) -> Difficulty {
        self.world_data.get_difficulty()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#isDifficultyLocked
    fn is_difficulty_locked(&self) -> bool {
        self.world_data.is_difficulty_locked()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getScheduledEvents
    fn get_scheduled_events(&self) -> TimerQueue<MinecraftServer> {
        self.wrapped.get_scheduled_events()
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getWanderingTraderSpawnDelay
    fn get_wandering_trader_spawn_delay(&self) -> i32 {
        0
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setWanderingTraderSpawnDelay
    fn set_wandering_trader_spawn_delay(&mut self, _value: i32) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getWanderingTraderSpawnChance
    fn get_wandering_trader_spawn_chance(&self) -> i32 {
        0
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setWanderingTraderSpawnChance
    fn set_wandering_trader_spawn_chance(&mut self, _value: i32) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#getWanderingTraderId
    fn get_wandering_trader_id(&self) -> Option<Uuid> {
        None
    }

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#setWanderingTraderId
    fn set_wandering_trader_id(&mut self, _id: Option<Uuid>) {}

    // JAVA: net.minecraft.world.level.storage.DerivedLevelData#fillCrashReportCategory
    fn fill_crash_report_category(
        &self,
        category: &mut CrashReportCategory,
        level: &dyn LevelHeightAccessor,
    ) {
        category.set_detail("Derived", true);
        self.wrapped.fill_crash_report_category(category, level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MockWorldData {
        level_name: String,
        game_type: GameType,
        hardcore: bool,
        allow_commands: bool,
        game_rules: GameRules,
        difficulty: Difficulty,
        difficulty_locked: bool,
    }

    impl Default for MockWorldData {
        fn default() -> Self {
            Self {
                level_name: "world".to_string(),
                game_type: GameType::Survival,
                hardcore: false,
                allow_commands: true,
                game_rules: GameRules,
                difficulty: Difficulty::Normal,
                difficulty_locked: false,
            }
        }
    }

    impl WorldData for MockWorldData {
        fn get_level_name(&self) -> String {
            self.level_name.clone()
        }
        fn get_game_type(&self) -> GameType {
            self.game_type
        }
        fn is_hardcore(&self) -> bool {
            self.hardcore
        }
        fn is_allow_commands(&self) -> bool {
            self.allow_commands
        }
        fn get_game_rules(&self) -> GameRules {
            self.game_rules.clone()
        }
        fn get_difficulty(&self) -> Difficulty {
            self.difficulty
        }
        fn is_difficulty_locked(&self) -> bool {
            self.difficulty_locked
        }
    }

    #[derive(Debug, Clone)]
    struct MockServerLevelData {
        respawn_data: RespawnData,
        game_time: i64,
        day_time: i64,
        clear_weather_time: i32,
        thundering: bool,
        thunder_time: i32,
        raining: bool,
        rain_time: i32,
        initialized: bool,
        legacy_border: Option<WorldBorderSettings>,
        scheduled_events: TimerQueue<MinecraftServer>,
        wandering_trader_spawn_delay: i32,
        wandering_trader_spawn_chance: i32,
        wandering_trader_id: Option<Uuid>,
        spawn_set_count: i32,
    }

    impl Default for MockServerLevelData {
        fn default() -> Self {
            Self {
                respawn_data: RespawnData,
                game_time: 0,
                day_time: 0,
                clear_weather_time: 0,
                thundering: false,
                thunder_time: 0,
                raining: false,
                rain_time: 0,
                initialized: false,
                legacy_border: None,
                scheduled_events: TimerQueue::new(),
                wandering_trader_spawn_delay: 0,
                wandering_trader_spawn_chance: 0,
                wandering_trader_id: None,
                spawn_set_count: 0,
            }
        }
    }

    impl ServerLevelData for MockServerLevelData {
        fn get_respawn_data(&self) -> RespawnData {
            self.respawn_data.clone()
        }
        fn get_game_time(&self) -> i64 {
            self.game_time
        }
        fn get_day_time(&self) -> i64 {
            self.day_time
        }
        fn get_level_name(&self) -> String {
            "wrapped".to_string()
        }
        fn get_clear_weather_time(&self) -> i32 {
            self.clear_weather_time
        }
        fn set_clear_weather_time(&mut self, value: i32) {
            self.clear_weather_time = value;
        }
        fn is_thundering(&self) -> bool {
            self.thundering
        }
        fn get_thunder_time(&self) -> i32 {
            self.thunder_time
        }
        fn is_raining(&self) -> bool {
            self.raining
        }
        fn get_rain_time(&self) -> i32 {
            self.rain_time
        }
        fn get_game_type(&self) -> GameType {
            GameType::Creative
        }
        fn set_game_time(&mut self, value: i64) {
            self.game_time = value;
        }
        fn set_day_time(&mut self, value: i64) {
            self.day_time = value;
        }
        fn set_spawn(&mut self, spawn: RespawnData) {
            self.respawn_data = spawn;
            self.spawn_set_count = self.spawn_set_count.wrapping_add(1);
        }
        fn set_thundering(&mut self, value: bool) {
            self.thundering = value;
        }
        fn set_thunder_time(&mut self, value: i32) {
            self.thunder_time = value;
        }
        fn set_raining(&mut self, value: bool) {
            self.raining = value;
        }
        fn set_rain_time(&mut self, value: i32) {
            self.rain_time = value;
        }
        fn set_game_type(&mut self, _game_type: GameType) {}
        fn is_hardcore(&self) -> bool {
            false
        }
        fn is_allow_commands(&self) -> bool {
            false
        }
        fn is_initialized(&self) -> bool {
            self.initialized
        }
        fn set_initialized(&mut self, value: bool) {
            self.initialized = value;
        }
        fn get_game_rules(&self) -> GameRules {
            GameRules
        }
        fn get_legacy_world_border_settings(&self) -> Option<WorldBorderSettings> {
            self.legacy_border.clone()
        }
        fn set_legacy_world_border_settings(&mut self, settings: Option<WorldBorderSettings>) {
            self.legacy_border = settings;
        }
        fn get_difficulty(&self) -> Difficulty {
            Difficulty::Peaceful
        }
        fn is_difficulty_locked(&self) -> bool {
            false
        }
        fn get_scheduled_events(&self) -> TimerQueue<MinecraftServer> {
            self.scheduled_events.clone()
        }
        fn get_wandering_trader_spawn_delay(&self) -> i32 {
            self.wandering_trader_spawn_delay
        }
        fn set_wandering_trader_spawn_delay(&mut self, value: i32) {
            self.wandering_trader_spawn_delay = value;
        }
        fn get_wandering_trader_spawn_chance(&self) -> i32 {
            self.wandering_trader_spawn_chance
        }
        fn set_wandering_trader_spawn_chance(&mut self, value: i32) {
            self.wandering_trader_spawn_chance = value;
        }
        fn get_wandering_trader_id(&self) -> Option<Uuid> {
            self.wandering_trader_id
        }
        fn set_wandering_trader_id(&mut self, id: Option<Uuid>) {
            self.wandering_trader_id = id;
        }
        fn fill_crash_report_category(
            &self,
            category: &mut CrashReportCategory,
            _level: &dyn LevelHeightAccessor,
        ) {
            category.set_detail("Wrapped", "yes");
        }
    }

    struct DummyLevel;
    impl LevelHeightAccessor for DummyLevel {}

    #[test]
    fn delegates_to_world_data_and_wrapped() {
        let world = MockWorldData {
            level_name: "MainWorld".to_string(),
            game_type: GameType::Adventure,
            hardcore: true,
            allow_commands: false,
            game_rules: GameRules,
            difficulty: Difficulty::Hard,
            difficulty_locked: true,
        };
        let wrapped = MockServerLevelData {
            game_time: 12_345,
            day_time: 67_890,
            clear_weather_time: 100,
            thundering: true,
            thunder_time: 200,
            raining: true,
            rain_time: 300,
            initialized: true,
            legacy_border: Some(WorldBorderSettings),
            ..Default::default()
        };
        let derived = DerivedLevelData::new(world, wrapped);

        assert_eq!(derived.get_level_name(), "MainWorld");
        assert_eq!(derived.get_game_type(), GameType::Adventure);
        assert!(derived.is_hardcore());
        assert!(!derived.is_allow_commands());
        assert_eq!(derived.get_difficulty(), Difficulty::Hard);
        assert!(derived.is_difficulty_locked());
        assert_eq!(derived.get_game_rules(), GameRules);

        assert_eq!(derived.get_game_time(), 12_345);
        assert_eq!(derived.get_day_time(), 67_890);
        assert_eq!(derived.get_clear_weather_time(), 100);
        assert!(derived.is_thundering());
        assert_eq!(derived.get_thunder_time(), 200);
        assert!(derived.is_raining());
        assert_eq!(derived.get_rain_time(), 300);
        assert!(derived.is_initialized());
        assert_eq!(
            derived.get_legacy_world_border_settings(),
            Some(WorldBorderSettings)
        );
    }

    #[test]
    fn setters_are_noops_except_set_spawn() {
        let world = MockWorldData::default();
        let wrapped = MockServerLevelData {
            game_time: 10,
            day_time: 20,
            clear_weather_time: 30,
            thundering: false,
            thunder_time: 40,
            raining: false,
            rain_time: 50,
            initialized: false,
            legacy_border: None,
            wandering_trader_spawn_delay: 7,
            wandering_trader_spawn_chance: 8,
            wandering_trader_id: Some(Uuid {
                most_significant_bits: 1,
                least_significant_bits: 2,
            }),
            ..Default::default()
        };
        let mut derived = DerivedLevelData::new(world, wrapped);

        derived.set_clear_weather_time(999);
        derived.set_game_time(999);
        derived.set_day_time(999);
        derived.set_thundering(true);
        derived.set_thunder_time(999);
        derived.set_raining(true);
        derived.set_rain_time(999);
        derived.set_game_type(GameType::Creative);
        derived.set_initialized(true);
        derived.set_legacy_world_border_settings(Some(WorldBorderSettings));
        derived.set_wandering_trader_spawn_delay(999);
        derived.set_wandering_trader_spawn_chance(999);
        derived.set_wandering_trader_id(Some(Uuid {
            most_significant_bits: 9,
            least_significant_bits: 9,
        }));

        assert_eq!(derived.wrapped.get_clear_weather_time(), 30);
        assert_eq!(derived.wrapped.get_game_time(), 10);
        assert_eq!(derived.wrapped.get_day_time(), 20);
        assert!(!derived.wrapped.is_thundering());
        assert_eq!(derived.wrapped.get_thunder_time(), 40);
        assert!(!derived.wrapped.is_raining());
        assert_eq!(derived.wrapped.get_rain_time(), 50);
        assert!(!derived.wrapped.is_initialized());
        assert_eq!(derived.wrapped.get_legacy_world_border_settings(), None);
        assert_eq!(derived.wrapped.get_wandering_trader_spawn_delay(), 7);
        assert_eq!(derived.wrapped.get_wandering_trader_spawn_chance(), 8);
        assert_eq!(
            derived.wrapped.get_wandering_trader_id(),
            Some(Uuid {
                most_significant_bits: 1,
                least_significant_bits: 2,
            })
        );

        derived.set_spawn(RespawnData);
        assert_eq!(derived.wrapped.spawn_set_count, 1);
    }

    #[test]
    fn wandering_trader_defaults_and_negative_id_ignored() {
        let wrapped = MockServerLevelData {
            wandering_trader_spawn_delay: 123,
            wandering_trader_spawn_chance: 456,
            wandering_trader_id: Some(Uuid {
                most_significant_bits: -1,
                least_significant_bits: i64::MIN,
            }),
            ..Default::default()
        };
        let derived = DerivedLevelData::new(MockWorldData::default(), wrapped);
        assert_eq!(derived.get_wandering_trader_spawn_delay(), 0);
        assert_eq!(derived.get_wandering_trader_spawn_chance(), 0);
        assert_eq!(derived.get_wandering_trader_id(), None);
    }

    #[test]
    fn negative_and_extreme_integer_passthrough() {
        let wrapped = MockServerLevelData {
            game_time: i64::MIN,
            day_time: i64::MAX,
            clear_weather_time: i32::MIN,
            thunder_time: i32::MAX,
            rain_time: -1,
            ..Default::default()
        };
        let derived = DerivedLevelData::new(MockWorldData::default(), wrapped);
        assert_eq!(derived.get_game_time(), i64::MIN);
        assert_eq!(derived.get_day_time(), i64::MAX);
        assert_eq!(derived.get_clear_weather_time(), i32::MIN);
        assert_eq!(derived.get_thunder_time(), i32::MAX);
        assert_eq!(derived.get_rain_time(), -1);
    }

    #[test]
    fn fill_crash_report_category_adds_derived_and_delegates() {
        let derived =
            DerivedLevelData::new(MockWorldData::default(), MockServerLevelData::default());
        let mut category = CrashReportCategory::new();
        let level = DummyLevel;
        derived.fill_crash_report_category(&mut category, &level);
        assert_eq!(category.details.len(), 2);
        assert_eq!(
            category.details[0],
            ("Derived".to_string(), "true".to_string())
        );
        assert_eq!(
            category.details[1],
            ("Wrapped".to_string(), "yes".to_string())
        );
    }

    #[test]
    fn crash_report_detail_preserves_float_bits_representation() {
        let mut category = CrashReportCategory::new();
        let nan_bits = f32::NAN.to_bits();
        let neg_zero_bits = (-0.0f64).to_bits();
        category.set_detail("NaNBits", nan_bits);
        category.set_detail("NegZeroBits", neg_zero_bits);
        assert_eq!(category.details[0], ("NaNBits".to_string(), nan_bits.to_string()));
        assert_eq!(
            category.details[1],
            ("NegZeroBits".to_string(), neg_zero_bits.to_string())
        );
        assert_ne!(neg_zero_bits, 0.0f64.to_bits());
    }
}
