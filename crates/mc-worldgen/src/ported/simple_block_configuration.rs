/// Marker trait for feature configurations.
/// JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration
pub trait FeatureConfiguration {}

/// Placeholder for net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider.
/// Kept minimal and self-contained for this configuration port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateProvider {
    pub state_id: i32,
}

/// Configuration for placing a simple block.
/// JAVA: net.minecraft.world.level.levelgen.feature.configurations.SimpleBlockConfiguration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleBlockConfiguration {
    to_place: BlockStateProvider,
    schedule_tick: bool,
}

impl FeatureConfiguration for SimpleBlockConfiguration {}

impl SimpleBlockConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SimpleBlockConfiguration#SimpleBlockConfiguration
    pub fn new(to_place: BlockStateProvider, schedule_tick: bool) -> Self {
        Self {
            to_place,
            schedule_tick,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SimpleBlockConfiguration#SimpleBlockConfiguration
    pub fn with_provider(to_place: BlockStateProvider) -> Self {
        Self::new(to_place, false)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SimpleBlockConfiguration#toPlace
    pub fn to_place(&self) -> &BlockStateProvider {
        &self.to_place
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SimpleBlockConfiguration#scheduleTick
    pub fn schedule_tick(&self) -> bool {
        self.schedule_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(id: i32) -> BlockStateProvider {
        BlockStateProvider { state_id: id }
    }

    #[test]
    fn new_with_schedule_tick() {
        let cfg = SimpleBlockConfiguration::new(provider(1), true);
        assert_eq!(cfg.to_place(), &provider(1));
        assert!(cfg.schedule_tick());
    }

    #[test]
    fn with_provider_defaults_schedule_tick_false() {
        let cfg = SimpleBlockConfiguration::with_provider(provider(42));
        assert_eq!(cfg.to_place(), &provider(42));
        assert!(!cfg.schedule_tick());
    }

    #[test]
    fn negative_state_id() {
        let cfg = SimpleBlockConfiguration::new(provider(-1), false);
        assert_eq!(cfg.to_place().state_id, -1);
        assert!(!cfg.schedule_tick());
    }

    #[test]
    fn integer_extreme_state_id() {
        let cfg = SimpleBlockConfiguration::new(provider(i32::MIN), true);
        assert_eq!(cfg.to_place().state_id, i32::MIN);
        let cfg_max = SimpleBlockConfiguration::new(provider(i32::MAX), false);
        assert_eq!(cfg_max.to_place().state_id, i32::MAX);
    }
}
