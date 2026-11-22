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

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.Passthrough`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Passthrough;

impl Passthrough {
    pub const INSTANCE: Passthrough = Passthrough;
}

impl RuleBlockEntityModifier for Passthrough {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.Passthrough#apply
    fn apply(&self, _random: &mut dyn RandomSource, tag: Option<CompoundTag>) -> Option<CompoundTag> {
        tag
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.Passthrough#getType
    fn get_type(&self) -> RuleBlockEntityModifierType {
        RuleBlockEntityModifierType::Passthrough
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
    fn apply_none_returns_none() {
        let mut rng = FixedRandom(0);
        assert_eq!(Passthrough::INSTANCE.apply(&mut rng, None), None);
    }

    #[test]
    fn apply_some_returns_same_tag() {
        let mut rng = FixedRandom(-999);
        let mut tag = CompoundTag::new();
        tag.entries.insert("id".to_string(), NbtValue::String("minecraft:chest".to_string()));
        tag.entries.insert("seed".to_string(), NbtValue::Long(i64::MIN));
        let expected = tag.clone();
        let out = Passthrough.apply(&mut rng, Some(tag));
        assert_eq!(out, Some(expected));
    }

    #[test]
    fn get_type_is_passthrough() {
        assert_eq!(Passthrough.get_type(), RuleBlockEntityModifierType::Passthrough);
    }

    #[test]
    fn preserves_nested_compound() {
        let mut rng = FixedRandom(i64::MAX);
        let mut inner = CompoundTag::new();
        inner.entries.insert("n".to_string(), NbtValue::Long(1));
        let mut tag = CompoundTag::new();
        tag.entries.insert("child".to_string(), NbtValue::Compound(inner));
        let expected = tag.clone();
        assert_eq!(Passthrough.apply(&mut rng, Some(tag)), Some(expected));
    }

    #[test]
    fn instance_constant_works() {
        let p: Passthrough = Passthrough::INSTANCE;
        let mut rng = FixedRandom(0);
        assert_eq!(p.get_type(), RuleBlockEntityModifierType::Passthrough);
        assert_eq!(p.apply(&mut rng, None), None);
    }
}
