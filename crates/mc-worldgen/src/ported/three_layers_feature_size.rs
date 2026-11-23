//! Port of `net.minecraft.world.level.levelgen.feature.featuresize.ThreeLayersFeatureSize`.

/// Minimal port of `net.minecraft.world.level.levelgen.feature.featuresize.FeatureSizeType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureSizeType {
    ThreeLayersFeatureSize,
}

/// Minimal port of `net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureSize {
    min_clipped_height: Option<i32>,
}

impl FeatureSize {
    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize#<init>
    pub fn new(min_clipped_height: Option<i32>) -> Self {
        Self {
            min_clipped_height,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize#minClippedHeight
    pub fn min_clipped_height(&self) -> Option<i32> {
        self.min_clipped_height
    }
}

/// Port of `net.minecraft.world.level.levelgen.feature.featuresize.ThreeLayersFeatureSize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThreeLayersFeatureSize {
    base: FeatureSize,
    limit: i32,
    upper_limit: i32,
    lower_size: i32,
    middle_size: i32,
    upper_size: i32,
}

impl ThreeLayersFeatureSize {
    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.ThreeLayersFeatureSize#<init>
    pub fn new(
        limit: i32,
        upper_limit: i32,
        lower_size: i32,
        middle_size: i32,
        upper_size: i32,
        min_clipped_height: Option<i32>,
    ) -> Self {
        Self {
            base: FeatureSize::new(min_clipped_height),
            limit,
            upper_limit,
            lower_size,
            middle_size,
            upper_size,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.ThreeLayersFeatureSize#type
    pub fn get_type(&self) -> FeatureSizeType {
        FeatureSizeType::ThreeLayersFeatureSize
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.ThreeLayersFeatureSize#getSizeAtHeight
    pub fn get_size_at_height(&self, tree_height: i32, height: i32) -> i32 {
        if height < self.limit {
            self.lower_size
        } else if height >= tree_height.wrapping_sub(self.upper_limit) {
            self.upper_size
        } else {
            self.middle_size
        }
    }

    /// Delegates to the base `FeatureSize`.
    pub fn min_clipped_height(&self) -> Option<i32> {
        self.base.min_clipped_height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_layer_by_height() {
        let fs = ThreeLayersFeatureSize::new(3, 2, 0, 1, 2, None);
        // height < limit -> lower
        assert_eq!(fs.get_size_at_height(10, 0), 0);
        assert_eq!(fs.get_size_at_height(10, 2), 0);
        // limit <= height < tree_height - upper_limit -> middle
        assert_eq!(fs.get_size_at_height(10, 3), 1);
        assert_eq!(fs.get_size_at_height(10, 7), 1);
        // height >= tree_height - upper_limit -> upper
        assert_eq!(fs.get_size_at_height(10, 8), 2);
        assert_eq!(fs.get_size_at_height(10, 9), 2);
    }

    #[test]
    fn boundary_at_limit_and_upper_threshold() {
        let fs = ThreeLayersFeatureSize::new(1, 1, 5, 6, 7, Some(0));
        assert_eq!(fs.get_size_at_height(5, 0), 5); // below limit
        assert_eq!(fs.get_size_at_height(5, 1), 6); // at limit, below upper threshold
        assert_eq!(fs.get_size_at_height(5, 3), 6); // middle
        assert_eq!(fs.get_size_at_height(5, 4), 7); // at upper threshold (5 - 1)
        assert_eq!(fs.get_size_at_height(5, 5), 7); // above upper threshold
    }

    #[test]
    fn wrapping_sub_on_tree_height_minus_upper_limit() {
        let fs = ThreeLayersFeatureSize::new(0, 1, 10, 20, 30, None);
        // i32::MIN.wrapping_sub(1) == i32::MAX in Java int arithmetic
        assert_eq!(fs.get_size_at_height(i32::MIN, i32::MAX - 1), 20);
        assert_eq!(fs.get_size_at_height(i32::MIN, i32::MAX), 30);
        assert_eq!(fs.get_size_at_height(i32::MIN, 0), 20);
    }

    #[test]
    fn negative_inputs() {
        let fs = ThreeLayersFeatureSize::new(-2, -3, 1, 2, 3, None);
        // height < limit (-2) -> lower
        assert_eq!(fs.get_size_at_height(0, -3), 1);
        // threshold = 0.wrapping_sub(-3) == 3
        assert_eq!(fs.get_size_at_height(0, -2), 2); // middle
        assert_eq!(fs.get_size_at_height(0, 2), 2); // middle
        assert_eq!(fs.get_size_at_height(0, 3), 3); // upper
        assert_eq!(fs.get_size_at_height(0, 4), 3); // upper
    }

    #[test]
    fn wrapping_sub_extreme_values() {
        let fs = ThreeLayersFeatureSize::new(0, -1, 1, 2, 3, None);
        // threshold = i32::MAX.wrapping_sub(-1) == i32::MIN
        assert_eq!(fs.get_size_at_height(i32::MAX, -1), 1); // below limit
        assert_eq!(fs.get_size_at_height(i32::MAX, 0), 3); // >= threshold
        assert_eq!(fs.get_size_at_height(i32::MAX, i32::MAX), 3);
    }

    #[test]
    fn constructor_type_and_min_clipped_height() {
        let fs = ThreeLayersFeatureSize::new(1, 2, 3, 4, 5, Some(7));
        assert_eq!(fs.get_type(), FeatureSizeType::ThreeLayersFeatureSize);
        assert_eq!(fs.min_clipped_height(), Some(7));

        let fs_none = ThreeLayersFeatureSize::new(0, 0, 0, 0, 0, None);
        assert_eq!(fs_none.get_type(), FeatureSizeType::ThreeLayersFeatureSize);
        assert_eq!(fs_none.min_clipped_height(), None);
    }
}
