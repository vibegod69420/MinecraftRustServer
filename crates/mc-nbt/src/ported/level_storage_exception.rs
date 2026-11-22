use std::error::Error;
use std::fmt;

/// Minimal self-contained stub for `net.minecraft.network.chat.Component`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    text: String,
}

impl Component {
    /// Creates a simple literal text component stub.
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Stub for `net.minecraft.network.chat.Component#getString`.
    pub fn get_string(&self) -> &str {
        &self.text
    }
}

/// Port of `net.minecraft.world.level.storage.LevelStorageException`.
#[derive(Debug)]
pub struct LevelStorageException {
    message_component: Component,
    message: String,
}

impl LevelStorageException {
    // JAVA: net.minecraft.world.level.storage.LevelStorageException#<init>
    pub fn new(message_component: Component) -> Self {
        let message = message_component.get_string().to_owned();
        Self {
            message_component,
            message,
        }
    }

    // JAVA: net.minecraft.world.level.storage.LevelStorageException#getMessageComponent
    pub fn get_message_component(&self) -> &Component {
        &self.message_component
    }
}

impl fmt::Display for LevelStorageException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for LevelStorageException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_captures_message_and_component() {
        let component = Component::literal("disk full");
        let err = LevelStorageException::new(component.clone());
        assert_eq!(err.get_message_component(), &component);
        assert_eq!(err.to_string(), "disk full");
    }

    #[test]
    fn empty_message_preserved() {
        let err = LevelStorageException::new(Component::literal(""));
        assert_eq!(err.to_string(), "");
        assert_eq!(err.get_message_component().get_string(), "");
    }

    #[test]
    fn unicode_and_negative_numeric_text_preserved() {
        let msg = "Ошибка -1 уровень";
        let err = LevelStorageException::new(Component::literal(msg));
        assert_eq!(err.to_string(), msg);
        assert!(err.to_string().contains("-1"));
    }

    #[test]
    fn error_trait_has_no_source() {
        let err = LevelStorageException::new(Component::literal("x"));
        let dyn_err: &dyn Error = &err;
        assert!(dyn_err.source().is_none());
    }

    #[test]
    fn float_bits_and_overflow_sanity() {
        let nan_a = f32::NAN.to_bits();
        let nan_b = f32::from_bits(nan_a).to_bits();
        assert_eq!(nan_a, nan_b);
        let wrapped = i64::MAX.wrapping_add(1);
        assert_eq!(wrapped, i64::MIN);
    }
}
