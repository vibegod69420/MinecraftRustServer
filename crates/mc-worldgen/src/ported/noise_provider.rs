//! Self-contained port of `net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

pub trait RandomSource {}

#[derive(Clone, Debug, Default)]
pub struct SimpleRandomSource;

impl RandomSource for SimpleRandomSource {}

#[derive(Clone, Debug, PartialEq)]
pub struct NoiseParameters {
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct LegacyRandomSource {
    seed: i64,
}

impl LegacyRandomSource {
    pub fn new(seed: i64) -> Self {
        Self { seed }
    }
}

#[derive(Clone, Debug)]
pub struct WorldgenRandom {
    seed: i64,
}

impl WorldgenRandom {
    pub fn new(source: LegacyRandomSource) -> Self {
        Self { seed: source.seed }
    }
}

#[derive(Clone, Debug)]
pub struct NormalNoise {
    seed: i64,
    parameters: NoiseParameters,
}

impl NormalNoise {
    pub fn create(random: WorldgenRandom, parameters: NoiseParameters) -> Self {
        Self {
            seed: random.seed,
            parameters,
        }
    }

    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut n = self.seed as u64;
        n ^= x.to_bits().wrapping_mul(0x9E37_79B1_85EB_CA6D_u64);
        n ^= y.to_bits().wrapping_mul(0xC2B2_AE3D_27D4_EB4F_u64);
        n ^= z.to_bits().wrapping_mul(0x1656_67B1_9E37_79F9_u64);
        n ^= (self.parameters.first_octave as i64 as u64)
            .wrapping_mul(0x27D4_EB2F_1656_67C5_u64);
        for (i, amp) in self.parameters.amplitudes.iter().enumerate() {
            n ^= amp
                .to_bits()
                .wrapping_mul((i as u64).wrapping_add(0x9E37_79B9_7F4A_7C15_u64));
        }
        n ^= n >> 33;
        n = n.wrapping_mul(0xFF51_AFD7_ED55_8CCD_u64);
        n ^= n >> 33;
        ((n >> 11) as f64) * (1.0 / ((1u64 << 53) as f64)) * 2.0 - 1.0
    }
}

#[derive(Clone, Debug)]
pub struct NoiseBasedStateProvider {
    pub seed: i64,
    pub parameters: NoiseParameters,
    pub scale: f32,
    pub noise: NormalNoise,
}

