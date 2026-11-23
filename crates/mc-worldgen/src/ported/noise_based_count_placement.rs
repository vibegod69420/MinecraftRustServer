#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub trait BiomeInfoNoise {
    fn get_value(&self, x: f64, z: f64, use_y: bool) -> f64;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseBasedCountPlacement {
    pub noise_to_count_ratio: i32,
    pub noise_factor: f64,
    pub noise_offset: f64,
}

impl NoiseBasedCountPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.NoiseBasedCountPlacement#<init>
    fn new(noise_to_count_ratio: i32, noise_factor: f64, noise_offset: f64) -> Self {
        Self {
            noise_to_count_ratio,
            noise_factor,
            noise_offset,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.NoiseBasedCountPlacement#of
    pub fn of(noise_to_count_ratio: i32, noise_factor: f64, noise_offset: f64) -> Self {
        Self::new(noise_to_count_ratio, noise_factor, noise_offset)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.NoiseBasedCountPlacement#count
    pub fn count(
        &self,
        _random: &mut impl RandomSource,
        pos: &BlockPos,
        biome_info_noise: &impl BiomeInfoNoise,
    ) -> i32 {
        let noise_x = (pos.get_x() as f64) / self.noise_factor;
        let noise_z = (pos.get_z() as f64) / self.noise_factor;
        let noise_value = biome_info_noise.get_value(noise_x, noise_z, false);
        let count = ((noise_value + self.noise_offset) * (self.noise_to_count_ratio as f64)).ceil();
        java_double_to_i32(count)
    }
}

fn java_double_to_i32(v: f64) -> i32 {
    if v.is_nan() {
        0
    } else if v >= i32::MAX as f64 {
        i32::MAX
    } else if v <= i32::MIN as f64 {
        i32::MIN
    } else {
        v as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRandom;
    impl RandomSource for DummyRandom {}

    struct FixedNoise(f64);
    impl BiomeInfoNoise for FixedNoise {
        fn get_value(&self, _x: f64, _z: f64, _use_y: bool) -> f64 {
            self.0
        }
    }

    struct RecordingNoise {
        value: f64,
        last_x: std::cell::Cell<f64>,
        last_z: std::cell::Cell<f64>,
        last_use_y: std::cell::Cell<bool>,
    }

    impl RecordingNoise {
        fn new(value: f64) -> Self {
            Self {
                value,
                last_x: std::cell::Cell::new(0.0),
                last_z: std::cell::Cell::new(0.0),
                last_use_y: std::cell::Cell::new(true),
            }
        }
    }

    impl BiomeInfoNoise for RecordingNoise {
        fn get_value(&self, x: f64, z: f64, use_y: bool) -> f64 {
            self.last_x.set(x);
            self.last_z.set(z);
            self.last_use_y.set(use_y);
            self.value
        }
    }

    #[test]
    fn count_basic_and_ceil() {
        let placement = NoiseBasedCountPlacement::of(10, 200.0, 0.0);
        let mut rng = DummyRandom;
        let pos = BlockPos::new(0, 64, 0);
        // noise 0.15 -> (0.15) * 10 = 1.5 -> ceil = 2
        assert_eq!(placement.count(&mut rng, &pos, &FixedNoise(0.15)), 2);
        // noise 0.1 -> 1.0 -> ceil = 1
        assert_eq!(placement.count(&mut rng, &pos, &FixedNoise(0.1)), 1);
    }

    #[test]
    fn count_uses_scaled_coordinates_and_no_y() {
        let placement = NoiseBasedCountPlacement::of(1, 100.0, 0.0);
        let mut rng = DummyRandom;
        let pos = BlockPos::new(250, 70, -50);
        let noise = RecordingNoise::new(0.0);
        let _ = placement.count(&mut rng, &pos, &noise);
        assert_eq!(noise.last_x.get().to_bits(), (2.5f64).to_bits());
        assert_eq!(noise.last_z.get().to_bits(), (-0.5f64).to_bits());
        assert!(!noise.last_use_y.get());
    }

    #[test]
    fn count_with_offset_and_negative_noise() {
        let placement = NoiseBasedCountPlacement::of(10, 1.0, 0.5);
        let mut rng = DummyRandom;
        let pos = BlockPos::new(0, 0, 0);
        // (-0.8 + 0.5) * 10 = -3.0 -> ceil = -3
        assert_eq!(placement.count(&mut rng, &pos, &FixedNoise(-0.8)), -3);
        // (-0.85 + 0.5) * 10 = -3.5 -> ceil = -3
        assert_eq!(placement.count(&mut rng, &pos, &FixedNoise(-0.85)), -3);
    }

    #[test]
    fn count_zero_ratio() {
        let placement = NoiseBasedCountPlacement::of(0, 200.0, 1.0);
        let mut rng = DummyRandom;
        let pos = BlockPos::new(123, 64, -456);
        assert_eq!(placement.count(&mut rng, &pos, &FixedNoise(999.0)), 0);
    }

    #[test]
    fn java_double_to_i32_clamps() {
        assert_eq!(super::java_double_to_i32(f64::NAN), 0);
        assert_eq!(super::java_double_to_i32(1e20), i32::MAX);
        assert_eq!(super::java_double_to_i32(-1e20), i32::MIN);
        assert_eq!(super::java_double_to_i32(42.9), 42);
        assert_eq!(super::java_double_to_i32(-42.9), -42);
    }
}
