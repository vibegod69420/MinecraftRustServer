use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    // JAVA: net.minecraft.core.BlockPos#immutable
    pub fn immutable(&self) -> Self {
        *self
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub location: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalPos {
    pub dimension: ResourceKey,
    pub pos: BlockPos,
}

impl GlobalPos {
    // JAVA: net.minecraft.core.GlobalPos#of
    pub fn of(dimension: ResourceKey, pos: BlockPos) -> Self {
        Self { dimension, pos }
    }

    // JAVA: net.minecraft.core.GlobalPos#dimension
    pub fn dimension(&self) -> &ResourceKey {
        &self.dimension
    }

    // JAVA: net.minecraft.core.GlobalPos#pos
    pub fn pos(&self) -> BlockPos {
        self.pos
    }
}

pub struct Level;

impl Level {
    pub fn overworld() -> ResourceKey {
        ResourceKey {
            location: "minecraft:overworld".to_string(),
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

pub struct Mth;

impl Mth {
    // JAVA: net.minecraft.util.Mth#wrapDegrees(float)
    pub fn wrap_degrees(mut f: f32) -> f32 {
        f %= 360.0;
        if f >= 180.0 {
            f -= 360.0;
        }
        if f < -180.0 {
            f += 360.0;
        }
        f
    }

    // JAVA: net.minecraft.util.Mth#clamp(float,float,float)
    pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
        if v < min {
            min
        } else if v > max {
            max
        } else {
            v
        }
    }
}

pub trait LevelHeightAccessor {}

pub struct CrashReportCategory {
    details: Vec<(String, String)>,
}

impl CrashReportCategory {
    pub fn new() -> Self {
        Self {
            details: Vec::new(),
        }
    }

    pub fn set_detail<F, S>(&mut self, name: &str, supplier: F)
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.details.push((name.to_string(), supplier().into()));
    }

    // JAVA: net.minecraft.CrashReportCategory#formatLocation
    pub fn format_location(_level: &dyn LevelHeightAccessor, pos: BlockPos) -> String {
        pos.to_string()
    }

    pub fn get_detail(&self, name: &str) -> Option<&str> {
        self.details
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

impl Default for CrashReportCategory {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.world.level.storage.LevelData$RespawnData
#[derive(Debug, Clone, PartialEq)]
pub struct RespawnData {
    pub global_pos: GlobalPos,
    pub yaw: f32,
    pub pitch: f32,
}

impl RespawnData {
    // JAVA: net.minecraft.world.level.storage.LevelData$RespawnData#DEFAULT
    pub fn default_data() -> Self {
        Self {
            global_pos: GlobalPos::of(Level::overworld(), BlockPos::ZERO),
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelData$RespawnData#of
    pub fn of(dimension: ResourceKey, pos: BlockPos, yaw: f32, pitch: f32) -> Self {
        Self {
            global_pos: GlobalPos::of(dimension, pos.immutable()),
            yaw: Mth::wrap_degrees(yaw),
            pitch: Mth::clamp(pitch, -90.0, 90.0),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelData$RespawnData#dimension
    pub fn dimension(&self) -> &ResourceKey {
        self.global_pos.dimension()
    }

    // JAVA: net.minecraft.world.level.storage.LevelData$RespawnData#pos
    pub fn pos(&self) -> BlockPos {
        self.global_pos.pos()
    }
}

impl Default for RespawnData {
    fn default() -> Self {
        Self::default_data()
    }
}

// JAVA: net.minecraft.world.level.storage.LevelData
pub trait LevelData {
    // JAVA: net.minecraft.world.level.storage.LevelData#getRespawnData
    fn get_respawn_data(&self) -> RespawnData;

    // JAVA: net.minecraft.world.level.storage.LevelData#getGameTime
    fn get_game_time(&self) -> i64;

    // JAVA: net.minecraft.world.level.storage.LevelData#getDayTime
    fn get_day_time(&self) -> i64;

    // JAVA: net.minecraft.world.level.storage.LevelData#isThundering
    fn is_thundering(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.LevelData#isRaining
    fn is_raining(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.LevelData#setRaining
    fn set_raining(&mut self, raining: bool);

    // JAVA: net.minecraft.world.level.storage.LevelData#isHardcore
    fn is_hardcore(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.LevelData#getDifficulty
    fn get_difficulty(&self) -> Difficulty;

    // JAVA: net.minecraft.world.level.storage.LevelData#isDifficultyLocked
    fn is_difficulty_locked(&self) -> bool;

    // JAVA: net.minecraft.world.level.storage.LevelData#fillCrashReportCategory
    fn fill_crash_report_category(
        &self,
        category: &mut CrashReportCategory,
        level: &dyn LevelHeightAccessor,
    ) {
        let pos = self.get_respawn_data().pos();
        category.set_detail("Level spawn location", || {
            CrashReportCategory::format_location(level, pos)
        });
        let game_time = self.get_game_time();
        let day_time = self.get_day_time();
        category.set_detail("Level time", || {
            format!("{} game time, {} day time", game_time, day_time)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyLevel;
    impl LevelHeightAccessor for DummyLevel {}

    struct DummyData {
        raining: bool,
    }

    impl LevelData for DummyData {
        fn get_respawn_data(&self) -> RespawnData {
            RespawnData::of(
                ResourceKey {
                    location: "minecraft:overworld".to_string(),
                },
                BlockPos { x: 1, y: 64, z: -2 },
                0.0,
                0.0,
            )
        }
        fn get_game_time(&self) -> i64 { 123 }
        fn get_day_time(&self) -> i64 { 456 }
        fn is_thundering(&self) -> bool { false }
        fn is_raining(&self) -> bool { self.raining }
        fn set_raining(&mut self, raining: bool) { self.raining = raining; }
        fn is_hardcore(&self) -> bool { false }
        fn get_difficulty(&self) -> Difficulty { Difficulty::Normal }
        fn is_difficulty_locked(&self) -> bool { false }
    }

    #[test]
    fn respawn_default_values() {
        let d = RespawnData::default_data();
        assert_eq!(d.pos(), BlockPos::ZERO);
        assert_eq!(d.dimension().location, "minecraft:overworld");
        assert_eq!(d.yaw.to_bits(), 0.0f32.to_bits());
        assert_eq!(d.pitch.to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn respawn_of_normalizes_yaw_and_pitch() {
        let dim = ResourceKey {
            location: "minecraft:the_nether".to_string(),
        };
        let r = RespawnData::of(dim.clone(), BlockPos { x: 7, y: 8, z: 9 }, 190.0, 120.0);
        assert_eq!(r.dimension(), &dim);
        assert_eq!(r.pos(), BlockPos { x: 7, y: 8, z: 9 });
        assert!((r.yaw - (-170.0)).abs() < 1e-4);
        assert_eq!(r.pitch.to_bits(), 90.0f32.to_bits());

        let r2 = RespawnData::of(dim, BlockPos::ZERO, -181.0, -120.0);
        assert!((r2.yaw - 179.0).abs() < 1e-4);
        assert_eq!(r2.pitch.to_bits(), (-90.0f32).to_bits());
    }

    #[test]
    fn wrap_degrees_edge_cases() {
        assert_eq!(Mth::wrap_degrees(0.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(Mth::wrap_degrees(360.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(Mth::wrap_degrees(180.0).to_bits(), (-180.0f32).to_bits());
        assert!((Mth::wrap_degrees(-181.0) - 179.0).abs() < 1e-4);
        assert!((Mth::wrap_degrees(540.0) - (-180.0)).abs() < 1e-4);
    }

    #[test]
    fn clamp_float_edges() {
        assert_eq!(Mth::clamp(-100.0, -90.0, 90.0).to_bits(), (-90.0f32).to_bits());
        assert_eq!(Mth::clamp(100.0, -90.0, 90.0).to_bits(), 90.0f32.to_bits());
        assert_eq!(Mth::clamp(f32::NAN, -90.0, 90.0).is_nan(), true);
    }

    #[test]
    fn fill_crash_report_category_populates_details() {
        let data = DummyData { raining: true };
        let mut cat = CrashReportCategory::new();
        data.fill_crash_report_category(&mut cat, &DummyLevel);
        assert_eq!(cat.get_detail("Level spawn location"), Some("1, 64, -2"));
        assert_eq!(cat.get_detail("Level time"), Some("123 game time, 456 day time"));
    }
}
