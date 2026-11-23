//! Self-contained port of `net.minecraft.world.level.levelgen.feature.stateproviders.RuleBasedBlockStateProvider`.

use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub id: u32,
}

impl Block {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

pub trait RandomSource {}

#[derive(Clone, Debug, Default)]
pub struct SimpleRandomSource;

impl RandomSource for SimpleRandomSource {}

pub trait WorldGenLevel {}

#[derive(Clone, Debug, Default)]
pub struct TestLevel;

impl WorldGenLevel for TestLevel {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockStateProvider {
    state: BlockState,
}

impl BlockStateProvider {
    pub fn simple(block: &Block) -> Self {
        Self {
            state: BlockState::new(block.id),
        }
    }

    pub fn from_state(state: BlockState) -> Self {
        Self { state }
    }

    pub fn get_state(&self, _random: &dyn RandomSource, _pos: &BlockPos) -> BlockState {
        self.state.clone()
    }
}

#[derive(Clone)]
pub struct BlockPredicate {
    f: Arc<dyn Fn(&dyn WorldGenLevel, &BlockPos) -> bool + Send + Sync>,
}

impl fmt::Debug for BlockPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BlockPredicate(..)")
    }
}

impl BlockPredicate {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&dyn WorldGenLevel, &BlockPos) -> bool + Send + Sync + 'static,
    {
        Self { f: Arc::new(f) }
    }

    pub fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        (self.f)(level, pos)
    }
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub if_true: BlockPredicate,
    pub then: BlockStateProvider,
}

impl Rule {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RuleBasedBlockStateProvider$Rule#<init>
    pub fn new(if_true: BlockPredicate, then_provider: BlockStateProvider) -> Self {
        Self {
            if_true,
            then: then_provider,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuleBasedBlockStateProvider {
    pub fallback: BlockStateProvider,
    pub rules: Vec<Rule>,
}

impl RuleBasedBlockStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RuleBasedBlockStateProvider#simple
    pub fn simple(fallback: BlockStateProvider) -> Self {
        Self {
            fallback,
            rules: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RuleBasedBlockStateProvider#simple
    pub fn simple_from_block(block: &Block) -> Self {
        Self::simple(BlockStateProvider::simple(block))
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.RuleBasedBlockStateProvider#getState
    pub fn get_state(
        &self,
        level: &dyn WorldGenLevel,
        random: &dyn RandomSource,
        pos: &BlockPos,
    ) -> BlockState {
        for rule in &self.rules {
            if rule.if_true.test(level, pos) {
                return rule.then.get_state(random, pos);
            }
        }
        self.fallback.get_state(random, pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_provider_uses_fallback_when_no_rules() {
        let provider = RuleBasedBlockStateProvider::simple(BlockStateProvider::from_state(
            BlockState::new(1),
        ));
        let level = TestLevel;
        let random = SimpleRandomSource;
        assert_eq!(
            provider.get_state(&level, &random, &BlockPos::new(0, 0, 0)),
            BlockState::new(1)
        );
    }

    #[test]
    fn simple_from_block_uses_block_state() {
        let provider = RuleBasedBlockStateProvider::simple_from_block(&Block::new(42));
        let level = TestLevel;
        let random = SimpleRandomSource;
        assert_eq!(
            provider.get_state(&level, &random, &BlockPos::new(1, -2, 3)),
            BlockState::new(42)
        );
    }

    #[test]
    fn first_matching_rule_wins() {
        let fallback = BlockStateProvider::from_state(BlockState::new(0));
        let rules = vec![
            Rule::new(
                BlockPredicate::new(|_, pos| pos.x > 0),
                BlockStateProvider::from_state(BlockState::new(1)),
            ),
            Rule::new(
                BlockPredicate::new(|_, pos| pos.x > 5),
                BlockStateProvider::from_state(BlockState::new(2)),
            ),
        ];
        let provider = RuleBasedBlockStateProvider { fallback, rules };
        let level = TestLevel;
        let random = SimpleRandomSource;
        assert_eq!(
            provider.get_state(&level, &random, &BlockPos::new(10, 0, 0)),
            BlockState::new(1)
        );
        assert_eq!(
            provider.get_state(&level, &random, &BlockPos::new(-1, 0, 0)),
            BlockState::new(0)
        );
    }

    #[test]
    fn negative_and_extreme_coordinates() {
        let fallback = BlockStateProvider::from_state(BlockState::new(7));
        let rules = vec![Rule::new(
            BlockPredicate::new(|_, pos| pos.y < 0),
            BlockStateProvider::from_state(BlockState::new(9)),
        )];
        let provider = RuleBasedBlockStateProvider { fallback, rules };
        let level = TestLevel;
        let random = SimpleRandomSource;
        assert_eq!(
            provider.get_state(&level, &random, &BlockPos::new(0, i32::MIN, 0)),
            BlockState::new(9)
        );
        assert_eq!(
            provider.get_state(&level, &random, &BlockPos::new(i32::MAX, 0, i32::MIN)),
            BlockState::new(7)
        );
    }

    #[test]
    fn no_rules_match_returns_fallback() {
        let provider = RuleBasedBlockStateProvider {
            fallback: BlockStateProvider::from_state(BlockState::new(3)),
            rules: vec![Rule::new(
                BlockPredicate::new(|_, _| false),
                BlockStateProvider::from_state(BlockState::new(99)),
            )],
        };
        let level = TestLevel;
        let random = SimpleRandomSource;
        assert_eq!(
            provider.get_state(&level, &random, &BlockPos::new(5, 5, 5)),
            BlockState::new(3)
        );
    }
}
