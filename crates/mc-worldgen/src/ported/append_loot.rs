use std::collections::BTreeMap;

/// Minimal self-contained NBT value representation.
#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    Long(i64),
    String(String),
    Compound(CompoundTag),
}

/// Minimal self-contained NBT compound tag representation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompoundTag {
    pub entries: BTreeMap<String, NbtValue>,
}

impl CompoundTag {
    // JAVA: net.minecraft.nbt.CompoundTag#<init>
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.nbt.CompoundTag#copy
    pub fn copy(&self) -> Self {
        self.clone()
    }

    // JAVA: net.minecraft.nbt.CompoundTag#putLong
    pub fn put_long(&mut self, key: &str, value: i64) {
        self.entries.insert(key.to_string(), NbtValue::Long(value));
    }

    /// Minimal representation of Java `store(key, codec, value)` for a loot table resource key.
    pub fn store_resource_key(&mut self, key: &str, value: &str) {
        self.entries
            .insert(key.to_string(), NbtValue::String(value.to_string()));
    }
}

/// Minimal self-contained random source.
pub trait RandomSource {
    // JAVA: net.minecraft.util.RandomSource#nextLong
    fn next_long(&mut self) -> i64;
}

/// Identifies the type of a rule block entity modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleBlockEntityModifierType {
    Clear,
    Passthrough,
    AppendStatic,
    AppendLoot,
}

/// Minimal trait for self-contained compilation.
pub trait RuleBlockEntityModifier {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifier#apply
    fn apply(&self, random: &mut dyn RandomSource, tag: Option<CompoundTag>) -> Option<CompoundTag>;

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifier#getType
    fn get_type(&self) -> RuleBlockEntityModifierType;
}

/// Minimal self-contained representation of a loot table resource key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub location: String,
}

impl ResourceKey {
    // JAVA: net.minecraft.resources.ResourceKey#<init>
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
        }
    }
}

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendLoot`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendLoot {
    loot_table: ResourceKey,
}

impl AppendLoot {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendLoot#<init>
    pub fn new(loot_table: ResourceKey) -> Self {
        Self { loot_table }
    }
}

impl RuleBlockEntityModifier for AppendLoot {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendLoot#apply
    fn apply(&self, random: &mut dyn RandomSource, tag: Option<CompoundTag>) -> Option<CompoundTag> {
        let mut result = match tag {
            None => CompoundTag::new(),
            Some(existing) => existing.copy(),
        };
        result.store_resource_key("LootTable", &self.loot_table.location);
        result.put_long("LootTableSeed", random.next_long());
        Some(result)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendLoot#getType
    fn get_type(&self) -> RuleBlockEntityModifierType {
        RuleBlockEntityModifierType::AppendLoot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(i64);

    impl RandomSource for FixedRandom {
        fn next_long(&mut self) -> i64 {
            self.0
        }
    }

    #[test]
    fn apply_none_creates_tag_with_loot_table_and_seed() {
        let modifier = AppendLoot::new(ResourceKey::new("minecraft:chests/simple_dungeon"));
        let mut rng = FixedRandom(12345);
        let out = modifier.apply(&mut rng, None).unwrap();

        assert_eq!(
            out.entries.get("LootTable"),
            Some(&NbtValue::String("minecraft:chests/simple_dungeon".to_string()))
        );
        assert_eq!(out.entries.get("LootTableSeed"), Some(&NbtValue::Long(12345)));
    }

    #[test]
    fn apply_some_copies_existing_and_sets_loot_fields() {
        let mut existing = CompoundTag::new();
        existing.entries.insert("id".to_string(), NbtValue::String("minecraft:chest".to_string()));
        existing.entries.insert("LootTableSeed".to_string(), NbtValue::Long(0));

        let modifier = AppendLoot::new(ResourceKey::new("minecraft:chests/village"));
        let mut rng = FixedRandom(i64::MIN);
        let out = modifier.apply(&mut rng, Some(existing.clone())).unwrap();

        assert_eq!(out.entries.get("id"), Some(&NbtValue::String("minecraft:chest".to_string())));
        assert_eq!(
            out.entries.get("LootTable"),
            Some(&NbtValue::String("minecraft:chests/village".to_string()))
        );
        assert_eq!(out.entries.get("LootTableSeed"), Some(&NbtValue::Long(i64::MIN)));
        // Original input should remain unchanged (apply used copy).
        assert_eq!(existing.entries.get("LootTableSeed"), Some(&NbtValue::Long(0)));
        assert!(!existing.entries.contains_key("LootTable"));
    }

    #[test]
    fn negative_and_max_seeds_are_preserved() {
        let modifier = AppendLoot::new(ResourceKey::new("m:t"));
        let mut rng_neg = FixedRandom(-1);
        let out_neg = modifier.apply(&mut rng_neg, None).unwrap();
        assert_eq!(out_neg.entries.get("LootTableSeed"), Some(&NbtValue::Long(-1)));

        let mut rng_max = FixedRandom(i64::MAX);
        let out_max = modifier.apply(&mut rng_max, None).unwrap();
        assert_eq!(out_max.entries.get("LootTableSeed"), Some(&NbtValue::Long(i64::MAX)));
    }

    #[test]
    fn get_type_is_append_loot() {
        let modifier = AppendLoot::new(ResourceKey::new("m:t"));
        assert_eq!(modifier.get_type(), RuleBlockEntityModifierType::AppendLoot);
    }

    #[test]
    fn overwrites_existing_loot_table_string() {
        let mut existing = CompoundTag::new();
        existing.entries.insert("LootTable".to_string(), NbtValue::String("old".to_string()));
        let modifier = AppendLoot::new(ResourceKey::new("new"));
        let mut rng = FixedRandom(7);
        let out = modifier.apply(&mut rng, Some(existing)).unwrap();
        assert_eq!(out.entries.get("LootTable"), Some(&NbtValue::String("new".to_string())));
    }
}
