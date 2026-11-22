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

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.Clear`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Clear;

impl Clear {
    pub const INSTANCE: Clear = Clear;
}

impl RuleBlockEntityModifier for Clear {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.Clear#apply
    fn apply(&self, _random: &mut dyn RandomSource, _tag: Option<CompoundTag>) -> Option<CompoundTag> {
        Some(CompoundTag::new())
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.Clear#getType
    fn get_type(&self) -> RuleBlockEntityModifierType {
        RuleBlockEntityModifierType::Clear
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
    fn apply_none_returns_empty_tag() {
        let mut rng = FixedRandom(0);
        let out = Clear::INSTANCE.apply(&mut rng, None).unwrap();
        assert!(out.entries.is_empty());
    }

    #[test]
    fn apply_some_returns_empty_tag_clearing_input() {
        let mut rng = FixedRandom(123);
        let mut tag = CompoundTag::new();
        tag.entries.insert("LootTable".to_string(), NbtValue::String("minecraft:chest".to_string()));
        tag.entries.insert("LootTableSeed".to_string(), NbtValue::Long(-1));
        let out = Clear.apply(&mut rng, Some(tag)).unwrap();
        assert!(out.entries.is_empty());
    }

    #[test]
    fn get_type_is_clear() {
        assert_eq!(Clear.get_type(), RuleBlockEntityModifierType::Clear);
    }

    #[test]
    fn random_is_not_consumed_semantically() {
        // Clear does not use the random source; any value should behave identically.
        let mut rng_min = FixedRandom(i64::MIN);
        let mut rng_max = FixedRandom(i64::MAX);
        let a = Clear.apply(&mut rng_min, None);
        let b = Clear.apply(&mut rng_max, None);
        assert_eq!(a, b);
    }

    #[test]
    fn instance_is_usable_as_constant() {
        let c: Clear = Clear::INSTANCE;
        let mut rng = FixedRandom(7);
        assert_eq!(c.get_type(), RuleBlockEntityModifierType::Clear);
        assert!(c.apply(&mut rng, None).unwrap().entries.is_empty());
    }
}