impl NoiseBasedStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseBasedStateProvider#<init>
    pub fn new(seed: i64, parameters: NoiseParameters, scale: f32) -> Self {
        let noise = NormalNoise::create(
            WorldgenRandom::new(LegacyRandomSource::new(seed)),
            parameters.clone(),
        );
        Self {
            seed,
            parameters,
            scale,
            noise,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseBasedStateProvider#getNoiseValue
    pub fn get_noise_value(&self, pos: &BlockPos, scale: f64) -> f64 {
        self.noise.get_value(
            f64::from(pos.get_x()) * scale,
            f64::from(pos.get_y()) * scale,
            f64::from(pos.get_z()) * scale,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockStateProviderKind {
    NoiseProvider,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockStateProviderType {
    pub kind: BlockStateProviderKind,
}

pub struct Mth;

impl Mth {
    pub fn clamp(v: f64, min: f64, max: f64) -> f64 {
        if v < min {
            min
        } else if v > max {
            max
        } else {
            v
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoiseProviderCodecGroup {
    pub seed_field: &'static str,
    pub noise_field: &'static str,
    pub scale_field: &'static str,
    pub states_field: &'static str,
}

#[derive(Clone, Debug)]
pub struct NoiseProvider {
    pub base: NoiseBasedStateProvider,
    pub states: Vec<BlockState>,
}

impl NoiseProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#noiseProviderCodec
    pub fn noise_provider_codec<P>() -> NoiseProviderCodecGroup {
        let _ = std::marker::PhantomData::<P>;
        NoiseProviderCodecGroup {
            seed_field: "seed",
            noise_field: "noise",
            scale_field: "scale",
            states_field: "states",
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#<init>
    pub fn new(seed: i64, parameters: NoiseParameters, scale: f32, states: Vec<BlockState>) -> Self {
        Self {
            base: NoiseBasedStateProvider::new(seed, parameters, scale),
            states,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#type
    pub fn provider_type(&self) -> BlockStateProviderType {
        BlockStateProviderType {
            kind: BlockStateProviderKind::NoiseProvider,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#getState
    pub fn get_state<R: RandomSource + ?Sized>(&self, _random: &R, pos: &BlockPos) -> BlockState {
        self.get_random_state_at_pos(&self.states, pos, f64::from(self.base.scale))
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#getRandomState
    pub fn get_random_state_at_pos(&self, states: &[BlockState], pos: &BlockPos, scale: f64) -> BlockState {
        let noise_val = self.base.get_noise_value(pos, scale);
        self.get_random_state_by_value(states, noise_val)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseProvider#getRandomState
    pub fn get_random_state_by_value(&self, states: &[BlockState], value: f64) -> BlockState {
        let v = (1.0_f64 + value) / 2.0_f64;
        let clamped = Mth::clamp(v, 0.0_f64, 0.9999_f64);
        let idx = (clamped * (states.len() as f64)) as i32;
        states[idx as usize].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> NoiseParameters {
        NoiseParameters {
            first_octave: -7,
            amplitudes: vec![1.0, 1.0],
        }
    }

    fn states(n: u32) -> Vec<BlockState> {
        (0..n).map(BlockState::new).collect()
    }

    #[test]
    fn get_random_state_by_value_maps_and_clamps() {
        let p = NoiseProvider::new(0, params(), 1.0_f32, states(10));
        let s = states(10);
        assert_eq!(p.get_random_state_by_value(&s, -1.0_f64), BlockState::new(0));
        assert_eq!(p.get_random_state_by_value(&s, 1.0_f64), BlockState::new(9));
        assert_eq!(p.get_random_state_by_value(&s, 0.0_f64), BlockState::new(5));
        assert_eq!(p.get_random_state_by_value(&s, -100.0_f64), BlockState::new(0));
        assert_eq!(p.get_random_state_by_value(&s, 100.0_f64), BlockState::new(9));
    }

    #[test]
    fn single_state_always_selected() {
        let p = NoiseProvider::new(1, params(), 1.0_f32, vec![BlockState::new(42)]);
        let s = vec![BlockState::new(42)];
        assert_eq!(p.get_random_state_by_value(&s, -1.0_f64), BlockState::new(42));
        assert_eq!(p.get_random_state_by_value(&s, 0.0_f64), BlockState::new(42));
        assert_eq!(p.get_random_state_by_value(&s, 1.0_f64), BlockState::new(42));
    }

    #[test]
    fn get_random_state_at_pos_uses_noise_and_scale() {
        let p = NoiseProvider::new(123, params(), 0.25_f32, states(5));
        let s = states(5);
        let pos = BlockPos::new(10, -20, 30);
        let got = p.get_random_state_at_pos(&s, &pos, 0.5_f64);
        let noise_val = p.base.get_noise_value(&pos, 0.5_f64);
        let expected = p.get_random_state_by_value(&s, noise_val);
        assert_eq!(got, expected);
    }

    #[test]
    fn get_state_uses_provider_scale() {
        let p = NoiseProvider::new(42, params(), 0.1_f32, states(8));
        let pos = BlockPos::new(-7, 64, -3);
        let got = p.get_state(&SimpleRandomSource, &pos);
        let expected = p.get_random_state_at_pos(&p.states, &pos, f64::from(0.1_f32));
        assert_eq!(got, expected);
    }

    #[test]
    fn provider_type_and_codec_fields() {
        let p = NoiseProvider::new(0, params(), 1.0_f32, states(1));
        assert_eq!(
            p.provider_type(),
            BlockStateProviderType {
                kind: BlockStateProviderKind::NoiseProvider
            }
        );
        let g = NoiseProvider::noise_provider_codec::<NoiseProvider>();
        assert_eq!(g.seed_field, "seed");
        assert_eq!(g.noise_field, "noise");
        assert_eq!(g.scale_field, "scale");
        assert_eq!(g.states_field, "states");
    }
}
