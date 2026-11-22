use std::marker::PhantomData;

/// Minimal self-contained placeholder for Mojang Codec.
#[derive(Debug, Clone, Copy)]
pub struct Codec<T>(PhantomData<T>);

/// Minimal self-contained placeholder for StringRepresentable.
pub trait StringRepresentable {
    fn get_serialized_name(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiquidSettings {
    IgnoreWaterlogging,
    ApplyWaterlogging,
}

impl LiquidSettings {
    pub const CODEC: Codec<LiquidSettings> = Codec(PhantomData);

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.LiquidSettings#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Self::IgnoreWaterlogging => "ignore_waterlogging",
            Self::ApplyWaterlogging => "apply_waterlogging",
        }
    }

    pub fn values() -> &'static [LiquidSettings] {
        &[Self::IgnoreWaterlogging, Self::ApplyWaterlogging]
    }
}

impl StringRepresentable for LiquidSettings {
    fn get_serialized_name(&self) -> &str {
        LiquidSettings::get_serialized_name(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_names_match_java() {
        assert_eq!(
            LiquidSettings::IgnoreWaterlogging.get_serialized_name(),
            "ignore_waterlogging"
        );
        assert_eq!(
            LiquidSettings::ApplyWaterlogging.get_serialized_name(),
            "apply_waterlogging"
        );
    }

    #[test]
    fn values_order_and_count() {
        let vals = LiquidSettings::values();
        assert_eq!(vals.len(), 2);
        assert_eq!(vals[0], LiquidSettings::IgnoreWaterlogging);
        assert_eq!(vals[1], LiquidSettings::ApplyWaterlogging);
    }

    #[test]
    fn string_representable_trait_dispatch() {
        let s: &dyn StringRepresentable = &LiquidSettings::ApplyWaterlogging;
        assert_eq!(s.get_serialized_name(), "apply_waterlogging");
    }

    #[test]
    fn equality_copy_and_hash_distinctness() {
        use std::collections::HashSet;
        let a = LiquidSettings::IgnoreWaterlogging;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, LiquidSettings::ApplyWaterlogging);
        let mut set = HashSet::new();
        set.insert(LiquidSettings::IgnoreWaterlogging);
        set.insert(LiquidSettings::ApplyWaterlogging);
        set.insert(LiquidSettings::IgnoreWaterlogging);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn codec_placeholder_exists() {
        let _codec: Codec<LiquidSettings> = LiquidSettings::CODEC;
    }
}
