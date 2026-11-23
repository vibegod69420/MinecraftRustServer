use std::path::PathBuf;

/// Minimal local representation of `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

/// Minimal local representation of `net.minecraft.nbt.Tag`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtTag(pub Vec<u8>);

/// Minimal local representation of `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

/// Minimal local representation of `net.minecraft.server.dialog.Dialog`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dialog;

/// Port of `net.minecraft.network.chat.ClickEvent.Action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    OpenUrl,
    OpenFile,
    RunCommand,
    SuggestCommand,
    ShowDialog,
    ChangePage,
    CopyToClipboard,
    Custom,
}

impl Action {
    // JAVA: net.minecraft.network.chat.ClickEvent.Action#isAllowedFromServer
    pub fn is_allowed_from_server(self) -> bool {
        match self {
            Self::OpenUrl => true,
            Self::OpenFile => false,
            Self::RunCommand => true,
            Self::SuggestCommand => true,
            Self::ShowDialog => true,
            Self::ChangePage => true,
            Self::CopyToClipboard => true,
            Self::Custom => true,
        }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.Action#getSerializedName
    pub fn get_serialized_name(self) -> &'static str {
        match self {
            Self::OpenUrl => "open_url",
            Self::OpenFile => "open_file",
            Self::RunCommand => "run_command",
            Self::SuggestCommand => "suggest_command",
            Self::ShowDialog => "show_dialog",
            Self::ChangePage => "change_page",
            Self::CopyToClipboard => "copy_to_clipboard",
            Self::Custom => "custom",
        }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.Action#filterForSerialization
    pub fn filter_for_serialization(self) -> Result<Self, String> {
        if !self.is_allowed_from_server() {
            let name = match self {
                Self::OpenUrl => "OPEN_URL",
                Self::OpenFile => "OPEN_FILE",
                Self::RunCommand => "RUN_COMMAND",
                Self::SuggestCommand => "SUGGEST_COMMAND",
                Self::ShowDialog => "SHOW_DIALOG",
                Self::ChangePage => "CHANGE_PAGE",
                Self::CopyToClipboard => "COPY_TO_CLIPBOARD",
                Self::Custom => "CUSTOM",
            };
            Err(format!("Click event type not allowed: {name}"))
        } else {
            Ok(self)
        }
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickEvent {
    OpenUrl(OpenUrl),
    OpenFile(OpenFile),
    RunCommand(RunCommand),
    SuggestCommand(SuggestCommand),
    ShowDialog(ShowDialog),
    ChangePage(ChangePage),
    CopyToClipboard(CopyToClipboard),
    Custom(Custom),
}

impl ClickEvent {
    // JAVA: net.minecraft.network.chat.ClickEvent#action
    pub fn action(&self) -> Action {
        match self {
            Self::OpenUrl(_) => Action::OpenUrl,
            Self::OpenFile(_) => Action::OpenFile,
            Self::RunCommand(_) => Action::RunCommand,
            Self::SuggestCommand(_) => Action::SuggestCommand,
            Self::ShowDialog(_) => Action::ShowDialog,
            Self::ChangePage(_) => Action::ChangePage,
            Self::CopyToClipboard(_) => Action::CopyToClipboard,
            Self::Custom(_) => Action::Custom,
        }
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent$ChangePage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangePage {
    pub page: i32,
}

impl ChangePage {
    // JAVA: net.minecraft.network.chat.ClickEvent.ChangePage#<init>
    pub fn new(page: i32) -> Self {
        Self { page }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.ChangePage#page
    pub fn page(&self) -> i32 {
        self.page
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.ChangePage#action
    pub fn action(&self) -> Action {
        Action::ChangePage
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent$CopyToClipboard`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyToClipboard {
    pub value: String,
}

impl CopyToClipboard {
    // JAVA: net.minecraft.network.chat.ClickEvent.CopyToClipboard#<init>
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.CopyToClipboard#value
    pub fn value(&self) -> &str {
        &self.value
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.CopyToClipboard#action
    pub fn action(&self) -> Action {
        Action::CopyToClipboard
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent$Custom`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Custom {
    pub id: Identifier,
    pub payload: Option<NbtTag>,
}

impl Custom {
    // JAVA: net.minecraft.network.chat.ClickEvent.Custom#<init>
    pub fn new(id: Identifier, payload: Option<NbtTag>) -> Self {
        Self { id, payload }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.Custom#id
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.Custom#payload
    pub fn payload(&self) -> Option<&NbtTag> {
        self.payload.as_ref()
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.Custom#action
    pub fn action(&self) -> Action {
        Action::Custom
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent$OpenFile`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenFile {
    pub path: String,
}

impl OpenFile {
    // JAVA: net.minecraft.network.chat.ClickEvent.OpenFile#<init>
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.OpenFile#<init>
    pub fn from_file(file: &std::path::Path) -> Self {
        Self {
            path: file.to_string_lossy().into_owned(),
        }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.OpenFile#<init>
    pub fn from_path(path: &std::path::Path) -> Self {
        Self {
            path: path.to_string_lossy().into_owned(),
        }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.OpenFile#file
    pub fn file(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.OpenFile#path
    pub fn path(&self) -> &str {
        &self.path
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.OpenFile#action
    pub fn action(&self) -> Action {
        Action::OpenFile
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent$OpenUrl`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenUrl {
    pub uri: String,
}

impl OpenUrl {
    // JAVA: net.minecraft.network.chat.ClickEvent.OpenUrl#<init>
    pub fn new(uri: impl Into<String>) -> Self {
        Self { uri: uri.into() }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.OpenUrl#uri
    pub fn uri(&self) -> &str {
        &self.uri
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.OpenUrl#action
    pub fn action(&self) -> Action {
        Action::OpenUrl
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent$RunCommand`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommand {
    pub command: String,
}

impl RunCommand {
    // JAVA: net.minecraft.network.chat.ClickEvent.RunCommand#<init>
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.RunCommand#command
    pub fn command(&self) -> &str {
        &self.command
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.RunCommand#action
    pub fn action(&self) -> Action {
        Action::RunCommand
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent$ShowDialog`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowDialog {
    pub dialog: Holder<Dialog>,
}

impl ShowDialog {
    // JAVA: net.minecraft.network.chat.ClickEvent.ShowDialog#<init>
    pub fn new(dialog: Holder<Dialog>) -> Self {
        Self { dialog }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.ShowDialog#dialog
    pub fn dialog(&self) -> &Holder<Dialog> {
        &self.dialog
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.ShowDialog#action
    pub fn action(&self) -> Action {
        Action::ShowDialog
    }
}

/// Port of `net.minecraft.network.chat.ClickEvent$SuggestCommand`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestCommand {
    pub command: String,
}

impl SuggestCommand {
    // JAVA: net.minecraft.network.chat.ClickEvent.SuggestCommand#<init>
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.SuggestCommand#command
    pub fn command(&self) -> &str {
        &self.command
    }

    // JAVA: net.minecraft.network.chat.ClickEvent.SuggestCommand#action
    pub fn action(&self) -> Action {
        Action::SuggestCommand
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn action_mapping_and_serialized_names() {
        let cases = [
            (Action::OpenUrl, "open_url", true),
            (Action::OpenFile, "open_file", false),
            (Action::RunCommand, "run_command", true),
            (Action::SuggestCommand, "suggest_command", true),
            (Action::ShowDialog, "show_dialog", true),
            (Action::ChangePage, "change_page", true),
            (Action::CopyToClipboard, "copy_to_clipboard", true),
            (Action::Custom, "custom", true),
        ];

        for (action, name, allowed) in cases {
            assert_eq!(action.get_serialized_name(), name);
            assert_eq!(action.is_allowed_from_server(), allowed);
        }

        let event = ClickEvent::RunCommand(RunCommand::new("/say hi"));
        assert_eq!(event.action(), Action::RunCommand);
    }

    #[test]
    fn filter_for_serialization_rejects_open_file() {
        assert_eq!(
            Action::OpenFile.filter_for_serialization(),
            Err("Click event type not allowed: OPEN_FILE".to_string())
        );
        assert_eq!(
            Action::OpenUrl.filter_for_serialization(),
            Ok(Action::OpenUrl)
        );
        assert_eq!(
            Action::Custom.filter_for_serialization(),
            Ok(Action::Custom)
        );
    }

    #[test]
    fn change_page_negative_and_extreme_values() {
        let neg = ChangePage::new(-1);
        assert_eq!(neg.page(), -1);
        assert_eq!(neg.action(), Action::ChangePage);

        let min = ChangePage::new(i32::MIN);
        assert_eq!(min.page(), i32::MIN);

        let max = ChangePage::new(i32::MAX);
        assert_eq!(max.page(), i32::MAX);

        let wrapped = i32::MAX.wrapping_add(1);
        let overflow_page = ChangePage::new(wrapped);
        assert_eq!(overflow_page.page(), i32::MIN);
    }

    #[test]
    fn open_file_path_conversions() {
        let path = Path::new("logs/latest.log");
        let from_path = OpenFile::from_path(path);
        let from_file = OpenFile::from_file(path);
        assert_eq!(from_path.path(), "logs/latest.log");
        assert_eq!(from_file.file(), PathBuf::from("logs/latest.log"));
        assert_eq!(from_path.action(), Action::OpenFile);
        assert_eq!(from_path, from_file);
    }

    #[test]
    fn custom_payload_optional_and_float_bits() {
        let no_payload = Custom::new(Identifier("minecraft:test".to_string()), None);
        assert_eq!(no_payload.id(), &Identifier("minecraft:test".to_string()));
        assert_eq!(no_payload.payload(), None);
        assert_eq!(no_payload.action(), Action::Custom);

        let bits = f32::NAN.to_bits().to_le_bytes();
        let tag = NbtTag(bits.to_vec());
        let event = ClickEvent::Custom(Custom::new(
            Identifier("minecraft:float".to_string()),
            Some(tag.clone()),
        ));
        match event {
            ClickEvent::Custom(custom) => {
                let payload = custom.payload().expect("payload present");
                let got_bits = u32::from_le_bytes(payload.0[..4].try_into().expect("4 bytes"));
                let got = f32::from_bits(got_bits);
                assert!(got.is_nan());
                assert_eq!(got.to_bits(), f32::NAN.to_bits());
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn click_event_enum_dispatches_actions() {
        let events = vec![
            ClickEvent::OpenUrl(OpenUrl::new("https://example.com")),
            ClickEvent::OpenFile(OpenFile::new("file.txt")),
            ClickEvent::RunCommand(RunCommand::new("/help")),
            ClickEvent::SuggestCommand(SuggestCommand::new("/msg ")),
            ClickEvent::ShowDialog(ShowDialog::new(Holder(Dialog))),
            ClickEvent::ChangePage(ChangePage::new(2)),
            ClickEvent::CopyToClipboard(CopyToClipboard::new("copy")),
            ClickEvent::Custom(Custom::new(Identifier("ns:id".to_string()), None)),
        ];
        let actions: Vec<Action> = events.iter().map(ClickEvent::action).collect();
        assert_eq!(
            actions,
            vec![
                Action::OpenUrl,
                Action::OpenFile,
                Action::RunCommand,
                Action::SuggestCommand,
                Action::ShowDialog,
                Action::ChangePage,
                Action::CopyToClipboard,
                Action::Custom,
            ]
        );
    }
}
