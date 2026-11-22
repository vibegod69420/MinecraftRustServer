/// Placeholder for `net.minecraft.util.valueproviders.IntProvider`.
///
/// This is a self-contained stub so the module compiles without unported
/// external Minecraft classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntProvider;

/// Placeholder for `net.minecraft.world.level.levelgen.blockpredicates.BlockPredicate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPredicate;

/// Placeholder for `net.minecraft.world.level.levelgen.feature.stateproviders.RuleBasedBlockStateProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleBasedBlockStateProvider;

/// Marker trait for `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration`.
pub trait FeatureConfiguration {}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.DiskConfiguration`.
///
/// JAVA: `public static final Codec<DiskConfiguration> CODEC = RecordCodecBuilder.create(...)`
/// - `state_provider`: `RuleBasedBlockStateProvider.CODEC`
/// - `target`: `BlockPredicate.CODEC`
/// - `radius`: `IntProvider.codec(0, 8)`
/// - `half_height`: `Codec.intRange(0, 4)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskConfiguration {
    state_provider: RuleBasedBlockStateProvider,
    target: BlockPredicate,
    radius: IntProvider,
    half_height: i32,
}

impl FeatureConfiguration for DiskConfiguration {}

impl DiskConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DiskConfiguration#<init>
    pub fn new(
        state_provider: RuleBasedBlockStateProvider,
        target: BlockPredicate,
        radius: IntProvider,
        half_height: i32,
    ) -> Self {
        Self {
            state_provider,
            target,
            radius,
            half_height,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DiskConfiguration#stateProvider
    pub fn state_provider(&self) -> &RuleBasedBlockStateProvider {
        &self.state_provider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DiskConfiguration#target
    pub fn target(&self) -> &BlockPredicate {
        &self.target
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DiskConfiguration#radius
    pub fn radius(&self) -> &IntProvider {
        &self.radius
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DiskConfiguration#halfHeight
    pub fn half_height(&self) -> i32 {
        self.half_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_and_accessors() {
        let cfg = DiskConfiguration::new(
            RuleBasedBlockStateProvider,
            BlockPredicate,
            IntProvider,
            3,
        );
        assert_eq!(cfg.state_provider(), &RuleBasedBlockStateProvider);
        assert_eq!(cfg.target(), &BlockPredicate);
        assert_eq!(cfg.radius(), &IntProvider);
        assert_eq!(cfg.half_height(), 3);
    }

    #[test]
    fn half_height_edge_cases() {
        let zero =
            DiskConfiguration::new(RuleBasedBlockStateProvider, BlockPredicate, IntProvider, 0);
        assert_eq!(zero.half_height(), 0);

        let negative =
            DiskConfiguration::new(RuleBasedBlockStateProvider, BlockPredicate, IntProvider, -1);
        assert_eq!(negative.half_height(), -1);

        let max = DiskConfiguration::new(
            RuleBasedBlockStateProvider,
            BlockPredicate,
            IntProvider,
            i32::MAX,
        );
        assert_eq!(max.half_height(), i32::MAX);

        let min = DiskConfiguration::new(
            RuleBasedBlockStateProvider,
            BlockPredicate,
            IntProvider,
            i32::MIN,
        );
        assert_eq!(min.half_height(), i32::MIN);
    }

    #[test]
    fn clone_and_equality() {
        let a = DiskConfiguration::new(RuleBasedBlockStateProvider, BlockPredicate, IntProvider, 2);
        let b = a.clone();
        assert_eq!(a, b);
    }
}
