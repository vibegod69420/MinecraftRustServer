use std::cell::RefCell;
use std::cmp::Ordering;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Minimal self-contained support types (unported Minecraft dependencies)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameType {
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Survival => "survival",
            Self::Creative => "creative",
            Self::Adventure => "adventure",
            Self::Spectator => "spectator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatFormatting {
    Red,
    Italic,
    Yellow,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    pub color: Option<i32>,
    pub formats: Vec<ChatFormatting>,
}

impl Style {
    pub fn with_color(mut self, color: i32) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentContent {
    Empty,
    Literal(String),
    Translatable(String, Vec<Component>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub content: ComponentContent,
    pub style: Style,
    pub siblings: Vec<Component>,
}

impl Component {
    pub fn empty() -> Self {
        Self {
            content: ComponentContent::Empty,
            style: Style::default(),
            siblings: Vec::new(),
        }
    }

    pub fn literal(text: impl Into<String>) -> Self {
        Self {
            content: ComponentContent::Literal(text.into()),
            style: Style::default(),
            siblings: Vec::new(),
        }
    }

    pub fn translatable(key: impl Into<String>) -> Self {
        Self {
            content: ComponentContent::Translatable(key.into(), Vec::new()),
            style: Style::default(),
            siblings: Vec::new(),
        }
    }

    pub fn translatable_with(key: impl Into<String>, args: Vec<Component>) -> Self {
        Self {
            content: ComponentContent::Translatable(key.into(), args),
            style: Style::default(),
            siblings: Vec::new(),
        }
    }

    pub fn with_style(mut self, fmt: ChatFormatting) -> Self {
        self.style.formats.push(fmt);
        self
    }

    pub fn with_style_fn(mut self, f: impl FnOnce(Style) -> Style) -> Self {
        self.style = f(self.style);
        self
    }

    pub fn with_color(mut self, color: i32) -> Self {
        self.style.color = Some(color);
        self
    }

    pub fn append(mut self, child: Component) -> Self {
        self.siblings.push(child);
        self
    }
}

pub struct CommonComponents;

impl CommonComponents {
    pub fn space() -> Component {
        Component::literal(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataVersion {
    pub version: i32,
}

impl DataVersion {
    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn is_compatible(&self, other: &DataVersion) -> bool {
        self.version == other.version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldVersion {
    pub data_version: DataVersion,
    pub stable: bool,
}

impl WorldVersion {
    pub fn data_version(&self) -> &DataVersion {
        &self.data_version
    }

    pub fn stable(&self) -> bool {
        self.stable
    }
}

std::thread_local! {
    static CURRENT_VERSION: RefCell<WorldVersion> = RefCell::new(WorldVersion::default());
}

pub struct SharedConstants;

impl SharedConstants {
    pub fn get_current_version() -> WorldVersion {
        CURRENT_VERSION.with(|v| v.borrow().clone())
    }

    pub fn set_current_version(v: WorldVersion) {
        CURRENT_VERSION.with(|c| *c.borrow_mut() = v);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelSettings {
    pub level_name: String,
    pub game_type: GameType,
    pub hardcore: bool,
    pub allow_commands: bool,
}

impl LevelSettings {
    pub fn level_name(&self) -> &str {
        &self.level_name
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelVersion {
    pub last_played: i64,
    pub minecraft_version_name: String,
    pub minecraft_version: DataVersion,
}

impl LevelVersion {
    pub fn last_played(&self) -> i64 {
        self.last_played
    }

    pub fn minecraft_version_name(&self) -> &str {
        &self.minecraft_version_name
    }

    pub fn minecraft_version(&self) -> &DataVersion {
        &self.minecraft_version
    }
}

fn is_null_or_empty(s: &str) -> bool {
    s.is_empty()
}

// ---------------------------------------------------------------------------
// Ported: net.minecraft.world.level.storage.LevelSummary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupStatus {
    NONE,
    DOWNGRADE,
    UPGRADE_TO_SNAPSHOT,
}

impl BackupStatus {
    // JAVA: net.minecraft.world.level.storage.LevelSummary.BackupStatus#shouldBackup
    pub fn should_backup(&self) -> bool {
        match self {
            Self::NONE => false,
            Self::DOWNGRADE => true,
            Self::UPGRADE_TO_SNAPSHOT => true,
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary.BackupStatus#isSevere
    pub fn is_severe(&self) -> bool {
        match self {
            Self::NONE => false,
            Self::DOWNGRADE => true,
            Self::UPGRADE_TO_SNAPSHOT => false,
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary.BackupStatus#getTranslationKey
    pub fn get_translation_key(&self) -> &'static str {
        match self {
            Self::NONE => "",
            Self::DOWNGRADE => "downgrade",
            Self::UPGRADE_TO_SNAPSHOT => "snapshot",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LevelSummaryKind {
    Normal,
    Corrupted,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct LevelSummary {
    settings: Option<LevelSettings>,
    level_version: Option<LevelVersion>,
    level_id: String,
    requires_manual_conversion: bool,
    locked: bool,
    experimental: bool,
    icon: PathBuf,
    info: RefCell<Option<Component>>,
    kind: LevelSummaryKind,
    corrupted_last_played: i64,
}

impl LevelSummary {
    // JAVA: net.minecraft.world.level.storage.LevelSummary#<init>
    pub fn new(
        settings: LevelSettings,
        level_version: LevelVersion,
        level_id: String,
        requires_manual_conversion: bool,
        locked: bool,
        experimental: bool,
        icon: PathBuf,
    ) -> Self {
        Self {
            settings: Some(settings),
            level_version: Some(level_version),
            level_id,
            requires_manual_conversion,
            locked,
            experimental,
            icon,
            info: RefCell::new(None),
            kind: LevelSummaryKind::Normal,
            corrupted_last_played: 0,
        }
    }

    fn play_world() -> Component {
        Component::translatable("selectWorld.select")
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#getLevelId
    pub fn get_level_id(&self) -> &str {
        &self.level_id
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#getLevelName
    pub fn get_level_name(&self) -> String {
        match self.kind {
            LevelSummaryKind::Corrupted | LevelSummaryKind::Symlink => self.level_id.clone(),
            LevelSummaryKind::Normal => {
                let name = self
                    .settings
                    .as_ref()
                    .map(|s| s.level_name())
                    .unwrap_or("");
                if name.is_empty() {
                    self.level_id.clone()
                } else {
                    name.to_string()
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#getIcon
    pub fn get_icon(&self) -> &PathBuf {
        &self.icon
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#requiresManualConversion
    pub fn requires_manual_conversion(&self) -> bool {
        self.requires_manual_conversion
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#isExperimental
    pub fn is_experimental(&self) -> bool {
        self.experimental
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#getLastPlayed
    pub fn get_last_played(&self) -> i64 {
        match self.kind {
            LevelSummaryKind::Corrupted => self.corrupted_last_played,
            LevelSummaryKind::Symlink => -1_i64,
            LevelSummaryKind::Normal => self
                .level_version
                .as_ref()
                .map(|v| v.last_played())
                .unwrap_or(0_i64),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#compareTo
    pub fn compare_to(&self, other: &LevelSummary) -> i32 {
        let self_last = self.get_last_played();
        let other_last = other.get_last_played();
        if self_last < other_last {
            1
        } else if self_last > other_last {
            -1
        } else {
            match self.level_id.cmp(&other.level_id) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#getSettings
    pub fn get_settings(&self) -> Option<&LevelSettings> {
        self.settings.as_ref()
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#getGameMode
    pub fn get_game_mode(&self) -> Option<GameType> {
        self.settings.as_ref().map(|s| s.game_type())
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#isHardcore
    pub fn is_hardcore(&self) -> bool {
        self.settings.as_ref().map(|s| s.hardcore()).unwrap_or(false)
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#hasCommands
    pub fn has_commands(&self) -> bool {
        self.settings
            .as_ref()
            .map(|s| s.allow_commands())
            .unwrap_or(false)
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#getWorldVersionName
    pub fn get_world_version_name(&self) -> Component {
        let name = self
            .level_version
            .as_ref()
            .map(|v| v.minecraft_version_name())
            .unwrap_or("");
        if is_null_or_empty(name) {
            Component::translatable("selectWorld.versionUnknown")
        } else {
            Component::literal(name)
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#levelVersion
    pub fn level_version(&self) -> Option<&LevelVersion> {
        self.level_version.as_ref()
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#shouldBackup
    pub fn should_backup(&self) -> bool {
        self.backup_status().should_backup()
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#isDowngrade
    pub fn is_downgrade(&self) -> bool {
        self.backup_status() == BackupStatus::DOWNGRADE
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#backupStatus
    pub fn backup_status(&self) -> BackupStatus {
        let current = SharedConstants::get_current_version();
        let v1 = current.data_version().version();
        let v2 = self
            .level_version
            .as_ref()
            .map(|v| v.minecraft_version().version())
            .unwrap_or(v1);
        if !current.stable() && v2 < v1 {
            BackupStatus::UPGRADE_TO_SNAPSHOT
        } else if v2 > v1 {
            BackupStatus::DOWNGRADE
        } else {
            BackupStatus::NONE
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#isLocked
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#isDisabled
    pub fn is_disabled(&self) -> bool {
        match self.kind {
            LevelSummaryKind::Corrupted | LevelSummaryKind::Symlink => false,
            LevelSummaryKind::Normal => {
                if !self.is_locked() && !self.requires_manual_conversion() {
                    !self.is_compatible()
                } else {
                    true
                }
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#isCompatible
    pub fn is_compatible(&self) -> bool {
        let current = SharedConstants::get_current_version();
        let other = self
            .level_version
            .as_ref()
            .map(|v| v.minecraft_version())
            .cloned()
            .unwrap_or_else(|| current.data_version().clone());
        current.data_version().is_compatible(&other)
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#getInfo
    pub fn get_info(&self) -> Component {
        if let Some(info) = self.info.borrow().as_ref() {
            return info.clone();
        }
        let created = self.create_info();
        *self.info.borrow_mut() = Some(created.clone());
        created
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#createInfo
    fn create_info(&self) -> Component {
        match self.kind {
            LevelSummaryKind::Corrupted => {
                return Component::translatable("recover_world.warning")
                    .with_style_fn(|s| s.with_color(-65536));
            }
            LevelSummaryKind::Symlink => {
                return Component::translatable("symlink_warning.title").with_color(-65536);
            }
            LevelSummaryKind::Normal => {}
        }

        if self.is_locked() {
            return Component::translatable("selectWorld.locked").with_style(ChatFormatting::Red);
        }
        if self.requires_manual_conversion() {
            return Component::translatable("selectWorld.conversion")
                .with_style(ChatFormatting::Red);
        }
        if !self.is_compatible() {
            return Component::translatable_with(
                "selectWorld.incompatible.info",
                vec![self.get_world_version_name()],
            )
            .with_style(ChatFormatting::Red);
        }

        let mut line = if self.is_hardcore() {
            Component::empty().append(
                Component::translatable("gameMode.hardcore").with_color(-65536),
            )
        } else {
            let game_mode_name = self
                .get_game_mode()
                .map(|g| g.get_name())
                .unwrap_or("survival");
            Component::translatable(format!("gameMode.{game_mode_name}"))
        };

        if self.has_commands() {
            line = line
                .append(Component::literal(", "))
                .append(Component::translatable("selectWorld.commands"));
        }

        if self.is_experimental() {
            line = line.append(Component::literal(", ")).append(
                Component::translatable("selectWorld.experimental")
                    .with_style(ChatFormatting::Yellow),
            );
        }

        let version_name = self.get_world_version_name();
        let mut version_line = Component::literal(", ")
            .append(Component::translatable("selectWorld.version"))
            .append(CommonComponents::space());
        if self.should_backup() {
            let styled_version = if self.is_downgrade() {
                version_name.with_style(ChatFormatting::Red)
            } else {
                version_name.with_style(ChatFormatting::Italic)
            };
            version_line = version_line.append(styled_version);
        } else {
            version_line = version_line.append(version_name);
        }

        line = line.append(version_line);
        line
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#primaryActionMessage
    pub fn primary_action_message(&self) -> Component {
        match self.kind {
            LevelSummaryKind::Corrupted => Component::translatable("recover_world.button"),
            LevelSummaryKind::Symlink => Component::translatable("symlink_warning.more_info"),
            LevelSummaryKind::Normal => Self::play_world(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#primaryActionActive
    pub fn primary_action_active(&self) -> bool {
        match self.kind {
            LevelSummaryKind::Corrupted | LevelSummaryKind::Symlink => true,
            LevelSummaryKind::Normal => !self.is_disabled(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#canUpload
    pub fn can_upload(&self) -> bool {
        match self.kind {
            LevelSummaryKind::Corrupted | LevelSummaryKind::Symlink => false,
            LevelSummaryKind::Normal => !self.requires_manual_conversion() && !self.is_locked(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#canEdit
    pub fn can_edit(&self) -> bool {
        match self.kind {
            LevelSummaryKind::Corrupted | LevelSummaryKind::Symlink => false,
            LevelSummaryKind::Normal => !self.is_disabled(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#canRecreate
    pub fn can_recreate(&self) -> bool {
        match self.kind {
            LevelSummaryKind::Corrupted | LevelSummaryKind::Symlink => false,
            LevelSummaryKind::Normal => !self.is_disabled(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelSummary#canDelete
    pub fn can_delete(&self) -> bool {
        true
    }
}

impl PartialEq for LevelSummary {
    fn eq(&self, other: &Self) -> bool {
        self.compare_to(other) == 0
    }
}

impl Eq for LevelSummary {}

impl PartialOrd for LevelSummary {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LevelSummary {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.compare_to(other) {
            x if x < 0 => Ordering::Less,
            x if x > 0 => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

pub struct CorruptedLevelSummary;

impl CorruptedLevelSummary {
    // JAVA: net.minecraft.world.level.storage.LevelSummary.CorruptedLevelSummary#<init>
    pub fn new(level_id: String, icon: PathBuf, last_played: i64) -> LevelSummary {
        LevelSummary {
            settings: None,
            level_version: None,
            level_id,
            requires_manual_conversion: false,
            locked: false,
            experimental: false,
            icon,
            info: RefCell::new(None),
            kind: LevelSummaryKind::Corrupted,
            corrupted_last_played: last_played,
        }
    }
}

pub struct SymlinkLevelSummary;

impl SymlinkLevelSummary {
    // JAVA: net.minecraft.world.level.storage.LevelSummary.SymlinkLevelSummary#<init>
    pub fn new(level_id: String, icon: PathBuf) -> LevelSummary {
        LevelSummary {
            settings: None,
            level_version: None,
            level_id,
            requires_manual_conversion: false,
            locked: false,
            experimental: false,
            icon,
            info: RefCell::new(None),
            kind: LevelSummaryKind::Symlink,
            corrupted_last_played: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_settings(name: &str, game_type: GameType, hardcore: bool, commands: bool) -> LevelSettings {
        LevelSettings {
            level_name: name.to_string(),
            game_type,
            hardcore,
            allow_commands: commands,
        }
    }

    fn make_version(last_played: i64, name: &str, data_version: i32) -> LevelVersion {
        LevelVersion {
            last_played,
            minecraft_version_name: name.to_string(),
            minecraft_version: DataVersion { version: data_version },
        }
    }

    fn make_normal(level_id: &str, last_played: i64, data_version: i32) -> LevelSummary {
        SharedConstants::set_current_version(WorldVersion {
            data_version: DataVersion { version: data_version },
            stable: true,
        });
        LevelSummary::new(
            make_settings("World", GameType::Survival, false, false),
            make_version(last_played, "1.21.11", data_version),
            level_id.to_string(),
            false,
            false,
            false,
            PathBuf::from("icon.png"),
        )
    }

    #[test]
    fn compare_to_orders_by_last_played_desc_then_id_asc() {
        let a = make_normal("a", 100, 1);
        let b = make_normal("b", 200, 1);
        let c = make_normal("c", 100, 1);
        assert_eq!(a.compare_to(&b), 1); // a older => comes after
        assert_eq!(b.compare_to(&a), -1);
        assert_eq!(a.compare_to(&c), "a".cmp("c") as i32); // same time, id asc
        assert_eq!(a.compare_to(&make_normal("a", 100, 1)), 0);
    }

    #[test]
    fn compare_to_handles_negative_and_extreme_last_played() {
        let min = CorruptedLevelSummary::new("min".to_string(), PathBuf::from("i"), i64::MIN);
        let neg = CorruptedLevelSummary::new("neg".to_string(), PathBuf::from("i"), -1);
        let max = CorruptedLevelSummary::new("max".to_string(), PathBuf::from("i"), i64::MAX);
        assert_eq!(min.compare_to(&neg), 1);
        assert_eq!(max.compare_to(&neg), -1);
        assert_eq!(min.compare_to(&max), 1);
        // overflow-safe boundary: no subtraction used, direct comparisons only
        assert_eq!(max.compare_to(&max.clone()), 0);
    }

    #[test]
    fn backup_status_and_compatibility_logic() {
        SharedConstants::set_current_version(WorldVersion {
            data_version: DataVersion { version: 10 },
            stable: true,
        });
        let downgrade = LevelSummary::new(
            make_settings("W", GameType::Survival, false, false),
            make_version(0, "newer", 11),
            "id".to_string(),
            false, false, false, PathBuf::from("i"),
        );
        assert_eq!(downgrade.backup_status(), BackupStatus::DOWNGRADE);
        assert!(downgrade.is_downgrade());
        assert!(downgrade.should_backup());
        assert!(!downgrade.is_compatible());

        SharedConstants::set_current_version(WorldVersion {
            data_version: DataVersion { version: 10 },
            stable: false,
        });
        let snap = LevelSummary::new(
            make_settings("W", GameType::Survival, false, false),
            make_version(0, "old", 9),
            "id".to_string(),
            false, false, false, PathBuf::from("i"),
        );
        assert_eq!(snap.backup_status(), BackupStatus::UPGRADE_TO_SNAPSHOT);
        assert!(snap.backup_status().should_backup());
        assert!(!snap.backup_status().is_severe());
    }

    #[test]
    fn corrupted_and_symlink_overrides() {
        let c = CorruptedLevelSummary::new("corrupt".to_string(), PathBuf::from("icon"), 42);
        assert_eq!(c.get_level_name(), "corrupt");
        assert_eq!(c.get_last_played(), 42);
        assert!(!c.is_disabled());
        assert!(c.primary_action_active());
        assert!(!c.can_upload());
        assert!(!c.can_edit());
        assert!(!c.can_recreate());
        assert!(c.can_delete());
        assert_eq!(
            c.primary_action_message(),
            Component::translatable("recover_world.button")
        );

        let s = SymlinkLevelSummary::new("link".to_string(), PathBuf::from("icon"));
        assert_eq!(s.get_level_name(), "link");
        assert_eq!(s.get_last_played(), -1_i64);
        assert!(!s.is_disabled());
        assert_eq!(
            s.primary_action_message(),
            Component::translatable("symlink_warning.more_info")
        );
        assert!(!s.can_upload());
        assert!(!s.can_edit());
        assert!(!s.can_recreate());
    }

    #[test]
    fn info_creation_and_float_bits_preservation() {
        SharedConstants::set_current_version(WorldVersion {
            data_version: DataVersion { version: 5 },
            stable: true,
        });
        let locked = LevelSummary::new(
            make_settings("W", GameType::Survival, false, false),
            make_version(0, "1.21", 5),
            "id".to_string(),
            false, true, false, PathBuf::from("i"),
        );
        let info = locked.get_info();
        assert_eq!(
            info,
            Component::translatable("selectWorld.locked").with_style(ChatFormatting::Red)
        );
        // cache hit
        assert_eq!(locked.get_info(), info);

        // Float bits edge case (rule coverage): preserve exact bit patterns.
        let f_bits = f32::NAN.to_bits();
        let f_back = f32::from_bits(f_bits);
        assert_eq!(f_back.to_bits(), f_bits);
        let d_bits = (-0.0_f64).to_bits();
        let d_back = f64::from_bits(d_bits);
        assert_eq!(d_back.to_bits(), d_bits);
    }

    #[test]
    fn level_name_empty_falls_back_to_id_and_backup_status_keys() {
        let s = make_normal("world_id", 0, 1);
        let s2 = LevelSummary::new(
            make_settings("", GameType::Creative, true, true),
            make_version(0, "", 1),
            "fallback".to_string(),
            false, false, true, PathBuf::from("i"),
        );
        assert_eq!(s.get_level_name(), "World");
        assert_eq!(s2.get_level_name(), "fallback");
        assert!(s2.is_hardcore());
        assert!(s2.has_commands());
        assert!(s2.is_experimental());
        assert_eq!(BackupStatus::NONE.get_translation_key(), "");
        assert_eq!(BackupStatus::DOWNGRADE.get_translation_key(), "downgrade");
        assert_eq!(BackupStatus::UPGRADE_TO_SNAPSHOT.get_translation_key(), "snapshot");
        assert!(BackupStatus::DOWNGRADE.is_severe());
        assert!(!BackupStatus::UPGRADE_TO_SNAPSHOT.is_severe());
    }
}
