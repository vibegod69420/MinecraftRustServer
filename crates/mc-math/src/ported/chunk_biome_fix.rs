#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkBiomeFix;

impl ChunkBiomeFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkBiomeFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkBiomeFix#makeRule
    pub fn fix_biomes(&self, biomes: Option<&[i32]>) -> Option<Vec<i32>> {
        let biomes = biomes?;
        if biomes.len() != 256 {
            return None;
        }

        let mut expanded = vec![0i32; 1024];
        for z in 0..4i32 {
            for x in 0..4i32 {
                let sample_x = (x << 2).wrapping_add(2);
                let sample_z = (z << 2).wrapping_add(2);
                let src_idx = ((sample_z << 4) | sample_x) as usize;
                let dst_idx = ((z << 2) | x) as usize;
                expanded[dst_idx] = biomes[src_idx];
            }
        }

        for section in 1..64usize {
            let start = section.wrapping_mul(16);
            expanded.copy_within(0..16, start);
        }

        Some(expanded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_256_biomes_to_1024() {
        let fix = ChunkBiomeFix::new();
        let biomes: Vec<i32> = (0..256).collect();
        let result = fix.fix_biomes(Some(&biomes)).expect("should expand");
        assert_eq!(result.len(), 1024);

        for z in 0..4usize {
            for x in 0..4usize {
                let sample_x = x * 4 + 2;
                let sample_z = z * 4 + 2;
                let src_idx = sample_z * 16 + sample_x;
                let dst_idx = z * 4 + x;
                assert_eq!(result[dst_idx], biomes[src_idx]);
            }
        }

        for section in 1..64usize {
            let start = section * 16;
            assert_eq!(&result[start..start + 16], &result[0..16]);
        }
    }

    #[test]
    fn wrong_length_returns_none() {
        let fix = ChunkBiomeFix::new();
        assert!(fix.fix_biomes(Some(&vec![0i32; 255])).is_none());
        assert!(fix.fix_biomes(Some(&vec![0i32; 257])).is_none());
        assert!(fix.fix_biomes(Some(&[])).is_none());
    }

    #[test]
    fn missing_biomes_returns_none() {
        let fix = ChunkBiomeFix::new();
        assert!(fix.fix_biomes(None).is_none());
    }

    #[test]
    fn negative_biome_ids_preserved() {
        let fix = ChunkBiomeFix::new();
        let biomes = vec![-7i32; 256];
        let result = fix.fix_biomes(Some(&biomes)).unwrap();
        assert!(result.iter().all(|&v| v == -7));
    }

    #[test]
    fn sampling_indices_match_java() {
        let fix = ChunkBiomeFix::new();
        let mut biomes = vec![0i32; 256];
        // (z=0,x=0) -> sample_x=2, sample_z=2, src = 2*16+2 = 34
        biomes[34] = 11;
        // (z=1,x=3) -> sample_x=14, sample_z=6, src = 6*16+14 = 110
        biomes[110] = 22;
        let result = fix.fix_biomes(Some(&biomes)).unwrap();
        assert_eq!(result[0], 11); // dst = z*4+x = 0
        assert_eq!(result[1 * 4 + 3], 22);
    }
}
