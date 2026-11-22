use std::collections::BTreeSet;

use crate::noise::SimplexNoise;
use crate::random::{LegacyRandomSource, RandomSource};

pub struct PerlinSimplexNoise {
    noise_levels: Vec<Option<SimplexNoise>>,
    highest_freq_value_factor: f64,
    highest_freq_input_factor: f64,
}

fn consume_count(rng: &mut dyn RandomSource, count: i32) {
    for _ in 0..count {
        rng.next_int();
    }
}

impl PerlinSimplexNoise {
    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinSimplexNoise#<init>
    pub fn new(rng: &mut dyn RandomSource, octaves: &[i32]) -> Self {
        let set: BTreeSet<i32> = octaves.iter().copied().collect();
        Self::new_from_set(rng, &set)
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinSimplexNoise#<init>
    fn new_from_set(rng: &mut dyn RandomSource, octaves: &BTreeSet<i32>) -> Self {
        if octaves.is_empty() {
            panic!("Need some octaves!");
        }

        let first = *octaves.first().unwrap();
        let last = *octaves.last().unwrap();
        let neg_first = -first;
        let count = neg_first + last + 1;
        if count < 1 {
            panic!("Total number of octaves needs to be >= 1");
        }

        let count_usize = count as usize;
        let base_noise = SimplexNoise::new(rng);
        let pivot = last;
        let mut noise_levels: Vec<Option<SimplexNoise>> = Vec::with_capacity(count_usize);
        for _ in 0..count_usize {
            noise_levels.push(None);
        }

        if pivot >= 0 && (pivot as usize) < count_usize && octaves.contains(&0) {
            noise_levels[pivot as usize] = Some(base_noise.clone());
        }

        let mut i = pivot + 1;
        while i < count {
            if i >= 0 && octaves.contains(&(pivot - i)) {
                noise_levels[i as usize] = Some(SimplexNoise::new(rng));
            } else {
                consume_count(rng, 262);
            }
            i += 1;
        }

        if last > 0 {
            let seed_f64 = base_noise.get_value_3d(base_noise.xo, base_noise.yo, base_noise.zo)
                * (9.223372E18_f32 as f64);
            let seed = seed_f64 as i64;
            let mut seed_rng = LegacyRandomSource::new(seed);
            // JAVA: new WorldgenRandom(new LegacyRandomSource(seed)) seeds from nextLong()
            let worldgen_seed = seed_rng.next_long();
            let mut worldgen_rng = LegacyRandomSource::new(worldgen_seed);

            let mut j = pivot - 1;
            while j >= 0 {
                if (j as usize) < count_usize && octaves.contains(&(pivot - j)) {
                    noise_levels[j as usize] = Some(SimplexNoise::new(&mut worldgen_rng));
                } else {
                    consume_count(&mut worldgen_rng, 262);
                }
                j -= 1;
            }
        }

        let highest_freq_input_factor = 2.0_f64.powi(last);
        let highest_freq_value_factor = 1.0 / (2.0_f64.powi(count) - 1.0);

        Self {
            noise_levels,
            highest_freq_value_factor,
            highest_freq_input_factor,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.synth.PerlinSimplexNoise#getValue
    pub fn get_value(&self, x: f64, y: f64, use_origin: bool) -> f64 {
        let mut value = 0.0_f64;
        let mut input_factor = self.highest_freq_input_factor;
        let mut value_factor = self.highest_freq_value_factor;

        for noise in &self.noise_levels {
            if let Some(noise) = noise {
                let ox = if use_origin { noise.xo } else { 0.0 };
                let oy = if use_origin { noise.yo } else { 0.0 };
                value += noise.get_value_2d(x * input_factor + ox, y * input_factor + oy)
                    * value_factor;
            }
            input_factor /= 2.0;
            value_factor *= 2.0;
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_octaves_panics() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut rng = LegacyRandomSource::new(0);
            let _ = PerlinSimplexNoise::new(&mut rng, &[]);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn deterministic_for_same_seed_and_input() {
        let mut rng1 = LegacyRandomSource::new(12345);
        let mut rng2 = LegacyRandomSource::new(12345);
        let n1 = PerlinSimplexNoise::new(&mut rng1, &[-3, -2, -1, 0, 1]);
        let n2 = PerlinSimplexNoise::new(&mut rng2, &[-3, -2, -1, 0, 1]);

        let v1 = n1.get_value(10.25, -7.5, false);
        let v2 = n2.get_value(10.25, -7.5, false);
        assert_eq!(v1.to_bits(), v2.to_bits());
    }

    #[test]
    fn use_origin_changes_value() {
        let mut rng = LegacyRandomSource::new(42);
        let noise = PerlinSimplexNoise::new(&mut rng, &[0, 1]);
        let a = noise.get_value(1.5, 2.5, false);
        let b = noise.get_value(1.5, 2.5, true);
        // For typical seeds/origins these differ; at minimum both should be finite.
        assert!(a.is_finite());
        assert!(b.is_finite());
        assert_ne!(a.to_bits(), b.to_bits());
    }

    #[test]
    fn negative_octaves_work() {
        let mut rng = LegacyRandomSource::new(7);
        let noise = PerlinSimplexNoise::new(&mut rng, &[-4, -3, -2, -1]);
        let v = noise.get_value(0.0, 0.0, false);
        assert!(v.is_finite());
    }

    #[test]
    fn consumes_rng_for_missing_octaves() {
        let mut rng_a = LegacyRandomSource::new(99);
        let mut rng_b = LegacyRandomSource::new(99);
        let _ = PerlinSimplexNoise::new(&mut rng_a, &[0, 2]); // gap at octave 1
        let _ = PerlinSimplexNoise::new(&mut rng_b, &[0, 2]);
        // Both RNGs should be in the same state after identical construction.
        assert_eq!(rng_a.next_long(), rng_b.next_long());
    }
}
