use std::fmt;

/// Port of `net.minecraft.util.valueproviders.IntProviderType`.
///
/// The Java interface is registry/codec-backed; this self-contained port
/// preserves the known type identities and register/name behavior.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntProviderType {
    name: &'static str,
}

impl IntProviderType {
    pub const CONSTANT: Self = Self { name: "constant" };
    pub const UNIFORM: Self = Self { name: "uniform" };
    pub const BIASED_TO_BOTTOM: Self = Self { name: "biased_to_bottom" };
    pub const CLAMPED: Self = Self { name: "clamped" };
    pub const WEIGHTED_LIST: Self = Self { name: "weighted_list" };
    pub const CLAMPED_NORMAL: Self = Self { name: "clamped_normal" };

    // JAVA: net.minecraft.util.valueproviders.IntProviderType#codec
    pub fn codec_name(&self) -> &'static str {
        self.name
    }

    // JAVA: net.minecraft.util.valueproviders.IntProviderType#register
    pub fn register(name: &'static str) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for IntProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IntProviderType({})", self.name)
    }
}

impl fmt::Display for IntProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_constants_have_expected_names() {
        assert_eq!(IntProviderType::CONSTANT.name(), "constant");
        assert_eq!(IntProviderType::UNIFORM.name(), "uniform");
        assert_eq!(IntProviderType::BIASED_TO_BOTTOM.name(), "biased_to_bottom");
        assert_eq!(IntProviderType::CLAMPED.name(), "clamped");
        assert_eq!(IntProviderType::WEIGHTED_LIST.name(), "weighted_list");
        assert_eq!(IntProviderType::CLAMPED_NORMAL.name(), "clamped_normal");
    }

    #[test]
    fn codec_name_matches_name() {
        assert_eq!(IntProviderType::CONSTANT.codec_name(), "constant");
        assert_eq!(IntProviderType::WEIGHTED_LIST.codec_name(), "weighted_list");
    }

    #[test]
    fn register_creates_type_with_name() {
        let custom = IntProviderType::register("custom");
        assert_eq!(custom.name(), "custom");
        assert_eq!(custom.codec_name(), "custom");
    }

    #[test]
    fn equality_and_display() {
        let a = IntProviderType::register("uniform");
        assert_eq!(a, IntProviderType::UNIFORM);
        assert_ne!(a, IntProviderType::CLAMPED);
        assert_eq!(a.to_string(), "uniform");
    }
}
