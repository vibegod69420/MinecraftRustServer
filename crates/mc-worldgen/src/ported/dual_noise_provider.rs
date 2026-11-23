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

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InclusiveRangeInt {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl InclusiveRangeInt {
    pub fn new(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    pub fn min_inclusive(&self) -> i32 {
        self.min_inclusive
    }

    pub fn max_inclusive(&self) -> i32 {
        self.max_inclusive
    }
}

#[derive(Debug, Clone)]
pub struct NoiseParameters {
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

impl NoiseParameters {
    pub fn new(first_octave: i32, amplitudes: Vec<f64>) -> Self {
        Self {
            first_octave,
            amplitudes,
        }
    }
}

pub struct NormalNoise {
    sampler: Box<dyn Fn(f64, f64, f64) -> f64 + Send + Sync>,
}

impl NormalNoise {
    pub fn create(seed: i64, _params: &NoiseParameters) -> Self {
        Self {
            sampler: Box::new(move |x: f64, y: f64, z: f64| {
                let mut n = seed as u64;
                n ^= x.to_bits().wrapping_mul(0x9E37_79B1_85EB_CA6D);
                n ^= y
                    .to_bits()
                    .wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
                    .rotate_left(21);
                n ^= z
                    .to_bits()
                    .wrapping_mul(0x1656_67B1_9E37_79F9)
                    .rotate_left(42);
                n = n.wrapping_mul(0x9E37_79B1_85EB_CA6D);
                n ^= n >> 33;
                ((n as f64 / u64::MAX as f64) * 2.0) - 1.0
            }),
        }
    }

    pub fn from_fn<F: Fn(f64, f64, f64) -> f64 + Send + Sync + 'static>(f: F) -> Self {
        Self {
            sampler: Box::new(f),
        }
    }

    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        (self.sampler)(x, y, z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStateProviderType {
    DualNoiseProvider,
}

// JAVA: net.minecraft.util.Mth#clampedMap
pub fn mth_clamped_map(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    if value <= from_min {
        return to_min;
    }
    if value >= from_max {
        return to_max;
    }
    (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
}

pub struct DualNoiseProvider {
    pub seed: i64,
    pub noise_parameters: NoiseParameters,
    pub scale: f32,
    noise: NormalNoise,
    pub variety: InclusiveRangeInt,
    pub slow_noise_parameters: NoiseParameters,
    pub slow_scale: f32,
    slow_noise: NormalNoise,
    pub states: Vec<BlockState>,
}

impl DualNoiseProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.DualNoiseProvider#<init>
    pub fn new(
        variety: InclusiveRangeInt,
        slow_noise_parameters: NoiseParameters,
        slow_scale: f32,
        seed: i64,
        noise_parameters: NoiseParameters,
        scale: f32,
        states: Vec<BlockState>,
    ) -> Self {
        let noise = NormalNoise::create(seed, &noise_parameters);
        let slow_noise = NormalNoise::create(seed, &slow_noise_parameters);
        Self {
            seed,
            noise_parameters,
            scale,
            noise,
            variety,
            slow_noise_parameters,
            slow_scale,
            slow_noise,
            states,
        }
    }

    pub fn new_with_noises(
        variety: InclusiveRangeInt,
        slow_scale: f32,
        scale: f32,
        noise: NormalNoise,
        slow_noise: NormalNoise,
        states: Vec<BlockState>,
    ) -> Self {
        Self {
            seed: 0,
            noise_parameters: NoiseParameters::new(0, Vec::new()),
            scale,
            noise,
            variety,
            slow_noise_parameters: NoiseParameters::new(0, Vec::new()),
            slow_scale,
            slow_noise,
            states,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#getNoiseValue
    pub fn get_noise_value(&self, pos: &BlockPos, scale: f64) -> f64 {
        self.noise.get_value(
            pos.get_x() as f64 * scale,
            pos.get_y() as f64 * scale,
            pos.get_z() as f64 * scale,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#getRandomState
    pub fn get_random_state_from_noise(&self, states: &[BlockState], value: f64) -> BlockState {
        let idx = mth_clamped_map(value, -1.0, 1.0, 0.0, states.len() as f64) as usize;
        let idx = if idx >= states.len() {
            states.len() - 1
        } else {
            idx
        };
        states[idx].clone()
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#getRandomState
    pub fn get_random_state(&self, states: &[BlockState], pos: &BlockPos, scale: f64) -> BlockState {
        let value = self.get_noise_value(pos, scale);
        self.get_random_state_from_noise(states, value)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.DualNoiseProvider#type
    pub fn provider_type(&self) -> BlockStateProviderType {
        BlockStateProviderType::DualNoiseProvider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.DualNoiseProvider#getSlowNoiseValue
    pub fn get_slow_noise_value(&self, pos: &BlockPos) -> f64 {
        self.slow_noise.get_value(
            pos.get_x() as f64 * self.slow_scale as f64,
            pos.get_y() as f64 * self.slow_scale as f64,
            pos.get_z() as f64 * self.slow_scale as f64,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.DualNoiseProvider#getState
    pub fn get_state(&self, _random: &mut impl RandomSource, pos: &BlockPos) -> BlockState {
        let slow_val = self.get_slow_noise_value(pos);
        let mapped = mth_clamped_map(
            slow_val,
            -1.0,
            1.0,
            self.variety.min_inclusive() as f64,
            (self.variety.max_inclusive() as f64) + 1.0,
        );
        let count = mapped as i32;
        let mut selected: Vec<BlockState> = Vec::with_capacity(count as usize);
        let mut i: i32 = 0;
        while i < count {
            let offset_pos = pos.offset(i.wrapping_mul(54545), 0, i.wrapping_mul(34234));
            let slow_at_offset = self.get_slow_noise_value(&offset_pos);
            let state = self.get_random_state_from_noise(&self.states, slow_at_offset);
            selected.push(state);
            i = i.wrapping_add(1);
        }
        self.get_random_state(&selected, pos, self.scale as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRandom;
    impl RandomSource for DummyRandom {
        fn next_float(&mut self) -> f32 {
            0.0
        }
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
    }

    fn state(id: u32) -> BlockState {
        BlockState::new(id)
    }

    #[test]
    fn clamped_map_bounds_and_middle() {
        assert_eq!(mth_clamped_map(-2.0, -1.0, 1.0, 0.0, 10.0), 0.0);
        assert_eq!(mth_clamped_map(2.0, -1.0, 1.0, 0.0, 10.0), 10.0);
        assert_eq!(mth_clamped_map(0.0, -1.0, 1.0, 0.0, 10.0), 5.0);
    }

    #[test]
    fn get_state_uses_variety_and_offsets() {
        let slow = NormalNoise::from_fn(|x, _y, z| {
            // deterministic: depends on x+z
            let v = ((x as i64).wrapping_add(z as i64) % 200) as f64 / 100.0 - 1.0;
            v.clamp(-1.0, 1.0)
        });
        let noise = NormalNoise::from_fn(|_, _, _| 0.0);
        let p = DualNoiseProvider::new_with_noises(
            InclusiveRangeInt::new(2, 4),
            1.0,
            1.0,
            noise,
            slow,
            vec![state(1), state(2), state(3), state(4)],
        );
        let mut rng = DummyRandom;
        let out = p.get_state(&mut rng, &BlockPos::new(10, 0, 10));
        assert!(matches!(out.id, 1 | 2 | 3 | 4));
    }

    #[test]
    fn slow_noise_value_scales_coordinates() {
        let slow = NormalNoise::from_fn(|x, y, z| x + y + z);
        let noise = NormalNoise::from_fn(|_, _, _| 0.0);
        let p = DualNoiseProvider::new_with_noises(
            InclusiveRangeInt::new(1, 1),
            0.5,
            1.0,
            noise,
            slow,
            vec![state(1)],
        );
        let v = p.get_slow_noise_value(&BlockPos::new(2, 4, 6));
        assert_eq!(v, (1.0 + 2.0 + 3.0));
    }

    #[test]
    fn negative_coords_and_wrapping_offsets() {
        let slow = NormalNoise::from_fn(|x, _y, z| {
            if x < 0.0 || z < 0.0 {
                -1.0
            } else {
                1.0
            }
        });
        let noise = NormalNoise::from_fn(|_, _, _| -1.0);
        let p = DualNoiseProvider::new_with_noises(
            InclusiveRangeInt::new(1, 2),
            1.0,
            1.0,
            noise,
            slow,
            vec![state(5), state(6)],
        );
        let mut rng = DummyRandom;
        let out = p.get_state(&mut rng, &BlockPos::new(-10, 0, -10));
        assert_eq!(out, state(5));
    }

    #[test]
    fn random_state_from_noise_clamps_index() {
        let slow = NormalNoise::from_fn(|_, _, _| 0.0);
        let noise = NormalNoise::from_fn(|_, _, _| 0.0);
        let p = DualNoiseProvider::new_with_noises(
            InclusiveRangeInt::new(1, 1),
            1.0,
            1.0,
            noise,
            slow,
            vec![state(9), state(8), state(7)],
        );
        assert_eq!(p.get_random_state_from_noise(&p.states, 1.0).id, 7);
        assert_eq!(p.get_random_state_from_noise(&p.states, -1.0).id, 9);
    }
}
