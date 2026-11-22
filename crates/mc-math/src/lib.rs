#![forbid(unsafe_code)]
//! Phase A faithful port of Minecraft 1.21.11 math, RNG, noise, and bit-packing primitives.

pub mod fdlibm;
pub mod mth;
pub mod noise;
pub mod packing;
pub mod random;
pub mod trace;

pub use mth::*;
pub use noise::{ImprovedNoise, NormalNoise, PerlinNoise, SimplexNoise};
pub use packing::{BlockPos, SectionPos, SimpleBitStorage};
pub use random::{
    LegacyPositionalRandomFactory, LegacyRandomSource, PositionalRandomFactory, RandomSource,
    Seed128Bit, Xoroshiro128PlusPlus, XoroshiroPositionalRandomFactory, XoroshiroRandomSource,
    mix_stafford13, seed_from_hash_of, seed_slime_chunk, upgrade_seed_to_128bit,
};
pub use trace::{RngTraceEvent, RngTracer, VecRngTracer};
