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

    pub fn at_y(&self, y: i32) -> Self {
        Self {
            x: self.x,
            y,
            z: self.z,
        }
    }
}

pub trait RandomSource {
    fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementContext {
    pub min_gen_y: i32,
    pub height: i32,
}

impl PlacementContext {
    pub fn new(min_gen_y: i32, height: i32) -> Self {
        Self { min_gen_y, height }
    }

    pub fn min_gen_y(&self) -> i32 {
        self.min_gen_y
    }

    pub fn max_gen_y(&self) -> i32 {
        self.min_gen_y
            .wrapping_add(self.height)
            .wrapping_sub(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAnchor {
    Absolute(i32),
    AboveBottom(i32),
    BelowTop(i32),
}

impl VerticalAnchor {
    pub fn absolute(value: i32) -> Self {
        Self::Absolute(value)
    }

    pub fn above_bottom(offset: i32) -> Self {
        Self::AboveBottom(offset)
    }

    pub fn below_top(offset: i32) -> Self {
        Self::BelowTop(offset)
    }

    pub fn resolve_y(&self, context: &PlacementContext) -> i32 {
        match *self {
            Self::Absolute(y) => y,
            Self::AboveBottom(offset) => context.min_gen_y().wrapping_add(offset),
            Self::BelowTop(offset) => context.max_gen_y().wrapping_sub(offset),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformHeight {
    min_inclusive: VerticalAnchor,
    max_inclusive: VerticalAnchor,
}

impl UniformHeight {
    pub fn of(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    fn sample(&self, random: &mut dyn RandomSource, context: &PlacementContext) -> i32 {
        let mut min_y = self.min_inclusive.resolve_y(context);
        let mut max_y = self.max_inclusive.resolve_y(context);
        if min_y > max_y {
            std::mem::swap(&mut min_y, &mut max_y);
        }
        random.next_int_between_inclusive(min_y, max_y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrapezoidHeight {
    min_inclusive: VerticalAnchor,
    max_inclusive: VerticalAnchor,
}

impl TrapezoidHeight {
    pub fn of(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    fn sample(&self, random: &mut dyn RandomSource, context: &PlacementContext) -> i32 {
        let mut min_y = self.min_inclusive.resolve_y(context);
        let mut max_y = self.max_inclusive.resolve_y(context);
        if min_y > max_y {
            std::mem::swap(&mut min_y, &mut max_y);
        }
        if min_y == max_y {
            return min_y;
        }
        let range = max_y.wrapping_sub(min_y);
        let mid = range / 2;
        let a = random.next_int_between_inclusive(0, mid);
        let b = random.next_int_between_inclusive(0, range.wrapping_sub(mid));
        min_y.wrapping_add(a).wrapping_add(b)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeightProvider {
    Uniform(UniformHeight),
    Trapezoid(TrapezoidHeight),
}

impl HeightProvider {
    pub fn sample(&self, random: &mut dyn RandomSource, context: &PlacementContext) -> i32 {
        match self {
            Self::Uniform(h) => h.sample(random, context),
            Self::Trapezoid(h) => h.sample(random, context),
        }
    }
}

impl From<UniformHeight> for HeightProvider {
    fn from(value: UniformHeight) -> Self {
        Self::Uniform(value)
    }
}

impl From<TrapezoidHeight> for HeightProvider {
    fn from(value: TrapezoidHeight) -> Self {
        Self::Trapezoid(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifierType {
    HeightRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightRangePlacement {
    height: HeightProvider,
}

impl HeightRangePlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.HeightRangePlacement#<init>
    fn new(height: HeightProvider) -> Self {
        Self { height }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.HeightRangePlacement#of
    pub fn of(height: impl Into<HeightProvider>) -> Self {
        Self::new(height.into())
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.HeightRangePlacement#uniform
    pub fn uniform(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> Self {
        Self::of(UniformHeight::of(min_inclusive, max_inclusive))
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.HeightRangePlacement#triangle
    pub fn triangle(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> Self {
        Self::of(TrapezoidHeight::of(min_inclusive, max_inclusive))
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.HeightRangePlacement#getPositions
    pub fn get_positions(
        &self,
        context: &PlacementContext,
        random: &mut dyn RandomSource,
        pos: &BlockPos,
    ) -> impl Iterator<Item = BlockPos> {
        let y = self.height.sample(random, context);
        std::iter::once(pos.at_y(y))
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.HeightRangePlacement#type
    pub fn placement_type(&self) -> PlacementModifierType {
        PlacementModifierType::HeightRange
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MinRandom;
    impl RandomSource for MinRandom {
        fn next_int_between_inclusive(&mut self, min: i32, _max: i32) -> i32 {
            min
        }
    }

    struct MaxRandom;
    impl RandomSource for MaxRandom {
        fn next_int_between_inclusive(&mut self, _min: i32, max: i32) -> i32 {
            max
        }
    }

    fn ctx() -> PlacementContext {
        PlacementContext::new(-64, 384)
    }

    #[test]
    fn get_positions_sets_y_from_height_provider() {
        let placement = HeightRangePlacement::uniform(
            VerticalAnchor::absolute(10),
            VerticalAnchor::absolute(10),
        );
        let pos = BlockPos::new(3, 64, -7);
        let mut rng = MinRandom;
        let positions: Vec<BlockPos> =
            placement.get_positions(&ctx(), &mut rng, &pos).collect();
        assert_eq!(positions, vec![BlockPos::new(3, 10, -7)]);
    }

    #[test]
    fn get_positions_preserves_xz_and_uses_sampled_y() {
        let placement = HeightRangePlacement::uniform(
            VerticalAnchor::absolute(-16),
            VerticalAnchor::absolute(32),
        );
        let pos = BlockPos::new(i32::MIN, 0, i32::MAX);
        let mut rng = MaxRandom;
        let positions: Vec<BlockPos> =
            placement.get_positions(&ctx(), &mut rng, &pos).collect();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].x, i32::MIN);
        assert_eq!(positions[0].z, i32::MAX);
        assert_eq!(positions[0].y, 32);
    }

    #[test]
    fn uniform_factory_uses_vertical_anchor_resolution() {
        let placement = HeightRangePlacement::uniform(
            VerticalAnchor::above_bottom(0),
            VerticalAnchor::below_top(0),
        );
        let context = ctx();
        let mut rng_min = MinRandom;
        let min_pos: Vec<BlockPos> = placement
            .get_positions(&context, &mut rng_min, &BlockPos::new(0, 0, 0))
            .collect();
        assert_eq!(min_pos[0].y, -64);

        let mut rng_max = MaxRandom;
        let max_pos: Vec<BlockPos> = placement
            .get_positions(&context, &mut rng_max, &BlockPos::new(0, 0, 0))
            .collect();
        assert_eq!(max_pos[0].y, 319);
    }

    #[test]
    fn triangle_factory_produces_single_position() {
        let placement = HeightRangePlacement::triangle(
            VerticalAnchor::absolute(0),
            VerticalAnchor::absolute(0),
        );
        let mut rng = MinRandom;
        let positions: Vec<BlockPos> = placement
            .get_positions(&ctx(), &mut rng, &BlockPos::new(5, 1, 9))
            .collect();
        assert_eq!(positions, vec![BlockPos::new(5, 0, 9)]);
    }

    #[test]
    fn placement_type_is_height_range() {
        let placement = HeightRangePlacement::uniform(
            VerticalAnchor::absolute(0),
            VerticalAnchor::absolute(1),
        );
        assert_eq!(placement.placement_type(), PlacementModifierType::HeightRange);
    }

    #[test]
    fn negative_and_extreme_y_values_are_preserved() {
        let placement = HeightRangePlacement::uniform(
            VerticalAnchor::absolute(i32::MIN),
            VerticalAnchor::absolute(i32::MIN),
        );
        let mut rng = MinRandom;
        let positions: Vec<BlockPos> = placement
            .get_positions(&ctx(), &mut rng, &BlockPos::new(-1, -2, -3))
            .collect();
        assert_eq!(positions[0], BlockPos::new(-1, i32::MIN, -3));

        let placement_max = HeightRangePlacement::uniform(
            VerticalAnchor::absolute(i32::MAX),
            VerticalAnchor::absolute(i32::MAX),
        );
        let mut rng2 = MaxRandom;
        let positions_max: Vec<BlockPos> = placement_max
            .get_positions(&ctx(), &mut rng2, &BlockPos::new(0, 0, 0))
            .collect();
        assert_eq!(positions_max[0].y, i32::MAX);
    }
}
