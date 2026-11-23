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
    NoiseThresholdProvider,
}

fn get_random(states: &[BlockState], random: &mut impl RandomSource) -> BlockState {
    let idx = random.next_int(states.len() as i32) as usize;
    states[idx].clone()
}

pub struct NoiseThresholdProvider {
    pub seed: i64,
    pub noise_parameters: NoiseParameters,
    pub scale: f32,
    noise: NormalNoise,
    pub threshold: f32,
    pub high_chance: f32,
    pub default_state: BlockState,
    pub low_states: Vec<BlockState>,
    pub high_states: Vec<BlockState>,
}

impl NoiseThresholdProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseThresholdProvider#<init>
    pub fn new(
        seed: i64,
        noise_parameters: NoiseParameters,
        scale: f32,
        threshold: f32,
        high_chance: f32,
        default_state: BlockState,
        low_states: Vec<BlockState>,
        high_states: Vec<BlockState>,
    ) -> Self {
        let noise = NormalNoise::create(seed, &noise_parameters);
        Self {
            seed,
            noise_parameters,
            scale,
            noise,
            threshold,
            high_chance,
            default_state,
            low_states,
            high_states,
        }
    }

    pub fn new_with_noise(
        noise: NormalNoise,
        scale: f32,
        threshold: f32,
        high_chance: f32,
        default_state: BlockState,
        low_states: Vec<BlockState>,
        high_states: Vec<BlockState>,
    ) -> Self {
        Self {
            seed: 0,
            noise_parameters: NoiseParameters::new(0, Vec::new()),
            scale,
            noise,
            threshold,
            high_chance,
            default_state,
            low_states,
            high_states,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseBasedStateProvider#getNoiseValue
    pub fn get_noise_value(&self, pos: &BlockPos, scale: f64) -> f64 {
        self.noise.get_value(
            pos.get_x() as f64 * scale,
            pos.get_y() as f64 * scale,
            pos.get_z() as f64 * scale,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseThresholdProvider#type
    pub fn provider_type(&self) -> BlockStateProviderType {
        BlockStateProviderType::NoiseThresholdProvider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseThresholdProvider#getState
    pub fn get_state(&self, random: &mut impl RandomSource, pos: &BlockPos) -> BlockState {
        let noise = self.get_noise_value(pos, self.scale as f64);
        if noise < self.threshold as f64 {
            get_random(&self.low_states, random)
        } else if random.next_float() < self.high_chance {
            get_random(&self.high_states, random)
        } else {
            self.default_state.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        float_idx: usize,
        int_idx: usize,
    }

    impl SeqRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                ints,
                float_idx: 0,
                int_idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.float_idx];
            self.float_idx += 1;
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints[self.int_idx];
            self.int_idx += 1;
            assert!(v >= 0 && v < bound);
            v
        }
    }

    fn state(id: u32) -> BlockState {
        BlockState::new(id)
    }

    fn provider_with_noise(noise: f64, threshold: f32, high_chance: f32) -> NoiseThresholdProvider {
        NoiseThresholdProvider::new_with_noise(
            NormalNoise::from_fn(move |_, _, _| noise),
            1.0,
            threshold,
            high_chance,
            state(0),
            vec![state(10), state(11)],
            vec![state(20), state(21)],
        )
    }

    #[test]
    fn below_threshold_selects_low_state() {
        let p = provider_with_noise(-0.5, 0.0, 1.0);
        let mut rng = SeqRandom::new(vec![], vec![1]);
        let out = p.get_state(&mut rng, &BlockPos::new(1, 2, 3));
        assert_eq!(out, state(11));
    }

    #[test]
    fn above_threshold_high_chance_selects_high_state() {
        let p = provider_with_noise(0.5, 0.0, 0.8);
        let mut rng = SeqRandom::new(vec![0.5], vec![0]);
        let out = p.get_state(&mut rng, &BlockPos::new(0, 0, 0));
        assert_eq!(out, state(20));
    }

    #[test]
    fn above_threshold_low_roll_returns_default() {
        let p = provider_with_noise(0.5, 0.0, 0.25);
        let mut rng = SeqRandom::new(vec![0.25], vec![]);
        let out = p.get_state(&mut rng, &BlockPos::new(0, 0, 0));
        assert_eq!(out, state(0));
    }

    #[test]
    fn threshold_boundary_uses_less_than() {
        let p = provider_with_noise(0.25, 0.25, 1.0);
        let mut rng = SeqRandom::new(vec![0.0], vec![0]);
        let out = p.get_state(&mut rng, &BlockPos::new(0, 0, 0));
        assert_eq!(out, state(20));
    }

    #[test]
    fn negative_coordinates_and_noise() {
        let p = NoiseThresholdProvider::new_with_noise(
            NormalNoise::from_fn(|x, y, z| (x + y + z) / 100.0),
            1.0,
            -0.5,
            1.0,
            state(0),
            vec![state(7)],
            vec![state(8)],
        );
        let mut rng_low = SeqRandom::new(vec![], vec![0]);
        assert_eq!(p.get_state(&mut rng_low, &BlockPos::new(-100, -10, -10)), state(7));

        let mut rng_high = SeqRandom::new(vec![0.0], vec![0]);
        assert_eq!(p.get_state(&mut rng_high, &BlockPos::new(10, 10, 10)), state(8));
    }

    #[test]
    fn float_bits_threshold_comparison() {
        let threshold = f32::from_bits(0x3F00_0001); // just above 0.5
        let noise = 0.5_f64;
        let p = provider_with_noise(noise, threshold, 1.0);
        let mut rng = SeqRandom::new(vec![], vec![0]);
        // noise (0.5) < threshold (~0.50000006) => low
        assert_eq!(p.get_state(&mut rng, &BlockPos::new(0, 0, 0)), state(10));
    }
}
