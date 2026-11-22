use crate::NbtTag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataVersion {
    version: i32,
    series: String,
}

impl DataVersion {
    pub fn new(version: i32, series: String) -> Self {
        Self { version, series }
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn series(&self) -> &str {
        &self.series
    }
}

// JAVA: net.minecraft.world.level.storage.LevelVersion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelVersion {
    level_data_version: i32,
    last_played: i64,
    minecraft_version_name: String,
    minecraft_version: DataVersion,
    snapshot: bool,
}

impl LevelVersion {
    // JAVA: net.minecraft.world.level.storage.LevelVersion#<init>
    fn new(
        level_data_version: i32,
        last_played: i64,
        minecraft_version_name: String,
        version: i32,
        series: String,
        snapshot: bool,
    ) -> Self {
        Self {
            level_data_version,
            last_played,
            minecraft_version_name,
            minecraft_version: DataVersion::new(version, series),
            snapshot,
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelVersion#parse
    pub fn parse(tag: &NbtTag) -> Self {
        Self::parse_with_defaults(tag, "Unknown", 0, false)
    }

    // JAVA: net.minecraft.world.level.storage.LevelVersion#parse
    pub fn parse_with_defaults(
        tag: &NbtTag,
        current_name: &str,
        current_data_version: i32,
        current_stable: bool,
    ) -> Self {
        let level_data_version = get_int(tag, "version", 0);
        let last_played = get_long(tag, "LastPlayed", 0i64);
        match tag.compound_get("Version") {
            Some(ver) => {
                let name = get_string(ver, "Name", current_name);
                let id = get_int(ver, "Id", current_data_version);
                let series = get_string(ver, "Series", "main");
                let snapshot = get_boolean(ver, "Snapshot", !current_stable);
                Self::new(level_data_version, last_played, name, id, series, snapshot)
            }
            None => Self::new(
                level_data_version,
                last_played,
                String::new(),
                0,
                "main".to_string(),
                false,
            ),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelVersion#levelDataVersion
    pub fn level_data_version(&self) -> i32 {
        self.level_data_version
    }

    // JAVA: net.minecraft.world.level.storage.LevelVersion#lastPlayed
    pub fn last_played(&self) -> i64 {
        self.last_played
    }

    // JAVA: net.minecraft.world.level.storage.LevelVersion#minecraftVersionName
    pub fn minecraft_version_name(&self) -> &str {
        &self.minecraft_version_name
    }

    // JAVA: net.minecraft.world.level.storage.LevelVersion#minecraftVersion
    pub fn minecraft_version(&self) -> &DataVersion {
        &self.minecraft_version
    }

    // JAVA: net.minecraft.world.level.storage.LevelVersion#snapshot
    pub fn snapshot(&self) -> bool {
        self.snapshot
    }
}

impl Default for LevelVersion {
    fn default() -> Self {
        Self::new(0, 0, String::new(), 0, "main".to_string(), false)
    }
}

fn get_int(tag: &NbtTag, key: &str, default: i32) -> i32 {
    match tag.compound_get(key) {
        Some(NbtTag::Int(v)) => *v,
        Some(NbtTag::Byte(v)) => i32::from(*v),
        Some(NbtTag::Short(v)) => i32::from(*v),
        Some(NbtTag::Long(v)) => *v as i32,
        _ => default,
    }
}

fn get_long(tag: &NbtTag, key: &str, default: i64) -> i64 {
    match tag.compound_get(key) {
        Some(NbtTag::Long(v)) => *v,
        Some(NbtTag::Int(v)) => i64::from(*v),
        Some(NbtTag::Byte(v)) => i64::from(*v),
        Some(NbtTag::Short(v)) => i64::from(*v),
        _ => default,
    }
}

fn get_string(tag: &NbtTag, key: &str, default: &str) -> String {
    match tag.compound_get(key) {
        Some(NbtTag::String(v)) => v.clone(),
        _ => default.to_string(),
    }
}

fn get_boolean(tag: &NbtTag, key: &str, default: bool) -> bool {
    match tag.compound_get(key) {
        Some(NbtTag::Byte(v)) => *v != 0,
        Some(NbtTag::Int(v)) => *v != 0,
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_with_version_compound() {
        let tag = NbtTag::Compound(vec![
            ("version".to_string(), NbtTag::Int(19133)),
            ("LastPlayed".to_string(), NbtTag::Long(123456789i64)),
            (
                "Version".to_string(),
                NbtTag::Compound(vec![
                    ("Name".to_string(), NbtTag::String("1.21.11".to_string())),
                    ("Id".to_string(), NbtTag::Int(4325)),
                    ("Series".to_string(), NbtTag::String("main".to_string())),
                    ("Snapshot".to_string(), NbtTag::Byte(0)),
                ]),
            ),
        ]);
        let lv = LevelVersion::parse_with_defaults(&tag, "fallback", 0, true);
        assert_eq!(lv.level_data_version(), 19133);
        assert_eq!(lv.last_played(), 123456789i64);
        assert_eq!(lv.minecraft_version_name(), "1.21.11");
        assert_eq!(lv.minecraft_version().version(), 4325);
        assert_eq!(lv.minecraft_version().series(), "main");
        assert!(!lv.snapshot());
    }

    #[test]
    fn parse_missing_version_uses_fallback() {
        let tag = NbtTag::Compound(vec![
            ("version".to_string(), NbtTag::Int(10)),
            ("LastPlayed".to_string(), NbtTag::Long(-1i64)),
        ]);
        let lv = LevelVersion::parse(&tag);
        assert_eq!(lv.level_data_version(), 10);
        assert_eq!(lv.last_played(), -1i64);
        assert_eq!(lv.minecraft_version_name(), "");
        assert_eq!(lv.minecraft_version().version(), 0);
        assert_eq!(lv.minecraft_version().series(), "main");
        assert!(!lv.snapshot());
    }

    #[test]
    fn parse_version_present_missing_fields_uses_defaults() {
        let tag = NbtTag::Compound(vec![("Version".to_string(), NbtTag::Compound(vec![]))]);
        let lv = LevelVersion::parse_with_defaults(&tag, "1.21.11", 4325, true);
        assert_eq!(lv.level_data_version(), 0);
        assert_eq!(lv.last_played(), 0);
        assert_eq!(lv.minecraft_version_name(), "1.21.11");
        assert_eq!(lv.minecraft_version().version(), 4325);
        assert_eq!(lv.minecraft_version().series(), "main");
        assert!(!lv.snapshot());
    }

    #[test]
    fn parse_negative_and_large_values() {
        let tag = NbtTag::Compound(vec![
            ("version".to_string(), NbtTag::Int(i32::MIN)),
            ("LastPlayed".to_string(), NbtTag::Long(i64::MIN)),
            (
                "Version".to_string(),
                NbtTag::Compound(vec![
                    ("Id".to_string(), NbtTag::Int(-1)),
                    ("Snapshot".to_string(), NbtTag::Byte(1)),
                    ("Name".to_string(), NbtTag::String(String::new())),
                ]),
            ),
        ]);
        let lv = LevelVersion::parse_with_defaults(&tag, "X", 99, false);
        assert_eq!(lv.level_data_version(), i32::MIN);
        assert_eq!(lv.last_played(), i64::MIN);
        assert_eq!(lv.minecraft_version().version(), -1);
        assert_eq!(lv.minecraft_version_name(), "");
        assert!(lv.snapshot());
    }

    #[test]
    fn parse_non_compound_tag_and_default() {
        let lv = LevelVersion::parse(&NbtTag::Int(0));
        assert_eq!(lv.level_data_version(), 0);
        assert_eq!(lv.last_played(), 0);
        assert_eq!(lv.minecraft_version_name(), "");
        let def = LevelVersion::default();
        assert_eq!(lv, def);
    }
}
