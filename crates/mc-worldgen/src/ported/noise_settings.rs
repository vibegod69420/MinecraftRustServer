pub mod dimension_type {
    pub const MIN_Y: i32 = -2032;
    pub const MAX_Y: i32 = 2031;
    pub const Y_SIZE: i32 = 4064;
}

/// Minimal stub for `net.minecraft.world.level.LevelHeightAccessor`.
pub trait LevelHeightAccessor {
    fn get_min_y(&self) -> i32;
    fn get_max_y(&self) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseSettings {
    pub min_y: i32,
    pub height: i32,
    pub noise_size_horizontal: i32,
    pub noise_size_vertical: i32,
}

impl NoiseSettings {
    pub const OVERWORLD_NOISE_SETTINGS: Self = Self {
        min_y: -64,
        height: 384,
        noise_size_horizontal: 1,
        noise_size_vertical: 2,
    };
    pub const NETHER_NOISE_SETTINGS: Self = Self {
        min_y: 0,
        height: 128,
        noise_size_horizontal: 1,
        noise_size_vertical: 2,
    };
    pub const END_NOISE_SETTINGS: Self = Self {
        min_y: 0,
        height: 128,
        noise_size_horizontal: 2,
        noise_size_vertical: 1,
    };
    pub const CAVES_NOISE_SETTINGS: Self = Self {
        min_y: -64,
        height: 192,
        noise_size_horizontal: 1,
        noise_size_vertical: 2,
    };
    pub const FLOATING_ISLANDS_NOISE_SETTINGS: Self = Self {
        min_y: 0,
        height: 256,
        noise_size_horizontal: 2,
        noise_size_vertical: 1,
    };

    // JAVA: net.minecraft.world.level.levelgen.NoiseSettings#<init>
    pub fn new(min_y: i32, height: i32, noise_size_horizontal: i32, noise_size_vertical: i32) -> Self {
        Self {
            min_y,
            height,
            noise_size_horizontal,
            noise_size_vertical,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseSettings#guardY
    fn guard_y(settings: Self) -> Result<Self, String> {
        let max_plus_one = dimension_type::MAX_Y.wrapping_add(1);
        if settings.min_y.wrapping_add(settings.height) > max_plus_one {
            return Err(format!(
                "min_y + height cannot be higher than: {}",
                max_plus_one
            ));
        } else if settings.height % 16 != 0 {
            return Err("height has to be a multiple of 16".to_string());
        } else if settings.min_y % 16 != 0 {
            return Err("min_y has to be a multiple of 16".to_string());
        }
        Ok(settings)
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseSettings#create
    pub fn create(min_y: i32, height: i32, noise_size_horizontal: i32, noise_size_vertical: i32) -> Self {
        let settings = Self::new(min_y, height, noise_size_horizontal, noise_size_vertical);
        match Self::guard_y(settings) {
            Ok(s) => s,
            Err(msg) => panic!("{}", msg),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseSettings#getCellHeight
    pub fn get_cell_height(&self) -> i32 {
        self.noise_size_vertical.wrapping_shl(2)
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseSettings#getCellWidth
    pub fn get_cell_width(&self) -> i32 {
        self.noise_size_horizontal.wrapping_shl(2)
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseSettings#clampToHeightAccessor
    pub fn clamp_to_height_accessor(&self, accessor: &dyn LevelHeightAccessor) -> Self {
        let min_y = self.min_y.max(accessor.get_min_y());
        let top = self
            .min_y
            .wrapping_add(self.height)
            .min(accessor.get_max_y().wrapping_add(1));
        let height = top.wrapping_sub(min_y);
        Self::new(min_y, height, self.noise_size_horizontal, self.noise_size_vertical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAccessor {
        min_y: i32,
        max_y: i32,
    }

    impl LevelHeightAccessor for MockAccessor {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn get_max_y(&self) -> i32 {
            self.max_y
        }
    }

    #[test]
    fn create_valid_and_predefined_constants() {
        let s = NoiseSettings::create(-64, 384, 1, 2);
        assert_eq!(s, NoiseSettings::OVERWORLD_NOISE_SETTINGS);
        assert_eq!(
            NoiseSettings::NETHER_NOISE_SETTINGS,
            NoiseSettings::new(0, 128, 1, 2)
        );
        assert_eq!(
            NoiseSettings::END_NOISE_SETTINGS,
            NoiseSettings::new(0, 128, 2, 1)
        );
        assert_eq!(
            NoiseSettings::CAVES_NOISE_SETTINGS,
            NoiseSettings::new(-64, 192, 1, 2)
        );
        assert_eq!(
            NoiseSettings::FLOATING_ISLANDS_NOISE_SETTINGS,
            NoiseSettings::new(0, 256, 2, 1)
        );
    }

    #[test]
    fn create_rejects_invalid_height_or_min_y_alignment() {
        assert!(std::panic::catch_unwind(|| NoiseSettings::create(0, 15, 1, 1)).is_err());
        assert!(std::panic::catch_unwind(|| NoiseSettings::create(1, 16, 1, 1)).is_err());
        assert!(std::panic::catch_unwind(|| NoiseSettings::create(-1, 16, 1, 1)).is_err());
    }

    #[test]
    fn create_rejects_sum_above_dimension_max() {
        let result = std::panic::catch_unwind(|| NoiseSettings::create(2032, 16, 1, 1));
        assert!(result.is_err());
    }

    #[test]
    fn cell_dimensions_match_quart_shift() {
        let s = NoiseSettings::new(0, 16, 3, 4);
        assert_eq!(s.get_cell_width(), 12); // 3 << 2
        assert_eq!(s.get_cell_height(), 16); // 4 << 2

        let neg = NoiseSettings::new(0, 16, -1, -2);
        assert_eq!(neg.get_cell_width(), -1i32.wrapping_shl(2));
        assert_eq!(neg.get_cell_height(), -2i32.wrapping_shl(2));
    }

    #[test]
    fn clamp_to_height_accessor_uses_wrapping_arithmetic() {
        let s = NoiseSettings::create(-64, 384, 1, 2);
        let acc = MockAccessor {
            min_y: 0,
            max_y: 127,
        };
        let clamped = s.clamp_to_height_accessor(&acc);
        assert_eq!(clamped.min_y, 0);
        assert_eq!(clamped.height, 128);
        assert_eq!(clamped.noise_size_horizontal, 1);
        assert_eq!(clamped.noise_size_vertical, 2);

        let no_change_acc = MockAccessor {
            min_y: -64,
            max_y: 319,
        };
        assert_eq!(s.clamp_to_height_accessor(&no_change_acc), s);
    }

    #[test]
    fn guard_y_uses_wrapping_add_for_overflow() {
        let min_y = i32::MAX - 15; // divisible by 16
        let s = NoiseSettings::new(min_y, 16, 1, 1);
        let r = NoiseSettings::guard_y(s);
        assert!(r.is_ok());

        let bad_min_y = NoiseSettings::new(i32::MAX, 16, 1, 1);
        let r_bad = NoiseSettings::guard_y(bad_min_y);
        assert!(r_bad.is_err());
    }
}
