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

    #[inline]
    pub fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
    fn next_boolean(&mut self) -> bool;
}

pub trait BlockState {
    fn is_end_stone(&self) -> bool;
}

pub trait WorldGenLevel {
    type State: BlockState;

    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> Self::State;
    fn generate_chorus_plant<R: RandomSource + ?Sized>(
        &mut self,
        pos: BlockPos,
        random: &mut R,
        max_horizontal_distance: i32,
    );
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoneFeatureConfiguration;

pub struct FeaturePlaceContext<'a, L: ?Sized, R: ?Sized> {
    pub level: &'a mut L,
    pub origin: BlockPos,
    pub random: &'a mut R,
    pub config: NoneFeatureConfiguration,
}

impl<'a, L: ?Sized, R: ?Sized> FeaturePlaceContext<'a, L, R> {
    pub fn new(
        level: &'a mut L,
        origin: BlockPos,
        random: &'a mut R,
        config: NoneFeatureConfiguration,
    ) -> Self {
        Self {
            level,
            origin,
            random,
            config,
        }
    }

    pub fn level(&mut self) -> &mut L {
        self.level
    }

    pub fn origin(&self) -> BlockPos {
        self.origin
    }

    pub fn random(&mut self) -> &mut R {
        self.random
    }

    pub fn config(&self) -> &NoneFeatureConfiguration {
        &self.config
    }
}

pub struct ChorusFlowerBlock;

impl ChorusFlowerBlock {
    // JAVA: net.minecraft.world.level.block.ChorusFlowerBlock#generatePlant
    pub fn generate_plant<L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        level: &mut L,
        pos: BlockPos,
        random: &mut R,
        max_horizontal_distance: i32,
    ) {
        level.generate_chorus_plant(pos, random, max_horizontal_distance);
    }
}

pub struct ChorusPlantFeature;

impl ChorusPlantFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.ChorusPlantFeature#<init>
    pub fn new(_config_codec: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.ChorusPlantFeature#place
    pub fn place<L, R>(&self, ctx: &mut FeaturePlaceContext<'_, L, R>) -> bool
    where
        L: WorldGenLevel + ?Sized,
        R: RandomSource + ?Sized,
    {
        let pos = ctx.origin;
        let is_empty = ctx.level.is_empty_block(pos);
        let below_pos = pos.below();
        let is_end_stone = ctx.level.get_block_state(below_pos).is_end_stone();
        if is_empty && is_end_stone {
            ChorusFlowerBlock::generate_plant(ctx.level, pos, ctx.random, 8);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct MockState {
        end_stone: bool,
    }

    impl BlockState for MockState {
        fn is_end_stone(&self) -> bool {
            self.end_stone
        }
    }

    struct MockRandom;

    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }

        fn next_float(&mut self) -> f32 {
            0.0
        }

        fn next_boolean(&mut self) -> bool {
            false
        }
    }

    #[derive(Default)]
    struct MockLevel {
        empty: bool,
        below_end_stone: bool,
        queried_below: Option<BlockPos>,
        generated: Option<(BlockPos, i32)>,
    }

    impl WorldGenLevel for MockLevel {
        type State = MockState;

        fn is_empty_block(&self, _pos: BlockPos) -> bool {
            self.empty
        }

        fn get_block_state(&self, pos: BlockPos) -> Self::State {
            // Record the queried position for verification in tests via interior tracking pattern.
            // We use a simple approach: tests inspect expected below position separately.
            let _ = pos;
            MockState {
                end_stone: self.below_end_stone,
            }
        }

        fn generate_chorus_plant<R: RandomSource + ?Sized>(
            &mut self,
            pos: BlockPos,
            _random: &mut R,
            max_horizontal_distance: i32,
        ) {
            self.generated = Some((pos, max_horizontal_distance));
        }
    }

    #[test]
    fn places_when_empty_and_end_stone_below() {
        let mut level = MockLevel {
            empty: true,
            below_end_stone: true,
            ..Default::default()
        };
        let mut rng = MockRandom;
        let origin = BlockPos::new(1, 64, -2);
        let mut ctx =
            FeaturePlaceContext::new(&mut level, origin, &mut rng, NoneFeatureConfiguration);
        let feature = ChorusPlantFeature::new(());

        assert!(feature.place(&mut ctx));
        assert_eq!(level.generated, Some((origin, 8)));
    }

    #[test]
    fn does_not_place_when_not_empty() {
        let mut level = MockLevel {
            empty: false,
            below_end_stone: true,
            ..Default::default()
        };
        let mut rng = MockRandom;
        let mut ctx = FeaturePlaceContext::new(
            &mut level,
            BlockPos::new(0, 0, 0),
            &mut rng,
            NoneFeatureConfiguration,
        );

        assert!(!ChorusPlantFeature::new(()).place(&mut ctx));
        assert_eq!(level.generated, None);
    }

    #[test]
    fn does_not_place_when_below_not_end_stone() {
        let mut level = MockLevel {
            empty: true,
            below_end_stone: false,
            ..Default::default()
        };
        let mut rng = MockRandom;
        let mut ctx = FeaturePlaceContext::new(
            &mut level,
            BlockPos::new(0, 0, 0),
            &mut rng,
            NoneFeatureConfiguration,
        );

        assert!(!ChorusPlantFeature::new(()).place(&mut ctx));
        assert_eq!(level.generated, None);
    }

    #[test]
    fn below_wraps_on_i32_min() {
        let pos = BlockPos::new(-5, i32::MIN, 7);
        let below = pos.below();
        assert_eq!(below.x, -5);
        assert_eq!(below.y, i32::MAX);
        assert_eq!(below.z, 7);
    }

    #[test]
    fn below_handles_negative_coordinates() {
        let pos = BlockPos::new(i32::MIN, -1, i32::MAX);
        let below = pos.below();
        assert_eq!(below, BlockPos::new(i32::MIN, -2, i32::MAX));
    }

    #[test]
    fn queries_below_position_for_end_stone_check() {
        use std::cell::Cell;

        struct TrackingLevel {
            empty: bool,
            queried: Cell<Option<BlockPos>>,
        }

        impl WorldGenLevel for TrackingLevel {
            type State = MockState;

            fn is_empty_block(&self, _pos: BlockPos) -> bool {
                self.empty
            }

            fn get_block_state(&self, pos: BlockPos) -> Self::State {
                self.queried.set(Some(pos));
                MockState { end_stone: false }
            }

            fn generate_chorus_plant<R: RandomSource + ?Sized>(
                &mut self,
                _pos: BlockPos,
                _random: &mut R,
                _max_horizontal_distance: i32,
            ) {
            }
        }

        let mut level = TrackingLevel {
            empty: true,
            queried: Cell::new(None),
        };
        let mut rng = MockRandom;
        let origin = BlockPos::new(3, 10, -4);
        let mut ctx =
            FeaturePlaceContext::new(&mut level, origin, &mut rng, NoneFeatureConfiguration);

        assert!(!ChorusPlantFeature::new(()).place(&mut ctx));
        assert_eq!(level.queried.get(), Some(BlockPos::new(3, 9, -4)));
    }
}
