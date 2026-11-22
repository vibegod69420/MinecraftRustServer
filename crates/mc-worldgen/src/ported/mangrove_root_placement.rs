use std::marker::PhantomData;

/// Placeholder for `net.minecraft.world.level.block.Block`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block;

/// Placeholder for `net.minecraft.core.HolderSet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T> {
    tag_name: &'static str,
    _marker: PhantomData<T>,
}

impl<T> HolderSet<T> {
    /// Creates a named placeholder holder set for self-contained porting.
    pub fn named(tag_name: &'static str) -> Self {
        Self {
            tag_name,
            _marker: PhantomData,
        }
    }

    /// Returns the placeholder tag/name.
    pub fn tag_name(&self) -> &'static str {
        self.tag_name
    }
}

/// Placeholder for `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStateProvider;

/// Port of `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement`.
///
/// The Mojang `Codec` field is omitted because codec infrastructure is not yet ported.
#[derive(Debug, Clone, PartialEq)]
pub struct MangroveRootPlacement {
    can_grow_through: HolderSet<Block>,
    muddy_roots_in: HolderSet<Block>,
    muddy_roots_provider: BlockStateProvider,
    max_root_width: i32,
    max_root_length: i32,
    random_skew_chance: f32,
}

impl MangroveRootPlacement {
    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement#<init>
    pub fn new(
        can_grow_through: HolderSet<Block>,
        muddy_roots_in: HolderSet<Block>,
        muddy_roots_provider: BlockStateProvider,
        max_root_width: i32,
        max_root_length: i32,
        random_skew_chance: f32,
    ) -> Self {
        Self {
            can_grow_through,
            muddy_roots_in,
            muddy_roots_provider,
            max_root_width,
            max_root_length,
            random_skew_chance,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement#canGrowThrough
    pub fn can_grow_through(&self) -> &HolderSet<Block> {
        &self.can_grow_through
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement#muddyRootsIn
    pub fn muddy_roots_in(&self) -> &HolderSet<Block> {
        &self.muddy_roots_in
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement#muddyRootsProvider
    pub fn muddy_roots_provider(&self) -> &BlockStateProvider {
        &self.muddy_roots_provider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement#maxRootWidth
    pub fn max_root_width(&self) -> i32 {
        self.max_root_width
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement#maxRootLength
    pub fn max_root_length(&self) -> i32 {
        self.max_root_length
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement#randomSkewChance
    pub fn random_skew_chance(&self) -> f32 {
        self.random_skew_chance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MangroveRootPlacement {
        MangroveRootPlacement::new(
            HolderSet::named("can_grow_through"),
            HolderSet::named("muddy_roots_in"),
            BlockStateProvider,
            4,
            8,
            0.5_f32,
        )
    }

    #[test]
    fn new_stores_all_fields() {
        let placement = sample();
        assert_eq!(placement.can_grow_through().tag_name(), "can_grow_through");
        assert_eq!(placement.muddy_roots_in().tag_name(), "muddy_roots_in");
        assert_eq!(placement.muddy_roots_provider(), &BlockStateProvider);
        assert_eq!(placement.max_root_width(), 4);
        assert_eq!(placement.max_root_length(), 8);
        assert_eq!(
            placement.random_skew_chance().to_bits(),
            0.5_f32.to_bits()
        );
    }

    #[test]
    fn codec_boundary_values_are_preserved() {
        let placement = MangroveRootPlacement::new(
            HolderSet::named("a"),
            HolderSet::named("b"),
            BlockStateProvider,
            1,
            64,
            1.0_f32,
        );
        assert_eq!(placement.max_root_width(), 1);
        assert_eq!(placement.max_root_length(), 64);
        assert_eq!(
            placement.random_skew_chance().to_bits(),
            1.0_f32.to_bits()
        );
    }

    #[test]
    fn negative_and_extreme_int_values_preserved() {
        let placement = MangroveRootPlacement::new(
            HolderSet::named("a"),
            HolderSet::named("b"),
            BlockStateProvider,
            -1,
            i32::MAX,
            0.0_f32,
        );
        assert_eq!(placement.max_root_width(), -1);
        assert_eq!(placement.max_root_length(), i32::MAX);

        let min_placement = MangroveRootPlacement::new(
            HolderSet::named("a"),
            HolderSet::named("b"),
            BlockStateProvider,
            i32::MIN,
            0,
            0.0_f32,
        );
        assert_eq!(min_placement.max_root_width(), i32::MIN);
        assert_eq!(min_placement.max_root_length(), 0);
    }

    #[test]
    fn float_special_bits_preserved() {
        let nan_placement = MangroveRootPlacement::new(
            HolderSet::named("a"),
            HolderSet::named("b"),
            BlockStateProvider,
            1,
            1,
            f32::NAN,
        );
        assert!(nan_placement.random_skew_chance().is_nan());

        let neg_zero = MangroveRootPlacement::new(
            HolderSet::named("a"),
            HolderSet::named("b"),
            BlockStateProvider,
            1,
            1,
            -0.0_f32,
        );
        assert_eq!(
            neg_zero.random_skew_chance().to_bits(),
            (-0.0_f32).to_bits()
        );

        let neg_inf = MangroveRootPlacement::new(
            HolderSet::named("a"),
            HolderSet::named("b"),
            BlockStateProvider,
            1,
            1,
            f32::NEG_INFINITY,
        );
        assert_eq!(
            neg_inf.random_skew_chance().to_bits(),
            f32::NEG_INFINITY.to_bits()
        );
    }
}
