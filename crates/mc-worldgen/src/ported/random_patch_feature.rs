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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#setWithOffset
    pub fn set_with_offset(&mut self, base: BlockPos, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = base.x.wrapping_add(dx);
        self.y = base.y.wrapping_add(dy);
        self.z = base.z.wrapping_add(dz);
        self
    }

    pub fn immutable(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

impl Default for MutableBlockPos {
    fn default() -> Self {
        Self::new()
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub struct Holder<T>(pub T);

impl<T> Holder<T> {
    pub fn value(&self) -> &T {
        &self.0
    }
}

pub trait PlacedFeatureValue<L, G, R: RandomSource + ?Sized> {
    fn place(&self, level: &mut L, chunk_generator: &G, random: &mut R, pos: BlockPos) -> bool;
}

pub struct RandomPatchConfiguration<P> {
    tries: i32,
    xz_spread: i32,
    y_spread: i32,
    feature: Holder<P>,
}

impl<P> RandomPatchConfiguration<P> {
    pub fn new(tries: i32, xz_spread: i32, y_spread: i32, feature: Holder<P>) -> Self {
        Self {
            tries,
            xz_spread,
            y_spread,
            feature,
        }
    }

    pub fn tries(&self) -> i32 {
        self.tries
    }

    pub fn xz_spread(&self) -> i32 {
        self.xz_spread
    }

    pub fn y_spread(&self) -> i32 {
        self.y_spread
    }

    pub fn feature(&self) -> &Holder<P> {
        &self.feature
    }
}

pub struct FeaturePlaceContext<'a, L, G, R: RandomSource + ?Sized, C> {
    pub config: &'a C,
    pub origin: BlockPos,
    pub level: &'a mut L,
    pub chunk_generator: &'a G,
    pub random: &'a mut R,
}

pub struct RandomPatchFeature;

impl RandomPatchFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.RandomPatchFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RandomPatchFeature#place
    pub fn place<L, G, R, P>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, G, R, RandomPatchConfiguration<P>>,
    ) -> bool
    where
        R: RandomSource + ?Sized,
        P: PlacedFeatureValue<L, G, R>,
    {
        let config = ctx.config;
        let origin = ctx.origin;
        let mut success_count: i32 = 0;
        let mut pos = MutableBlockPos::new();
        let xz_spread_bound = config.xz_spread().wrapping_add(1);
        let y_spread_bound = config.y_spread().wrapping_add(1);

        let mut i: i32 = 0;
        while i < config.tries() {
            pos.set_with_offset(
                origin,
                ctx.random
                    .next_int(xz_spread_bound)
                    .wrapping_sub(ctx.random.next_int(xz_spread_bound)),
                ctx.random
                    .next_int(y_spread_bound)
                    .wrapping_sub(ctx.random.next_int(y_spread_bound)),
                ctx.random
                    .next_int(xz_spread_bound)
                    .wrapping_sub(ctx.random.next_int(xz_spread_bound)),
            );
            if config.feature().value().place(
                ctx.level,
                ctx.chunk_generator,
                ctx.random,
                pos.immutable(),
            ) {
                success_count = success_count.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        success_count > 0
    }
}

impl Default for RandomPatchFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    struct MockFeature {
        result: bool,
        positions: RefCell<Vec<BlockPos>>,
    }

    impl MockFeature {
        fn new(result: bool) -> Self {
            Self {
                result,
                positions: RefCell::new(Vec::new()),
            }
        }
    }

    impl<L, G, R: RandomSource + ?Sized> PlacedFeatureValue<L, G, R> for MockFeature {
        fn place(&self, _level: &mut L, _cg: &G, _random: &mut R, pos: BlockPos) -> bool {
            self.positions.borrow_mut().push(pos);
            self.result
        }
    }

    #[test]
    fn place_returns_true_when_any_succeeds() {
        let feature = MockFeature::new(true);
        let config = RandomPatchConfiguration::new(3, 4, 2, Holder(feature));
        let mut level = ();
        let chunk_gen = ();
        let mut rng = SeqRandom::new(vec![0; 32]);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin: BlockPos::new(10, 64, 20),
            level: &mut level,
            chunk_generator: &chunk_gen,
            random: &mut rng,
        };
        assert!(RandomPatchFeature::new().place(&mut ctx));
        assert_eq!(ctx.config.feature().value().positions.borrow().len(), 3);
    }

    #[test]
    fn place_returns_false_when_none_succeed() {
        let feature = MockFeature::new(false);
        let config = RandomPatchConfiguration::new(5, 3, 3, Holder(feature));
        let mut level = ();
        let chunk_gen = ();
        let mut rng = SeqRandom::new(vec![0; 64]);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin: BlockPos::new(0, 0, 0),
            level: &mut level,
            chunk_generator: &chunk_gen,
            random: &mut rng,
        };
        assert!(!RandomPatchFeature::new().place(&mut ctx));
        assert_eq!(ctx.config.feature().value().positions.borrow().len(), 5);
    }

    #[test]
    fn offset_calculation_and_negative_inputs() {
        let feature = MockFeature::new(true);
        // xz_spread=1 -> bound 2, y_spread=0 -> bound 1
        let config = RandomPatchConfiguration::new(1, 1, 0, Holder(feature));
        let mut level = ();
        let chunk_gen = ();
        // dx: 1-0 => 1; dy: 0-0 => 0; dz: 0-1 => -1
        let mut rng = SeqRandom::new(vec![1, 0, 0, 0, 0, 1]);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin: BlockPos::new(5, -10, 15),
            level: &mut level,
            chunk_generator: &chunk_gen,
            random: &mut rng,
        };
        assert!(RandomPatchFeature::new().place(&mut ctx));
        let positions = ctx.config.feature().value().positions.borrow();
        assert_eq!(positions[0], BlockPos::new(6, -10, 14));
    }

    #[test]
    fn overflow_wrapping_in_spread_and_offset() {
        let feature = MockFeature::new(true);
        let config = RandomPatchConfiguration::new(1, i32::MAX, 0, Holder(feature));
        let mut level = ();
        let chunk_gen = ();
        // xz bound wraps to i32::MIN; mock returns 0 for bound <= 0
        let mut rng = SeqRandom::new(vec![0; 8]);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin: BlockPos::new(i32::MAX, 0, i32::MIN),
            level: &mut level,
            chunk_generator: &chunk_gen,
            random: &mut rng,
        };
        assert!(RandomPatchFeature::new().place(&mut ctx));
        let positions = ctx.config.feature().value().positions.borrow();
        assert_eq!(positions[0], BlockPos::new(i32::MAX, 0, i32::MIN));
    }

    #[test]
    fn zero_or_negative_tries_returns_false() {
        let feature = MockFeature::new(true);
        let config = RandomPatchConfiguration::new(0, 3, 3, Holder(feature));
        let mut level = ();
        let chunk_gen = ();
        let mut rng = SeqRandom::new(vec![0; 8]);
        let mut ctx = FeaturePlaceContext {
            config: &config,
            origin: BlockPos::new(0, 0, 0),
            level: &mut level,
            chunk_generator: &chunk_gen,
            random: &mut rng,
        };
        assert!(!RandomPatchFeature::new().place(&mut ctx));
        assert!(ctx.config.feature().value().positions.borrow().is_empty());

        let feature2 = MockFeature::new(true);
        let config2 = RandomPatchConfiguration::new(-5, 3, 3, Holder(feature2));
        let mut rng2 = SeqRandom::new(vec![0; 8]);
        let mut ctx2 = FeaturePlaceContext {
            config: &config2,
            origin: BlockPos::new(0, 0, 0),
            level: &mut level,
            chunk_generator: &chunk_gen,
            random: &mut rng2,
        };
        assert!(!RandomPatchFeature::new().place(&mut ctx2));
        assert!(ctx2.config.feature().value().positions.borrow().is_empty());
    }
}
