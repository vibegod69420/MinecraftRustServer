use std::fmt;

/// Port of `net.minecraft.util.valueproviders.FloatProviderType`.
///
/// The Java interface is registry/codec-backed; this self-contained port
/// preserves the known type identities and register/name behavior.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FloatProviderType {
    name: &'static str,
}

impl FloatProviderType {
    pub const CONSTANT: Self = Self { name: "constant" };
    pub const UNIFORM: Self = Self { name: "uniform" };
    pub const CLAMPED_NORMAL: Self = Self { name: "clamped_normal" };
    pub const TRAPEZOID: Self = Self { name: "trapezoid" };

    // JAVA: net.minecraft.util.valueproviders.FloatProviderType#codec
    pub fn codec_name(&self) -> &'static str {
        self.name
    }

    // JAVA: net.minecraft.util.valueproviders.FloatProviderType#register
    pub fn register(name: &'static str) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for FloatProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FloatProviderType({})", self.name)
    }
}

impl fmt::Display for FloatProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_constants_have_expected_names() {
        assert_eq!(FloatProviderType::CONSTANT.name(), "constant");
        assert_eq!(FloatProviderType::UNIFORM.name(), "uniform");
        assert_eq!(FloatProviderType::CLAMPED_NORMAL.name(), "clamped_normal");
        assert_eq!(FloatProviderType::TRAPEZOID.name(), "trapezoid");
    }

    #[test]
    fn codec_name_matches_name() {
        assert_eq!(
            FloatProviderType::CONSTANT.codec_name(),
            FloatProviderType::CONSTANT.name()
        );
        assert_eq!(
            FloatProviderType::TRAPEZOID.codec_name(),
            "trapezoid"
        );
    }

    #[test]
    fn register_creates_type_with_name() {
        let custom = FloatProviderType::register("custom");
        assert_eq!(custom.name(), "custom");
        assert_eq!(custom.codec_name(), "custom");
    }

    #[test]
    fn equality_and_display() {
        let a = FloatProviderType::register("constant");
        assert_eq!(a, FloatProviderType::CONSTANT);
        assert_ne!(a, FloatProviderType::UNIFORM);
        assert_eq!(a.to_string(), "constant");
    }
}
