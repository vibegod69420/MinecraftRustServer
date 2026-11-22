use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

const MODULUS_MASK: i64 = 281_474_976_710_655i64; // (1L << 48) - 1
const MULTIPLIER: i64 = 25_214_903_917i64;
const INCREMENT: i64 = 11i64;
const DOUBLE_MULTIPLIER: f64 = 1.1102230246251565E-16f64; // 1.0 / (1L << 53)

/// Minimal local port of `net.minecraft.util.MarsagliaPolarGaussian`.
#[derive(Debug, Clone)]
struct MarsagliaPolarGaussian {
    next_next_gaussian: f64,
    have_next_next_gaussian: bool,
}

impl MarsagliaPolarGaussian {
    // JAVA: net.minecraft.util.MarsagliaPolarGaussian#<init>
    fn new() -> Self {
        Self {
            next_next_gaussian: 0.0,
            have_next_next_gaussian: false,
        }
    }

    // JAVA: net.minecraft.util.MarsagliaPolarGaussian#reset
    fn reset(&mut self) {
        self.have_next_next_gaussian = false;
    }
}

/// Minimal local placeholder for `LegacyRandomSource.LegacyPositionalRandomFactory`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyPositionalRandomFactory {
    seed: i64,
}

impl LegacyPositionalRandomFactory {
    // JAVA: net.minecraft.world.level.levelgen.LegacyRandomSource$LegacyPositionalRandomFactory#<init>
    pub fn new(seed: i64) -> Self {
        Self { seed }
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }
}

/// Port of `net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource`.
#[derive(Debug)]
pub struct ThreadSafeLegacyRandomSource {
    seed: AtomicI64,
    gaussian: Mutex<MarsagliaPolarGaussian>,
}

impl ThreadSafeLegacyRandomSource {
    // JAVA: net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource#<init>
    pub fn new(seed: i64) -> Self {
        let s = Self {
            seed: AtomicI64::new(0),
            gaussian: Mutex::new(MarsagliaPolarGaussian::new()),
        };
        s.set_seed(seed);
        s
    }

    // JAVA: net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource#setSeed
    pub fn set_seed(&self, seed: i64) {
        self.seed
            .store((seed ^ MULTIPLIER) & MODULUS_MASK, Ordering::Relaxed);
    }

    // JAVA: net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource#next
    pub fn next(&self, bits: i32) -> i32 {
        loop {
            let current = self.seed.load(Ordering::Relaxed);
            let next = current
                .wrapping_mul(MULTIPLIER)
                .wrapping_add(INCREMENT)
                & MODULUS_MASK;
            if self
                .seed
                .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                let shift = 48i32.wrapping_sub(bits) & 63;
                return ((next as u64) >> (shift as u32)) as i32;
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextInt
    pub fn next_int(&self) -> i32 {
        self.next(32)
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextLong
    pub fn next_long(&self) -> i64 {
        let i = self.next(32) as i64;
        let j = self.next(32) as i64;
        (i << 32).wrapping_add(j)
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextDouble
    pub fn next_double(&self) -> f64 {
        let i = self.next(26) as i64;
        let j = self.next(27) as i64;
        let l = (i << 27).wrapping_add(j);
        (l as f64) * DOUBLE_MULTIPLIER
    }

    // JAVA: net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource#fork
    pub fn fork(&self) -> ThreadSafeLegacyRandomSource {
        ThreadSafeLegacyRandomSource::new(self.next_long())
    }

    // JAVA: net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource#forkPositional
    pub fn fork_positional(&self) -> LegacyPositionalRandomFactory {
        LegacyPositionalRandomFactory::new(self.next_long())
    }

    // JAVA: net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource#nextGaussian
    pub fn next_gaussian(&self) -> f64 {
        let mut gaussian = self.gaussian.lock().expect("gaussian lock poisoned");
        if gaussian.have_next_next_gaussian {
            gaussian.have_next_next_gaussian = false;
            return gaussian.next_next_gaussian;
        }
        loop {
            let d0 = 2.0 * self.next_double() - 1.0;
            let d1 = 2.0 * self.next_double() - 1.0;
            let d2 = d0 * d0 + d1 * d1;
            if d2 < 1.0 && d2 != 0.0 {
                let d3 = (-2.0 * d2.ln() / d2).sqrt();
                gaussian.next_next_gaussian = d1 * d3;
                gaussian.have_next_next_gaussian = true;
                return d0 * d3;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn known_sequence_seed_zero() {
        let rng = ThreadSafeLegacyRandomSource::new(0);
        assert_eq!(rng.next_int(), -1_155_484_576);
        let rng2 = ThreadSafeLegacyRandomSource::new(0);
        assert_eq!(rng2.next_long(), -4_962_768_465_676_381_896i64);
    }

    #[test]
    fn set_seed_does_not_reset_gaussian_state() {
        let rng = ThreadSafeLegacyRandomSource::new(123);
        let _ = rng.next_gaussian(); // may populate cached gaussian
        // set_seed in Java does not reset gaussian; ensure it still produces finite values
        rng.set_seed(123);
        let g = rng.next_gaussian();
        assert!(g.is_finite());
    }

    #[test]
    fn negative_seed_deterministic() {
        let a = ThreadSafeLegacyRandomSource::new(-1);
        let b = ThreadSafeLegacyRandomSource::new(-1);
        for _ in 0..8 {
            assert_eq!(a.next_int(), b.next_int());
        }
    }

    #[test]
    fn next_double_range() {
        let rng = ThreadSafeLegacyRandomSource::new(42);
        for _ in 0..64 {
            let d = rng.next_double();
            assert!(d.is_finite());
            assert!(d >= 0.0 && d < 1.0);
        }
    }

    #[test]
    fn concurrent_next_no_panic() {
        let rng = Arc::new(ThreadSafeLegacyRandomSource::new(42));
        let mut handles = Vec::new();
        for _ in 0..4 {
            let r = Arc::clone(&rng);
            handles.push(std::thread::spawn(move || {
                for _ in 0..512 {
                    let _ = r.next(32);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn fork_positional_uses_next_long_seed() {
        let rng = ThreadSafeLegacyRandomSource::new(7);
        let expected = ThreadSafeLegacyRandomSource::new(7);
        let expected_seed = expected.next_long();
        let factory = rng.fork_positional();
        assert_eq!(factory.seed(), expected_seed);
    }
}
