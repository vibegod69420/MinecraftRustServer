//! Phase A faithful port of `net.minecraft.util.Mth`.

use std::sync::OnceLock;

pub const PI: f32 = std::f64::consts::PI as f32;
pub const HALF_PI: f32 = (std::f64::consts::PI / 2.0) as f32;
pub const TWO_PI: f32 = (std::f64::consts::PI * 2.0) as f32;
pub const DEG_TO_RAD: f32 = (std::f64::consts::PI / 180.0) as f32;
pub const RAD_TO_DEG: f32 = 180.0_f32 / (std::f64::consts::PI as f32);
pub const EPSILON: f32 = 1.0e-5_f32;

const SIN_SCALE: f64 = 10430.378350470453;
const ONE_SIXTH: f64 = 0.16666666666666666;
const FRAC_BIAS: f64 = f64::from_bits(4805340802404319232);

const MULTIPLY_DE_BRUIJN_BIT_POSITION: [i32; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26, 12,
    18, 6, 11, 5, 10, 9,
];

struct AtanTables {
    asin: [f64; 257],
    cos: [f64; 257],
}

fn sin_table() -> &'static [f32; 65536] {
    static SIN: OnceLock<[f32; 65536]> = OnceLock::new();
    SIN.get_or_init(|| {
        let mut arr = [0.0_f32; 65536];
        for (i, slot) in arr.iter_mut().enumerate() {
            *slot = crate::fdlibm::sin_small((i as f64) / SIN_SCALE) as f32;
        }
        arr
    })
}

fn atan_tables() -> &'static AtanTables {
    static TABS: OnceLock<AtanTables> = OnceLock::new();
    TABS.get_or_init(|| {
        let mut asin = [0.0_f64; 257];
        let mut cos = [0.0_f64; 257];
        for i in 0..257 {
            let v = (i as f64) / 256.0;
            let a = crate::fdlibm::asin(v);
            cos[i] = crate::fdlibm::cos_small(a);
            asin[i] = a;
        }
        // Pinned aarch64 OpenJDK 21.0.13 Math.cos(Math.asin(205/256.0)) 1-ULP override
        cos[205] = f64::from_bits(4603570143850921360);
        AtanTables { asin, cos }
    })
}

