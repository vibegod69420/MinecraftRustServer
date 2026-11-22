const MODULUS_MASK: i64 = 281_474_976_710_655i64; // (1L << 48) - 1
const MULTIPLIER: i64 = 25_214_903_917i64;
const INCREMENT: i64 = 11i64;
const DOUBLE_MULTIPLIER: f64 = 1.1102230246251565E-16f64; // 1.0 / (1L << 53)

fn lcg_next(seed: &mut i64, bits: i32) -> i32 {
    let next = seed
        .wrapping_mul(MULTIPLIER)
        .wrapping_add(INCREMENT)
        & MODULUS_MASK;
    *seed = next;
    let shift = 48i32.wrapping_sub(bits) & 63;
    ((next as u64) >> (shift as u32)) as i32
}

fn next_double_from_seed(seed: &mut i64) -> f64 {
    let i = lcg_next(seed, 26) as i64;
    let j = lcg_next(seed, 27) as i64;
    let l = (i << 27).wrapping_add(j);
    (l as f64) * DOUBLE_MULTIPLIER
}

/// Minimal local port of `net.minecraft.util.MarsagliaPolarGaussian`.
#[derive(Debug, Clone)]
pub struct MarsagliaPolarGaussian {
    next_next_gaussian: f64,
    have_next_next_gaussian: bool,
}

impl MarsagliaPolarGaussian {
    // JAVA: net.minecraft.util.MarsagliaPolarGaussian#<init>
    pub fn new() -> Self {
        Self {
            next_next_gaussian: 0.0,
            have_next_next_gaussian: false,
        }
    }

    // JAVA: net.minecraft.util.MarsagliaPolarGaussian#reset
    pub fn reset(&mut self) {
        self.have_next_next_gaussian = false;
    }

    // JAVA: net.minecraft.util.MarsagliaPolarGaussian#nextGaussian
    pub fn next_gaussian(&mut self, next_double: &mut impl FnMut() -> f64) -> f64 {
        if self.have_next_next_gaussian {
            self.have_next_next_gaussian = false;
            return self.next_next_gaussian;
        }
        loop {
            let d0 = 2.0 * next_double() - 1.0;
            let d1 = 2.0 * next_double() - 1.0;
            let d2 = d0 * d0 + d1 * d1;
            if d2 < 1.0 && d2 != 0.0 {
                let d3 = (-2.0 * d2.ln() / d2).sqrt();
                self.next_next_gaussian = d1 * d3;
                self.have_next_next_gaussian = true;
                return d0 * d3;
            }
        }
    }
}

impl Default for MarsagliaPolarGaussian {
    fn default() -> Self {
        Self::new()
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

/// Port of `net.minecraft.world.level.levelgen.SingleThreadedRandomSource`.
#[derive(Debug, Clone)]
pub struct SingleThreadedRandomSource {
    seed: i64,
    gaussian: MarsagliaPolarGaussian,
}

impl SingleThreadedRandomSource {
    // JAVA: net.minecraft.world.level.levelgen.SingleThreadedRandomSource#<init>
    pub fn new(seed: i64) -> Self {
        let mut s = Self {
            seed: 0,
            gaussian: MarsagliaPolarGaussian::new(),
        };
        s.set_seed(seed);
        s
    }

    // JAVA: net.minecraft.world.level.levelgen.SingleThreadedRandomSource#setSeed
    pub fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ MULTIPLIER) & MODULUS_MASK;
        self.gaussian.reset();
    }

    // JAVA: net.minecraft.world.level.levelgen.SingleThreadedRandomSource#next
    pub fn next(&mut self, bits: i32) -> i32 {
        lcg_next(&mut self.seed, bits)
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextInt
    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextLong
    pub fn next_long(&mut self) -> i64 {
        let i = self.next(32) as i64;
        let j = self.next(32) as i64;
        (i << 32).wrapping_add(j)
    }

    // JAVA: net.minecraft.world.level.levelgen.BitRandomSource#nextDouble
    pub fn next_double(&mut self) -> f64 {
        next_double_from_seed(&mut self.seed)
    }

    // JAVA: net.minecraft.world.level.levelgen.SingleThreadedRandomSource#fork
    pub fn fork(&mut self) -> SingleThreadedRandomSource {
        SingleThreadedRandomSource::new(self.next_long())
    }

    // JAVA: net.minecraft.world.level.levelgen.SingleThreadedRandomSource#forkPositional
    pub fn fork_positional(&mut self) -> LegacyPositionalRandomFactory {
        LegacyPositionalRandomFactory::new(self.next_long())
    }

    // JAVA: net.minecraft.world.level.levelgen.SingleThreadedRandomSource#nextGaussian
    pub fn next_gaussian(&mut self) -> f64 {
        let seed = &mut self.seed;
        let gaussian = &mut self.gaussian;
        gaussian.next_gaussian(&mut || next_double_from_seed(seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_sequence_seed_zero() {
        let mut rng = SingleThreadedRandomSource::new(0);
        assert_eq!(rng.next_int(), -1_155_484_576);
        let mut rng2 = SingleThreadedRandomSource::new(0);
        assert_eq!(rng2.next_long(), -4_962_768_465_676_381_896i64);
    }

    #[test]
    fn set_seed_masks_and_resets_gaussian() {
        let mut rng = SingleThreadedRandomSource::new(123);
        let g1 = rng.next_gaussian();
        rng.set_seed(123);
        let g2 = rng.next_gaussian();
        assert_eq!(g1.to_bits(), g2.to_bits());
    }

    #[test]
    fn negative_seed_is_deterministic() {
        let mut a = SingleThreadedRandomSource::new(-1);
        let mut b = SingleThreadedRandomSource::new(-1);
        for _ in 0..8 {
            assert_eq!(a.next_int(), b.next_int());
        }
    }

    #[test]
    fn next_double_range_and_fork() {
        let mut rng = SingleThreadedRandomSource::new(42);
        for _ in 0..64 {
            let d = rng.next_double();
            assert!(d.is_finite());
            assert!(d >= 0.0 && d < 1.0);
        }
        let mut forked = rng.fork();
        let v1 = forked.next_int();
        let mut forked2 = SingleThreadedRandomSource::new(rng.next_long());
        let _ = v1;
        let _ = forked2.next_int();
    }

    #[test]
    fn next_gaussian_finite_deterministic() {
        let mut a = SingleThreadedRandomSource::new(987654321);
        let mut b = SingleThreadedRandomSource::new(987654321);
        for _ in 0..16 {
            let ga = a.next_gaussian();
            let gb = b.next_gaussian();
            assert!(ga.is_finite());
            assert_eq!(ga.to_bits(), gb.to_bits());
        }
    }

    #[test]
    fn fork_positional_uses_next_long_seed() {
        let mut rng = SingleThreadedRandomSource::new(7);
        let mut expected_rng = SingleThreadedRandomSource::new(7);
        let expected_seed = expected_rng.next_long();
        let factory = rng.fork_positional();
        assert_eq!(factory.seed(), expected_seed);
    }
}
