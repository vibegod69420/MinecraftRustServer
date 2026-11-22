//! Port of `net.minecraft.world.level.levelgen.placement.BlockPredicateFilter`.

use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

pub trait RandomSource: Debug {}

#[derive(Debug, Default)]
pub struct RandomSourceStub;

impl RandomSource for RandomSourceStub {}

#[derive(Debug, Default)]
pub struct Level;

#[derive(Debug)]
pub struct PlacementContext<'a> {
    level: &'a Level,
}

impl<'a> PlacementContext<'a> {
    pub fn new(level: &'a Level) -> Self {
        Self { level }
    }

    pub fn get_level(&self) -> &Level {
        self.level
    }
}

pub trait BlockPredicate: Debug {
    fn test(&self, level: &Level, pos: BlockPos) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifierType {
    BlockPredicateFilter,
}

/// Port of `net.minecraft.world.level.levelgen.placement.BlockPredicateFilter`.
///
/// Extends `PlacementFilter` in Java; the inherited filtering behavior is provided by
/// `get_positions`.
#[derive(Debug)]
pub struct BlockPredicateFilter<P> {
    predicate: P,
}

impl<P: BlockPredicate> BlockPredicateFilter<P> {
    // JAVA: net.minecraft.world.level.levelgen.placement.BlockPredicateFilter#BlockPredicateFilter
    fn new(predicate: P) -> Self {
        Self { predicate }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.BlockPredicateFilter#forPredicate
    pub fn for_predicate(predicate: P) -> Self {
        Self::new(predicate)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.BlockPredicateFilter#shouldPlace
    pub fn should_place(
        &self,
        ctx: &PlacementContext<'_>,
        _random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> bool {
        self.predicate.test(ctx.get_level(), pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementFilter#getPositions
    pub fn get_positions(
        &self,
        ctx: &PlacementContext<'_>,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        if self.should_place(ctx, random, pos) {
            vec![pos]
        } else {
            Vec::new()
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.BlockPredicateFilter#type
    pub fn type_(&self) -> PlacementModifierType {
        PlacementModifierType::BlockPredicateFilter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct AlwaysTrue;
    impl BlockPredicate for AlwaysTrue {
        fn test(&self, _level: &Level, _pos: BlockPos) -> bool {
            true
        }
    }

    #[derive(Debug)]
    struct AlwaysFalse;
    impl BlockPredicate for AlwaysFalse {
        fn test(&self, _level: &Level, _pos: BlockPos) -> bool {
            false
        }
    }

    #[derive(Debug)]
    struct YIsEven;
    impl BlockPredicate for YIsEven {
        fn test(&self, _level: &Level, pos: BlockPos) -> bool {
            pos.get_y() % 2 == 0
        }
    }

    #[test]
    fn should_place_delegates_to_predicate() {
        let level = Level;
        let ctx = PlacementContext::new(&level);
        let mut rng = RandomSourceStub;
        let pos = BlockPos::new(1, 2, 3);

        assert!(
            BlockPredicateFilter::for_predicate(AlwaysTrue).should_place(&ctx, &mut rng, pos)
        );
        assert!(
            !BlockPredicateFilter::for_predicate(AlwaysFalse).should_place(&ctx, &mut rng, pos)
        );
    }

    #[test]
    fn should_place_with_negative_and_extreme_coordinates() {
        let level = Level;
        let ctx = PlacementContext::new(&level);
        let mut rng = RandomSourceStub;
        let filter = BlockPredicateFilter::for_predicate(YIsEven);

        assert!(filter.should_place(&ctx, &mut rng, BlockPos::new(-10, -4, -20)));
        assert!(!filter.should_place(&ctx, &mut rng, BlockPos::new(i32::MIN, i32::MAX, 0)));
        assert!(filter.should_place(&ctx, &mut rng, BlockPos::new(i32::MAX, i32::MIN, i32::MIN)));
    }

    #[test]
    fn get_positions_filters_by_predicate() {
        let level = Level;
        let ctx = PlacementContext::new(&level);
        let mut rng = RandomSourceStub;
        let pos = BlockPos::new(5, 64, -7);

        let kept =
            BlockPredicateFilter::for_predicate(AlwaysTrue).get_positions(&ctx, &mut rng, pos);
        assert_eq!(kept, vec![pos]);

        let dropped =
            BlockPredicateFilter::for_predicate(AlwaysFalse).get_positions(&ctx, &mut rng, pos);
        assert!(dropped.is_empty());
    }

    #[test]
    fn type_returns_block_predicate_filter() {
        let filter = BlockPredicateFilter::for_predicate(AlwaysTrue);
        assert_eq!(
            filter.type_(),
            PlacementModifierType::BlockPredicateFilter
        );
    }
}
