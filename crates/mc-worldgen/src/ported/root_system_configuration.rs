use std::marker::PhantomData;

/// Marker trait for `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration`.
pub trait FeatureConfiguration {}

/// Minimal self-contained placeholder for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub value: T,
}

impl<T> Holder<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.block.Block`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Block;

/// Minimal self-contained placeholder for `net.minecraft.tags.TagKey<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagKey<T> {
    pub location: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T> TagKey<T> {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            _marker: PhantomData,
        }
    }
}

/// Port of `net.minecraft.world.level.levelgen.feature.configurations.RootSystemConfiguration`.
///
/// Generic parameters keep this module self-contained without requiring unported external
/// Minecraft classes:
/// - `PF`: placed feature type (`Holder<PlacedFeature>` in Java)
/// - `BSP`: block state provider type (`BlockStateProvider` in Java)
/// - `BP`: block predicate type (`BlockPredicate` in Java)
#[derive(Debug, Clone, PartialEq)]
pub struct RootSystemConfiguration<PF, BSP, BP> {
    pub tree_feature: Holder<PF>,
    pub required_vertical_space_for_tree: i32,
    pub root_radius: i32,
    pub root_replaceable: TagKey<Block>,
    pub root_state_provider: BSP,
    pub root_placement_attempts: i32,
    pub root_column_max_height: i32,
    pub hanging_root_radius: i32,
    pub hanging_roots_vertical_span: i32,
    pub hanging_root_state_provider: BSP,
    pub hanging_root_placement_attempts: i32,
    pub allowed_vertical_water_for_tree: i32,
    pub allowed_tree_position: BP,
}

impl<PF, BSP, BP> FeatureConfiguration for RootSystemConfiguration<PF, BSP, BP> {}

