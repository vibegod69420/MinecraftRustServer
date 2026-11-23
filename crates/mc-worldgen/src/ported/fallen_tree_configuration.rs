pub trait FeatureConfiguration {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateProvider(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntProvider(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeDecorator(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallenTreeConfiguration {
    pub trunk_provider: BlockStateProvider,
    pub log_length: IntProvider,
    pub stump_decorators: Vec<TreeDecorator>,
    pub log_decorators: Vec<TreeDecorator>,
}

impl FeatureConfiguration for FallenTreeConfiguration {}

impl FallenTreeConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FallenTreeConfiguration#<init>
    pub fn new(
        trunk_provider: BlockStateProvider,
        log_length: IntProvider,
        stump_decorators: Vec<TreeDecorator>,
        log_decorators: Vec<TreeDecorator>,
    ) -> Self {
        Self {
            trunk_provider,
            log_length,
            stump_decorators,
            log_decorators,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallenTreeConfigurationBuilder {
    trunk_provider: BlockStateProvider,
    log_length: IntProvider,
    stump_decorators: Vec<TreeDecorator>,
    log_decorators: Vec<TreeDecorator>,
}

impl FallenTreeConfigurationBuilder {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FallenTreeConfiguration.FallenTreeConfigurationBuilder#<init>
    pub fn new(trunk_provider: BlockStateProvider, log_length: IntProvider) -> Self {
        Self {
            trunk_provider,
            log_length,
            stump_decorators: Vec::new(),
            log_decorators: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FallenTreeConfiguration.FallenTreeConfigurationBuilder#stumpDecorators
    pub fn stump_decorators(mut self, stump_decorators: Vec<TreeDecorator>) -> Self {
        self.stump_decorators = stump_decorators;
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FallenTreeConfiguration.FallenTreeConfigurationBuilder#logDecorators
    pub fn log_decorators(mut self, log_decorators: Vec<TreeDecorator>) -> Self {
        self.log_decorators = log_decorators;
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FallenTreeConfiguration.FallenTreeConfigurationBuilder#build
    pub fn build(self) -> FallenTreeConfiguration {
        FallenTreeConfiguration::new(
            self.trunk_provider,
            self.log_length,
            self.stump_decorators,
            self.log_decorators,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_defaults_empty_decorators() {
        let cfg = FallenTreeConfigurationBuilder::new(BlockStateProvider(1), IntProvider(4)).build();
        assert_eq!(cfg.trunk_provider, BlockStateProvider(1));
        assert_eq!(cfg.log_length, IntProvider(4));
        assert!(cfg.stump_decorators.is_empty());
        assert!(cfg.log_decorators.is_empty());
    }

    #[test]
    fn builder_sets_decorators() {
        let stumps = vec![TreeDecorator(10), TreeDecorator(20)];
        let logs = vec![TreeDecorator(30)];
        let cfg = FallenTreeConfigurationBuilder::new(BlockStateProvider(5), IntProvider(7))
            .stump_decorators(stumps.clone())
            .log_decorators(logs.clone())
            .build();
        assert_eq!(cfg.stump_decorators, stumps);
        assert_eq!(cfg.log_decorators, logs);
    }

    #[test]
    fn builder_replaces_decorator_lists() {
        let cfg = FallenTreeConfigurationBuilder::new(BlockStateProvider(1), IntProvider(2))
            .stump_decorators(vec![TreeDecorator(1)])
            .stump_decorators(vec![TreeDecorator(2), TreeDecorator(3)])
            .log_decorators(vec![TreeDecorator(9)])
            .log_decorators(Vec::new())
            .build();
        assert_eq!(cfg.stump_decorators, vec![TreeDecorator(2), TreeDecorator(3)]);
        assert!(cfg.log_decorators.is_empty());
    }

    #[test]
    fn negative_and_extreme_integer_values_preserved() {
        let cfg = FallenTreeConfiguration::new(
            BlockStateProvider(i32::MIN),
            IntProvider(i32::MAX),
            vec![TreeDecorator(-1)],
            vec![TreeDecorator(i32::MIN)],
        );
        assert_eq!(cfg.trunk_provider.0, i32::MIN);
        assert_eq!(cfg.log_length.0, i32::MAX);
        assert_eq!(cfg.stump_decorators[0].0, -1);
        assert_eq!(cfg.log_decorators[0].0, i32::MIN);
    }

    #[test]
    fn float_bit_patterns_preserved_via_integer_storage() {
        let nan_bits = f32::NAN.to_bits() as i32;
        let inf_bits = f32::INFINITY.to_bits() as i32;
        let cfg =
            FallenTreeConfigurationBuilder::new(BlockStateProvider(nan_bits), IntProvider(inf_bits))
                .build();
        assert_eq!(cfg.trunk_provider.0 as u32, f32::NAN.to_bits());
        assert_eq!(cfg.log_length.0 as u32, f32::INFINITY.to_bits());
    }
}
