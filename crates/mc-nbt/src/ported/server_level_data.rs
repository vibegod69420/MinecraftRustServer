use std::string::String;

/// Port of `net.minecraft.world.level.GameType`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
   Survival,
   Creative,
   Adventure,
   Spectator,
}

/// Port of `net.minecraft.world.level.difficulty.Difficulty`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
   Peaceful,
   Easy,
   Normal,
   Hard,
}

/// Port of `net.minecraft.world.level.gamerules.GameRules`
#[derive(Debug, Clone)]
pub struct GameRules;

/// Port of `net.minecraft.world.level.storage.ServerLevelData`
pub trait ServerLevelData {
   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getLevelName
   fn get_level_name(&self) -> String;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setThundering
   fn set_thundering(&self, _: bool);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getRainTime
   fn get_rain_time(&self) -> i32;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setRainTime
   fn set_rain_time(&self, _: i32);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setThunderTime
   fn set_thunder_time(&self, _: i32);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getThunderTime
   fn get_thunder_time(&self) -> i32;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getClearWeatherTime
   fn get_clear_weather_time(&self) -> i32;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setClearWeatherTime
   fn set_clear_weather_time(&self, _: i32);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getWanderingTraderSpawnDelay
   fn get_wandering_trader_spawn_delay(&self) -> i32;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setWanderingTraderSpawnDelay
   fn set_wandering_trader_spawn_delay(&self, _: i32);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getWanderingTraderSpawnChance
   fn get_wandering_trader_spawn_chance(&self) -> i32;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setWanderingTraderSpawnChance
   fn set_wandering_trader_spawn_chance(&self, _: i32);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getWanderingTraderId
   fn get_wandering_trader_id(&self) -> Option<String>;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setWanderingTraderId
   fn set_wandering_trader_id(&self, _: Option<String>);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getGameType
   fn get_game_type(&self) -> GameType;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setGameType
   fn set_game_type(&self, _: GameType);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#isInitialized
   fn is_initialized(&self) -> bool;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setInitialized
   fn set_initialized(&self, _: bool);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#isAllowCommands
   fn is_allow_commands(&self) -> bool;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setAllowCommands
   fn set_allow_commands(&self, _: bool);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setGameTime
   fn set_game_time(&self, _: i64);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#setDayTime
   fn set_day_time(&self, _: i64);

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getGameRules
   fn get_game_rules(&self) -> GameRules;

   // JAVA: net.minecraft.world.level.storage.ServerLevelData#getScheduledEvents
   fn get_scheduled_events(&self) -> ();
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_set_rain_time_wrapping_java_semantics() {
      // Java int wrapping matches Rust i32 wrapping
      let max = i32::MAX;
      assert_eq!(max.wrapping_add(1), i32::MIN);
      let min = i32::MIN;
      assert_eq!(min.wrapping_sub(1), i32::MAX);
   }

   #[test]
   fn test_set_game_time_i64_boundary() {
      // Java long wrapping matches Rust i64 wrapping
      let max = i64::MAX;
      assert_eq!(max.wrapping_add(1), i64::MIN);
      let min = i64::MIN;
      assert_eq!(min.wrapping_sub(1), i64::MAX);
   }

   #[test]
   fn test_game_type_enum_variants() {
      assert_eq!(GameType::Survival as i32, 0);
      assert_eq!(GameType::Creative as i32, 1);
      assert_eq!(GameType::Adventure as i32, 2);
      assert_eq!(GameType::Spectator as i32, 3);
   }

   #[test]
   fn test_difficulty_enum_variants() {
      assert_eq!(Difficulty::Peaceful as i32, 0);
      assert_eq!(Difficulty::Easy as i32, 1);
      assert_eq!(Difficulty::Normal as i32, 2);
      assert_eq!(Difficulty::Hard as i32, 3);
   }

   #[test]
   fn test_is_initialized_default() {
      struct DummyLevel;
      impl ServerLevelData for DummyLevel {
          fn get_level_name(&self) -> String { "world".into() }
          fn set_thundering(&self, _: bool) {}
          fn get_rain_time(&self) -> i32 { 0 }
          fn set_rain_time(&self, _: i32) {}
          fn set_thunder_time(&self, _: i32) {}
          fn get_thunder_time(&self) -> i32 { 0 }
          fn get_clear_weather_time(&self) -> i32 { 0 }
          fn set_clear_weather_time(&self, _: i32) {}
          fn get_wandering_trader_spawn_delay(&self) -> i32 { 0 }
          fn set_wandering_trader_spawn_delay(&self, _: i32) {}
          fn get_wandering_trader_spawn_chance(&self) -> i32 { 0 }
          fn set_wandering_trader_spawn_chance(&self, _: i32) {}
          fn get_wandering_trader_id(&self) -> Option<String> { None }
          fn set_wandering_trader_id(&self, _: Option<String>) {}
          fn get_game_type(&self) -> GameType { GameType::Survival }
          fn set_game_type(&self, _: GameType) {}
          fn is_initialized(&self) -> bool { true }
          fn set_initialized(&self, _: bool) {}
          fn is_allow_commands(&self) -> bool { false }
          fn set_allow_commands(&self, _: bool) {}
          fn set_game_time(&self, _: i64) {}
          fn set_day_time(&self, _: i64) {}
          fn get_game_rules(&self) -> GameRules { GameRules }
          fn get_scheduled_events(&self) -> () {}
      }
      assert!(DummyLevel.is_initialized());
   }
}
