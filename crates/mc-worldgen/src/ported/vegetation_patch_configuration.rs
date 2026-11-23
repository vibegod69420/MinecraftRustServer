use std::marker::PhantomData;

/// Marker trait mirroring `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration`.
pub trait FeatureConfiguration {}

/// Placeholder for `net.minecraft.tags.TagKey<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TagKey<T> {
    pub name: String,
    pub _phantom: PhantomData<T>,
}

/// Placeholder for `net.minecraft.world.level.block.Block`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Block;

/// Placeholder for `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BlockStateProvider {
    pub id: String,
}

/// Placeholder for `net.minecraft.world.level.levelgen.placement.PlacedFeature`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PlacedFeature {
    pub id: String,
}

/// Placeholder for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Holder<T> {
    pub value: T,
}

/// Placeholder for `net.minecraft.world.level.levelgen.placement.CaveSurface`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CaveSurface {
    #[default]
    Floor,
    Ceiling,
}

/// Placeholder for `net.minecraft.util.valueproviders.IntProvider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IntProvider {
    pub min: i32,
    pub max: i32,
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.VegetationPatchConfiguration`.
///
/// The Java `CODEC` field is not ported because no codec framework is available in this crate.
#[derive(Debug, Clone, PartialEq)]
pub struct VegetationPatchConfiguration {
    pub replaceable: TagKey<Block>,
    pub ground_state: BlockStateProvider,
    pub vegetation_feature: Holder<PlacedFeature>,
    pub surface: CaveSurface,
    pub depth: IntProvider,
    pub extra_bottom_block_chance: f32,
    pub vertical_range: i32,
    pub vegetation_chance: f32,
    pub xz_radius: IntProvider,
    pub extra_edge_column_chance: f32,
}

impl FeatureConfiguration for VegetationPatchConfiguration {}

impl VegetationPatchConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.VegetationPatchConfiguration#<init>
    pub fn new(
        replaceable: TagKey<Block>,
        ground_state: BlockStateProvider,
        vegetation_feature: Holder<PlacedFeature>,
        surface: CaveSurface,
        depth: IntProvider,
        extra_bottom_block_chance: f32,
        vertical_range: i32,
        vegetation_chance: f32,
        xz_radius: IntProvider,
        extra_edge_column_chance: f32,
    ) -> Self {
        Self {
            replaceable,
            ground_state,
            vegetation_feature,
            surface,
            depth,
            extra_bottom_block_chance,
            vertical_range,
            vegetation_chance,
            xz_radius,
            extra_edge_column_chance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> VegetationPatchConfiguration {
        VegetationPatchConfiguration::new(
            TagKey {
                name: "minecraft:lush_ground_replaceable".to_string(),
                _phantom: PhantomData,
            },
            BlockStateProvider {
                id: "minecraft:moss_block".to_string(),
            },
            Holder {
                value: PlacedFeature {
                    id: "minecraft:lush_caves_vegetation".to_string(),
                },
            },
            CaveSurface::Floor,
            IntProvider { min: 1, max: 2 },
            0.0_f32,
            5_i32,
            0.8_f32,
            IntProvider { min: 4, max: 7 },
            0.3_f32,
        )
    }

    #[test]
    fn constructor_stores_all_fields() {
        let cfg = sample();
        assert_eq!(cfg.replaceable.name, "minecraft:lush_ground_replaceable");
        assert_eq!(cfg.ground_state.id, "minecraft:moss_block");
        assert_eq!(
            cfg.vegetation_feature.value.id,
            "minecraft:lush_caves_vegetation"
        );
        assert_eq!(cfg.surface, CaveSurface::Floor);
        assert_eq!(cfg.depth, IntProvider { min: 1, max: 2 });
        assert_eq!(cfg.extra_bottom_block_chance.to_bits(), 0.0_f32.to_bits());
        assert_eq!(cfg.vertical_range, 5);
        assert_eq!(cfg.vegetation_chance.to_bits(), 0.8_f32.to_bits());
        assert_eq!(cfg.xz_radius, IntProvider { min: 4, max: 7 });
        assert_eq!(cfg.extra_edge_column_chance.to_bits(), 0.3_f32.to_bits());
    }

    #[test]
    fn float_fields_preserve_exact_bits() {
        let nan_bits = 0x7FC0_1234_u32;
        let cfg = VegetationPatchConfiguration::new(
            TagKey::default(),
            BlockStateProvider::default(),
            Holder {
                value: PlacedFeature::default(),
            },
            CaveSurface::Ceiling,
            IntProvider { min: 1, max: 1 },
            f32::from_bits(nan_bits),
            1,
            -0.0_f32,
            IntProvider { min: 1, max: 1 },
            f32::INFINITY,
        );
        assert_eq!(cfg.extra_bottom_block_chance.to_bits(), nan_bits);
        assert_eq!(cfg.vegetation_chance.to_bits(), (-0.0_f32).to_bits());
        assert!(cfg.extra_edge_column_chance.is_infinite());
        assert!(cfg.extra_edge_column_chance.is_sign_positive());
        assert_eq!(cfg.surface, CaveSurface::Ceiling);
    }

    #[test]
    fn integer_edge_and_negative_values() {
        let cfg = VegetationPatchConfiguration::new(
            TagKey::default(),
            BlockStateProvider::default(),
            Holder {
                value: PlacedFeature::default(),
            },
            CaveSurface::Floor,
            IntProvider {
                min: i32::MIN,
                max: i32::MAX,
            },
            -1.0_f32,
            i32::MIN,
            f32::NEG_INFINITY,
            IntProvider { min: -128, max: -1 },
            1.0_f32,
        );
        assert_eq!(cfg.vertical_range, i32::MIN);
        assert_eq!(cfg.depth.min, i32::MIN);
        assert_eq!(cfg.depth.max, i32::MAX);
        assert_eq!(cfg.xz_radius.min, -128);
        assert_eq!(cfg.xz_radius.max, -1);
        assert!(cfg.vegetation_chance.is_infinite());
        assert!(cfg.vegetation_chance.is_sign_negative());
    }

    #[test]
    fn wrapping_overflow_values_are_stored_as_i32() {
        let overflowed = i32::MAX.wrapping_add(1);
        let cfg = VegetationPatchConfiguration::new(
            TagKey::default(),
            BlockStateProvider::default(),
            Holder {
                value: PlacedFeature::default(),
            },
            CaveSurface::Floor,
            IntProvider {
                min: overflowed,
                max: i32::MIN.wrapping_sub(1),
            },
            0.0_f32,
            overflowed,
            0.0_f32,
            IntProvider { min: 0, max: 0 },
            0.0_f32,
        );
        assert_eq!(cfg.vertical_range, i32::MIN);
        assert_eq!(cfg.depth.min, i32::MIN);
        assert_eq!(cfg.depth.max, i32::MAX);
    }
}
