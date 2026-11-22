/// Placeholder for `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
/// Kept self-contained because external state-provider types are not yet ported.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStateProvider;

/// Port of `net.minecraft.world.level.levelgen.feature.rootplacers.AboveRootPlacement`.
///
/// The Mojang `Codec` field is omitted because codec infrastructure is not yet ported.
#[derive(Debug, Clone, PartialEq)]
pub struct AboveRootPlacement {
    above_root_provider: BlockStateProvider,
    above_root_placement_chance: f32,
}

impl AboveRootPlacement {
    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.AboveRootPlacement#<init>
    pub fn new(above_root_provider: BlockStateProvider, above_root_placement_chance: f32) -> Self {
        Self {
            above_root_provider,
            above_root_placement_chance,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.AboveRootPlacement#aboveRootProvider
    pub fn above_root_provider(&self) -> &BlockStateProvider {
        &self.above_root_provider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.AboveRootPlacement#aboveRootPlacementChance
    pub fn above_root_placement_chance(&self) -> f32 {
        self.above_root_placement_chance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_fields() {
        let provider = BlockStateProvider;
        let placement = AboveRootPlacement::new(provider.clone(), 0.5_f32);
        assert_eq!(placement.above_root_provider(), &provider);
        assert_eq!(
            placement.above_root_placement_chance().to_bits(),
            0.5_f32.to_bits()
        );
    }

    #[test]
    fn chance_boundary_values() {
        let zero = AboveRootPlacement::new(BlockStateProvider, 0.0_f32);
        let one = AboveRootPlacement::new(BlockStateProvider, 1.0_f32);
        assert_eq!(zero.above_root_placement_chance().to_bits(), 0.0_f32.to_bits());
        assert_eq!(one.above_root_placement_chance().to_bits(), 1.0_f32.to_bits());
    }

    #[test]
    fn chance_negative_and_infinity_bits_preserved() {
        let negative = AboveRootPlacement::new(BlockStateProvider, -0.25_f32);
        let inf = AboveRootPlacement::new(BlockStateProvider, f32::INFINITY);
        let neg_inf = AboveRootPlacement::new(BlockStateProvider, f32::NEG_INFINITY);
        assert_eq!(
            negative.above_root_placement_chance().to_bits(),
            (-0.25_f32).to_bits()
        );
        assert_eq!(
            inf.above_root_placement_chance().to_bits(),
            f32::INFINITY.to_bits()
        );
        assert_eq!(
            neg_inf.above_root_placement_chance().to_bits(),
            f32::NEG_INFINITY.to_bits()
        );
    }

    #[test]
    fn chance_nan_and_signed_zero_bits_preserved() {
        let nan = AboveRootPlacement::new(BlockStateProvider, f32::NAN);
        let neg_zero = AboveRootPlacement::new(BlockStateProvider, -0.0_f32);
        assert!(nan.above_root_placement_chance().is_nan());
        assert_eq!(
            neg_zero.above_root_placement_chance().to_bits(),
            (-0.0_f32).to_bits()
        );
        assert_ne!(
            (-0.0_f32).to_bits(),
            0.0_f32.to_bits(),
            "signed zero bit patterns should differ"
        );
    }
}
