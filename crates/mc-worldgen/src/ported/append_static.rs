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

    // JAVA: net.minecraft.nbt.CompoundTag#merge
    pub fn merge(&mut self, other: &CompoundTag) -> &mut Self {
        for (key, value) in &other.entries {
            match (self.entries.get_mut(key), value) {
                (Some(NbtValue::Compound(dst)), NbtValue::Compound(src)) => {
                    dst.merge(src);
                }
                _ => {
                    self.entries.insert(key.clone(), value.clone());
                }
            }
        }
        self
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

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendStatic`.
#[derive(Debug, Clone, PartialEq)]
pub struct AppendStatic {
    tag: CompoundTag,
}

impl AppendStatic {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendStatic#<init>
    pub fn new(tag: CompoundTag) -> Self {
        Self { tag }
    }
}

impl RuleBlockEntityModifier for AppendStatic {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendStatic#apply
    fn apply(&self, _random: &mut dyn RandomSource, tag: Option<CompoundTag>) -> Option<CompoundTag> {
        match tag {
            None => Some(self.tag.copy()),
            Some(mut existing) => {
                existing.merge(&self.tag);
                Some(existing)
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendStatic#getType
    fn get_type(&self) -> RuleBlockEntityModifierType {
        RuleBlockEntityModifierType::AppendStatic
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
    fn apply_none_returns_copy_of_template() {
        let mut template = CompoundTag::new();
        template.entries.insert("id".to_string(), NbtValue::String("minecraft:chest".to_string()));
        let modifier = AppendStatic::new(template.clone());
        let mut rng = FixedRandom(0);

        let out = modifier.apply(&mut rng, None).unwrap();
        assert_eq!(out, template);
    }

    #[test]
    fn apply_some_merges_template_into_existing() {
        let mut template = CompoundTag::new();
        template.entries.insert("LootTable".to_string(), NbtValue::String("minecraft:chest".to_string()));

        let mut existing = CompoundTag::new();
        existing.entries.insert("x".to_string(), NbtValue::Long(-1));

        let modifier = AppendStatic::new(template);
        let mut rng = FixedRandom(i64::MIN);
        let out = modifier.apply(&mut rng, Some(existing)).unwrap();

        assert_eq!(out.entries.get("x"), Some(&NbtValue::Long(-1)));
        assert_eq!(
            out.entries.get("LootTable"),
            Some(&NbtValue::String("minecraft:chest".to_string()))
        );
    }

    #[test]
    fn merge_overwrites_non_compound_and_recurses_compounds() {
        let mut template_inner = CompoundTag::new();
        template_inner.entries.insert("b".to_string(), NbtValue::Long(2));
        let mut template = CompoundTag::new();
        template.entries.insert("nested".to_string(), NbtValue::Compound(template_inner));
        template.entries.insert("value".to_string(), NbtValue::Long(i64::MAX));

        let mut existing_inner = CompoundTag::new();
        existing_inner.entries.insert("a".to_string(), NbtValue::Long(1));
        let mut existing = CompoundTag::new();
        existing.entries.insert("nested".to_string(), NbtValue::Compound(existing_inner));
        existing.entries.insert("value".to_string(), NbtValue::Long(0));

        let modifier = AppendStatic::new(template);
        let mut rng = FixedRandom(1);
        let out = modifier.apply(&mut rng, Some(existing)).unwrap();

        assert_eq!(out.entries.get("value"), Some(&NbtValue::Long(i64::MAX)));
        match out.entries.get("nested") {
            Some(NbtValue::Compound(c)) => {
                assert_eq!(c.entries.get("a"), Some(&NbtValue::Long(1)));
                assert_eq!(c.entries.get("b"), Some(&NbtValue::Long(2)));
            }
            other => panic!("expected nested compound, got {:?}", other),
        }
    }

    #[test]
    fn get_type_is_append_static() {
        let modifier = AppendStatic::new(CompoundTag::new());
        assert_eq!(modifier.get_type(), RuleBlockEntityModifierType::AppendStatic);
    }

    #[test]
    fn apply_none_does_not_alias_template() {
        let mut template = CompoundTag::new();
        template.entries.insert("k".to_string(), NbtValue::Long(1));
        let modifier = AppendStatic::new(template);
        let mut rng = FixedRandom(0);

        let mut out = modifier.apply(&mut rng, None).unwrap();
        out.entries.insert("k".to_string(), NbtValue::Long(2));

        let out2 = modifier.apply(&mut rng, None).unwrap();
        assert_eq!(out2.entries.get("k"), Some(&NbtValue::Long(1)));
    }
}