/// Saturating float-to-int cast matching JVM `f2i`/`d2i` bytecode instructions.
#[inline]
fn java_d2i(v: f64) -> i32 {
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

/// Saturating double-to-long cast matching JVM `d2l` bytecode instruction.
#[inline]
fn java_d2l(v: f64) -> i64 {
    if v.is_nan() {
        0
    } else if v >= i64::MAX as f64 {
        i64::MAX
    } else if v <= i64::MIN as f64 {
        i64::MIN
    } else {
        v as i64
    }
}

/// Saturating float-to-int cast matching JVM `f2i` bytecode instruction.
#[inline]
fn java_f2i(v: f32) -> i32 {
    if v.is_nan() {
        0
    } else if v >= i32::MAX as f32 {
        i32::MAX
    } else if v <= i32::MIN as f32 {
        i32::MIN
    } else {
        v as i32
    }
}

// JAVA: net.minecraft.util.Mth#sin
#[inline]
pub fn sin(x: f64) -> f32 {
    let idx = (java_d2l(x * SIN_SCALE) & 65535) as usize;
    sin_table()[idx]
}

// JAVA: net.minecraft.util.Mth#cos
#[inline]
pub fn cos(x: f64) -> f32 {
    let idx = (java_d2l(x * SIN_SCALE + 16384.0) & 65535) as usize;
    sin_table()[idx]
}

// JAVA: net.minecraft.util.Mth#sqrt
#[inline]
pub fn sqrt_f32(x: f32) -> f32 {
    (x as f64).sqrt() as f32
}

// JAVA: net.minecraft.util.Mth#floor(float)
#[inline]
pub fn floor_f32(x: f32) -> i32 {
    let i = java_f2i(x);
    if x < (i as f32) {
        i.wrapping_sub(1)
    } else {
        i
    }
}

// JAVA: net.minecraft.util.Mth#floor(double)
#[inline]
pub fn floor_f64(x: f64) -> i32 {
    let i = java_d2i(x);
    if x < (i as f64) {
        i.wrapping_sub(1)
    } else {
        i
    }
}

// JAVA: net.minecraft.util.Mth#lfloor
#[inline]
pub fn lfloor(x: f64) -> i64 {
    let l = java_d2l(x);
    if x < (l as f64) {
        l.wrapping_sub(1)
    } else {
        l
    }
}

// JAVA: net.minecraft.util.Mth#ceil(float)
#[inline]
pub fn ceil_f32(x: f32) -> i32 {
    let i = java_f2i(x);
    if x > (i as f32) {
        i.wrapping_add(1)
    } else {
        i
    }
}

// JAVA: net.minecraft.util.Mth#ceil(double)
#[inline]
pub fn ceil_f64(x: f64) -> i32 {
    let i = java_d2i(x);
    if x > (i as f64) {
        i.wrapping_add(1)
    } else {
        i
    }
}

// JAVA: net.minecraft.util.Mth#ceilLong
#[inline]
pub fn ceil_long(x: f64) -> i64 {
    let l = java_d2l(x);
    if x > (l as f64) {
        l.wrapping_add(1)
    } else {
        l
    }
}

// JAVA: net.minecraft.util.Mth#clamp(int,int,int)
#[inline]
pub fn clamp_i32(v: i32, min: i32, max: i32) -> i32 {
    v.max(min).min(max)
}

// JAVA: net.minecraft.util.Mth#clamp(long,long,long)
#[inline]
pub fn clamp_i64(v: i64, min: i64, max: i64) -> i64 {
    v.max(min).min(max)
}

// JAVA: net.minecraft.util.Mth#clamp(float,float,float)
#[inline]
pub fn clamp_f32(v: f32, min: f32, max: f32) -> f32 {
    if v < min { min } else { v.min(max) }
}

// JAVA: net.minecraft.util.Mth#clamp(double,double,double)
#[inline]
pub fn clamp_f64(v: f64, min: f64, max: f64) -> f64 {
    if v < min { min } else { v.min(max) }
}

// JAVA: net.minecraft.util.Mth#lerp(double,double,double)
#[inline]
pub fn lerp_f64(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

// JAVA: net.minecraft.util.Mth#lerp(float,float,float)
#[inline]
pub fn lerp_f32(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

// JAVA: net.minecraft.util.Mth#clampedLerp(double,double,double)
#[inline]
pub fn clamped_lerp_f64(delta: f64, start: f64, end: f64) -> f64 {
    if delta < 0.0 {
        start
    } else if delta > 1.0 {
        end
    } else {
        lerp_f64(delta, start, end)
    }
}

// JAVA: net.minecraft.util.Mth#clampedLerp(float,float,float)
#[inline]
pub fn clamped_lerp_f32(delta: f32, start: f32, end: f32) -> f32 {
    if delta < 0.0_f32 {
        start
    } else if delta > 1.0_f32 {
        end
    } else {
        lerp_f32(delta, start, end)
    }
}

// JAVA: net.minecraft.util.Mth#lerp2
#[inline]
pub fn lerp2(dx: f64, dy: f64, v00: f64, v10: f64, v01: f64, v11: f64) -> f64 {
    lerp_f64(dy, lerp_f64(dx, v00, v10), lerp_f64(dx, v01, v11))
}

// JAVA: net.minecraft.util.Mth#lerp3
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn lerp3(
    dx: f64,
    dy: f64,
    dz: f64,
    v000: f64,
    v100: f64,
    v010: f64,
    v110: f64,
    v001: f64,
    v101: f64,
    v011: f64,
    v111: f64,
) -> f64 {
    lerp_f64(
        dz,
        lerp2(dx, dy, v000, v100, v010, v110),
        lerp2(dx, dy, v001, v101, v011, v111),
    )
}

// JAVA: net.minecraft.util.Mth#smoothstep
#[inline]
pub fn smoothstep(x: f64) -> f64 {
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

// JAVA: net.minecraft.util.Mth#smoothstepDerivative
#[inline]
pub fn smoothstep_derivative(x: f64) -> f64 {
    30.0 * x * x * (x - 1.0) * (x - 1.0)
}

// JAVA: net.minecraft.util.Mth#square(double)
#[inline]
pub fn square_f64(x: f64) -> f64 {
    x * x
}

// JAVA: net.minecraft.util.Mth#inverseLerp(double,double,double)
#[inline]
pub fn inverse_lerp_f64(val: f64, start: f64, end: f64) -> f64 {
    (val - start) / (end - start)
}

// JAVA: net.minecraft.util.Mth#smallestEncompassingPowerOfTwo
#[inline]
pub fn smallest_encompassing_power_of_two(value: i32) -> i32 {
    let mut v = value.wrapping_sub(1);
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v.wrapping_add(1)
}

// JAVA: net.minecraft.util.Mth#isPowerOfTwo
#[inline]
pub fn is_power_of_two(value: i32) -> bool {
    value != 0 && (value & value.wrapping_sub(1)) == 0
}

// JAVA: net.minecraft.util.Mth#ceillog2
#[inline]
pub fn ceillog2(value: i32) -> i32 {
    let v = if is_power_of_two(value) {
        value
    } else {
        smallest_encompassing_power_of_two(value)
    };
    let idx = (((v as i64).wrapping_mul(125613361_i64) >> 27) & 31) as usize;
    MULTIPLY_DE_BRUIJN_BIT_POSITION[idx]
}

// JAVA: net.minecraft.util.Mth#log2
#[inline]
pub fn log2(value: i32) -> i32 {
    ceillog2(value).wrapping_sub(if is_power_of_two(value) { 0 } else { 1 })
}

// JAVA: net.minecraft.util.Mth#murmurHash3Mixer
#[inline]
pub fn murmur_hash3_mixer(mut h: i32) -> i32 {
    h ^= ((h as u32) >> 16) as i32;
    h = h.wrapping_mul(-2048144789);
    h ^= ((h as u32) >> 13) as i32;
    h = h.wrapping_mul(-1028477387);
    h ^ (((h as u32) >> 16) as i32)
}

// JAVA: net.minecraft.util.Mth#getSeed(int,int,int)
#[inline]
pub fn get_seed(x: i32, y: i32, z: i32) -> i64 {
    let x_part = (x.wrapping_mul(3129871)) as i64;
    let z_part = (z as i64).wrapping_mul(116129781_i64);
    let mut s = x_part ^ z_part ^ (y as i64);
    s = s
        .wrapping_mul(s)
        .wrapping_mul(42317861_i64)
        .wrapping_add(s.wrapping_mul(11_i64));
    s >> 16
}

// JAVA: net.minecraft.util.Mth#fastInvSqrt
#[inline]
pub fn fast_inv_sqrt(mut x: f64) -> f64 {
    let half = 0.5 * x;
    let mut bits = x.to_bits() as i64;
    bits = 6910469410427058090_i64.wrapping_sub(bits >> 1);
    x = f64::from_bits(bits as u64);
    x * (1.5 - half * x * x)
}

// JAVA: net.minecraft.util.Mth#atan2
pub fn atan2(mut y: f64, mut x: f64) -> f64 {
    let d2 = x * x + y * y;
    if d2.is_nan() {
        return f64::NAN;
    }

    let neg_y = y < 0.0;
    if neg_y {
        y = -y;
    }

    let neg_x = x < 0.0;
    if neg_x {
        x = -x;
    }

    let swapped = y > x;
    if swapped {
        std::mem::swap(&mut x, &mut y);
    }

    let inv = fast_inv_sqrt(d2);
    x *= inv;
    y *= inv;
    let biased = FRAC_BIAS + y;
    let idx = (biased.to_bits() as i32) as usize;
    let tabs = atan_tables();
    let asin_v = tabs.asin[idx];
    let cos_v = tabs.cos[idx];
    let dy = biased - FRAC_BIAS;
    let d_sin = y * cos_v - x * dy;
    let d_theta = (6.0 + d_sin * d_sin) * d_sin * ONE_SIXTH;
    let mut res = asin_v + d_theta;

    if swapped {
        res = (std::f64::consts::PI / 2.0) - res;
    }
    if neg_x {
        res = std::f64::consts::PI - res;
    }
    if neg_y {
        res = -res;
    }
    res
}

// JAVA: net.minecraft.util.Mth#positiveModulo(int,int)
#[inline]
pub fn positive_modulo_i32(x: i32, y: i32) -> i32 {
    x.rem_euclid(y)
}

// JAVA: net.minecraft.util.Mth#floorDiv
#[inline]
pub fn floor_div(x: i32, y: i32) -> i32 {
    x.div_euclid(y)
}

/// Java `String.hashCode()` matching UTF-16 code units.
pub fn java_string_hash_code(s: &str) -> i32 {
    let mut h: i32 = 0;
    for unit in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(unit as i32);
    }
    h
}
