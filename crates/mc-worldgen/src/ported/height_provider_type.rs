//! Port of `net.minecraft.world.level.levelgen.heightproviders.HeightProviderType`.
//!
//! Registry and codec infrastructure are represented as self-contained placeholders.

/// Placeholder for Mojang serialization `MapCodec<P>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MapCodec;

/// Port of `net.minecraft.world.level.levelgen.heightproviders.HeightProviderType`.
///
/// The Java type is generic (`HeightProviderType<P extends HeightProvider>`).
/// This self-contained port models the known type constants as an enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightProviderType {
    Constant,
    Uniform,
    BiasedToBottom,
    VeryBiasedToBottom,
    Trapezoid,
    WeightedList,
}

impl HeightProviderType {
    pub const CONSTANT: Self = Self::Constant;
    pub const UNIFORM: Self = Self::Uniform;
    pub const BIASED_TO_BOTTOM: Self = Self::BiasedToBottom;
    pub const VERY_BIASED_TO_BOTTOM: Self = Self::VeryBiasedToBottom;
    pub const TRAPEZOID: Self = Self::Trapezoid;
    pub const WEIGHTED_LIST: Self = Self::WeightedList;

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.HeightProviderType#codec
    pub fn codec(self) -> MapCodec {
        MapCodec
    }

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.HeightProviderType#register
    fn register(name: &str, _codec: MapCodec) -> Self {
        match name {
            "constant" => Self::Constant,
            "uniform" => Self::Uniform,
            "biased_to_bottom" => Self::BiasedToBottom,
            "very_biased_to_bottom" => Self::VeryBiasedToBottom,
            "trapezoid" => Self::Trapezoid,
            "weighted_list" => Self::WeightedList,
            _ => Self::Constant,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Constant => "constant",
            Self::Uniform => "uniform",
            Self::BiasedToBottom => "biased_to_bottom",
            Self::VeryBiasedToBottom => "very_biased_to_bottom",
            Self::Trapezoid => "trapezoid",
            Self::WeightedList => "weighted_list",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_variants() {
        assert_eq!(HeightProviderType::CONSTANT, HeightProviderType::Constant);
        assert_eq!(HeightProviderType::UNIFORM, HeightProviderType::Uniform);
        assert_eq!(
            HeightProviderType::BIASED_TO_BOTTOM,
            HeightProviderType::BiasedToBottom
        );
        assert_eq!(
            HeightProviderType::VERY_BIASED_TO_BOTTOM,
            HeightProviderType::VeryBiasedToBottom
        );
        assert_eq!(HeightProviderType::TRAPEZOID, HeightProviderType::Trapezoid);
        assert_eq!(
            HeightProviderType::WEIGHTED_LIST,
            HeightProviderType::WeightedList
        );
    }

    #[test]
    fn register_known_names() {
        assert_eq!(
            HeightProviderType::register("constant", MapCodec),
            HeightProviderType::Constant
        );
        assert_eq!(
            HeightProviderType::register("uniform", MapCodec),
            HeightProviderType::Uniform
        );
        assert_eq!(
            HeightProviderType::register("biased_to_bottom", MapCodec),
            HeightProviderType::BiasedToBottom
        );
        assert_eq!(
            HeightProviderType::register("very_biased_to_bottom", MapCodec),
            HeightProviderType::VeryBiasedToBottom
        );
        assert_eq!(
            HeightProviderType::register("trapezoid", MapCodec),
            HeightProviderType::Trapezoid
        );
        assert_eq!(
            HeightProviderType::register("weighted_list", MapCodec),
            HeightProviderType::WeightedList
        );
    }

    #[test]
    fn register_unknown_falls_back_to_constant() {
        assert_eq!(
            HeightProviderType::register("unknown", MapCodec),
            HeightProviderType::Constant
        );
        assert_eq!(
            HeightProviderType::register("", MapCodec),
            HeightProviderType::Constant
        );
    }

    #[test]
    fn codec_returns_placeholder_and_name_roundtrip() {
        for ty in [
            HeightProviderType::Constant,
            HeightProviderType::Uniform,
            HeightProviderType::BiasedToBottom,
            HeightProviderType::VeryBiasedToBottom,
            HeightProviderType::Trapezoid,
            HeightProviderType::WeightedList,
        ] {
            assert_eq!(ty.codec(), MapCodec);
            assert_eq!(HeightProviderType::register(ty.name(), MapCodec), ty);
        }
    }
}
