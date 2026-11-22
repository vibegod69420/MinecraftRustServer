/// Stub for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
}

/// Stub for `net.minecraft.world.level.levelgen.feature.ConfiguredFeature<?, ?>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredFeature;

/// Marker trait for feature configurations.
pub trait FeatureConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration#getFeatures
    fn get_features(&self) -> Vec<ConfiguredFeature> {
        Vec::new()
    }
}

/// Stub for `net.minecraft.world.level.levelgen.structure.templatesystem.BlockStateMatchTest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateMatchTest {
    pub state: BlockState,
}

impl BlockStateMatchTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockStateMatchTest#<init>
    pub fn new(state: BlockState) -> Self {
        Self { state }
    }
}

/// Minimal stub for `net.minecraft.world.level.levelgen.structure.templatesystem.RuleTest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleTest {
    BlockStateMatch(BlockStateMatchTest),
    AlwaysTrue,
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.OreConfiguration.TargetBlockState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetBlockState {
    pub target: RuleTest,
    pub state: BlockState,
}

/// Stub namespace for `net.minecraft.world.level.levelgen.feature.configurations.OreConfiguration`.
pub mod ore_configuration {
    use super::{BlockState, RuleTest, TargetBlockState};

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.OreConfiguration#target
    pub fn target(rule_test: RuleTest, state: BlockState) -> TargetBlockState {
        TargetBlockState {
            target: rule_test,
            state,
        }
    }
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.ReplaceBlockConfiguration`.
///
/// Codec shape (Java): RecordCodecBuilder with
/// `Codec.list(OreConfiguration.TargetBlockState.CODEC).fieldOf("targets")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceBlockConfiguration {
    pub target_states: Vec<TargetBlockState>,
}

impl ReplaceBlockConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.ReplaceBlockConfiguration#<init>(List)
    pub fn new(target_states: Vec<TargetBlockState>) -> Self {
        Self { target_states }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.ReplaceBlockConfiguration#<init>(BlockState,BlockState)
    pub fn new_simple(from: BlockState, to: BlockState) -> Self {
        Self::new(vec![ore_configuration::target(
            RuleTest::BlockStateMatch(BlockStateMatchTest::new(from)),
            to,
        )])
    }
}

impl FeatureConfiguration for ReplaceBlockConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    fn bs(id: u32) -> BlockState {
        BlockState { id }
    }

    #[test]
    fn simple_constructor_creates_single_block_state_match_target() {
        let cfg = ReplaceBlockConfiguration::new_simple(bs(1), bs(2));
        assert_eq!(cfg.target_states.len(), 1);
        let target = &cfg.target_states[0];
        assert_eq!(target.state, bs(2));
        match &target.target {
            RuleTest::BlockStateMatch(m) => assert_eq!(m.state, bs(1)),
            other => panic!("expected BlockStateMatch, got {other:?}"),
        }
    }

    #[test]
    fn list_constructor_stores_targets_in_order() {
        let targets = vec![
            ore_configuration::target(RuleTest::AlwaysTrue, bs(10)),
            ore_configuration::target(
                RuleTest::BlockStateMatch(BlockStateMatchTest::new(bs(3))),
                bs(4),
            ),
        ];
        let cfg = ReplaceBlockConfiguration::new(targets.clone());
        assert_eq!(cfg.target_states, targets);
    }

    #[test]
    fn empty_target_list_is_preserved() {
        let cfg = ReplaceBlockConfiguration::new(Vec::new());
        assert!(cfg.target_states.is_empty());
        assert!(cfg.get_features().is_empty());
    }

    #[test]
    fn block_state_match_test_constructor_stores_state() {
        let test = BlockStateMatchTest::new(bs(u32::MAX));
        assert_eq!(test.state.id, u32::MAX);
    }

    #[test]
    fn ore_configuration_target_builds_target_block_state() {
        let t = ore_configuration::target(RuleTest::AlwaysTrue, bs(99));
        assert_eq!(t.state, bs(99));
        assert_eq!(t.target, RuleTest::AlwaysTrue);
    }
}
