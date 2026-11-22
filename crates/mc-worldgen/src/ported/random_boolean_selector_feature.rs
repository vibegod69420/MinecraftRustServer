#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
    fn next_boolean(&mut self) -> bool;
}

pub trait WorldGenLevel {}

pub trait ChunkGenerator {}

pub trait PlacedFeature {
    fn place<L: WorldGenLevel + ?Sized, G: ChunkGenerator + ?Sized, R: RandomSource + ?Sized>(
        &self,
        level: &mut L,
        chunk_generator: &mut G,
        random: &mut R,
        pos: BlockPos,
    ) -> bool;
}

#[derive(Debug, Clone)]
pub struct Holder<T>(pub T);

impl<T> Holder<T> {
    pub fn value(&self) -> &T {
        &self.0
    }
}

pub struct RandomBooleanFeatureConfiguration<PT, PF> {
    pub feature_true: Holder<PT>,
    pub feature_false: Holder<PF>,
}

pub struct FeaturePlaceContext<'a, L: ?Sized, G: ?Sized, R: ?Sized, C> {
    pub level: &'a mut L,
    pub chunk_generator: &'a mut G,
    pub random: &'a mut R,
    pub origin: BlockPos,
    pub config: &'a C,
}

impl<'a, L: ?Sized, G: ?Sized, R: ?Sized, C> FeaturePlaceContext<'a, L, G, R, C> {
    pub fn level(&mut self) -> &mut L {
        self.level
    }

    pub fn chunk_generator(&mut self) -> &mut G {
        self.chunk_generator
    }

    pub fn random(&mut self) -> &mut R {
        self.random
    }

    pub fn origin(&self) -> BlockPos {
        self.origin
    }

    pub fn config(&self) -> &C {
        self.config
    }
}

pub struct RandomBooleanSelectorFeature;

impl RandomBooleanSelectorFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.RandomBooleanSelectorFeature#<init>
    pub fn new(_config_codec: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RandomBooleanSelectorFeature#place
    pub fn place<L, G, R, PT, PF>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, G, R, RandomBooleanFeatureConfiguration<PT, PF>>,
    ) -> bool
    where
        L: WorldGenLevel + ?Sized,
        G: ChunkGenerator + ?Sized,
        R: RandomSource + ?Sized,
        PT: PlacedFeature,
        PF: PlacedFeature,
    {
        let next_bool = ctx.random.next_boolean();
        let origin = ctx.origin;
        if next_bool {
            ctx.config
                .feature_true
                .value()
                .place(ctx.level, ctx.chunk_generator, ctx.random, origin)
        } else {
            ctx.config
                .feature_false
                .value()
                .place(ctx.level, ctx.chunk_generator, ctx.random, origin)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct MockLevel;
    impl WorldGenLevel for MockLevel {}

    struct MockChunkGenerator;
    impl ChunkGenerator for MockChunkGenerator {}

    struct MockRandom {
        bools: Vec<bool>,
        idx: usize,
    }

    impl MockRandom {
        fn new(bools: Vec<bool>) -> Self {
            Self { bools, idx: 0 }
        }
    }

    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }

        fn next_float(&mut self) -> f32 {
            0.0
        }

        fn next_boolean(&mut self) -> bool {
            let v = self.bools[self.idx];
            self.idx += 1;
            v
        }
    }

    struct MockFeature {
        result: bool,
        called: Cell<bool>,
        received_pos: Cell<BlockPos>,
    }

    impl MockFeature {
        fn new(result: bool) -> Self {
            Self {
                result,
                called: Cell::new(false),
                received_pos: Cell::new(BlockPos::new(0, 0, 0)),
            }
        }
    }

    impl PlacedFeature for MockFeature {
        fn place<
            L: WorldGenLevel + ?Sized,
            G: ChunkGenerator + ?Sized,
            R: RandomSource + ?Sized,
        >(
            &self,
            _level: &mut L,
            _chunk_generator: &mut G,
            _random: &mut R,
            pos: BlockPos,
        ) -> bool {
            self.called.set(true);
            self.received_pos.set(pos);
            self.result
        }
    }

    #[test]
    fn selects_true_feature_when_random_is_true() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = MockRandom::new(vec![true]);
        let config = RandomBooleanFeatureConfiguration {
            feature_true: Holder(MockFeature::new(true)),
            feature_false: Holder(MockFeature::new(false)),
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin: BlockPos::new(1, 2, 3),
            config: &config,
        };

        assert!(RandomBooleanSelectorFeature::new(()).place(&mut ctx));
        assert!(config.feature_true.0.called.get());
        assert!(!config.feature_false.0.called.get());
    }

    #[test]
    fn selects_false_feature_when_random_is_false() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = MockRandom::new(vec![false]);
        let config = RandomBooleanFeatureConfiguration {
            feature_true: Holder(MockFeature::new(true)),
            feature_false: Holder(MockFeature::new(false)),
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin: BlockPos::new(0, 0, 0),
            config: &config,
        };

        assert!(!RandomBooleanSelectorFeature::new(()).place(&mut ctx));
        assert!(!config.feature_true.0.called.get());
        assert!(config.feature_false.0.called.get());
    }

    #[test]
    fn propagates_result_and_passes_negative_origin() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = MockRandom::new(vec![true]);
        let config = RandomBooleanFeatureConfiguration {
            feature_true: Holder(MockFeature::new(false)),
            feature_false: Holder(MockFeature::new(true)),
        };
        let origin = BlockPos::new(i32::MIN, -1, i32::MAX);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin,
            config: &config,
        };

        assert!(!RandomBooleanSelectorFeature::new(()).place(&mut ctx));
        assert_eq!(config.feature_true.0.received_pos.get(), origin);
    }

    #[test]
    fn false_branch_can_return_true_and_receives_origin() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = MockRandom::new(vec![false]);
        let config = RandomBooleanFeatureConfiguration {
            feature_true: Holder(MockFeature::new(false)),
            feature_false: Holder(MockFeature::new(true)),
        };
        let origin = BlockPos::new(7, -64, -8);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin,
            config: &config,
        };

        assert!(RandomBooleanSelectorFeature::new(()).place(&mut ctx));
        assert_eq!(config.feature_false.0.received_pos.get(), origin);
    }
}
