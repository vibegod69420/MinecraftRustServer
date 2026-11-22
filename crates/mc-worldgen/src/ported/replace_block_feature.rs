use std::marker::PhantomData;

/// Placeholder for `com.mojang.serialization.Codec<T>`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Codec<T>(pub PhantomData<T>);

/// Placeholder for `net.minecraft.util.RandomSource`.
pub trait RandomSource {}

/// Minimal block state abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u32,
}

/// Minimal equivalent of `net.minecraft.core.BlockPos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#new(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Minimal equivalent of `net.minecraft.world.level.WorldGenLevel`.
pub trait WorldGenLevel {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

/// Placeholder for `net.minecraft.world.level.chunk.ChunkGenerator`.
pub trait ChunkGenerator {}

/// Minimal rule target abstraction.
pub trait RuleTest<R: RandomSource + ?Sized> {
    fn test(&self, state: &BlockState, random: &mut R) -> bool;
}

/// Minimal equivalent of `net.minecraft.world.level.levelgen.feature.configurations.OreConfiguration.TargetBlockState`.
pub struct TargetBlockState<T> {
    pub target: T,
    pub state: BlockState,
}

/// Minimal equivalent of `net.minecraft.world.level.levelgen.feature.configurations.ReplaceBlockConfiguration`.
pub struct ReplaceBlockConfiguration<T> {
    pub target_states: Vec<TargetBlockState<T>>,
}

/// Minimal equivalent of `net.minecraft.world.level.levelgen.feature.FeaturePlaceContext`.
pub struct FeaturePlaceContext<'a, C, Level: ?Sized, Generator: ?Sized, R: ?Sized> {
    pub config: &'a C,
    pub level: &'a mut Level,
    pub chunk_generator: &'a mut Generator,
    pub random: &'a mut R,
    pub origin: BlockPos,
}

