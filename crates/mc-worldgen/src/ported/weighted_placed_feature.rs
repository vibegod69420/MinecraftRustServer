#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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

pub struct Holder<T>(pub T);

impl<T> Holder<T> {
    pub fn value(&self) -> &T {
        &self.0
    }
}

pub trait PlacedFeature<L, G, R: ?Sized> {
    fn place(&self, level: &mut L, chunk_generator: &G, random: &mut R, pos: BlockPos) -> bool;
}

pub struct WeightedPlacedFeature<P> {
    pub feature: Holder<P>,
    pub chance: f32,
}

impl<P> WeightedPlacedFeature<P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.WeightedPlacedFeature#<init>
    pub fn new(feature: Holder<P>, chance: f32) -> Self {
        Self { feature, chance }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WeightedPlacedFeature#place
    pub fn place<L, G, R: ?Sized>(
        &self,
        level: &mut L,
        chunk_generator: &G,
        random: &mut R,
        pos: BlockPos,
    ) -> bool
    where
        P: PlacedFeature<L, G, R>,
    {
        self.feature
            .value()
            .place(level, chunk_generator, random, pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct MockPlaced {
        result: bool,
        last_pos: Cell<BlockPos>,
    }

    impl MockPlaced {
        fn new(result: bool) -> Self {
            Self {
                result,
                last_pos: Cell::new(BlockPos::new(0, 0, 0)),
            }
        }
    }

    impl<L, G, R: ?Sized> PlacedFeature<L, G, R> for MockPlaced {
        fn place(&self, _level: &mut L, _chunk_generator: &G, _random: &mut R, pos: BlockPos) -> bool {
            self.last_pos.set(pos);
            self.result
        }
    }

    #[test]
    fn place_delegates_and_returns_true() {
        let f = WeightedPlacedFeature::new(Holder(MockPlaced::new(true)), 0.75_f32);
        let mut level = ();
        let chunk_gen = ();
        let mut rng = ();
        assert!(f.place(&mut level, &chunk_gen, &mut rng, BlockPos::new(1, 2, 3)));
        assert_eq!(f.feature.value().last_pos.get(), BlockPos::new(1, 2, 3));
    }

    #[test]
    fn place_delegates_and_returns_false() {
        let f = WeightedPlacedFeature::new(Holder(MockPlaced::new(false)), 0.25_f32);
        let mut level = ();
        let chunk_gen = ();
        let mut rng = ();
        assert!(!f.place(&mut level, &chunk_gen, &mut rng, BlockPos::new(-10, -20, -30)));
        assert_eq!(
            f.feature.value().last_pos.get(),
            BlockPos::new(-10, -20, -30)
        );
    }

    #[test]
    fn chance_preserves_float_bits() {
        let nan_feature = WeightedPlacedFeature::new(Holder(MockPlaced::new(true)), f32::NAN);
        assert!(nan_feature.chance.is_nan());

        let neg_zero = -0.0_f32;
        let nz_feature = WeightedPlacedFeature::new(Holder(MockPlaced::new(true)), neg_zero);
        assert_eq!(nz_feature.chance.to_bits(), neg_zero.to_bits());
        assert_ne!(nz_feature.chance.to_bits(), 0.0_f32.to_bits());

        let inf_feature = WeightedPlacedFeature::new(Holder(MockPlaced::new(true)), f32::INFINITY);
        assert_eq!(inf_feature.chance.to_bits(), f32::INFINITY.to_bits());
    }

    #[test]
    fn negative_and_extreme_coordinates_pass_through() {
        let f = WeightedPlacedFeature::new(Holder(MockPlaced::new(true)), 1.0_f32);
        let mut level = ();
        let chunk_gen = ();
        let mut rng = ();
        let pos = BlockPos::new(i32::MIN, -1, i32::MAX);
        assert!(f.place(&mut level, &chunk_gen, &mut rng, pos));
        assert_eq!(f.feature.value().last_pos.get(), pos);
    }
}
