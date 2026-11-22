#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LevelResource {
    id: &'static str,
}

// JAVA: net.minecraft.world.level.storage.LevelResource#PLAYER_ADVANCEMENTS_DIR
pub const PLAYER_ADVANCEMENTS_DIR: LevelResource = LevelResource::new("advancements");
// JAVA: net.minecraft.world.level.storage.LevelResource#PLAYER_STATS_DIR
pub const PLAYER_STATS_DIR: LevelResource = LevelResource::new("stats");
// JAVA: net.minecraft.world.level.storage.LevelResource#PLAYER_DATA_DIR
pub const PLAYER_DATA_DIR: LevelResource = LevelResource::new("playerdata");
// JAVA: net.minecraft.world.level.storage.LevelResource#PLAYER_OLD_DATA_DIR
pub const PLAYER_OLD_DATA_DIR: LevelResource = LevelResource::new("players");
// JAVA: net.minecraft.world.level.storage.LevelResource#LEVEL_DATA_FILE
pub const LEVEL_DATA_FILE: LevelResource = LevelResource::new("level.dat");
// JAVA: net.minecraft.world.level.storage.LevelResource#OLD_LEVEL_DATA_FILE
pub const OLD_LEVEL_DATA_FILE: LevelResource = LevelResource::new("level.dat_old");
// JAVA: net.minecraft.world.level.storage.LevelResource#ICON_FILE
pub const ICON_FILE: LevelResource = LevelResource::new("icon.png");
// JAVA: net.minecraft.world.level.storage.LevelResource#LOCK_FILE
pub const LOCK_FILE: LevelResource = LevelResource::new("session.lock");
// JAVA: net.minecraft.world.level.storage.LevelResource#GENERATED_DIR
pub const GENERATED_DIR: LevelResource = LevelResource::new("generated");
// JAVA: net.minecraft.world.level.storage.LevelResource#DATAPACK_DIR
pub const DATAPACK_DIR: LevelResource = LevelResource::new("datapacks");
// JAVA: net.minecraft.world.level.storage.LevelResource#MAP_RESOURCE_FILE
pub const MAP_RESOURCE_FILE: LevelResource = LevelResource::new("resources.zip");
// JAVA: net.minecraft.world.level.storage.LevelResource#ROOT
pub const ROOT: LevelResource = LevelResource::new(".");

impl LevelResource {
    // JAVA: net.minecraft.world.level.storage.LevelResource#<init>
    const fn new(id: &'static str) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.world.level.storage.LevelResource#getId
    pub fn get_id(&self) -> &str {
        self.id
    }
}

impl std::fmt::Display for LevelResource {
    // JAVA: net.minecraft.world.level.storage.LevelResource#toString
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_and_ids() {
        assert_eq!(PLAYER_ADVANCEMENTS_DIR.get_id(), "advancements");
        assert_eq!(PLAYER_STATS_DIR.get_id(), "stats");
        assert_eq!(PLAYER_DATA_DIR.get_id(), "playerdata");
        assert_eq!(PLAYER_OLD_DATA_DIR.get_id(), "players");
        assert_eq!(LEVEL_DATA_FILE.get_id(), "level.dat");
    }

    #[test]
    fn more_constants_and_ids() {
        assert_eq!(OLD_LEVEL_DATA_FILE.get_id(), "level.dat_old");
        assert_eq!(ICON_FILE.get_id(), "icon.png");
        assert_eq!(LOCK_FILE.get_id(), "session.lock");
        assert_eq!(GENERATED_DIR.get_id(), "generated");
        assert_eq!(DATAPACK_DIR.get_id(), "datapacks");
        assert_eq!(MAP_RESOURCE_FILE.get_id(), "resources.zip");
        assert_eq!(ROOT.get_id(), ".");
    }

    #[test]
    fn to_string_format() {
        assert_eq!(LEVEL_DATA_FILE.to_string(), "/level.dat");
        assert_eq!(ROOT.to_string(), "/.");
        assert_eq!(PLAYER_ADVANCEMENTS_DIR.to_string(), "/advancements");
    }

    #[test]
    fn equality_and_clone() {
        let a = LEVEL_DATA_FILE.clone();
        assert_eq!(a, LEVEL_DATA_FILE);
        assert_ne!(LEVEL_DATA_FILE, LOCK_FILE);
    }
}
