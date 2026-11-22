//! Phase A faithful port of `RandomSupport`, `RandomSource`, `LegacyRandomSource`,
//! `Xoroshiro128PlusPlus`, `XoroshiroRandomSource`, `MarsagliaPolarGaussian`, and `WorldgenRandom`.

use std::sync::Arc;

use crate::mth::{get_seed, java_string_hash_code, square_f64};
use crate::trace::{RngTraceEvent, RngTracer};

pub const GOLDEN_RATIO_64: i64 = -7046029254386353131;
pub const SILVER_RATIO_64: i64 = 7640891576956012809;

const FLOAT_UNIT: f32 = 5.9604645e-8_f32;
const DOUBLE_UNIT: f64 = 1.110223e-16_f32 as f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed128Bit {
    pub seed_lo: i64,
    pub seed_hi: i64,
}

impl Seed128Bit {
    // JAVA: net.minecraft.world.level.levelgen.RandomSupport$Seed128bit#xor
    #[inline]
    pub const fn xor_parts(self, lo: i64, hi: i64) -> Self {
        Self {
            seed_lo: self.seed_lo ^ lo,
            seed_hi: self.seed_hi ^ hi,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomSupport$Seed128bit#xor
    #[inline]
    pub const fn xor(self, other: Self) -> Self {
        self.xor_parts(other.seed_lo, other.seed_hi)
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomSupport$Seed128bit#mixed
    #[inline]
    pub fn mixed(self) -> Self {
        Self {
            seed_lo: mix_stafford13(self.seed_lo),
            seed_hi: mix_stafford13(self.seed_hi),
        }
    }
}

// JAVA: net.minecraft.world.level.levelgen.RandomSupport#mixStafford13
#[inline]
pub fn mix_stafford13(mut z: i64) -> i64 {
    z = (z ^ (((z as u64) >> 30) as i64)).wrapping_mul(-4658895280553007687);
    z = (z ^ (((z as u64) >> 27) as i64)).wrapping_mul(-7723592293110705685);
    z ^ (((z as u64) >> 31) as i64)
}

// JAVA: net.minecraft.world.level.levelgen.RandomSupport#upgradeSeedTo128bitUnmixed
#[inline]
pub fn upgrade_seed_to_128bit_unmixed(seed: i64) -> Seed128Bit {
    let lo = seed ^ SILVER_RATIO_64;
    let hi = lo.wrapping_add(GOLDEN_RATIO_64);
    Seed128Bit {
        seed_lo: lo,
        seed_hi: hi,
    }
}

// JAVA: net.minecraft.world.level.levelgen.RandomSupport#upgradeSeedTo128bit
#[inline]
pub fn upgrade_seed_to_128bit(seed: i64) -> Seed128Bit {
    upgrade_seed_to_128bit_unmixed(seed).mixed()
}

/// RFC 1321 MD5 implementation (zero external dependency, `#![forbid(unsafe_code)]`).
fn md5(input: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    let mut msg = input.to_vec();
    let bit_len = (input.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    for chunk in msg.chunks_exact(64) {
        let mut m = [0_u32; 16];
        for (i, word) in m.iter_mut().enumerate() {
            *word = u32::from_le_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | (!d)), (7 * i) % 16),
            };
            let sum = a.wrapping_add(f).wrapping_add(K[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(sum.rotate_left(S[i]));
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0_u8; 16];
    out[0..4].copy_from_slice(&a0.to_le_bytes());
    out[4..8].copy_from_slice(&b0.to_le_bytes());
    out[8..12].copy_from_slice(&c0.to_le_bytes());
    out[12..16].copy_from_slice(&d0.to_le_bytes());
    out
}

// JAVA: net.minecraft.world.level.levelgen.RandomSupport#seedFromHashOf
pub fn seed_from_hash_of(s: &str) -> Seed128Bit {
    let bytes = md5(s.as_bytes());
    let lo = i64::from_be_bytes(bytes[0..8].try_into().unwrap());
    let hi = i64::from_be_bytes(bytes[8..16].try_into().unwrap());
    Seed128Bit {
        seed_lo: lo,
        seed_hi: hi,
    }
}

pub trait RandomSource {
    fn set_seed(&mut self, seed: i64);
    fn next_int(&mut self) -> i32;
    fn next_int_bound(&mut self, bound: i32) -> i32;
    fn next_long(&mut self) -> i64;
    fn next_boolean(&mut self) -> bool;
    fn next_float(&mut self) -> f32;
    fn next_double(&mut self) -> f64;
    fn next_gaussian(&mut self) -> f64;
    fn fork(&mut self) -> Box<dyn RandomSource>;
    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory>;

    // JAVA: net.minecraft.util.RandomSource#nextIntBetweenInclusive
    fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32 {
        self.next_int_bound(max - min + 1) + min
    }

    // JAVA: net.minecraft.util.RandomSource#consumeCount
    fn consume_count(&mut self, count: i32) {
        for _ in 0..count {
            let _ = self.next_int();
        }
    }

    // JAVA: net.minecraft.util.RandomSource#triangle(double,double)
    fn triangle_f64(&mut self, origin: f64, spread: f64) -> f64 {
        origin + spread * (self.next_double() - self.next_double())
    }

    // JAVA: net.minecraft.util.RandomSource#triangle(float,float)
    fn triangle_f32(&mut self, origin: f32, spread: f32) -> f32 {
        origin + spread * (self.next_float() - self.next_float())
    }
}

pub trait PositionalRandomFactory {
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource>;
    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource>;
    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource>;
}

#[derive(Debug, Clone, Default)]
struct MarsagliaPolarGaussian {
    next_next_gaussian: f64,
    have_next_next_gaussian: bool,
}

impl MarsagliaPolarGaussian {
    // JAVA: net.minecraft.world.level.levelgen.MarsagliaPolarGaussian#reset
    fn reset(&mut self) {
        self.have_next_next_gaussian = false;
    }

    // JAVA: net.minecraft.world.level.levelgen.MarsagliaPolarGaussian#nextGaussian
    fn next_gaussian(&mut self, mut next_double: impl FnMut() -> f64) -> f64 {
        if self.have_next_next_gaussian {
            self.have_next_next_gaussian = false;
            return self.next_next_gaussian;
        }
        loop {
            let v1 = 2.0 * next_double() - 1.0;
            let v2 = 2.0 * next_double() - 1.0;
            let s = square_f64(v1) + square_f64(v2);
            if s < 1.0 && s != 0.0 {
                let mult = (-2.0 * crate::fdlibm::log(s) / s).sqrt();
                self.next_next_gaussian = v2 * mult;
                self.have_next_next_gaussian = true;
                return v1 * mult;
            }
        }
    }
}

#[derive(Clone)]
pub struct LegacyRandomSource {
    seed: i64,
    gaussian: MarsagliaPolarGaussian,
    stream_id: u64,
    tracer: Option<Arc<dyn RngTracer>>,
}

impl LegacyRandomSource {
    const MODULUS_MASK: i64 = 281474976710655;
    const MULTIPLIER: i64 = 25214903917;
    const INCREMENT: i64 = 11;

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#<init>
    pub fn new(seed: i64) -> Self {
        let mut r = Self {
            seed: 0,
            gaussian: MarsagliaPolarGaussian::default(),
            stream_id: 0,
            tracer: None,
        };
        r.set_seed(seed);
        r
    }

    pub fn with_tracer(seed: i64, stream_id: u64, tracer: Arc<dyn RngTracer>) -> Self {
        let mut r = Self {
            seed: 0,
            gaussian: MarsagliaPolarGaussian::default(),
            stream_id,
            tracer: Some(tracer),
        };
        r.set_seed(seed);
        r
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#next
    #[inline]
    pub fn next_bits_i32(&mut self, bits: i32) -> i32 {
        let cur = self.seed;
        let next = cur
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT)
            & Self::MODULUS_MASK;
        self.seed = next;
        let out = (next >> ((48 - bits) & 63)) as i32;
        if let Some(tr) = &self.tracer {
            tr.record(RngTraceEvent {
                seq: 0,
                stream_id: self.stream_id,
                impl_name: "LegacyRandomSource",
                method: "next",
                arg: bits as i64,
                result: out as i64,
            });
        }
        out
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextDouble
    fn bit_next_double(&mut self) -> f64 {
        let hi = self.next_bits_i32(26) as i64;
        let lo = self.next_bits_i32(27) as i64;
        let combined = (hi << 27).wrapping_add(lo);
        (combined as f64) * DOUBLE_UNIT
    }
}

impl RandomSource for LegacyRandomSource {
    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#setSeed
    fn set_seed(&mut self, seed: i64) {
        if let Some(tr) = &self.tracer {
            tr.record(RngTraceEvent {
                seq: 0,
                stream_id: self.stream_id,
                impl_name: "LegacyRandomSource",
                method: "setSeed",
                arg: seed,
                result: 0,
            });
        }
        self.seed = (seed ^ Self::MULTIPLIER) & Self::MODULUS_MASK;
        self.gaussian.reset();
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextInt()
    fn next_int(&mut self) -> i32 {
        self.next_bits_i32(32)
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextInt(int)
    fn next_int_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "Bound must be positive");
        if (bound & bound.wrapping_sub(1)) == 0 {
            return (((bound as i64).wrapping_mul(self.next_bits_i32(31) as i64)) >> 31) as i32;
        }
        loop {
            let bits = self.next_bits_i32(31);
            let val = bits % bound;
            if bits.wrapping_sub(val).wrapping_add(bound - 1) >= 0 {
                return val;
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextLong
    fn next_long(&mut self) -> i64 {
        let hi = self.next_bits_i32(32) as i64;
        let lo = self.next_bits_i32(32) as i64;
        (hi << 32).wrapping_add(lo)
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextBoolean
    fn next_boolean(&mut self) -> bool {
        self.next_bits_i32(1) != 0
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextFloat
    fn next_float(&mut self) -> f32 {
        (self.next_bits_i32(24) as f32) * FLOAT_UNIT
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextDouble
    fn next_double(&mut self) -> f64 {
        self.bit_next_double()
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#nextGaussian
    fn next_gaussian(&mut self) -> f64 {
        let mut g = std::mem::take(&mut self.gaussian);
        let res = g.next_gaussian(|| self.bit_next_double());
        self.gaussian = g;
        res
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#fork
    fn fork(&mut self) -> Box<dyn RandomSource> {
        Box::new(LegacyRandomSource::new(self.next_long()))
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource#forkPositional
    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        Box::new(LegacyPositionalRandomFactory {
            seed: self.next_long(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LegacyPositionalRandomFactory {
    pub seed: i64,
}

impl PositionalRandomFactory for LegacyPositionalRandomFactory {
    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource$LegacyPositionalRandomFactory#at
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource> {
        Box::new(LegacyRandomSource::new(get_seed(x, y, z) ^ self.seed))
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource$LegacyPositionalRandomFactory#fromHashOf
    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource> {
        let h = java_string_hash_code(name) as i64;
        Box::new(LegacyRandomSource::new(h ^ self.seed))
    }

    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource$LegacyPositionalRandomFactory#fromSeed
    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource> {
        Box::new(LegacyRandomSource::new(seed))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Xoroshiro128PlusPlus {
    pub seed_lo: i64,
    pub seed_hi: i64,
}

impl Xoroshiro128PlusPlus {
    // JAVA: net.minecraft.world.level.levelgen.Xoroshiro128PlusPlus#<init>(long,long)
    pub fn new(mut seed_lo: i64, mut seed_hi: i64) -> Self {
        if (seed_lo | seed_hi) == 0 {
            seed_lo = GOLDEN_RATIO_64;
            seed_hi = SILVER_RATIO_64;
        }
        Self { seed_lo, seed_hi }
    }

    // JAVA: net.minecraft.world.level.levelgen.Xoroshiro128PlusPlus#<init>(RandomSupport$Seed128bit)
    pub fn from_seed128(seed: Seed128Bit) -> Self {
        Self::new(seed.seed_lo, seed.seed_hi)
    }

    // JAVA: net.minecraft.world.level.levelgen.Xoroshiro128PlusPlus#nextLong
    #[inline]
    pub fn next_long(&mut self) -> i64 {
        let s0 = self.seed_lo;
        let mut s1 = self.seed_hi;
        let res = (s0.wrapping_add(s1) as u64)
            .rotate_left(17)
            .wrapping_add(s0 as u64) as i64;
        s1 ^= s0;
        self.seed_lo = ((s0 as u64).rotate_left(49) as i64) ^ s1 ^ (s1 << 21);
        self.seed_hi = (s1 as u64).rotate_left(28) as i64;
        res
    }
}

#[derive(Clone)]
pub struct XoroshiroRandomSource {
    rng: Xoroshiro128PlusPlus,
    gaussian: MarsagliaPolarGaussian,
    stream_id: u64,
    tracer: Option<Arc<dyn RngTracer>>,
}

impl XoroshiroRandomSource {
    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#<init>(long)
    pub fn new(seed: i64) -> Self {
        Self {
            rng: Xoroshiro128PlusPlus::from_seed128(upgrade_seed_to_128bit(seed)),
            gaussian: MarsagliaPolarGaussian::default(),
            stream_id: 0,
            tracer: None,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#<init>(long,long)
    pub fn from_pair(seed_lo: i64, seed_hi: i64) -> Self {
        Self {
            rng: Xoroshiro128PlusPlus::new(seed_lo, seed_hi),
            gaussian: MarsagliaPolarGaussian::default(),
            stream_id: 0,
            tracer: None,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#<init>(RandomSupport$Seed128bit)
    pub fn from_seed128(seed: Seed128Bit) -> Self {
        Self::from_pair(seed.seed_lo, seed.seed_hi)
    }

    pub fn with_tracer(seed: i64, stream_id: u64, tracer: Arc<dyn RngTracer>) -> Self {
        Self {
            rng: Xoroshiro128PlusPlus::from_seed128(upgrade_seed_to_128bit(seed)),
            gaussian: MarsagliaPolarGaussian::default(),
            stream_id,
            tracer: Some(tracer),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextBits
    #[inline]
    fn next_bits(&mut self, bits: i32) -> i64 {
        let raw = self.rng.next_long();
        let out = ((raw as u64) >> ((64 - bits) & 63)) as i64;
        if let Some(tr) = &self.tracer {
            tr.record(RngTraceEvent {
                seq: 0,
                stream_id: self.stream_id,
                impl_name: "XoroshiroRandomSource",
                method: "nextBits",
                arg: bits as i64,
                result: out,
            });
        }
        out
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextDouble
    #[inline]
    fn xoro_next_double(&mut self) -> f64 {
        (self.next_bits(53) as f64) * DOUBLE_UNIT
    }
}

impl RandomSource for XoroshiroRandomSource {
    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#setSeed
    fn set_seed(&mut self, seed: i64) {
        if let Some(tr) = &self.tracer {
            tr.record(RngTraceEvent {
                seq: 0,
                stream_id: self.stream_id,
                impl_name: "XoroshiroRandomSource",
                method: "setSeed",
                arg: seed,
                result: 0,
            });
        }
        self.rng = Xoroshiro128PlusPlus::from_seed128(upgrade_seed_to_128bit(seed));
        self.gaussian.reset();
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextInt()
    fn next_int(&mut self) -> i32 {
        let out = self.rng.next_long() as i32;
        if let Some(tr) = &self.tracer {
            tr.record(RngTraceEvent {
                seq: 0,
                stream_id: self.stream_id,
                impl_name: "XoroshiroRandomSource",
                method: "nextInt",
                arg: 0,
                result: out as i64,
            });
        }
        out
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextInt(int)
    fn next_int_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "Bound must be positive");
        let bound_u = bound as u32;
        let bound_l = bound as i64;
        let mut r = (self.next_int() as u32) as i64;
        let mut prod = r.wrapping_mul(bound_l);
        let mut low = prod & 0xFFFF_FFFF;
        if low < bound_l {
            let thresh = ((!bound_u).wrapping_add(1) % bound_u) as i64;
            while low < thresh {
                r = (self.next_int() as u32) as i64;
                prod = r.wrapping_mul(bound_l);
                low = prod & 0xFFFF_FFFF;
            }
        }
        (prod >> 32) as i32
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextLong
    fn next_long(&mut self) -> i64 {
        let out = self.rng.next_long();
        if let Some(tr) = &self.tracer {
            tr.record(RngTraceEvent {
                seq: 0,
                stream_id: self.stream_id,
                impl_name: "XoroshiroRandomSource",
                method: "nextLong",
                arg: 0,
                result: out,
            });
        }
        out
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextBoolean
    fn next_boolean(&mut self) -> bool {
        (self.rng.next_long() & 1) != 0
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextFloat
    fn next_float(&mut self) -> f32 {
        (self.next_bits(24) as f32) * FLOAT_UNIT
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextDouble
    fn next_double(&mut self) -> f64 {
        self.xoro_next_double()
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#nextGaussian
    fn next_gaussian(&mut self) -> f64 {
        let mut g = std::mem::take(&mut self.gaussian);
        let res = g.next_gaussian(|| self.xoro_next_double());
        self.gaussian = g;
        res
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#consumeCount
    fn consume_count(&mut self, count: i32) {
        for _ in 0..count {
            let _ = self.rng.next_long();
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#fork
    fn fork(&mut self) -> Box<dyn RandomSource> {
        let lo = self.rng.next_long();
        let hi = self.rng.next_long();
        Box::new(XoroshiroRandomSource::from_pair(lo, hi))
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource#forkPositional
    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        let lo = self.rng.next_long();
        let hi = self.rng.next_long();
        Box::new(XoroshiroPositionalRandomFactory {
            seed_lo: lo,
            seed_hi: hi,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct XoroshiroPositionalRandomFactory {
    pub seed_lo: i64,
    pub seed_hi: i64,
}

impl PositionalRandomFactory for XoroshiroPositionalRandomFactory {
    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource$XoroshiroPositionalRandomFactory#at
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource> {
        let pos_seed = get_seed(x, y, z);
        Box::new(XoroshiroRandomSource::from_pair(
            pos_seed ^ self.seed_lo,
            self.seed_hi,
        ))
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource$XoroshiroPositionalRandomFactory#fromHashOf
    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource> {
        let h = seed_from_hash_of(name);
        Box::new(XoroshiroRandomSource::from_seed128(
            h.xor_parts(self.seed_lo, self.seed_hi),
        ))
    }

    // JAVA: net.minecraft.world.level.levelgen.XoroshiroRandomSource$XoroshiroPositionalRandomFactory#fromSeed
    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource> {
        Box::new(XoroshiroRandomSource::from_pair(
            seed ^ self.seed_lo,
            seed ^ self.seed_hi,
        ))
    }
}

// JAVA: net.minecraft.world.level.levelgen.WorldgenRandom#seedSlimeChunk
pub fn seed_slime_chunk(cx: i32, cz: i32, world_seed: i64, salt: i64) -> LegacyRandomSource {
    let x1 = cx.wrapping_mul(cx).wrapping_mul(4987142) as i64;
    let x2 = cx.wrapping_mul(5947611) as i64;
    let z1 = (cz as i64).wrapping_mul(cz as i64).wrapping_mul(4392871_i64);
    let z2 = cz.wrapping_mul(389711) as i64;
    let s = world_seed
        .wrapping_add(x1)
        .wrapping_add(x2)
        .wrapping_add(z1)
        .wrapping_add(z2)
        ^ salt;
    LegacyRandomSource::new(s)
}
