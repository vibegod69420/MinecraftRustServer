use std::marker::PhantomData;

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

#[derive(Debug, Clone, Copy, Default)]
pub struct PlacementContext;

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlacementModifierType {
    RarityFilter,
    InSquare,
    Other(u32),
}

/// Placeholder for Mojang MapCodec; serialization framework not ported.
#[derive(Debug, Clone, Copy)]
pub struct MapCodec<T>(pub PhantomData<T>);

impl<T> MapCodec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RarityFilter {
    pub chance: i32,
}

/// Placeholder for ExtraCodecs.POSITIVE_INT.fieldOf("chance").xmap(RarityFilter::new, $$0 -> $$0.chance)
/// JAVA: net.minecraft.world.level.levelgen.placement.RarityFilter#CODEC
pub const RARITY_FILTER_CODEC: MapCodec<RarityFilter> = MapCodec::placeholder();

impl RarityFilter {
    // JAVA: net.minecraft.world.level.levelgen.placement.RarityFilter#<init>
    fn new(chance: i32) -> Self {
        Self { chance }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.RarityFilter#onAverageOnceEvery
    pub fn on_average_once_every(chance: i32) -> Self {
        Self::new(chance)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.RarityFilter#shouldPlace
    pub fn should_place(&self, _ctx: &PlacementContext, random: &mut dyn RandomSource, _pos: BlockPos) -> bool {
        random.next_float() < (1.0f32 / self.chance as f32)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.RarityFilter#type
    pub fn modifier_type(&self) -> PlacementModifierType {
        PlacementModifierType::RarityFilter
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementFilter#getPositions
    pub fn get_positions(
        &self,
        ctx: &PlacementContext,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        if self.should_place(ctx, random, pos) {
            vec![pos]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRandom {
        float_val: f32,
    }
    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
        fn next_float(&mut self) -> f32 {
            self.float_val
        }
    }

    #[test]
    fn places_when_below_threshold() {
        let f = RarityFilter::on_average_once_every(4); // threshold 0.25
        let mut rng = MockRandom { float_val: 0.1 };
        let pos = BlockPos::new(1, 2, 3);
        assert!(f.should_place(&PlacementContext, &mut rng, pos));
        let mut rng2 = MockRandom { float_val: 0.1 };
        assert_eq!(f.get_positions(&PlacementContext, &mut rng2, pos), vec![pos]);
    }

    #[test]
    fn rejects_when_at_or_above_threshold() {
        let f = RarityFilter::on_average_once_every(4); // threshold 0.25
        let mut rng = MockRandom { float_val: 0.25 };
        let pos = BlockPos::new(0, 0, 0);
        assert!(!f.should_place(&PlacementContext, &mut rng, pos));
        let mut rng2 = MockRandom { float_val: 0.9 };
        assert!(f.get_positions(&PlacementContext, &mut rng2, pos).is_empty());
    }

    #[test]
    fn chance_one_always_places_for_small_float() {
        let f = RarityFilter::on_average_once_every(1); // threshold 1.0
        let mut rng = MockRandom { float_val: 0.999_999_94 }; // < 1.0f32
        assert!(f.should_place(&PlacementContext, &mut rng, BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn type_is_rarity_filter() {
        let f = RarityFilter::on_average_once_every(10);
        assert_eq!(f.modifier_type(), PlacementModifierType::RarityFilter);
    }

    #[test]
    fn float_bits_boundary() {
        // 1.0f32 / 2 = 0.5f32 exactly
        let f = RarityFilter::on_average_once_every(2);
        let mut rng_eq = MockRandom { float_val: 0.5 };
        assert!(!f.should_place(&PlacementContext, &mut rng_eq, BlockPos::new(0, 0, 0)));
        let mut rng_below = MockRandom {
            float_val: f32::from_bits(0x3eff_ffff), // just below 0.5
        };
        assert!(f.should_place(&PlacementContext, &mut rng_below, BlockPos::new(0, 0, 0)));
    }
}
