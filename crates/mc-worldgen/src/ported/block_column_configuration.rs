pub trait FeatureConfiguration {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntProvider(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateProvider(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPredicate(pub u32);

impl BlockPredicate {
    pub const ONLY_IN_AIR_PREDICATE: BlockPredicate = BlockPredicate(0);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub height: IntProvider,
    pub state: BlockStateProvider,
}

impl Layer {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration.Layer#<init>
    pub fn new(height: IntProvider, state: BlockStateProvider) -> Self {
        Self { height, state }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration.Layer#height
    pub fn height(&self) -> &IntProvider {
        &self.height
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration.Layer#state
    pub fn state(&self) -> &BlockStateProvider {
        &self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockColumnConfiguration {
    pub layers: Vec<Layer>,
    pub direction: Direction,
    pub allowed_placement: BlockPredicate,
    pub prioritize_tip: bool,
}

impl FeatureConfiguration for BlockColumnConfiguration {}

impl BlockColumnConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration#<init>
    pub fn new(
        layers: Vec<Layer>,
        direction: Direction,
        allowed_placement: BlockPredicate,
        prioritize_tip: bool,
    ) -> Self {
        Self {
            layers,
            direction,
            allowed_placement,
            prioritize_tip,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration#layers
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration#direction
    pub fn direction(&self) -> Direction {
        self.direction
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration#allowedPlacement
    pub fn allowed_placement(&self) -> &BlockPredicate {
        &self.allowed_placement
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration#prioritizeTip
    pub fn prioritize_tip(&self) -> bool {
        self.prioritize_tip
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration#layer
    pub fn layer(height: IntProvider, state: BlockStateProvider) -> Layer {
        Layer::new(height, state)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration#simple
    pub fn simple(height: IntProvider, state: BlockStateProvider) -> Self {
        Self::new(
            vec![Self::layer(height, state)],
            Direction::Up,
            BlockPredicate::ONLY_IN_AIR_PREDICATE,
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_and_simple_factory() {
        let h = IntProvider(3);
        let s = BlockStateProvider(42);
        let layer = BlockColumnConfiguration::layer(h.clone(), s.clone());
        assert_eq!(layer.height(), &h);
        assert_eq!(layer.state(), &s);

        let cfg = BlockColumnConfiguration::simple(IntProvider(3), BlockStateProvider(42));
        assert_eq!(cfg.layers().len(), 1);
        assert_eq!(cfg.layers()[0], layer);
        assert_eq!(cfg.direction(), Direction::Up);
        assert_eq!(
            cfg.allowed_placement(),
            &BlockPredicate::ONLY_IN_AIR_PREDICATE
        );
        assert!(!cfg.prioritize_tip());
    }

    #[test]
    fn new_preserves_all_fields() {
        let layers = vec![
            Layer::new(IntProvider(1), BlockStateProvider(10)),
            Layer::new(IntProvider(2), BlockStateProvider(20)),
        ];
        let cfg = BlockColumnConfiguration::new(
            layers.clone(),
            Direction::Down,
            BlockPredicate(7),
            true,
        );
        assert_eq!(cfg.layers(), layers.as_slice());
        assert_eq!(cfg.direction(), Direction::Down);
        assert_eq!(cfg.allowed_placement(), &BlockPredicate(7));
        assert!(cfg.prioritize_tip());
    }

    #[test]
    fn negative_and_extreme_integer_values_preserved() {
        let layer = Layer::new(IntProvider(i32::MIN), BlockStateProvider(i32::MAX));
        assert_eq!(layer.height().0, i32::MIN);
        assert_eq!(layer.state().0, i32::MAX);

        let cfg = BlockColumnConfiguration::simple(IntProvider(-1), BlockStateProvider(-999));
        assert_eq!(cfg.layers()[0].height().0, -1);
        assert_eq!(cfg.layers()[0].state().0, -999);
    }

    #[test]
    fn float_bit_patterns_preserved_via_integer_storage() {
        let nan_bits = f32::NAN.to_bits() as i32;
        let neg_zero_bits = (-0.0f32).to_bits() as i32;
        let layer = Layer::new(IntProvider(nan_bits), BlockStateProvider(neg_zero_bits));
        assert_eq!(layer.height().0 as u32, f32::NAN.to_bits());
        assert_eq!(layer.state().0 as u32, (-0.0f32).to_bits());
    }

    #[test]
    fn direction_variants_and_predicate_constant() {
        assert_ne!(Direction::Up, Direction::Down);
        assert_ne!(Direction::North, Direction::South);
        assert_ne!(Direction::West, Direction::East);
        assert_eq!(
            BlockPredicate::ONLY_IN_AIR_PREDICATE,
            BlockPredicate(0)
        );
    }
}
