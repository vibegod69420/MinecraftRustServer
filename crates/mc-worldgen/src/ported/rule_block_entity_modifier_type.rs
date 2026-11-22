use std::marker::PhantomData;

/// Marker type for `Clear` in this self-contained module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clear;

/// Marker type for `Passthrough` in this self-contained module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Passthrough;

/// Marker type for `AppendStatic` in this self-contained module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendStatic;

/// Marker type for `AppendLoot` in this self-contained module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendLoot;

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifierType`.
///
/// Java uses Mojang `MapCodec` and registry dispatch; this self-contained port preserves the
/// type identity, registered names, and method structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuleBlockEntityModifierType<P> {
    pub name: &'static str,
    _marker: PhantomData<fn() -> P>,
}

impl<P> RuleBlockEntityModifierType<P> {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifierType#codec
    pub fn codec_name(&self) -> &'static str {
        // In Java this returns a MapCodec<P>. In this self-contained port we expose the
        // registered codec/type name.
        self.name
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifierType#register
    const fn register(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

pub const CLEAR: RuleBlockEntityModifierType<Clear> =
    RuleBlockEntityModifierType::register("clear");
pub const PASSTHROUGH: RuleBlockEntityModifierType<Passthrough> =
    RuleBlockEntityModifierType::register("passthrough");
pub const APPEND_STATIC: RuleBlockEntityModifierType<AppendStatic> =
    RuleBlockEntityModifierType::register("append_static");
pub const APPEND_LOOT: RuleBlockEntityModifierType<AppendLoot> =
    RuleBlockEntityModifierType::register("append_loot");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_names_match_java() {
        assert_eq!(CLEAR.name, "clear");
        assert_eq!(PASSTHROUGH.name, "passthrough");
        assert_eq!(APPEND_STATIC.name, "append_static");
        assert_eq!(APPEND_LOOT.name, "append_loot");
    }

    #[test]
    fn codec_name_returns_registered_name() {
        assert_eq!(CLEAR.codec_name(), "clear");
        assert_eq!(PASSTHROUGH.codec_name(), "passthrough");
        assert_eq!(APPEND_STATIC.codec_name(), "append_static");
        assert_eq!(APPEND_LOOT.codec_name(), "append_loot");
    }

    #[test]
    fn same_type_constants_are_equal() {
        let a = CLEAR;
        let b: RuleBlockEntityModifierType<Clear> = RuleBlockEntityModifierType::register("clear");
        assert_eq!(a, b);
    }

    #[test]
    fn different_names_are_not_equal_for_same_marker() {
        let custom: RuleBlockEntityModifierType<Clear> =
            RuleBlockEntityModifierType::register("custom");
        assert_ne!(CLEAR, custom);
        assert_eq!(custom.codec_name(), "custom");
    }

    #[test]
    fn type_markers_are_distinct_zero_sized_types() {
        assert_eq!(std::mem::size_of::<Clear>(), 0);
        assert_eq!(std::mem::size_of::<Passthrough>(), 0);
        assert_eq!(std::mem::size_of::<AppendStatic>(), 0);
        assert_eq!(std::mem::size_of::<AppendLoot>(), 0);
    }
}
