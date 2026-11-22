/// Placeholder for `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
///
/// This is a self-contained stub so the module compiles without unported
/// external Minecraft classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateProvider;

/// Marker trait for `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration`.
pub trait FeatureConfiguration {}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.HugeMushroomFeatureConfiguration`.
///
/// JAVA: `public static final Codec<HugeMushroomFeatureConfiguration> CODEC = RecordCodecBuilder.create(...)`
/// - `cap_provider`: `BlockStateProvider.CODEC`
/// - `stem_provider`: `BlockStateProvider.CODEC`
/// - `foliage_radius`: `Codec.INT.fieldOf("foliage_radius").orElse(2)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HugeMushroomFeatureConfiguration {
    pub cap_provider: BlockStateProvider,
    pub stem_provider: BlockStateProvider,
    pub foliage_radius: i32,
}

impl FeatureConfiguration for HugeMushroomFeatureConfiguration {}

impl HugeMushroomFeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.HugeMushroomFeatureConfiguration#<init>
    pub fn new(
        cap_provider: BlockStateProvider,
        stem_provider: BlockStateProvider,
        foliage_radius: i32,
    ) -> Self {
        Self {
            cap_provider,
            stem_provider,
            foliage_radius,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction() {
        let cfg = HugeMushroomFeatureConfiguration::new(BlockStateProvider, BlockStateProvider, 2);
        assert_eq!(cfg.cap_provider, BlockStateProvider);
        assert_eq!(cfg.stem_provider, BlockStateProvider);
        assert_eq!(cfg.foliage_radius, 2);
    }

    #[test]
    fn foliage_radius_edge_cases() {
        let zero =
            HugeMushroomFeatureConfiguration::new(BlockStateProvider, BlockStateProvider, 0);
        assert_eq!(zero.foliage_radius, 0);

        let negative =
            HugeMushroomFeatureConfiguration::new(BlockStateProvider, BlockStateProvider, -1);
        assert_eq!(negative.foliage_radius, -1);

        let max = HugeMushroomFeatureConfiguration::new(
            BlockStateProvider,
            BlockStateProvider,
            i32::MAX,
        );
        assert_eq!(max.foliage_radius, i32::MAX);

        let min = HugeMushroomFeatureConfiguration::new(
            BlockStateProvider,
            BlockStateProvider,
            i32::MIN,
        );
        assert_eq!(min.foliage_radius, i32::MIN);
    }

    #[test]
    fn clone_and_equality() {
        let a = HugeMushroomFeatureConfiguration::new(BlockStateProvider, BlockStateProvider, 3);
        let b = a.clone();
        assert_eq!(a, b);
    }
}
