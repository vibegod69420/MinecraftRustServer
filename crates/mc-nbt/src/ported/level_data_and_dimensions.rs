/// Minimal self-contained stub for `net.minecraft.world.level.storage.WorldData`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldData;

/// Minimal self-contained stub for `net.minecraft.world.level.levelgen.WorldDimensions.Complete`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldDimensionsComplete;

/// Port of `net.minecraft.world.level.storage.LevelDataAndDimensions`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelDataAndDimensions {
    pub world_data: WorldData,
    pub dimensions: WorldDimensionsComplete,
}

impl LevelDataAndDimensions {
    // JAVA: net.minecraft.world.level.storage.LevelDataAndDimensions#worldData
    pub fn world_data(&self) -> &WorldData {
        &self.world_data
    }

    // JAVA: net.minecraft.world.level.storage.LevelDataAndDimensions#dimensions
    pub fn dimensions(&self) -> &WorldDimensionsComplete {
        &self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_components() {
        let ldd = LevelDataAndDimensions {
            world_data: WorldData,
            dimensions: WorldDimensionsComplete,
        };
        assert_eq!(ldd.world_data(), &WorldData);
        assert_eq!(ldd.dimensions(), &WorldDimensionsComplete);
    }

    #[test]
    fn clone_and_equality() {
        let a = LevelDataAndDimensions {
            world_data: WorldData,
            dimensions: WorldDimensionsComplete,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_format_contains_type_name() {
        let ldd = LevelDataAndDimensions {
            world_data: WorldData,
            dimensions: WorldDimensionsComplete,
        };
        let dbg = format!("{:?}", ldd);
        assert!(dbg.contains("LevelDataAndDimensions"));
    }

    #[test]
    fn float_bits_and_wrapping_sanity() {
        let neg_zero = (-0.0f64).to_bits();
        let pos_zero = 0.0f64.to_bits();
        assert_ne!(neg_zero, pos_zero);
        let wrapped = i32::MIN.wrapping_sub(1);
        assert_eq!(wrapped, i32::MAX);
        let neg = -42i32;
        assert!(neg < 0);
    }
}
