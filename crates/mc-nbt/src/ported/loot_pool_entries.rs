#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootPoolEntryType {
    pub name: Identifier,
    pub codec_id: String,
}

pub struct LootPoolEntries;

impl LootPoolEntries {
    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#register
    fn register(name: &str, codec_id: &str) -> LootPoolEntryType {
        LootPoolEntryType {
            name: Identifier::with_default_namespace(name),
            codec_id: codec_id.to_string(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#EMPTY
    pub fn empty() -> LootPoolEntryType {
        Self::register("empty", "EmptyLootItem.CODEC")
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#ITEM
    pub fn item() -> LootPoolEntryType {
        Self::register("item", "LootItem.CODEC")
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#LOOT_TABLE
    pub fn loot_table() -> LootPoolEntryType {
        Self::register("loot_table", "NestedLootTable.CODEC")
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#DYNAMIC
    pub fn dynamic() -> LootPoolEntryType {
        Self::register("dynamic", "DynamicLoot.CODEC")
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#TAG
    pub fn tag() -> LootPoolEntryType {
        Self::register("tag", "TagEntry.CODEC")
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#SLOTS
    pub fn slots() -> LootPoolEntryType {
        Self::register("slots", "SlotLoot.CODEC")
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#ALTERNATIVES
    pub fn alternatives() -> LootPoolEntryType {
        Self::register("alternatives", "AlternativesEntry.CODEC")
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#SEQUENCE
    pub fn sequence() -> LootPoolEntryType {
        Self::register("sequence", "SequentialEntry.CODEC")
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.LootPoolEntries#GROUP
    pub fn group() -> LootPoolEntryType {
        Self::register("group", "EntryGroup.CODEC")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_uses_default_namespace() {
        let entry = LootPoolEntries::empty();
        assert_eq!(entry.name.namespace, "minecraft");
        assert_eq!(entry.name.path, "empty");
        assert_eq!(entry.codec_id, "EmptyLootItem.CODEC");
    }

    #[test]
    fn all_known_entry_types_have_expected_names() {
        let entries = vec![
            (LootPoolEntries::empty(), "empty", "EmptyLootItem.CODEC"),
            (LootPoolEntries::item(), "item", "LootItem.CODEC"),
            (
                LootPoolEntries::loot_table(),
                "loot_table",
                "NestedLootTable.CODEC",
            ),
            (LootPoolEntries::dynamic(), "dynamic", "DynamicLoot.CODEC"),
            (LootPoolEntries::tag(), "tag", "TagEntry.CODEC"),
            (LootPoolEntries::slots(), "slots", "SlotLoot.CODEC"),
            (
                LootPoolEntries::alternatives(),
                "alternatives",
                "AlternativesEntry.CODEC",
            ),
            (
                LootPoolEntries::sequence(),
                "sequence",
                "SequentialEntry.CODEC",
            ),
            (LootPoolEntries::group(), "group", "EntryGroup.CODEC"),
        ];
        for (entry, path, codec) in entries {
            assert_eq!(entry.name.namespace, "minecraft");
            assert_eq!(entry.name.path, path);
            assert_eq!(entry.codec_id, codec);
        }
    }

    #[test]
    fn identifier_with_default_namespace_preserves_path() {
        let id = Identifier::with_default_namespace("custom_path");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "custom_path");
    }

    #[test]
    fn different_entry_types_are_distinct() {
        assert_ne!(LootPoolEntries::item(), LootPoolEntries::dynamic());
        assert_ne!(LootPoolEntries::slots(), LootPoolEntries::group());
    }

    #[test]
    fn same_entry_type_factory_returns_equal_value() {
        assert_eq!(LootPoolEntries::sequence(), LootPoolEntries::sequence());
    }
}
