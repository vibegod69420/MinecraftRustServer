#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureSizeType {
    name: &'static str,
}

impl FeatureSizeType {
    pub const TWO_LAYERS_FEATURE_SIZE: FeatureSizeType =
        Self::register("two_layers_feature_size");
    pub const THREE_LAYERS_FEATURE_SIZE: FeatureSizeType =
        Self::register("three_layers_feature_size");

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSizeType#register
    const fn register(name: &'static str) -> FeatureSizeType {
        // Original: Registry.register(BuiltInRegistries.FEATURE_SIZE_TYPE, name, new FeatureSizeType<>(codec))
        // Registry and codec infrastructure omitted in this self-contained port.
        FeatureSizeType { name }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSizeType#<init>
    const fn new(name: &'static str) -> FeatureSizeType {
        FeatureSizeType { name }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSizeType#codec
    pub fn codec(&self) -> &'static str {
        // Original returns MapCodec<P>; returning type name in this self-contained port.
        self.name
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_have_expected_names() {
        assert_eq!(
            FeatureSizeType::TWO_LAYERS_FEATURE_SIZE.name(),
            "two_layers_feature_size"
        );
        assert_eq!(
            FeatureSizeType::THREE_LAYERS_FEATURE_SIZE.name(),
            "three_layers_feature_size"
        );
    }

    #[test]
    fn codec_returns_name() {
        assert_eq!(
            FeatureSizeType::TWO_LAYERS_FEATURE_SIZE.codec(),
            "two_layers_feature_size"
        );
        assert_eq!(
            FeatureSizeType::THREE_LAYERS_FEATURE_SIZE.codec(),
            "three_layers_feature_size"
        );
    }

    #[test]
    fn register_and_new_create_type() {
        let a = FeatureSizeType::register("custom_a");
        let b = FeatureSizeType::new("custom_b");
        assert_eq!(a.name(), "custom_a");
        assert_eq!(b.name(), "custom_b");
        assert_ne!(a, b);
    }

    #[test]
    fn equality_and_copy_semantics() {
        let a = FeatureSizeType::TWO_LAYERS_FEATURE_SIZE;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, FeatureSizeType::THREE_LAYERS_FEATURE_SIZE);
    }

    #[test]
    fn empty_and_edge_case_names() {
        let empty = FeatureSizeType::new("");
        assert_eq!(empty.name(), "");
        assert_eq!(empty.codec(), "");
        let long = FeatureSizeType::register("very_long_feature_size_type_name_for_edge_case");
        assert_eq!(
            long.name(),
            "very_long_feature_size_type_name_for_edge_case"
        );
    }
}
