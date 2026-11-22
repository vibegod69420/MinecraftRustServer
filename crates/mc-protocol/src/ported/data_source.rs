use std::fmt;

/// Minimal self-contained placeholder for `net.minecraft.commands.CommandSourceStack`.
#[derive(Debug, Clone, Default)]
pub struct CommandSourceStack;

/// Minimal self-contained placeholder for `net.minecraft.nbt.CompoundTag`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundTag;

/// Minimal self-contained placeholder for brigadier `CommandSyntaxException`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

/// Minimal self-contained placeholder for Mojang `MapCodec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapCodec {
    pub type_id: String,
}

impl MapCodec {
    pub fn new(type_id: impl Into<String>) -> Self {
        Self {
            type_id: type_id.into(),
        }
    }
}

/// Port of `net.minecraft.network.chat.contents.data.DataSource`.
pub trait DataSource {
    // JAVA: net.minecraft.network.chat.contents.data.DataSource#getData
    fn get_data(
        &self,
        source: &CommandSourceStack,
    ) -> Result<Vec<CompoundTag>, CommandSyntaxException>;

    // JAVA: net.minecraft.network.chat.contents.data.DataSource#codec
    fn codec(&self) -> MapCodec;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummySource;

    impl DataSource for DummySource {
        fn get_data(
            &self,
            _source: &CommandSourceStack,
        ) -> Result<Vec<CompoundTag>, CommandSyntaxException> {
            Ok(vec![CompoundTag::default()])
        }

        fn codec(&self) -> MapCodec {
            MapCodec::new("dummy")
        }
    }

    #[test]
    fn dummy_source_returns_single_tag() {
        let src = DummySource;
        let data = src.get_data(&CommandSourceStack::default()).unwrap();
        assert_eq!(data, vec![CompoundTag::default()]);
    }

    #[test]
    fn dummy_source_codec_type_id() {
        let src = DummySource;
        assert_eq!(src.codec().type_id, "dummy");
    }

    #[test]
    fn command_syntax_exception_display() {
        let err = CommandSyntaxException {
            message: "syntax error".to_string(),
        };
        assert_eq!(err.to_string(), "syntax error");
    }

    #[test]
    fn map_codec_equality() {
        assert_eq!(MapCodec::new("entity"), MapCodec::new("entity"));
        assert_ne!(MapCodec::new("entity"), MapCodec::new("block"));
    }
}
