#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

pub trait FeatureConfiguration {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndGatewayConfiguration {
    exit: Option<BlockPos>,
    exact: bool,
}

impl FeatureConfiguration for EndGatewayConfiguration {}

impl EndGatewayConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.EndGatewayConfiguration#<init>
    fn new(exit: Option<BlockPos>, exact: bool) -> Self {
        Self { exit, exact }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.EndGatewayConfiguration#knownExit
    pub fn known_exit(exit: BlockPos, exact: bool) -> Self {
        Self::new(Some(exit), exact)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.EndGatewayConfiguration#delayedExitSearch
    pub fn delayed_exit_search() -> Self {
        Self::new(None, false)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.EndGatewayConfiguration#getExit
    pub fn get_exit(&self) -> Option<BlockPos> {
        self.exit
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.EndGatewayConfiguration#isExitExact
    pub fn is_exit_exact(&self) -> bool {
        self.exact
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_exit_stores_position_and_exact() {
        let pos = BlockPos::new(10, 64, -20);
        let cfg = EndGatewayConfiguration::known_exit(pos, true);
        assert_eq!(cfg.get_exit(), Some(pos));
        assert!(cfg.is_exit_exact());
    }

    #[test]
    fn delayed_exit_search_is_empty_and_inexact() {
        let cfg = EndGatewayConfiguration::delayed_exit_search();
        assert_eq!(cfg.get_exit(), None);
        assert!(!cfg.is_exit_exact());
    }

    #[test]
    fn known_exit_inexact() {
        let pos = BlockPos::new(0, 0, 0);
        let cfg = EndGatewayConfiguration::known_exit(pos, false);
        assert_eq!(cfg.get_exit(), Some(pos));
        assert!(!cfg.is_exit_exact());
    }

    #[test]
    fn negative_and_extreme_coordinates() {
        let pos = BlockPos::new(i32::MIN, -1, i32::MAX);
        let cfg = EndGatewayConfiguration::known_exit(pos, true);
        assert_eq!(
            cfg.get_exit(),
            Some(BlockPos::new(i32::MIN, -1, i32::MAX))
        );
        assert!(cfg.is_exit_exact());
    }

    #[test]
    fn clone_and_eq() {
        let cfg = EndGatewayConfiguration::known_exit(BlockPos::new(1, 2, 3), false);
        let cfg2 = cfg.clone();
        assert_eq!(cfg, cfg2);
    }
}
