/// Placeholder for `net.minecraft.world.level.block.state.BlockState`.
///
/// This is a self-contained stub so the module compiles without unported
/// external Minecraft classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState;

/// Placeholder for `net.minecraft.util.valueproviders.IntProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntProvider;

/// Marker trait for `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration`.
pub trait FeatureConfiguration {}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.ReplaceSphereConfiguration`.
///
/// JAVA: `public static final Codec<ReplaceSphereConfiguration> CODEC = RecordCodecBuilder.create(...)`
/// - `target`: `BlockState.CODEC`
/// - `state`: `BlockState.CODEC`
/// - `radius`: `IntProvider.codec(0, 12)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceSphereConfiguration {
    pub target_state: BlockState,
    pub replace_state: BlockState,
    radius: IntProvider,
}

impl FeatureConfiguration for ReplaceSphereConfiguration {}

impl ReplaceSphereConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.ReplaceSphereConfiguration#<init>
    pub fn new(target_state: BlockState, replace_state: BlockState, radius: IntProvider) -> Self {
        Self {
            target_state,
            replace_state,
            radius,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.ReplaceSphereConfiguration#radius
    pub fn radius(&self) -> &IntProvider {
        &self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_and_public_fields() {
        let cfg = ReplaceSphereConfiguration::new(BlockState, BlockState, IntProvider);
        assert_eq!(cfg.target_state, BlockState);
        assert_eq!(cfg.replace_state, BlockState);
    }

    #[test]
    fn radius_getter() {
        let cfg = ReplaceSphereConfiguration::new(BlockState, BlockState, IntProvider);
        assert_eq!(cfg.radius(), &IntProvider);
    }

    #[test]
    fn clone_and_equality() {
        let a = ReplaceSphereConfiguration::new(BlockState, BlockState, IntProvider);
        let b = a.clone();
        assert_eq!(a, b);
    }
}
