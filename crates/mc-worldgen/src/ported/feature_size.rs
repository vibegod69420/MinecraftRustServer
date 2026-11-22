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

pub const MAX_WIDTH: i32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureSize {
    min_clipped_height: Option<i32>,
}

pub trait FeatureSizeOps {
    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize#type
    fn get_type(&self) -> FeatureSizeType;

    // JAVA: net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize#getSizeAtHeight
    fn get_size_at_height(&self, tree_height: i32, current_y: i32) -> i32;
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

#[cfg(test)]
mod tests {
    use super::*;

    struct DummySize {
        _base: FeatureSize,
    }

    impl FeatureSizeOps for DummySize {
        fn get_type(&self) -> FeatureSizeType {
            FeatureSizeType::TWO_LAYERS_FEATURE_SIZE
        }

        fn get_size_at_height(&self, _tree_height: i32, current_y: i32) -> i32 {
            current_y
        }
    }

    #[test]
    fn new_stores_min_clipped_height() {
        let fs = FeatureSize::new(Some(5));
        assert_eq!(fs.min_clipped_height(), Some(5));
        let fs_none = FeatureSize::new(None);
        assert_eq!(fs_none.min_clipped_height(), None);
    }

    #[test]
    fn min_clipped_height_edge_values() {
        let fs_min = FeatureSize::new(Some(i32::MIN));
        assert_eq!(fs_min.min_clipped_height(), Some(i32::MIN));
        let fs_max = FeatureSize::new(Some(i32::MAX));
        assert_eq!(fs_max.min_clipped_height(), Some(i32::MAX));
        let fs_neg = FeatureSize::new(Some(-1));
        assert_eq!(fs_neg.min_clipped_height(), Some(-1));
    }

    #[test]
    fn max_width_constant_matches_java() {
        assert_eq!(MAX_WIDTH, 16);
    }

    #[test]
    fn trait_ops_work_on_dummy() {
        let d = DummySize {
            _base: FeatureSize::new(None),
        };
        assert_eq!(d.get_type(), FeatureSizeType::TWO_LAYERS_FEATURE_SIZE);
        assert_eq!(d.get_size_at_height(10, -3), -3);
        assert_eq!(d.get_size_at_height(i32::MIN, 0), 0);
        assert_eq!(d.get_size_at_height(i32::MAX, i32::MAX), i32::MAX);
    }

    #[test]
    fn base_copy_preserves_value() {
        let fs = FeatureSize::new(Some(7));
        let copied = fs;
        assert_eq!(fs.min_clipped_height(), copied.min_clipped_height());
        assert_eq!(copied.min_clipped_height(), Some(7));
    }
}
