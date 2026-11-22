/// Marker trait for feature configurations.
/// JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration
pub trait FeatureConfiguration {}

/// Dimension Y size used by LayerConfiguration codec bounds.
/// JAVA: net.minecraft.world.level.dimension.DimensionType#Y_SIZE
pub const DIMENSION_TYPE_Y_SIZE: i32 = 4064;

/// Placeholder for net.minecraft.world.level.block.state.BlockState.
/// Kept minimal and self-contained for this configuration port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub id: i32,
}

/// Configuration for a single layer feature.
/// JAVA: net.minecraft.world.level.levelgen.feature.configurations.LayerConfiguration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerConfiguration {
    pub height: i32,
    pub state: BlockState,
}

impl FeatureConfiguration for LayerConfiguration {}

impl LayerConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.LayerConfiguration#LayerConfiguration
    pub fn new(height: i32, state: BlockState) -> Self {
        Self { height, state }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_fields() {
        let state = BlockState { id: 5 };
        let cfg = LayerConfiguration::new(10, state);
        assert_eq!(cfg.height, 10);
        assert_eq!(cfg.state, state);
    }

    #[test]
    fn negative_height() {
        let cfg = LayerConfiguration::new(-1, BlockState { id: 0 });
        assert_eq!(cfg.height, -1);
        assert_eq!(cfg.state.id, 0);
    }

    #[test]
    fn integer_extremes() {
        let cfg = LayerConfiguration::new(i32::MAX, BlockState { id: i32::MIN });
        assert_eq!(cfg.height, i32::MAX);
        assert_eq!(cfg.state.id, i32::MIN);
    }

    #[test]
    fn dimension_y_size_constant() {
        assert_eq!(DIMENSION_TYPE_Y_SIZE, 4064);
    }

    #[test]
    fn copy_and_eq() {
        let cfg = LayerConfiguration::new(3, BlockState { id: 7 });
        let copied = cfg;
        assert_eq!(cfg, copied);
    }
}
