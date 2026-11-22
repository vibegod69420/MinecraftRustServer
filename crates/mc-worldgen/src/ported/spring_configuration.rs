pub trait FeatureConfiguration {}

#[derive(Debug, Clone, PartialEq)]
pub struct SpringConfiguration<S, V> {
    pub state: S,
    pub requires_block_below: bool,
    pub rock_count: i32,
    pub hole_count: i32,
    pub valid_blocks: V,
}

impl<S, V> FeatureConfiguration for SpringConfiguration<S, V> {}

impl<S, V> SpringConfiguration<S, V> {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SpringConfiguration#<init>
    pub fn new(
        state: S,
        requires_block_below: bool,
        rock_count: i32,
        hole_count: i32,
        valid_blocks: V,
    ) -> Self {
        Self {
            state,
            requires_block_below,
            rock_count,
            hole_count,
            valid_blocks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockFluidState(u32);

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockHolderSet(Vec<i32>);

    #[test]
    fn new_stores_fields() {
        let c = SpringConfiguration::new(
            MockFluidState(1),
            true,
            4,
            1,
            MockHolderSet(vec![10, 20]),
        );
        assert_eq!(c.state, MockFluidState(1));
        assert!(c.requires_block_below);
        assert_eq!(c.rock_count, 4);
        assert_eq!(c.hole_count, 1);
        assert_eq!(c.valid_blocks.0, vec![10, 20]);
    }

    #[test]
    fn handles_negative_and_extreme_ints() {
        let c = SpringConfiguration::new(
            MockFluidState(0),
            false,
            -1,
            i32::MIN,
            MockHolderSet(vec![i32::MAX, -1]),
        );
        assert!(!c.requires_block_below);
        assert_eq!(c.rock_count, -1);
        assert_eq!(c.hole_count, i32::MIN);
        assert_eq!(c.valid_blocks.0[0], i32::MAX);
        assert_eq!(c.valid_blocks.0[1], -1);
    }

    #[test]
    fn handles_overflow_boundary_values_with_wrapping_semantics() {
        let c = SpringConfiguration::new(MockFluidState(0), true, i32::MAX, i32::MIN, ());
        assert_eq!(c.rock_count, i32::MAX);
        assert_eq!(c.hole_count, i32::MIN);
        assert_eq!(c.rock_count.wrapping_add(1), i32::MIN);
        assert_eq!(c.hole_count.wrapping_sub(1), i32::MAX);
    }

    #[test]
    fn preserves_float_bits_in_generic_fields() {
        let nan = f32::NAN;
        let neg_zero = -0.0_f64;
        let c = SpringConfiguration::new(nan, false, 0, 0, neg_zero);
        assert!(c.state.is_nan());
        assert_eq!(c.valid_blocks.to_bits(), neg_zero.to_bits());
    }
}