impl<'a, C, Level: ?Sized, Generator: ?Sized, R: ?Sized> FeaturePlaceContext<'a, C, Level, Generator, R> {
    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#level
    pub fn level(&mut self) -> &mut Level {
        self.level
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#origin
    pub fn origin(&self) -> BlockPos {
        self.origin
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#config
    pub fn config(&self) -> &C {
        self.config
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.FeaturePlaceContext#random
    pub fn random(&mut self) -> &mut R {
        self.random
    }
}

/// Minimal equivalent of `net.minecraft.world.level.levelgen.feature.Feature`.
pub trait Feature<C, Level: ?Sized, Generator: ?Sized, R: ?Sized> {
    fn place(&mut self, ctx: &mut FeaturePlaceContext<'_, C, Level, Generator, R>) -> bool;
}

/// Port of `net.minecraft.world.level.levelgen.feature.ReplaceBlockFeature`.
pub struct ReplaceBlockFeature<C> {
    _codec: Codec<C>,
}

impl<C> ReplaceBlockFeature<C> {
    // JAVA: net.minecraft.world.level.levelgen.feature.ReplaceBlockFeature#<init>
    pub fn new(codec: Codec<C>) -> Self {
        Self { _codec: codec }
    }
}

impl<T, Level, Generator, R> Feature<ReplaceBlockConfiguration<T>, Level, Generator, R>
    for ReplaceBlockFeature<ReplaceBlockConfiguration<T>>
where
    T: RuleTest<R>,
    Level: WorldGenLevel + ?Sized,
    Generator: ChunkGenerator + ?Sized,
    R: RandomSource + ?Sized,
{
    // JAVA: net.minecraft.world.level.levelgen.feature.ReplaceBlockFeature#place
    fn place(
        &mut self,
        ctx: &mut FeaturePlaceContext<'_, ReplaceBlockConfiguration<T>, Level, Generator, R>,
    ) -> bool {
        let origin = ctx.origin();
        let current_state = ctx.level.get_block_state(origin);
        let config = ctx.config;

        for target_state in &config.target_states {
            if target_state.target.test(&current_state, ctx.random) {
                ctx.level.set_block(origin, target_state.state, 2);
                break;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestRandom {
        calls: u32,
    }

    impl RandomSource for TestRandom {}

    #[derive(Default)]
    struct TestGenerator;
    impl ChunkGenerator for TestGenerator {}

    struct TestLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
        set_calls: Vec<(BlockPos, BlockState, i32)>,
    }

    impl TestLevel {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
                set_calls: Vec::new(),
            }
        }
    }

    impl WorldGenLevel for TestLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            *self.blocks.get(&(pos.x, pos.y, pos.z)).unwrap_or(&BlockState { id: 0 })
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            self.set_calls.push((pos, state, flags));
            true
        }
    }

    struct MatchIdRule {
        id: u32,
    }

    impl RuleTest<TestRandom> for MatchIdRule {
        fn test(&self, state: &BlockState, random: &mut TestRandom) -> bool {
            random.calls = random.calls.wrapping_add(1);
            state.id == self.id
        }
    }

    #[test]
    fn replaces_on_first_matching_target() {
        let mut level = TestLevel::new();
        let origin = BlockPos::new(1, 2, 3);
        level.blocks.insert((1, 2, 3), BlockState { id: 10 });
        let config = ReplaceBlockConfiguration {
            target_states: vec![
                TargetBlockState { target: MatchIdRule { id: 5 }, state: BlockState { id: 100 } },
                TargetBlockState { target: MatchIdRule { id: 10 }, state: BlockState { id: 200 } },
                TargetBlockState { target: MatchIdRule { id: 10 }, state: BlockState { id: 300 } },
            ],
        };
        let mut generator = TestGenerator;
        let mut random = TestRandom::default();
        let mut ctx = FeaturePlaceContext {
            config: &config,
            level: &mut level,
            chunk_generator: &mut generator,
            random: &mut random,
            origin,
        };
        let mut feature: ReplaceBlockFeature<ReplaceBlockConfiguration<MatchIdRule>> =
            ReplaceBlockFeature::new(Codec(PhantomData));
        let ok = feature.place(&mut ctx);

        assert!(ok);
        assert_eq!(level.get_block_state(origin), BlockState { id: 200 });
        assert_eq!(level.set_calls, vec![(origin, BlockState { id: 200 }, 2)]);
        assert_eq!(random.calls, 2); // checked first (no), second (yes), then broke
    }

    #[test]
    fn no_match_returns_true_without_setting_block() {
        let mut level = TestLevel::new();
        let origin = BlockPos::new(0, 0, 0);
        level.blocks.insert((0, 0, 0), BlockState { id: 1 });
        let config = ReplaceBlockConfiguration {
            target_states: vec![
                TargetBlockState { target: MatchIdRule { id: 2 }, state: BlockState { id: 9 } },
            ],
        };
        let mut generator = TestGenerator;
        let mut random = TestRandom::default();
        let mut ctx = FeaturePlaceContext {
            config: &config,
            level: &mut level,
            chunk_generator: &mut generator,
            random: &mut random,
            origin,
        };
        let mut feature: ReplaceBlockFeature<ReplaceBlockConfiguration<MatchIdRule>> =
            ReplaceBlockFeature::new(Codec(PhantomData));
        let ok = feature.place(&mut ctx);

        assert!(ok);
        assert!(level.set_calls.is_empty());
        assert_eq!(level.get_block_state(origin), BlockState { id: 1 });
    }

    #[test]
    fn empty_targets_returns_true() {
        let mut level = TestLevel::new();
        let config: ReplaceBlockConfiguration<MatchIdRule> = ReplaceBlockConfiguration {
            target_states: vec![],
        };
        let mut generator = TestGenerator;
        let mut random = TestRandom::default();
        let mut ctx = FeaturePlaceContext {
            config: &config,
            level: &mut level,
            chunk_generator: &mut generator,
            random: &mut random,
            origin: BlockPos::new(-1, -2, -3),
        };
        let mut feature = ReplaceBlockFeature::new(Codec(PhantomData));
        assert!(feature.place(&mut ctx));
        assert!(level.set_calls.is_empty());
    }

    #[test]
    fn negative_origin_coordinates_are_preserved() {
        let mut level = TestLevel::new();
        let origin = BlockPos::new(i32::MIN, -1, i32::MAX);
        level.blocks.insert((i32::MIN, -1, i32::MAX), BlockState { id: 7 });
        let config = ReplaceBlockConfiguration {
            target_states: vec![
                TargetBlockState { target: MatchIdRule { id: 7 }, state: BlockState { id: 8 } },
            ],
        };
        let mut generator = TestGenerator;
        let mut random = TestRandom::default();
        let mut ctx = FeaturePlaceContext {
            config: &config,
            level: &mut level,
            chunk_generator: &mut generator,
            random: &mut random,
            origin,
        };
        let mut feature: ReplaceBlockFeature<ReplaceBlockConfiguration<MatchIdRule>> =
            ReplaceBlockFeature::new(Codec(PhantomData));
        assert!(feature.place(&mut ctx));
        assert_eq!(level.set_calls[0].0, origin);
    }
}
