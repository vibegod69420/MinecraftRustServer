//! Phase A faithful port of `SimplexNoise`, `ImprovedNoise`, `PerlinNoise`, and `NormalNoise`.

use crate::mth::{floor_f64, lerp2, lerp3, lfloor, smoothstep, smoothstep_derivative};
use crate::random::RandomSource;

const GRADIENT: [[i32; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

#[inline]
fn dot(g: [i32; 3], x: f64, y: f64, z: f64) -> f64 {
    (g[0] as f64) * x + (g[1] as f64) * y + (g[2] as f64) * z
}

#[derive(Debug, Clone)]
pub struct SimplexNoise {
    p: [i32; 512],
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
}

impl SimplexNoise {
    // JAVA: net.minecraft.world.level.levelgen.synth.SimplexNoise#<init>
    pub fn new(rng: &mut dyn RandomSource) -> Self {
        let xo = rng.next_double() * 256.0;
        let yo = rng.next_double() * 256.0;
        let zo = rng.next_double() * 256.0;
        let mut p = [0_i32; 512];
        for (i, slot) in p.iter_mut().take(256).enumerate() {
            *slot = i as i32;
        }
        for i in 0..256 {
            let j = rng.next_int_bound(256 - i as i32) as usize;
            p.swap(i, i + j);
        }
        Self { p, xo, yo, zo }
    }

    #[inline]
    fn p_idx(&self, idx: i32) -> i32 {
        self.p[(idx as usize) & 0xFF]
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.SimplexNoise#getCornerNoise3D
    fn get_corner_noise_3d(&self, grad_idx: usize, x: f64, y: f64, z: f64, r2: f64) -> f64 {
        let t = r2 - x * x - y * y - z * z;
        if t < 0.0 {
            0.0
        } else {
            let t2 = t * t;
            t2 * t2 * dot(GRADIENT[grad_idx], x, y, z)
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.SimplexNoise#getValue(double,double)
    pub fn get_value_2d(&self, x: f64, y: f64) -> f64 {
        let sqrt3 = 3.0_f64.sqrt();
        let f2 = 0.5 * (sqrt3 - 1.0);
        let g2 = (3.0 - sqrt3) / 6.0;

        let s = (x + y) * f2;
        let i = floor_f64(x + s);
        let j = floor_f64(y + s);
        let t = ((i + j) as f64) * g2;
        let x0 = x - ((i as f64) - t);
        let y0 = y - ((j as f64) - t);
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
        let x1 = x0 - (i1 as f64) + g2;
        let y1 = y0 - (j1 as f64) + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let y2 = y0 - 1.0 + 2.0 * g2;

        let ii = i & 0xFF;
        let jj = j & 0xFF;
        let gi0 = (self.p_idx(ii + self.p_idx(jj)) % 12) as usize;
        let gi1 = (self.p_idx(ii + i1 + self.p_idx(jj + j1)) % 12) as usize;
        let gi2 = (self.p_idx(ii + 1 + self.p_idx(jj + 1)) % 12) as usize;

        let n0 = self.get_corner_noise_3d(gi0, x0, y0, 0.0, 0.5);
        let n1 = self.get_corner_noise_3d(gi1, x1, y1, 0.0, 0.5);
        let n2 = self.get_corner_noise_3d(gi2, x2, y2, 0.0, 0.5);
        70.0 * (n0 + n1 + n2)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.SimplexNoise#getValue(double,double,double)
    pub fn get_value_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        const F3: f64 = 0.3333333333333333;
        const G3: f64 = 0.16666666666666666;
        let s = (x + y + z) * F3;
        let i = floor_f64(x + s);
        let j = floor_f64(y + s);
        let k = floor_f64(z + s);
        let t = ((i + j + k) as f64) * G3;
        let x0 = x - ((i as f64) - t);
        let y0 = y - ((j as f64) - t);
        let z0 = z - ((k as f64) - t);

        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 {
                (1, 0, 0, 1, 1, 0)
            } else if x0 >= z0 {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if y0 < z0 {
            (0, 0, 1, 0, 1, 1)
        } else if x0 < z0 {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };

        let x1 = x0 - (i1 as f64) + G3;
        let y1 = y0 - (j1 as f64) + G3;
        let z1 = z0 - (k1 as f64) + G3;
        let x2 = x0 - (i2 as f64) + F3;
        let y2 = y0 - (j2 as f64) + F3;
        let z2 = z0 - (k2 as f64) + F3;
        let x3 = x0 - 1.0 + 0.5;
        let y3 = y0 - 1.0 + 0.5;
        let z3 = z0 - 1.0 + 0.5;

        let ii = i & 0xFF;
        let jj = j & 0xFF;
        let kk = k & 0xFF;
        let gi0 = (self.p_idx(ii + self.p_idx(jj + self.p_idx(kk))) % 12) as usize;
        let gi1 = (self.p_idx(ii + i1 + self.p_idx(jj + j1 + self.p_idx(kk + k1))) % 12) as usize;
        let gi2 = (self.p_idx(ii + i2 + self.p_idx(jj + j2 + self.p_idx(kk + k2))) % 12) as usize;
        let gi3 = (self.p_idx(ii + 1 + self.p_idx(jj + 1 + self.p_idx(kk + 1))) % 12) as usize;

        let n0 = self.get_corner_noise_3d(gi0, x0, y0, z0, 0.6);
        let n1 = self.get_corner_noise_3d(gi1, x1, y1, z1, 0.6);
        let n2 = self.get_corner_noise_3d(gi2, x2, y2, z2, 0.6);
        let n3 = self.get_corner_noise_3d(gi3, x3, y3, z3, 0.6);
        32.0 * (n0 + n1 + n2 + n3)
    }
}

#[derive(Debug, Clone)]
pub struct ImprovedNoise {
    p: [i8; 256],
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
}

impl ImprovedNoise {
    const SHIFT_UP_EPSILON: f64 = 1.0e-7_f32 as f64;

    // JAVA: net.minecraft.world.level.levelgen.synth.ImprovedNoise#<init>
    pub fn new(rng: &mut dyn RandomSource) -> Self {
        let xo = rng.next_double() * 256.0;
        let yo = rng.next_double() * 256.0;
        let zo = rng.next_double() * 256.0;
        let mut p = [0_i8; 256];
        for (i, slot) in p.iter_mut().enumerate() {
            *slot = i as i8;
        }
        for i in 0..256 {
            let j = rng.next_int_bound(256 - i as i32) as usize;
            p.swap(i, i + j);
        }
        Self { p, xo, yo, zo }
    }

    #[inline]
    fn perm(&self, idx: i32) -> i32 {
        (self.p[(idx as usize) & 0xFF] as i32) & 0xFF
    }

    #[inline]
    fn grad_dot(hash: i32, x: f64, y: f64, z: f64) -> f64 {
        dot(GRADIENT[(hash & 15) as usize], x, y, z)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.ImprovedNoise#noise(double,double,double)
    pub fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        self.noise_with_y_scale(x, y, z, 0.0, 0.0)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.ImprovedNoise#noise(double,double,double,double,double)
    pub fn noise_with_y_scale(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let sx = x + self.xo;
        let sy = y + self.yo;
        let sz = z + self.zo;
        let gx = floor_f64(sx);
        let gy = floor_f64(sy);
        let gz = floor_f64(sz);
        let fx = sx - (gx as f64);
        let fy = sy - (gy as f64);
        let fz = sz - (gz as f64);
        let y_shift = if y_scale != 0.0 {
            let clamp_y = if y_max >= 0.0 && y_max < fy { y_max } else { fy };
            (floor_f64(clamp_y / y_scale + Self::SHIFT_UP_EPSILON) as f64) * y_scale
        } else {
            0.0
        };
        self.sample_and_lerp(gx, gy, gz, fx, fy - y_shift, fz, fy)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.ImprovedNoise#noiseWithDerivative
    pub fn noise_with_derivative(&self, x: f64, y: f64, z: f64, deriv: &mut [f64; 3]) -> f64 {
        let sx = x + self.xo;
        let sy = y + self.yo;
        let sz = z + self.zo;
        let gx = floor_f64(sx);
        let gy = floor_f64(sy);
        let gz = floor_f64(sz);
        let fx = sx - (gx as f64);
        let fy = sy - (gy as f64);
        let fz = sz - (gz as f64);
        self.sample_with_derivative(gx, gy, gz, fx, fy, fz, deriv)
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_and_lerp(&self, gx: i32, gy: i32, gz: i32, fx: f64, fy_adj: f64, fz: f64, fy: f64) -> f64 {
        let a = self.perm(gx);
        let b = self.perm(gx.wrapping_add(1));
        let aa = self.perm(a.wrapping_add(gy));
        let ab = self.perm(a.wrapping_add(gy).wrapping_add(1));
        let ba = self.perm(b.wrapping_add(gy));
        let bb = self.perm(b.wrapping_add(gy).wrapping_add(1));

        let g000 = Self::grad_dot(self.perm(aa.wrapping_add(gz)), fx, fy_adj, fz);
        let g100 = Self::grad_dot(self.perm(ba.wrapping_add(gz)), fx - 1.0, fy_adj, fz);
        let g010 = Self::grad_dot(self.perm(ab.wrapping_add(gz)), fx, fy_adj - 1.0, fz);
        let g110 = Self::grad_dot(self.perm(bb.wrapping_add(gz)), fx - 1.0, fy_adj - 1.0, fz);
        let g001 = Self::grad_dot(self.perm(aa.wrapping_add(gz).wrapping_add(1)), fx, fy_adj, fz - 1.0);
        let g101 = Self::grad_dot(self.perm(ba.wrapping_add(gz).wrapping_add(1)), fx - 1.0, fy_adj, fz - 1.0);
        let g011 = Self::grad_dot(self.perm(ab.wrapping_add(gz).wrapping_add(1)), fx, fy_adj - 1.0, fz - 1.0);
        let g111 = Self::grad_dot(self.perm(bb.wrapping_add(gz).wrapping_add(1)), fx - 1.0, fy_adj - 1.0, fz - 1.0);

        let u = smoothstep(fx);
        let v = smoothstep(fy);
        let w = smoothstep(fz);
        lerp3(u, v, w, g000, g100, g010, g110, g001, g101, g011, g111)
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_with_derivative(
        &self,
        gx: i32,
        gy: i32,
        gz: i32,
        fx: f64,
        fy: f64,
        fz: f64,
        deriv: &mut [f64; 3],
    ) -> f64 {
        let a = self.perm(gx);
        let b = self.perm(gx.wrapping_add(1));
        let aa = self.perm(a.wrapping_add(gy));
        let ab = self.perm(a.wrapping_add(gy).wrapping_add(1));
        let ba = self.perm(b.wrapping_add(gy));
        let bb = self.perm(b.wrapping_add(gy).wrapping_add(1));

        let i000 = (self.perm(aa.wrapping_add(gz)) & 15) as usize;
        let i100 = (self.perm(ba.wrapping_add(gz)) & 15) as usize;
        let i010 = (self.perm(ab.wrapping_add(gz)) & 15) as usize;
        let i110 = (self.perm(bb.wrapping_add(gz)) & 15) as usize;
        let i001 = (self.perm(aa.wrapping_add(gz).wrapping_add(1)) & 15) as usize;
        let i101 = (self.perm(ba.wrapping_add(gz).wrapping_add(1)) & 15) as usize;
        let i011 = (self.perm(ab.wrapping_add(gz).wrapping_add(1)) & 15) as usize;
        let i111 = (self.perm(bb.wrapping_add(gz).wrapping_add(1)) & 15) as usize;

        let g000 = GRADIENT[i000];
        let g100 = GRADIENT[i100];
        let g010 = GRADIENT[i010];
        let g110 = GRADIENT[i110];
        let g001 = GRADIENT[i001];
        let g101 = GRADIENT[i101];
        let g011 = GRADIENT[i011];
        let g111 = GRADIENT[i111];

        let v000 = dot(g000, fx, fy, fz);
        let v100 = dot(g100, fx - 1.0, fy, fz);
        let v010 = dot(g010, fx, fy - 1.0, fz);
        let v110 = dot(g110, fx - 1.0, fy - 1.0, fz);
        let v001 = dot(g001, fx, fy, fz - 1.0);
        let v101 = dot(g101, fx - 1.0, fy, fz - 1.0);
        let v011 = dot(g011, fx, fy - 1.0, fz - 1.0);
        let v111 = dot(g111, fx - 1.0, fy - 1.0, fz - 1.0);

        let u = smoothstep(fx);
        let v = smoothstep(fy);
        let w = smoothstep(fz);

        let gx_interp = lerp3(
            u, v, w,
            g000[0] as f64, g100[0] as f64, g010[0] as f64, g110[0] as f64,
            g001[0] as f64, g101[0] as f64, g011[0] as f64, g111[0] as f64,
        );
        let gy_interp = lerp3(
            u, v, w,
            g000[1] as f64, g100[1] as f64, g010[1] as f64, g110[1] as f64,
            g001[1] as f64, g101[1] as f64, g011[1] as f64, g111[1] as f64,
        );
        let gz_interp = lerp3(
            u, v, w,
            g000[2] as f64, g100[2] as f64, g010[2] as f64, g110[2] as f64,
            g001[2] as f64, g101[2] as f64, g011[2] as f64, g111[2] as f64,
        );

        let dx_diff = lerp2(v, w, v100 - v000, v110 - v010, v101 - v001, v111 - v011);
        let dy_diff = lerp2(w, u, v010 - v000, v011 - v001, v110 - v100, v111 - v101);
        let dz_diff = lerp2(u, v, v001 - v000, v101 - v100, v011 - v010, v111 - v110);

        let du = smoothstep_derivative(fx);
        let dv = smoothstep_derivative(fy);
        let dw = smoothstep_derivative(fz);

        deriv[0] += gx_interp + du * dx_diff;
        deriv[1] += gy_interp + dv * dy_diff;
        deriv[2] += gz_interp + dw * dz_diff;

        lerp3(u, v, w, v000, v100, v010, v110, v001, v101, v011, v111)
    }
}

#[derive(Debug, Clone)]
pub struct PerlinNoise {
    noise_levels: Vec<Option<ImprovedNoise>>,
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
    lowest_freq_value_factor: f64,
    lowest_freq_input_factor: f64,
    pub max_value: f64,
}

impl PerlinNoise {
    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#create(RandomSource,int,DoubleList)
    pub fn create(rng: &mut dyn RandomSource, first_octave: i32, amplitudes: Vec<f64>) -> Self {
        Self::new_internal(rng, first_octave, amplitudes, true)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#createLegacyForLegacyNetherBiome
    pub fn create_legacy(rng: &mut dyn RandomSource, first_octave: i32, amplitudes: Vec<f64>) -> Self {
        Self::new_internal(rng, first_octave, amplitudes, false)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#<init>
    fn new_internal(
        rng: &mut dyn RandomSource,
        first_octave: i32,
        amplitudes: Vec<f64>,
        use_positional: bool,
    ) -> Self {
        let count = amplitudes.len();
        let zero_idx = -first_octave;
        let mut noise_levels: Vec<Option<ImprovedNoise>> = vec![None; count];

        if use_positional {
            let pos = rng.fork_positional();
            for (i, &amp) in amplitudes.iter().enumerate() {
                if amp != 0.0 {
                    let octave = first_octave + (i as i32);
                    let mut oct_rng = pos.from_hash_of(&format!("octave_{octave}"));
                    noise_levels[i] = Some(ImprovedNoise::new(oct_rng.as_mut()));
                }
            }
        } else {
            let oct0 = ImprovedNoise::new(rng);
            if zero_idx >= 0 && (zero_idx as usize) < count && amplitudes[zero_idx as usize] != 0.0 {
                noise_levels[zero_idx as usize] = Some(oct0);
            }
            for i in (0..zero_idx).rev() {
                let idx = i as usize;
                if idx < count && amplitudes[idx] != 0.0 {
                    noise_levels[idx] = Some(ImprovedNoise::new(rng));
                } else {
                    rng.consume_count(262);
                }
            }
        }

        let lowest_freq_input_factor = 2.0_f64.powi(-zero_idx);
        let lowest_freq_value_factor =
            2.0_f64.powi((count as i32) - 1) / (2.0_f64.powi(count as i32) - 1.0);

        let mut p = Self {
            noise_levels,
            first_octave,
            amplitudes,
            lowest_freq_value_factor,
            lowest_freq_input_factor,
            max_value: 0.0,
        };
        p.max_value = p.edge_value(2.0);
        p
    }

    fn edge_value(&self, scale: f64) -> f64 {
        let mut acc = 0.0;
        let mut vf = self.lowest_freq_value_factor;
        for (i, lvl) in self.noise_levels.iter().enumerate() {
            if lvl.is_some() {
                acc += self.amplitudes[i] * scale * vf;
            }
            vf /= 2.0;
        }
        acc
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#wrap
    #[inline]
    pub fn wrap(v: f64) -> f64 {
        v - (lfloor(v / 3.3554432e7 + 0.5) as f64) * 3.3554432e7
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#getValue(double,double,double)
    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        self.get_value_full(x, y, z, 0.0, 0.0, false)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinNoise#getValue(double,double,double,double,double,boolean)
    pub fn get_value_full(
        &self,
        x: f64,
        y: f64,
        z: f64,
        y_scale: f64,
        y_max: f64,
        use_origin: bool,
    ) -> f64 {
        let mut sum = 0.0;
        let mut inf = self.lowest_freq_input_factor;
        let mut vf = self.lowest_freq_value_factor;
        for (i, lvl_opt) in self.noise_levels.iter().enumerate() {
            if let Some(lvl) = lvl_opt {
                let sample = lvl.noise_with_y_scale(
                    Self::wrap(x * inf),
                    if use_origin { -lvl.yo } else { Self::wrap(y * inf) },
                    Self::wrap(z * inf),
                    y_scale * inf,
                    y_max * inf,
                );
                sum += self.amplitudes[i] * sample * vf;
            }
            inf *= 2.0;
            vf /= 2.0;
        }
        sum
    }
}

#[derive(Debug, Clone)]
pub struct NormalNoise {
    value_factor: f64,
    first: PerlinNoise,
    second: PerlinNoise,
    pub max_value: f64,
}

impl NormalNoise {
    const INPUT_FACTOR: f64 = 1.0181268882175227;

    // JAVA: net.minecraft.world.level.levelgen.synth.NormalNoise#create(RandomSource,int,double...)
    pub fn create(rng: &mut dyn RandomSource, first_octave: i32, amplitudes: Vec<f64>) -> Self {
        let first = PerlinNoise::create(rng, first_octave, amplitudes.clone());
        let second = PerlinNoise::create(rng, first_octave, amplitudes.clone());
        let mut min_idx = i32::MAX;
        let mut max_idx = i32::MIN;
        for (i, &amp) in amplitudes.iter().enumerate() {
            if amp != 0.0 {
                min_idx = min_idx.min(i as i32);
                max_idx = max_idx.max(i as i32);
            }
        }
        let expected_dev = 0.1 * (1.0 + 1.0 / ((max_idx - min_idx + 1) as f64));
        let value_factor = 0.16666666666666666 / expected_dev;
        let max_value = (first.max_value + second.max_value) * value_factor;
        Self {
            value_factor,
            first,
            second,
            max_value,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.NormalNoise#getValue
    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        let sx = x * Self::INPUT_FACTOR;
        let sy = y * Self::INPUT_FACTOR;
        let sz = z * Self::INPUT_FACTOR;
        (self.first.get_value(x, y, z) + self.second.get_value(sx, sy, sz)) * self.value_factor
    }
}
