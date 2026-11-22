use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

/// Minimal self-contained placeholder for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

/// Minimal self-contained placeholder for `net.minecraft.nbt.CompoundTag`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundTag {
    pub entries: HashMap<String, String>,
}

/// Minimal self-contained placeholder for brigadier `CommandSyntaxException`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

/// Minimal self-contained placeholder for `net.minecraft.commands.CommandSourceStack`.
/// Includes a simplified command storage map to model
/// `source.getServer().getCommandStorage().get(id)`.
#[derive(Debug, Clone, Default)]
pub struct CommandSourceStack {
    pub command_storage: HashMap<Identifier, CompoundTag>,
}

impl CommandSourceStack {
    fn get_storage(&self, id: &Identifier) -> CompoundTag {
        self.command_storage.get(id).cloned().unwrap_or_default()
    }
}

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

/// Minimal self-contained port of `net.minecraft.network.chat.contents.data.DataSource`.
pub trait DataSource {
    // JAVA: net.minecraft.network.chat.contents.data.DataSource#getData
    fn get_data(
        &self,
        source: &CommandSourceStack,
    ) -> Result<Vec<CompoundTag>, CommandSyntaxException>;

    // JAVA: net.minecraft.network.chat.contents.data.DataSource#codec
    fn codec(&self) -> MapCodec;
}

/// Port of `net.minecraft.network.chat.contents.data.StorageDataSource`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageDataSource {
    id: Identifier,
}

pub static MAP_CODEC: LazyLock<MapCodec> = LazyLock::new(|| MapCodec::new("storage"));

impl StorageDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.StorageDataSource#<init>
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.network.chat.contents.data.StorageDataSource#id
    pub fn id(&self) -> &Identifier {
        &self.id
    }
}

impl DataSource for StorageDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.StorageDataSource#getData
    fn get_data(
        &self,
        source: &CommandSourceStack,
    ) -> Result<Vec<CompoundTag>, CommandSyntaxException> {
        let tag = source.get_storage(&self.id);
        Ok(vec![tag])
    }

    // JAVA: net.minecraft.network.chat.contents.data.StorageDataSource#codec
    fn codec(&self) -> MapCodec {
        MAP_CODEC.clone()
    }
}

impl fmt::Display for StorageDataSource {
    // JAVA: net.minecraft.network.chat.contents.data.StorageDataSource#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "storage={}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string_format() {
        let src = StorageDataSource::new(Identifier::new("minecraft", "test"));
        assert_eq!(src.to_string(), "storage=minecraft:test");
    }

    #[test]
    fn get_data_returns_stored_tag() {
        let id = Identifier::new("ns", "path");
        let mut tag = CompoundTag::default();
        tag.entries.insert("k".to_string(), "v".to_string());

        let mut stack = CommandSourceStack::default();
        stack.command_storage.insert(id.clone(), tag.clone());

        let src = StorageDataSource::new(id);
        let data = src.get_data(&stack).unwrap();
        assert_eq!(data, vec![tag]);
    }

    #[test]
    fn get_data_missing_returns_default_tag() {
        let src = StorageDataSource::new(Identifier::new("missing", "id"));
        let data = src.get_data(&CommandSourceStack::default()).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], CompoundTag::default());
    }

    #[test]
    fn codec_has_storage_type_id() {
        let src = StorageDataSource::new(Identifier::new("x", "y"));
        assert_eq!(src.codec().type_id, "storage");
        assert_eq!(MAP_CODEC.type_id, "storage");
    }

    #[test]
    fn id_accessor_returns_identifier() {
        let id = Identifier::new("a", "b");
        let src = StorageDataSource::new(id.clone());
        assert_eq!(src.id(), &id);
    }
}
