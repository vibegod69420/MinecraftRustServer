pub trait FeatureConfiguration {}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaFeatureConfiguration<B, P> {
    contents: B,
    rim: B,
    size: P,
    rim_size: P,
}

impl<B, P> FeatureConfiguration for DeltaFeatureConfiguration<B, P> {}

impl<B, P> DeltaFeatureConfiguration<B, P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DeltaFeatureConfiguration#<init>
    pub fn new(contents: B, rim: B, size: P, rim_size: P) -> Self {
        Self {
            contents,
            rim,
            size,
            rim_size,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DeltaFeatureConfiguration#contents
    pub fn contents(&self) -> &B {
        &self.contents
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DeltaFeatureConfiguration#rim
    pub fn rim(&self) -> &B {
        &self.rim
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DeltaFeatureConfiguration#size
    pub fn size(&self) -> &P {
        &self.size
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.DeltaFeatureConfiguration#rimSize
    pub fn rim_size(&self) -> &P {
        &self.rim_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockBlock(u32);

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockIntProvider {
        min: i32,
        max: i32,
        value: i32,
    }

    #[test]
    fn new_and_accessors() {
        let c = DeltaFeatureConfiguration::new(
            MockBlock(1),
            MockBlock(2),
            MockIntProvider {
                min: 0,
                max: 16,
                value: 5,
            },
            MockIntProvider {
                min: 0,
                max: 16,
                value: 3,
            },
        );
        assert_eq!(c.contents(), &MockBlock(1));
        assert_eq!(c.rim(), &MockBlock(2));
        assert_eq!(c.size().value, 5);
        assert_eq!(c.rim_size().value, 3);
    }

    #[test]
    fn handles_negative_and_extreme_integer_values() {
        let c = DeltaFeatureConfiguration::new(
            MockBlock(u32::MAX),
            MockBlock(0),
            MockIntProvider {
                min: -16,
                max: 16,
                value: -1,
            },
            MockIntProvider {
                min: i32::MIN,
                max: i32::MAX,
                value: i32::MIN,
            },
        );
        assert_eq!(c.contents().0, u32::MAX);
        assert_eq!(c.rim().0, 0);
        assert_eq!(c.size().value, -1);
        assert_eq!(c.rim_size().min, i32::MIN);
        assert_eq!(c.rim_size().max, i32::MAX);
        assert_eq!(c.rim_size().max.wrapping_add(1), i32::MIN);
    }

    #[test]
    fn clone_preserves_fields() {
        let c = DeltaFeatureConfiguration::new(10_u32, 20_u32, 5_i32, 7_i32);
        let c2 = c.clone();
        assert_eq!(c, c2);
        assert_eq!(*c2.contents(), 10_u32);
        assert_eq!(*c2.rim(), 20_u32);
    }

    #[test]
    fn preserves_float_bits_in_generic_fields() {
        let nan = f32::NAN;
        let neg_zero = -0.0_f32;
        let c = DeltaFeatureConfiguration::new(nan, neg_zero, f32::INFINITY, f32::NEG_INFINITY);
        assert!(c.contents().is_nan());
        assert_eq!(c.rim().to_bits(), neg_zero.to_bits());
        assert_eq!(c.size().to_bits(), f32::INFINITY.to_bits());
        assert_eq!(c.rim_size().to_bits(), f32::NEG_INFINITY.to_bits());
    }
}