impl<PF, BSP, BP> RootSystemConfiguration<PF, BSP, BP> {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.RootSystemConfiguration#<init>
    pub fn new(
        tree_feature: Holder<PF>,
        required_vertical_space_for_tree: i32,
        root_radius: i32,
        root_replaceable: TagKey<Block>,
        root_state_provider: BSP,
        root_placement_attempts: i32,
        root_column_max_height: i32,
        hanging_root_radius: i32,
        hanging_roots_vertical_span: i32,
        hanging_root_state_provider: BSP,
        hanging_root_placement_attempts: i32,
        allowed_vertical_water_for_tree: i32,
        allowed_tree_position: BP,
    ) -> Self {
        Self {
            tree_feature,
            required_vertical_space_for_tree,
            root_radius,
            root_replaceable,
            root_state_provider,
            root_placement_attempts,
            root_column_max_height,
            hanging_root_radius,
            hanging_roots_vertical_span,
            hanging_root_state_provider,
            hanging_root_placement_attempts,
            allowed_vertical_water_for_tree,
            allowed_tree_position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct DummyPlacedFeature(&'static str);

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct DummyBlockStateProvider(u32);

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct DummyBlockPredicate(bool);

    fn sample_config() -> RootSystemConfiguration<DummyPlacedFeature, DummyBlockStateProvider, DummyBlockPredicate>
    {
        RootSystemConfiguration::new(
            Holder::new(DummyPlacedFeature("azalea_tree")),
            1,
            3,
            TagKey::new("minecraft:azalea_root_replaceable"),
            DummyBlockStateProvider(10),
            20,
            100,
            3,
            2,
            DummyBlockStateProvider(20),
            20,
            2,
            DummyBlockPredicate(true),
        )
    }

    #[test]
    fn constructor_assigns_all_fields() {
        let cfg = sample_config();
        assert_eq!(cfg.tree_feature, Holder::new(DummyPlacedFeature("azalea_tree")));
        assert_eq!(cfg.required_vertical_space_for_tree, 1);
        assert_eq!(cfg.root_radius, 3);
        assert_eq!(
            cfg.root_replaceable,
            TagKey::new("minecraft:azalea_root_replaceable")
        );
        assert_eq!(cfg.root_state_provider, DummyBlockStateProvider(10));
        assert_eq!(cfg.root_placement_attempts, 20);
        assert_eq!(cfg.root_column_max_height, 100);
        assert_eq!(cfg.hanging_root_radius, 3);
        assert_eq!(cfg.hanging_roots_vertical_span, 2);
        assert_eq!(cfg.hanging_root_state_provider, DummyBlockStateProvider(20));
        assert_eq!(cfg.hanging_root_placement_attempts, 20);
        assert_eq!(cfg.allowed_vertical_water_for_tree, 2);
        assert_eq!(cfg.allowed_tree_position, DummyBlockPredicate(true));
    }

    #[test]
    fn negative_and_extreme_int_inputs_are_preserved() {
        let cfg = RootSystemConfiguration::new(
            Holder::new(DummyPlacedFeature("edge")),
            -1,
            i32::MIN,
            TagKey::new("edge:replaceable"),
            DummyBlockStateProvider(u32::MAX),
            i32::MAX,
            -256,
            0,
            -16,
            DummyBlockStateProvider(0),
            i32::MIN,
            i32::MAX,
            DummyBlockPredicate(false),
        );

        assert_eq!(cfg.required_vertical_space_for_tree, -1);
        assert_eq!(cfg.root_radius, i32::MIN);
        assert_eq!(cfg.root_placement_attempts, i32::MAX);
        assert_eq!(cfg.root_column_max_height, -256);
        assert_eq!(cfg.hanging_root_radius, 0);
        assert_eq!(cfg.hanging_roots_vertical_span, -16);
        assert_eq!(cfg.hanging_root_placement_attempts, i32::MIN);
        assert_eq!(cfg.allowed_vertical_water_for_tree, i32::MAX);
    }

    #[test]
    fn wrapping_int_and_masked_shift_semantics_match_java() {
        let cfg = RootSystemConfiguration::new(
            Holder::new(DummyPlacedFeature("wrap")),
            i32::MAX,
            i32::MIN,
            TagKey::new("wrap:replaceable"),
            DummyBlockStateProvider(1),
            i32::MAX,
            i32::MIN,
            -1,
            -16,
            DummyBlockStateProvider(2),
            2,
            -3,
            DummyBlockPredicate(true),
        );

        assert_eq!(
            cfg.required_vertical_space_for_tree.wrapping_add(1),
            i32::MIN
        );
        assert_eq!(cfg.root_radius.wrapping_sub(1), i32::MAX);
        assert_eq!(cfg.root_placement_attempts.wrapping_mul(2), -2);
        assert_eq!(cfg.root_column_max_height.wrapping_add(-1), i32::MAX);

        // Java int shift count is masked to 5 bits.
        let shift_count: i32 = 33; // 33 & 31 == 1
        let masked = (shift_count as u32) & 31;
        assert_eq!(cfg.hanging_roots_vertical_span << masked, -32);
        assert_eq!((-1_i32) << ((32_u32) & 31), -1);
    }

    #[test]
    fn float_bit_patterns_are_preserved_for_porting_discipline() {
        // This configuration has no float fields, but Java-faithful ports must preserve
        // exact f32/f64 bit patterns and operation order when floats are involved.
        let f32_cases: [f32; 6] = [
            0.0,
            -0.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            f32::from_bits(0x7FC0_0001),
        ];
        for v in f32_cases {
            let bits = v.to_bits();
            let rt = f32::from_bits(bits);
            assert_eq!(rt.to_bits(), bits);
        }

        let a: f64 = -1.234_567_890_123_456_7;
        let b: f64 = 9.876_543_210_987_654e-3;
        let sum_bits = (a + b).to_bits();
        assert_eq!(f64::from_bits(sum_bits).to_bits(), sum_bits);
        assert_eq!(f64::from_bits(a.to_bits()).to_bits(), a.to_bits());
    }
}
