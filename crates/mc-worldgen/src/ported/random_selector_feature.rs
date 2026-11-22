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

pub struct WeightedPlacedFeature<PF> {
    pub feature: Holder<PF>,
    pub chance: f32,
}

impl<PF: PlacedFeature> WeightedPlacedFeature<PF> {
    // JAVA: net.minecraft.world.level.levelgen.feature.WeightedPlacedFeature#place
    pub fn place<L: WorldGenLevel + ?Sized, G: ChunkGenerator + ?Sized, R: RandomSource + ?Sized>(
        &self,
        level: &mut L,
        chunk_generator: &mut G,
        random: &mut R,
        pos: BlockPos,
    ) -> bool {
        self.feature
            .value()
            .place(level, chunk_generator, random, pos)
    }
}

pub struct RandomFeatureConfiguration<PF, DF> {
    pub features: Vec<WeightedPlacedFeature<PF>>,
    pub default_feature: Holder<DF>,
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

pub struct RandomSelectorFeature;

impl RandomSelectorFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.RandomSelectorFeature#<init>
    pub fn new(_config_codec: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RandomSelectorFeature#place
    pub fn place<L, G, R, PF, DF>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, G, R, RandomFeatureConfiguration<PF, DF>>,
    ) -> bool
    where
        L: WorldGenLevel + ?Sized,
        G: ChunkGenerator + ?Sized,
        R: RandomSource + ?Sized,
        PF: PlacedFeature,
        DF: PlacedFeature,
    {
        let origin = ctx.origin;
        let features_len = ctx.config.features.len();
        let mut i = 0usize;
        while i < features_len {
            let chance = ctx.config.features[i].chance;
            let roll = ctx.random.next_float();
            if roll < chance {
                return ctx.config.features[i].place(ctx.level, ctx.chunk_generator, ctx.random, origin);
            }
            i += 1;
        }

        ctx.config
            .default_feature
            .value()
            .place(ctx.level, ctx.chunk_generator, ctx.random, origin)
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

    struct SeqRandom {
        floats: Vec<f32>,
        idx: usize,
    }

    impl SeqRandom {
        fn floats(floats: Vec<f32>) -> Self {
            Self { floats, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }

        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.idx];
            self.idx += 1;
            v
        }

        fn next_boolean(&mut self) -> bool {
            false
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
    fn selects_first_matching_feature_with_strict_less_than() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = SeqRandom::floats(vec![0.5, 0.3]);
        let config = RandomFeatureConfiguration {
            features: vec![
                WeightedPlacedFeature {
                    feature: Holder(MockFeature::new(true)),
                    chance: 0.5,
                },
                WeightedPlacedFeature {
                    feature: Holder(MockFeature::new(false)),
                    chance: 0.5,
                },
            ],
            default_feature: Holder(MockFeature::new(true)),
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin: BlockPos::new(0, 0, 0),
            config: &config,
        };

        assert!(!RandomSelectorFeature::new(()).place(&mut ctx));
        assert!(!config.features[0].feature.0.called.get());
        assert!(config.features[1].feature.0.called.get());
        assert!(!config.default_feature.0.called.get());
    }

    #[test]
    fn uses_default_when_no_feature_matches_and_passes_origin() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = SeqRandom::floats(vec![0.9, 0.8]);
        let config = RandomFeatureConfiguration {
            features: vec![WeightedPlacedFeature {
                feature: Holder(MockFeature::new(true)),
                chance: 0.5,
            }],
            default_feature: Holder(MockFeature::new(false)),
        };
        let origin = BlockPos::new(7, -1, i32::MIN);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin,
            config: &config,
        };

        assert!(!RandomSelectorFeature::new(()).place(&mut ctx));
        assert!(!config.features[0].feature.0.called.get());
        assert!(config.default_feature.0.called.get());
        assert_eq!(config.default_feature.0.received_pos.get(), origin);
    }

    #[test]
    fn nan_roll_never_matches() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let nan_bits = f32::from_bits(0x7FC0_0001);
        let mut rng = SeqRandom::floats(vec![nan_bits]);
        let config = RandomFeatureConfiguration {
            features: vec![WeightedPlacedFeature {
                feature: Holder(MockFeature::new(true)),
                chance: 1.0,
            }],
            default_feature: Holder(MockFeature::new(false)),
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin: BlockPos::new(0, 0, 0),
            config: &config,
        };

        assert!(!RandomSelectorFeature::new(()).place(&mut ctx));
        assert!(!config.features[0].feature.0.called.get());
        assert!(config.default_feature.0.called.get());
    }

    #[test]
    fn negative_and_infinite_chance_edge_cases() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = SeqRandom::floats(vec![0.0, 0.0]);
        let config = RandomFeatureConfiguration {
            features: vec![
                WeightedPlacedFeature {
                    feature: Holder(MockFeature::new(true)),
                    chance: -1.0,
                },
                WeightedPlacedFeature {
                    feature: Holder(MockFeature::new(true)),
                    chance: f32::INFINITY,
                },
            ],
            default_feature: Holder(MockFeature::new(false)),
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin: BlockPos::new(0, 0, 0),
            config: &config,
        };

        assert!(RandomSelectorFeature::new(()).place(&mut ctx));
        assert!(!config.features[0].feature.0.called.get());
        assert!(config.features[1].feature.0.called.get());
        assert!(!config.default_feature.0.called.get());
    }

    #[test]
    fn empty_features_uses_default_with_extreme_origin() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = SeqRandom::floats(vec![]);
        let config: RandomFeatureConfiguration<MockFeature, MockFeature> = RandomFeatureConfiguration {
            features: vec![],
            default_feature: Holder(MockFeature::new(true)),
        };
        let origin = BlockPos::new(i32::MAX, i32::MIN, -123);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin,
            config: &config,
        };

        assert!(RandomSelectorFeature::new(()).place(&mut ctx));
        assert!(config.default_feature.0.called.get());
        assert_eq!(config.default_feature.0.received_pos.get(), origin);
    }
}
