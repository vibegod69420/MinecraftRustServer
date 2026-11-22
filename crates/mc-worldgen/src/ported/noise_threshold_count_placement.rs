use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

pub trait RandomSource {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifierType {
    NoiseThresholdCount,
}

thread_local! {
    static TEST_BIOME_INFO_NOISE: Cell<f64> = Cell::new(0.0);
    static LAST_NOISE_X: Cell<f64> = Cell::new(0.0);
    static LAST_NOISE_Z: Cell<f64> = Cell::new(0.0);
    static LAST_NOISE_USE_Y: Cell<bool> = Cell::new(false);
}

pub fn set_test_biome_info_noise(value: f64) {
    TEST_BIOME_INFO_NOISE.with(|c| c.set(value));
}

pub fn last_biome_info_noise_args() -> (f64, f64, bool) {
    (
        LAST_NOISE_X.with(|c| c.get()),
        LAST_NOISE_Z.with(|c| c.get()),
        LAST_NOISE_USE_Y.with(|c| c.get()),
    )
}

pub mod biome {
    use super::{LAST_NOISE_USE_Y, LAST_NOISE_X, LAST_NOISE_Z, TEST_BIOME_INFO_NOISE};

    #[derive(Debug, Clone, Copy, Default)]
    pub struct BiomeInfoNoise;

    impl BiomeInfoNoise {
        pub fn get_value(&self, x: f64, z: f64, use_y: bool) -> f64 {
            LAST_NOISE_X.with(|c| c.set(x));
            LAST_NOISE_Z.with(|c| c.set(z));
            LAST_NOISE_USE_Y.with(|c| c.set(use_y));
            TEST_BIOME_INFO_NOISE.with(|c| c.get())
        }
    }

    pub static BIOME_INFO_NOISE: BiomeInfoNoise = BiomeInfoNoise;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseThresholdCountPlacement {
    noise_level: f64,
    below_noise: i32,
    above_noise: i32,
}

impl NoiseThresholdCountPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.NoiseThresholdCountPlacement#<init>
    fn new(noise_level: f64, below_noise: i32, above_noise: i32) -> Self {
        Self {
            noise_level,
            below_noise,
            above_noise,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.NoiseThresholdCountPlacement#of
    pub fn of(noise_level: f64, below_noise: i32, above_noise: i32) -> Self {
        Self::new(noise_level, below_noise, above_noise)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.NoiseThresholdCountPlacement#count
    pub fn count<R: RandomSource + ?Sized>(&self, _random: &R, pos: &BlockPos) -> i32 {
        let noise_val = biome::BIOME_INFO_NOISE.get_value(
            f64::from(pos.get_x()) / 200.0,
            f64::from(pos.get_z()) / 200.0,
            false,
        );
        if noise_val < self.noise_level {
            self.below_noise
        } else {
            self.above_noise
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.NoiseThresholdCountPlacement#type
    pub fn placement_type(&self) -> PlacementModifierType {
        PlacementModifierType::NoiseThresholdCount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRandom;
    impl RandomSource for DummyRandom {}

    #[test]
    fn count_below_threshold_returns_below_noise() {
        set_test_biome_info_noise(-0.5);
        let p = NoiseThresholdCountPlacement::of(0.0, 10, 20);
        let pos = BlockPos::new(100, 64, 200);
        assert_eq!(p.count(&DummyRandom, &pos), 10);
    }

    #[test]
    fn count_at_or_above_threshold_returns_above_noise() {
        let p = NoiseThresholdCountPlacement::of(0.0, 10, 20);
        let pos = BlockPos::new(0, 0, 0);

        set_test_biome_info_noise(0.0);
        assert_eq!(p.count(&DummyRandom, &pos), 20);

        set_test_biome_info_noise(0.5);
        assert_eq!(p.count(&DummyRandom, &pos), 20);
    }

    #[test]
    fn count_passes_scaled_coordinates_to_biome_noise() {
        set_test_biome_info_noise(0.0);
        let p = NoiseThresholdCountPlacement::of(1.0, 1, 2);
        let pos = BlockPos::new(400, 7, -600);
        let _ = p.count(&DummyRandom, &pos);
        let (x, z, use_y) = last_biome_info_noise_args();
        assert_eq!(x, 2.0);
        assert_eq!(z, -3.0);
        assert!(!use_y);
    }

    #[test]
    fn count_nan_noise_returns_above_noise() {
        set_test_biome_info_noise(f64::NAN);
        let p = NoiseThresholdCountPlacement::of(0.0, 10, 20);
        let pos = BlockPos::new(0, 0, 0);
        // NaN < noise_level is false in Java/IEEE 754
        assert_eq!(p.count(&DummyRandom, &pos), 20);
    }

    #[test]
    fn count_negative_coordinates_and_extreme_int_counts() {
        set_test_biome_info_noise(-1.0);
        let p = NoiseThresholdCountPlacement::of(0.0, i32::MIN, i32::MAX);
        let pos = BlockPos::new(i32::MIN, -64, i32::MAX);
        assert_eq!(p.count(&DummyRandom, &pos), i32::MIN);

        set_test_biome_info_noise(1.0);
        assert_eq!(p.count(&DummyRandom, &pos), i32::MAX);
    }

    #[test]
    fn placement_type_is_noise_threshold_count() {
        let p = NoiseThresholdCountPlacement::of(-1.0, 1, 2);
        assert_eq!(p.placement_type(), PlacementModifierType::NoiseThresholdCount);
    }
}
