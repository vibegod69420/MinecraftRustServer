/// Stub for unported `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec;

/// Port of `net.minecraft.world.level.levelgen.placement.PlacementModifierType`.
///
/// In Java this is a generic interface `PlacementModifierType<P extends PlacementModifier>`.
/// Here it is represented as a lightweight named type handle. Registry and codec
/// infrastructure are stubbed so the module remains self-contained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacementModifierType {
    name: &'static str,
}

impl PlacementModifierType {
    pub const BLOCK_PREDICATE_FILTER: Self = Self::register("block_predicate_filter", MapCodec);
    pub const RARITY_FILTER: Self = Self::register("rarity_filter", MapCodec);
    pub const SURFACE_RELATIVE_THRESHOLD_FILTER: Self =
        Self::register("surface_relative_threshold_filter", MapCodec);
    pub const SURFACE_WATER_DEPTH_FILTER: Self =
        Self::register("surface_water_depth_filter", MapCodec);
    pub const BIOME_FILTER: Self = Self::register("biome", MapCodec);
    pub const COUNT: Self = Self::register("count", MapCodec);
    pub const NOISE_BASED_COUNT: Self = Self::register("noise_based_count", MapCodec);
    pub const NOISE_THRESHOLD_COUNT: Self = Self::register("noise_threshold_count", MapCodec);
    pub const COUNT_ON_EVERY_LAYER: Self = Self::register("count_on_every_layer", MapCodec);
    pub const ENVIRONMENT_SCAN: Self = Self::register("environment_scan", MapCodec);
    pub const HEIGHTMAP: Self = Self::register("heightmap", MapCodec);
    pub const HEIGHT_RANGE: Self = Self::register("height_range", MapCodec);
    pub const IN_SQUARE: Self = Self::register("in_square", MapCodec);
    pub const RANDOM_OFFSET: Self = Self::register("random_offset", MapCodec);
    pub const FIXED_PLACEMENT: Self = Self::register("fixed_placement", MapCodec);

    /// Returns the registry name for this placement modifier type.
    pub fn name(&self) -> &'static str {
        self.name
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementModifierType#codec
    pub fn codec(&self) -> MapCodec {
        MapCodec
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementModifierType#register
    const fn register(name: &'static str, _codec: MapCodec) -> Self {
        // Java: Registry.register(BuiltInRegistries.PLACEMENT_MODIFIER_TYPE, name, () -> codec)
        Self { name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_constants_have_expected_names() {
        let cases = [
            (
                PlacementModifierType::BLOCK_PREDICATE_FILTER,
                "block_predicate_filter",
            ),
            (PlacementModifierType::RARITY_FILTER, "rarity_filter"),
            (
                PlacementModifierType::SURFACE_RELATIVE_THRESHOLD_FILTER,
                "surface_relative_threshold_filter",
            ),
            (
                PlacementModifierType::SURFACE_WATER_DEPTH_FILTER,
                "surface_water_depth_filter",
            ),
            (PlacementModifierType::BIOME_FILTER, "biome"),
            (PlacementModifierType::COUNT, "count"),
            (PlacementModifierType::NOISE_BASED_COUNT, "noise_based_count"),
            (
                PlacementModifierType::NOISE_THRESHOLD_COUNT,
                "noise_threshold_count",
            ),
            (
                PlacementModifierType::COUNT_ON_EVERY_LAYER,
                "count_on_every_layer",
            ),
            (PlacementModifierType::ENVIRONMENT_SCAN, "environment_scan"),
            (PlacementModifierType::HEIGHTMAP, "heightmap"),
            (PlacementModifierType::HEIGHT_RANGE, "height_range"),
            (PlacementModifierType::IN_SQUARE, "in_square"),
            (PlacementModifierType::RANDOM_OFFSET, "random_offset"),
            (PlacementModifierType::FIXED_PLACEMENT, "fixed_placement"),
        ];
        assert_eq!(cases.len(), 15);
        for (ty, expected) in cases {
            assert_eq!(ty.name(), expected);
        }
    }

    #[test]
    fn register_returns_named_type() {
        let t = PlacementModifierType::register("custom", MapCodec);
        assert_eq!(t.name(), "custom");
    }

    #[test]
    fn codec_returns_stub() {
        let t = PlacementModifierType::COUNT;
        assert_eq!(t.codec(), MapCodec);
    }

    #[test]
    fn equality_and_copy_semantics() {
        let a = PlacementModifierType::BIOME_FILTER;
        let b = a;
        let c = PlacementModifierType::COUNT;
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
