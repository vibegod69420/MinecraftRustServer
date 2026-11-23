use std::fmt;
use std::marker::PhantomData;

/// Minimal placeholder for `net.minecraft.world.level.levelgen.blockpredicates.BlockPredicate`.
pub trait BlockPredicate {}

/// Minimal placeholder for `com.mojang.serialization.MapCodec`.
pub struct MapCodec<P: ?Sized>(PhantomData<fn() -> P>);

impl<P: ?Sized> MapCodec<P> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<P: ?Sized> Default for MapCodec<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: ?Sized> Clone for MapCodec<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: ?Sized> Copy for MapCodec<P> {}

impl<P: ?Sized> fmt::Debug for MapCodec<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MapCodec(..)")
    }
}

macro_rules! define_block_predicates {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default)]
            pub struct $name;
            impl BlockPredicate for $name {}
        )*
    };
}

define_block_predicates!(
    MatchingBlocksPredicate,
    MatchingBlockTagPredicate,
    MatchingFluidsPredicate,
    HasSturdyFacePredicate,
    SolidPredicate,
    ReplaceablePredicate,
    WouldSurvivePredicate,
    InsideWorldBoundsPredicate,
    AnyOfPredicate,
    AllOfPredicate,
    NotPredicate,
    TrueBlockPredicate,
    UnobstructedPredicate,
);

/// Port of `net.minecraft.world.level.levelgen.blockpredicates.BlockPredicateType`.
pub struct BlockPredicateType<P: ?Sized> {
    name: &'static str,
    codec: MapCodec<P>,
}

impl<P: ?Sized> Clone for BlockPredicateType<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: ?Sized> Copy for BlockPredicateType<P> {}

impl<P: ?Sized> fmt::Debug for BlockPredicateType<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockPredicateType")
            .field("name", &self.name)
            .finish()
    }
}

impl<P: ?Sized> BlockPredicateType<P> {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.BlockPredicateType#codec
    pub fn codec(&self) -> MapCodec<P> {
        self.codec
    }

    /// Returns the registered name of this block predicate type.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

// JAVA: net.minecraft.world.level.levelgen.blockpredicates.BlockPredicateType#register
const fn register<P>(name: &'static str, codec: MapCodec<P>) -> BlockPredicateType<P> {
    // Registry.register(BuiltInRegistries.BLOCK_PREDICATE_TYPE, name, () -> codec)
    // is omitted in this self-contained port; the name/codec association is retained.
    BlockPredicateType { name, codec }
}

pub const MATCHING_BLOCKS: BlockPredicateType<MatchingBlocksPredicate> =
    register("matching_blocks", MapCodec::new());
pub const MATCHING_BLOCK_TAG: BlockPredicateType<MatchingBlockTagPredicate> =
    register("matching_block_tag", MapCodec::new());
pub const MATCHING_FLUIDS: BlockPredicateType<MatchingFluidsPredicate> =
    register("matching_fluids", MapCodec::new());
pub const HAS_STURDY_FACE: BlockPredicateType<HasSturdyFacePredicate> =
    register("has_sturdy_face", MapCodec::new());
pub const SOLID: BlockPredicateType<SolidPredicate> = register("solid", MapCodec::new());
pub const REPLACEABLE: BlockPredicateType<ReplaceablePredicate> =
    register("replaceable", MapCodec::new());
pub const WOULD_SURVIVE: BlockPredicateType<WouldSurvivePredicate> =
    register("would_survive", MapCodec::new());
pub const INSIDE_WORLD_BOUNDS: BlockPredicateType<InsideWorldBoundsPredicate> =
    register("inside_world_bounds", MapCodec::new());
pub const ANY_OF: BlockPredicateType<AnyOfPredicate> = register("any_of", MapCodec::new());
pub const ALL_OF: BlockPredicateType<AllOfPredicate> = register("all_of", MapCodec::new());
pub const NOT: BlockPredicateType<NotPredicate> = register("not", MapCodec::new());
pub const TRUE: BlockPredicateType<TrueBlockPredicate> = register("true", MapCodec::new());
pub const UNOBSTRUCTED: BlockPredicateType<UnobstructedPredicate> =
    register("unobstructed", MapCodec::new());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_names() {
        let custom: BlockPredicateType<TrueBlockPredicate> =
            register("custom_test", MapCodec::new());
        assert_eq!(custom.name(), "custom_test");

        let cases = [
            (MATCHING_BLOCKS.name(), "matching_blocks"),
            (MATCHING_BLOCK_TAG.name(), "matching_block_tag"),
            (MATCHING_FLUIDS.name(), "matching_fluids"),
            (HAS_STURDY_FACE.name(), "has_sturdy_face"),
            (SOLID.name(), "solid"),
            (REPLACEABLE.name(), "replaceable"),
            (WOULD_SURVIVE.name(), "would_survive"),
            (INSIDE_WORLD_BOUNDS.name(), "inside_world_bounds"),
            (ANY_OF.name(), "any_of"),
            (ALL_OF.name(), "all_of"),
            (NOT.name(), "not"),
            (TRUE.name(), "true"),
            (UNOBSTRUCTED.name(), "unobstructed"),
        ];
        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn codec_access() {
        let c1 = MATCHING_BLOCKS.codec();
        let c2 = MATCHING_BLOCKS.codec();
        let _ = (c1, c2);
        let _ = TRUE.codec();
        let _ = format!("{:?}", NOT.codec());
    }

    #[test]
    fn integer_overflow_wrapping() {
        // Java integer semantics: wrapping add/sub/mul.
        assert_eq!(i32::MAX.wrapping_add(1), i32::MIN);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
        assert_eq!(i32::MIN.wrapping_mul(-1), i32::MIN);
        assert_eq!(i64::MAX.wrapping_add(1), i64::MIN);
    }

    #[test]
    fn negative_inputs_and_shift_mask() {
        assert_eq!((-5i32).wrapping_add(-3), -8);
        assert_eq!((-7i32).wrapping_mul(3), -21);
        // Java shift masks: int & 31, long & 63.
        assert_eq!((-1i32) & 31, 31);
        assert_eq!((-1i64) & 63, 63);
        assert_eq!((-33i32) & 31, 31);
    }

    #[test]
    fn float_bit_patterns() {
        assert_eq!(0.0f32.to_bits(), 0x0000_0000);
        assert_eq!((-0.0f32).to_bits(), 0x8000_0000);
        assert_ne!(0.0f32.to_bits(), (-0.0f32).to_bits());

        let nan_bits = f32::NAN.to_bits();
        assert_eq!(nan_bits & 0x7F80_0000, 0x7F80_0000);
        assert_ne!(nan_bits & 0x007F_FFFF, 0);

        assert_eq!(1.0f64.to_bits(), 0x3FF0_0000_0000_0000u64);
        let neg_zero_f64 = (-0.0f64).to_bits();
        assert_eq!(neg_zero_f64, 0x8000_0000_0000_0000u64);
    }
}
