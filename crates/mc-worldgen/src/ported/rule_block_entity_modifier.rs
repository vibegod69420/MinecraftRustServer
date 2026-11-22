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

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifier`.
///
/// The Java `CODEC` field is registry/codec infrastructure and is not represented in this
/// self-contained port.
pub trait RuleBlockEntityModifier {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifier#apply
    fn apply(&self, random: &mut dyn RandomSource, tag: Option<CompoundTag>) -> Option<CompoundTag>;

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifier#getType
    fn get_type(&self) -> RuleBlockEntityModifierType;
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

    struct PassthroughMock;

    impl RuleBlockEntityModifier for PassthroughMock {
        fn apply(
            &self,
            _random: &mut dyn RandomSource,
            tag: Option<CompoundTag>,
        ) -> Option<CompoundTag> {
            tag
        }

        fn get_type(&self) -> RuleBlockEntityModifierType {
            RuleBlockEntityModifierType::Passthrough
        }
    }

    struct ClearMock;

    impl RuleBlockEntityModifier for ClearMock {
        fn apply(
            &self,
            _random: &mut dyn RandomSource,
            _tag: Option<CompoundTag>,
        ) -> Option<CompoundTag> {
            Some(CompoundTag::new())
        }

        fn get_type(&self) -> RuleBlockEntityModifierType {
            RuleBlockEntityModifierType::Clear
        }
    }

    #[test]
    fn mock_passthrough_returns_none() {
        let m = PassthroughMock;
        let mut rng = FixedRandom(42);
        assert_eq!(m.apply(&mut rng, None), None);
        assert_eq!(m.get_type(), RuleBlockEntityModifierType::Passthrough);
    }

    #[test]
    fn mock_passthrough_returns_some() {
        let m = PassthroughMock;
        let mut rng = FixedRandom(i64::MIN);
        let mut tag = CompoundTag::new();
        tag.entries
            .insert("k".to_string(), NbtValue::Long(-1));
        let out = m.apply(&mut rng, Some(tag.clone()));
        assert_eq!(out, Some(tag));
    }

    #[test]
    fn mock_clear_returns_empty_tag() {
        let m = ClearMock;
        let mut rng = FixedRandom(i64::MAX);
        let mut tag = CompoundTag::new();
        tag.entries
            .insert("x".to_string(), NbtValue::String("y".to_string()));
        let out = m.apply(&mut rng, Some(tag)).unwrap();
        assert!(out.entries.is_empty());
        assert_eq!(m.get_type(), RuleBlockEntityModifierType::Clear);
    }

    #[test]
    fn compound_tag_new_is_empty() {
        let tag = CompoundTag::new();
        assert!(tag.entries.is_empty());
        assert_eq!(tag, CompoundTag::default());
    }
}
