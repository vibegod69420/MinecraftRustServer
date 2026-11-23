//! Self-contained port of `net.minecraft.world.level.levelgen.feature.stateproviders.NoiseBasedStateProvider`.

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
        n ^= x
            .to_bits()
            .wrapping_mul(0x9E37_79B1_85EB_CA6D_u64);
        n ^= y
            .to_bits()
            .wrapping_mul(0xC2B2_AE3D_27D4_EB4F_u64);
        n ^= z
            .to_bits()
            .wrapping_mul(0x1656_67B1_9E37_79F9_u64);
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

/// Stub descriptor for the DFU `noiseCodec` group fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoiseCodecGroup {
    pub seed_field: &'static str,
    pub noise_field: &'static str,
    pub scale_field: &'static str,
}

#[derive(Clone, Debug)]
pub struct NoiseBasedStateProvider {
    pub seed: i64,
    pub parameters: NoiseParameters,
    pub scale: f32,
    pub noise: NormalNoise,
}

impl NoiseBasedStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.NoiseBasedStateProvider#noiseCodec
    pub fn noise_codec<P>() -> NoiseCodecGroup {
        let _ = std::marker::PhantomData::<P>;
        NoiseCodecGroup {
            seed_field: "seed",
            noise_field: "noise",
            scale_field: "scale",
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_fields_and_creates_noise() {
        let params = NoiseParameters {
            first_octave: -7,
            amplitudes: vec![1.0, 1.0],
        };
        let provider = NoiseBasedStateProvider::new(12345, params.clone(), 0.05_f32);
        assert_eq!(provider.seed, 12345);
        assert_eq!(provider.parameters, params);
        assert_eq!(provider.scale.to_bits(), 0.05_f32.to_bits());
        let expected_noise = NormalNoise::create(
            WorldgenRandom::new(LegacyRandomSource::new(12345)),
            params,
        );
        let p = BlockPos::new(1, 2, 3);
        assert_eq!(
            provider.noise.get_value(1.0, 2.0, 3.0).to_bits(),
            expected_noise.get_value(1.0, 2.0, 3.0).to_bits()
        );
    }

    #[test]
    fn get_noise_value_scales_coordinates() {
        let params = NoiseParameters {
            first_octave: -3,
            amplitudes: vec![1.0],
        };
        let provider = NoiseBasedStateProvider::new(42, params, 0.1_f32);
        let pos = BlockPos::new(10, 20, 30);
        let scale = 0.5_f64;
        let got = provider.get_noise_value(&pos, scale);
        let expected = provider
            .noise
            .get_value(10.0_f64 * scale, 20.0_f64 * scale, 30.0_f64 * scale);
        assert_eq!(got.to_bits(), expected.to_bits());
    }

    #[test]
    fn get_noise_value_negative_and_extreme_coordinates() {
        let params = NoiseParameters {
            first_octave: -2,
            amplitudes: vec![1.0, 0.5],
        };
        let provider = NoiseBasedStateProvider::new(99, params, 1.0_f32);
        let neg = BlockPos::new(-100, -64, -200);
        let got_neg = provider.get_noise_value(&neg, 0.01_f64);
        let exp_neg = provider
            .noise
            .get_value(-100.0_f64 * 0.01_f64, -64.0_f64 * 0.01_f64, -200.0_f64 * 0.01_f64);
        assert_eq!(got_neg.to_bits(), exp_neg.to_bits());

        let extreme = BlockPos::new(i32::MIN, i32::MAX, 0);
        let got_ext = provider.get_noise_value(&extreme, 1.0_f64);
        let exp_ext = provider.noise.get_value(
            f64::from(i32::MIN),
            f64::from(i32::MAX),
            0.0_f64,
        );
        assert_eq!(got_ext.to_bits(), exp_ext.to_bits());
    }

    #[test]
    fn noise_codec_group_fields() {
        let g = NoiseBasedStateProvider::noise_codec::<NoiseBasedStateProvider>();
        assert_eq!(g.seed_field, "seed");
        assert_eq!(g.noise_field, "noise");
        assert_eq!(g.scale_field, "scale");
    }

    #[test]
    fn deterministic_for_same_seed_and_params() {
        let params = NoiseParameters {
            first_octave: -4,
            amplitudes: vec![1.0, 2.0],
        };
        let a = NoiseBasedStateProvider::new(7, params.clone(), 0.25_f32);
        let b = NoiseBasedStateProvider::new(7, params, 0.25_f32);
        let pos = BlockPos::new(1, -2, 3);
        assert_eq!(
            a.get_noise_value(&pos, 0.1_f64).to_bits(),
            b.get_noise_value(&pos, 0.1_f64).to_bits()
        );
    }
}
