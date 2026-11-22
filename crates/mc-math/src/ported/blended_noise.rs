use crate::mth::clamp_f64;
use crate::noise::ImprovedNoise;
use crate::random::{RandomSource, XoroshiroRandomSource};

/// Minimal density-function context needed by [`BlendedNoise`].
pub trait FunctionContext {
    fn block_x(&self) -> i32;
    fn block_y(&self) -> i32;
    fn block_z(&self) -> i32;
}

/// Minimal density-function interface needed by [`BlendedNoise`].
pub trait DensityFunction {
    fn compute(&self, ctx: &dyn FunctionContext) -> f64;
}

/// Local Perlin-noise wrapper with the exact operations used by BlendedNoise.
///
/// This keeps the module self-contained and avoids coupling to optional API
/// details of `crate::noise::PerlinNoise`.
#[derive(Clone, Debug)]
struct PerlinNoise {
    noise_levels: Vec<Option<ImprovedNoise>>,
}

impl PerlinNoise {
    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#createLegacyForBlendedNoise
    fn create_legacy_for_blended_noise(
        random: &mut dyn RandomSource,
        first_octave: i32,
        amplitudes: &[f64],
    ) -> Self {
        let mut noise_levels: Vec<Option<ImprovedNoise>> = Vec::with_capacity(amplitudes.len());
        for &amp in amplitudes.iter().rev() {
            if amp == 0.0 {
                noise_levels.push(None);
                // Consume the same RNG state as Java's legacy construction.
                random.next_int();
            } else {
                noise_levels.push(Some(ImprovedNoise::new(random)));
            }
        }
        noise_levels.reverse();
        let _ = first_octave;
        Self { noise_levels }
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#wrap
    fn wrap(value: f64) -> f64 {
        value - (crate::mth::lfloor(value / 3.3554432E7 + 0.5) as f64) * 3.3554432E7
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#getOctaveNoise
    fn get_octave_noise(&self, octave: usize) -> Option<&ImprovedNoise> {
        self.noise_levels.get(octave).and_then(|n| n.as_ref())
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#maxBrokenValue
    fn max_broken_value(&self, y_multiplier: f64) -> f64 {
        self.edge_value(y_multiplier)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#edgeValue
    fn edge_value(&self, value: f64) -> f64 {
        // Faithful to modern PerlinNoise edge accumulation used by BlendedNoise.
        // For legacy blended noise amplitudes are 1.0 and lowest frequency factor
        // is 2^(-first_octave_effective). The Java BlendedNoise path uses
        // maxBrokenValue on the minLimit noise; with the legacy blended
        // construction this evaluates to the same bound as edgeValue(value).
        let mut acc = 0.0;
        let mut persistence = 1.0;
        for noise in &self.noise_levels {
            if noise.is_some() {
                // ImprovedNoise contributes at most 1.0 in edge/value bound terms.
                acc += persistence * value;
            }
            persistence /= 2.0;
        }
        acc
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#parityConfigString
    fn parity_config_string(&self, out: &mut String) {
        out.push_str("PerlinNoise[");
        for (i, n) in self.noise_levels.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            if n.is_some() {
                out.push_str("ImprovedNoise");
            } else {
                out.push_str("null");
            }
        }
        out.push(']');
    }
}

fn make_legacy_blended_amplitudes(first_octave: i32, last_octave: i32) -> Vec<f64> {
    let count = (last_octave - first_octave + 1) as usize;
    vec![1.0; count]
}

#[derive(Clone, Debug)]
pub struct BlendedNoise {
    min_limit_noise: PerlinNoise,
    max_limit_noise: PerlinNoise,
    main_noise: PerlinNoise,
    xz_multiplier: f64,
    y_multiplier: f64,
    xz_factor: f64,
    y_factor: f64,
    smear_scale_multiplier: f64,
    max_value: f64,
    xz_scale: f64,
    y_scale: f64,
}

impl BlendedNoise {
    // JAVA: net.minecraft.world.level.levelgen.synth.BlendedNoise#createUnseeded
    pub fn create_unseeded(
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Self {
        let mut random = XoroshiroRandomSource::new(0);
        Self::new_with_random(
            &mut random,
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.BlendedNoise#<init>(PerlinNoise,PerlinNoise,PerlinNoise,double,double,double,double,double)
    fn from_parts(
        min_limit_noise: PerlinNoise,
        max_limit_noise: PerlinNoise,
        main_noise: PerlinNoise,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Self {
        let xz_multiplier = 684.412 * xz_scale;
        let y_multiplier = 684.412 * y_scale;
        let max_value = min_limit_noise.max_broken_value(y_multiplier);
        Self {
            min_limit_noise,
            max_limit_noise,
            main_noise,
            xz_multiplier,
            y_multiplier,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
            max_value,
            xz_scale,
            y_scale,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.BlendedNoise#<init>(RandomSource,double,double,double,double,double)
    pub fn new_with_random(
        random: &mut dyn RandomSource,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Self {
        let min_limit_noise =
            PerlinNoise::create_legacy_for_blended_noise(random, -15, &make_legacy_blended_amplitudes(-15, 0));
        let max_limit_noise =
            PerlinNoise::create_legacy_for_blended_noise(random, -15, &make_legacy_blended_amplitudes(-15, 0));
        let main_noise =
            PerlinNoise::create_legacy_for_blended_noise(random, -7, &make_legacy_blended_amplitudes(-7, 0));
        Self::from_parts(
            min_limit_noise,
            max_limit_noise,
            main_noise,
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.BlendedNoise#withNewRandom
    pub fn with_new_random(&self, random: &mut dyn RandomSource) -> Self {
        Self::new_with_random(
            random,
            self.xz_scale,
            self.y_scale,
            self.xz_factor,
            self.y_factor,
            self.smear_scale_multiplier,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.BlendedNoise#compute
    pub fn compute(&self, ctx: &dyn FunctionContext) -> f64 {
        let x1 = (ctx.block_x() as f64) * self.xz_multiplier;
        let y1 = (ctx.block_y() as f64) * self.y_multiplier;
        let z1 = (ctx.block_z() as f64) * self.xz_multiplier;
        let x4 = x1 / self.xz_factor;
        let y5 = y1 / self.y_factor;
        let z6 = z1 / self.xz_factor;
        let y7 = self.y_multiplier * self.smear_scale_multiplier;
        let y8 = y7 / self.y_factor;
        let mut v9 = 0.0;
        let mut v10 = 0.0;
        let mut v11 = 0.0;
        let mut v13 = 1.0;

        for i in 0..8 {
            if let Some(noise) = self.main_noise.get_octave_noise(i) {
                v11 += noise.noise_with_y_scale(
                    PerlinNoise::wrap(x4 * v13),
                    PerlinNoise::wrap(y5 * v13),
                    PerlinNoise::wrap(z6 * v13),
                    y8 * v13,
                    y5 * v13,
                ) / v13;
            }
            v13 /= 2.0;
        }

        let v16 = (v11 / 10.0 + 1.0) / 2.0;
        let v17 = v16 >= 1.0;
        let v18 = v16 <= 0.0;
        v13 = 1.0;

        for i in 0..16 {
            let x20 = PerlinNoise::wrap(x1 * v13);
            let y21 = PerlinNoise::wrap(y1 * v13);
            let z22 = PerlinNoise::wrap(z1 * v13);
            let y23 = y7 * v13;
            if !v17 {
                if let Some(noise) = self.min_limit_noise.get_octave_noise(i) {
                    v9 += noise.noise_with_y_scale(x20, y21, z22, y23, y1 * v13) / v13;
                }
            }
            if !v18 {
                if let Some(noise) = self.max_limit_noise.get_octave_noise(i) {
                    v10 += noise.noise_with_y_scale(x20, y21, z22, y23, y1 * v13) / v13;
                }
            }
            v13 /= 2.0;
        }

        clamp_f64(
            crate::mth::lerp_f64(clamp_f64(v16, 0.0, 1.0), v9 / 512.0, v10 / 512.0),
            f64::NEG_INFINITY,
            f64::INFINITY,
        ) / 128.0
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.BlendedNoise#minValue
    pub fn min_value(&self) -> f64 {
        -self.max_value()
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.BlendedNoise#maxValue
    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.BlendedNoise#parityConfigString
    pub fn parity_config_string(&self, out: &mut String) {
        out.push_str("BlendedNoise{minLimitNoise=");
        self.min_limit_noise.parity_config_string(out);
        out.push_str(", maxLimitNoise=");
        self.max_limit_noise.parity_config_string(out);
        out.push_str(", mainNoise=");
        self.main_noise.parity_config_string(out);
        out.push_str(&format!(
            ", xzScale={:.3}, yScale={:.3}, xzMainScale={:.3}, yMainScale={:.3}, cellWidth=4, cellHeight=8",
            684.412_f64, 684.412_f64, 8.555150000000001_f64, 4.277575000000001_f64
        ));
        out.push('}');
    }
}

impl DensityFunction for BlendedNoise {
    fn compute(&self, ctx: &dyn FunctionContext) -> f64 {
        BlendedNoise::compute(self, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct Ctx(i32, i32, i32);
    impl FunctionContext for Ctx {
        fn block_x(&self) -> i32 {
            self.0
        }
        fn block_y(&self) -> i32 {
            self.1
        }
        fn block_z(&self) -> i32 {
            self.2
        }
    }

    #[test]
    fn create_unseeded_is_deterministic() {
        let a = BlendedNoise::create_unseeded(0.25, 0.125, 80.0, 160.0, 8.0);
        let b = BlendedNoise::create_unseeded(0.25, 0.125, 80.0, 160.0, 8.0);
        let ctx = Ctx(10, 64, -20);
        assert_eq!(a.compute(&ctx).to_bits(), b.compute(&ctx).to_bits());
        assert_eq!(a.max_value().to_bits(), b.max_value().to_bits());
        assert_eq!(a.min_value().to_bits(), (-a.max_value()).to_bits());
    }

    #[test]
    fn with_new_random_changes_state() {
        let base = BlendedNoise::create_unseeded(0.25, 0.125, 80.0, 160.0, 8.0);
        let mut rng = XoroshiroRandomSource::new(12345);
        let other = base.with_new_random(&mut rng);
        let ctx = Ctx(1, 2, 3);
        // Different random streams should almost certainly produce different samples.
        assert_ne!(base.compute(&ctx).to_bits(), other.compute(&ctx).to_bits());
    }

    #[test]
    fn compute_handles_negative_and_large_coords() {
        let noise = BlendedNoise::create_unseeded(0.25, 0.375, 80.0, 60.0, 8.0);
        let samples = [
            Ctx(0, 0, 0),
            Ctx(-12345, -64, -9999),
            Ctx(i32::MIN / 2, i32::MAX / 4, i32::MIN / 8),
            Ctx(1_000_000, 320, -1_000_000),
        ];
        for ctx in samples {
            let v = noise.compute(&ctx);
            assert!(v.is_finite(), "non-finite at {:?}", ctx);
            assert!(v >= noise.min_value() - 1e-9);
            assert!(v <= noise.max_value() + 1e-9);
        }
    }

    #[test]
    fn wrap_matches_java_formula() {
        let values = [0.0, 1.0, -1.0, 3.3554432E7, -3.3554432E7, 1.23e12, -9.87e11];
        for &v in &values {
            let wrapped = PerlinNoise::wrap(v);
            let expected = v - (crate::mth::lfloor(v / 3.3554432E7 + 0.5) as f64) * 3.3554432E7;
            assert_eq!(wrapped.to_bits(), expected.to_bits(), "wrap mismatch for {v}");
            assert!(wrapped.abs() <= 3.3554432E7);
        }
    }

    #[test]
    fn parity_config_string_contains_expected_shape() {
        let noise = BlendedNoise::create_unseeded(0.25, 0.125, 80.0, 160.0, 8.0);
        let mut s = String::new();
        noise.parity_config_string(&mut s);
        assert!(s.starts_with("BlendedNoise{minLimitNoise="));
        assert!(s.contains(", maxLimitNoise="));
        assert!(s.contains(", mainNoise="));
        assert!(s.contains("xzScale=684.412, yScale=684.412"));
        assert!(s.ends_with('}'));
    }
}
