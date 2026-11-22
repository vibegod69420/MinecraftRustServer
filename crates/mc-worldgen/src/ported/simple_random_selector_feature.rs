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

pub struct SimpleRandomFeatureConfiguration<PF> {
    pub features: Vec<Holder<PF>>,
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

pub struct SimpleRandomSelectorFeature;

impl SimpleRandomSelectorFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.SimpleRandomSelectorFeature#<init>
    pub fn new(_config_codec: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SimpleRandomSelectorFeature#place
    pub fn place<L, G, R, PF>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, G, R, SimpleRandomFeatureConfiguration<PF>>,
    ) -> bool
    where
        L: WorldGenLevel + ?Sized,
        G: ChunkGenerator + ?Sized,
        R: RandomSource + ?Sized,
        PF: PlacedFeature,
    {
        let size = ctx.config.features.len() as i32;
        let idx = ctx.random.next_int(size);
        let origin = ctx.origin;
        ctx.config.features[idx as usize].value().place(
            ctx.level,
            ctx.chunk_generator,
            ctx.random,
            origin,
        )
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
        ints: Vec<i32>,
        bounds: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn ints(ints: Vec<i32>) -> Self {
            Self {
                ints,
                bounds: Vec::new(),
                idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            self.bounds.push(bound);
            let v = self.ints[self.idx];
            self.idx += 1;
            v
        }

        fn next_float(&mut self) -> f32 {
            0.0
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
    fn selects_feature_by_next_int_index() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = SeqRandom::ints(vec![1]);
        let config = SimpleRandomFeatureConfiguration {
            features: vec![
                Holder(MockFeature::new(false)),
                Holder(MockFeature::new(true)),
                Holder(MockFeature::new(false)),
            ],
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin: BlockPos::new(1, 2, 3),
            config: &config,
        };

        assert!(SimpleRandomSelectorFeature::new(()).place(&mut ctx));
        assert!(!config.features[0].0.called.get());
        assert!(config.features[1].0.called.get());
        assert!(!config.features[2].0.called.get());
    }

    #[test]
    fn next_int_bound_matches_feature_count() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = SeqRandom::ints(vec![2]);
        let config = SimpleRandomFeatureConfiguration {
            features: vec![
                Holder(MockFeature::new(true)),
                Holder(MockFeature::new(true)),
                Holder(MockFeature::new(false)),
            ],
        };
        {
            let mut ctx = FeaturePlaceContext {
                level: &mut level,
                chunk_generator: &mut cg,
                random: &mut rng,
                origin: BlockPos::new(0, 0, 0),
                config: &config,
            };
            assert!(!SimpleRandomSelectorFeature::new(()).place(&mut ctx));
        }
        assert_eq!(rng.bounds, vec![3]);
    }

    #[test]
    fn passes_negative_and_extreme_origin() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = SeqRandom::ints(vec![0]);
        let config = SimpleRandomFeatureConfiguration {
            features: vec![Holder(MockFeature::new(true))],
        };
        let origin = BlockPos::new(i32::MIN, -64, i32::MAX);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin,
            config: &config,
        };

        assert!(SimpleRandomSelectorFeature::new(()).place(&mut ctx));
        assert_eq!(config.features[0].0.received_pos.get(), origin);
    }

    #[test]
    fn propagates_result_from_selected_feature() {
        let mut level = MockLevel;
        let mut cg = MockChunkGenerator;
        let mut rng = SeqRandom::ints(vec![0]);
        let config = SimpleRandomFeatureConfiguration {
            features: vec![Holder(MockFeature::new(false))],
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &mut cg,
            random: &mut rng,
            origin: BlockPos::new(0, 0, 0),
            config: &config,
        };

        assert!(!SimpleRandomSelectorFeature::new(()).place(&mut ctx));
        assert!(config.features[0].0.called.get());
    }
}
