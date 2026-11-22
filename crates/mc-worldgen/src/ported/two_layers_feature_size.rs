#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureSizeType {
    name: &'static str,
}

impl FeatureSizeType {
    pub const TWO_LAYERS_FEATURE_SIZE: FeatureSizeType =
        FeatureSizeType {
            name: "two_layers_feature_size",
        };
    pub const THREE_LAYERS_FEATURE_SIZE: FeatureSizeType =
        FeatureSizeType {
            name: "three_layers_feature_size",
        };

    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureSize {
    min_clipped_height: Option<i32>,
}

impl FeatureSize {
    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize#<init>
    pub fn new(min_clipped_height: Option<i32>) -> Self {
        Self { min_clipped_height }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize#minClippedHeight
    pub fn min_clipped_height(&self) -> Option<i32> {
        self.min_clipped_height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TwoLayersFeatureSize {
    base: FeatureSize,
    limit: i32,
    lower_size: i32,
    upper_size: i32,
}

impl TwoLayersFeatureSize {
    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.TwoLayersFeatureSize#<init>
    pub fn new(limit: i32, lower_size: i32, upper_size: i32) -> Self {
        Self::new_with_min_clipped_height(limit, lower_size, upper_size, None)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.TwoLayersFeatureSize#<init>
    pub fn new_with_min_clipped_height(
        limit: i32,
        lower_size: i32,
        upper_size: i32,
        min_clipped_height: Option<i32>,
    ) -> Self {
        Self {
            base: FeatureSize::new(min_clipped_height),
            limit,
            lower_size,
            upper_size,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.TwoLayersFeatureSize#type
    pub fn get_type(&self) -> FeatureSizeType {
        FeatureSizeType::TWO_LAYERS_FEATURE_SIZE
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.TwoLayersFeatureSize#getSizeAtHeight
    pub fn get_size_at_height(&self, _tree_height: i32, current_y: i32) -> i32 {
        if current_y < self.limit {
            self.lower_size
        } else {
            self.upper_size
        }
    }

    pub fn min_clipped_height(&self) -> Option<i32> {
        self.base.min_clipped_height()
    }

    pub fn limit(&self) -> i32 {
        self.limit
    }

    pub fn lower_size(&self) -> i32 {
        self.lower_size
    }

    pub fn upper_size(&self) -> i32 {
        self.upper_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_is_two_layers() {
        let fs = TwoLayersFeatureSize::new(1, 0, 1);
        assert_eq!(fs.get_type(), FeatureSizeType::TWO_LAYERS_FEATURE_SIZE);
        assert_eq!(fs.get_type().name(), "two_layers_feature_size");
    }

    #[test]
    fn get_size_at_height_uses_limit() {
        let fs = TwoLayersFeatureSize::new(3, 10, 20);
        assert_eq!(fs.get_size_at_height(5, 0), 10);
        assert_eq!(fs.get_size_at_height(5, 2), 10);
        assert_eq!(fs.get_size_at_height(5, 3), 20);
        assert_eq!(fs.get_size_at_height(5, 100), 20);
    }

    #[test]
    fn negative_inputs_and_boundaries() {
        let fs = TwoLayersFeatureSize::new(0, -5, 7);
        assert_eq!(fs.get_size_at_height(-1, -1), -5);
        assert_eq!(fs.get_size_at_height(i32::MIN, 0), 7);
        assert_eq!(fs.get_size_at_height(i32::MAX, i32::MIN), -5);
        assert_eq!(fs.get_size_at_height(0, i32::MAX), 7);
    }

    #[test]
    fn min_clipped_height_constructors() {
        let fs = TwoLayersFeatureSize::new(1, 0, 1);
        assert_eq!(fs.min_clipped_height(), None);

        let fs2 = TwoLayersFeatureSize::new_with_min_clipped_height(1, 0, 1, Some(5));
        assert_eq!(fs2.min_clipped_height(), Some(5));

        let fs3 = TwoLayersFeatureSize::new_with_min_clipped_height(1, 0, 1, Some(i32::MIN));
        assert_eq!(fs3.min_clipped_height(), Some(i32::MIN));
    }

    #[test]
    fn extreme_limit_values() {
        let fs = TwoLayersFeatureSize::new(i32::MAX, 1, 2);
        assert_eq!(fs.get_size_at_height(0, i32::MAX - 1), 1);
        assert_eq!(fs.get_size_at_height(0, i32::MAX), 2);

        let fs2 = TwoLayersFeatureSize::new(i32::MIN, 1, 2);
        assert_eq!(fs2.get_size_at_height(0, i32::MIN), 2);
        assert_eq!(fs2.get_size_at_height(0, i32::MIN.wrapping_add(1)), 2);
    }

    #[test]
    fn accessors_return_constructor_values() {
        let fs = TwoLayersFeatureSize::new_with_min_clipped_height(4, 2, 8, Some(12));
        assert_eq!(fs.limit(), 4);
        assert_eq!(fs.lower_size(), 2);
        assert_eq!(fs.upper_size(), 8);
        assert_eq!(fs.min_clipped_height(), Some(12));
    }
}
