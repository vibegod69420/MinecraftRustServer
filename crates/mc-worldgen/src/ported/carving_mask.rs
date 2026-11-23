#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#getBlockAt
    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos {
            x: self.x.wrapping_shl(4).wrapping_add(x),
            y,
            z: self.z.wrapping_shl(4).wrapping_add(z),
        }
    }
}

pub trait Mask {
    // JAVA: net.minecraft.world.level.chunk.CarvingMask$Mask#test
    fn test(&self, x: i32, y: i32, z: i32) -> bool;
}

impl<F: Fn(i32, i32, i32) -> bool> Mask for F {
    // JAVA: net.minecraft.world.level.chunk.CarvingMask$Mask#test
    fn test(&self, x: i32, y: i32, z: i32) -> bool {
        self(x, y, z)
    }
}

#[derive(Clone, Debug, Default)]
struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    // JAVA: java.util.BitSet#<init>
    fn new(nbits: usize) -> Self {
        let word_count = (nbits.wrapping_add(63)) / 64;
        Self {
            words: vec![0u64; word_count],
        }
    }

    // JAVA: java.util.BitSet#valueOf
    fn value_of(longs: &[i64]) -> Self {
        Self {
            words: longs.iter().map(|&v| v as u64).collect(),
        }
    }

    // JAVA: java.util.BitSet#set
    fn set(&mut self, bit_index: usize) {
        let word_index = bit_index / 64;
        if word_index >= self.words.len() {
            self.words.resize(word_index + 1, 0u64);
        }
        let bit = bit_index % 64;
        self.words[word_index] |= 1u64 << bit;
    }

    // JAVA: java.util.BitSet#get
    fn get(&self, bit_index: usize) -> bool {
        let word_index = bit_index / 64;
        if word_index >= self.words.len() {
            return false;
        }
        let bit = bit_index % 64;
        (self.words[word_index] & (1u64 << bit)) != 0
    }

    // JAVA: java.util.BitSet#stream
    fn stream_set_bits(&self) -> Vec<i32> {
        let mut out = Vec::new();
        for (wi, &word) in self.words.iter().enumerate() {
            let mut w = word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                let idx = wi.wrapping_mul(64).wrapping_add(bit);
                if idx <= i32::MAX as usize {
                    out.push(idx as i32);
                }
                w &= w.wrapping_sub(1);
            }
        }
        out
    }

    // JAVA: java.util.BitSet#toLongArray
    fn to_long_array(&self) -> Vec<i64> {
        let mut len = self.words.len();
        while len > 0 && self.words[len - 1] == 0 {
            len -= 1;
        }
        self.words[..len].iter().map(|&w| w as i64).collect()
    }
}

struct FalseMask;

impl Mask for FalseMask {
    // JAVA: net.minecraft.world.level.chunk.CarvingMask$Mask#test
    fn test(&self, _x: i32, _y: i32, _z: i32) -> bool {
        false
    }
}

pub struct CarvingMask {
    min_y: i32,
    mask: BitSet,
    additional_mask: Box<dyn Mask>,
}

impl CarvingMask {
    // JAVA: net.minecraft.world.level.chunk.CarvingMask#<init>
    pub fn new(height: i32, min_y: i32) -> Self {
        let nbits = 256i32.wrapping_mul(height);
        let nbits_usize = if nbits < 0 { 0usize } else { nbits as usize };
        Self {
            min_y,
            mask: BitSet::new(nbits_usize),
            additional_mask: Box::new(FalseMask),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.CarvingMask#<init>
    pub fn from_long_array(longs: &[i64], min_y: i32) -> Self {
        Self {
            min_y,
            mask: BitSet::value_of(longs),
            additional_mask: Box::new(FalseMask),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.CarvingMask#setAdditionalMask
    pub fn set_additional_mask(&mut self, mask: Box<dyn Mask>) {
        self.additional_mask = mask;
    }

    // JAVA: net.minecraft.world.level.chunk.CarvingMask#getIndex
    fn get_index(&self, x: i32, y: i32, z: i32) -> i32 {
        (x & 15) | ((z & 15) << 4) | (y.wrapping_sub(self.min_y) << 8)
    }

    // JAVA: net.minecraft.world.level.chunk.CarvingMask#set
    pub fn set(&mut self, x: i32, y: i32, z: i32) {
        let idx = self.get_index(x, y, z);
        if idx >= 0 {
            self.mask.set(idx as usize);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.CarvingMask#get
    pub fn get(&self, x: i32, y: i32, z: i32) -> bool {
        if self.additional_mask.test(x, y, z) {
            return true;
        }
        let idx = self.get_index(x, y, z);
        idx >= 0 && self.mask.get(idx as usize)
    }

    // JAVA: net.minecraft.world.level.chunk.CarvingMask#stream
    pub fn stream(&self, chunk_pos: ChunkPos) -> Vec<BlockPos> {
        self.mask
            .stream_set_bits()
            .into_iter()
            .map(|i| {
                let x = i & 15;
                let z = (i >> 4) & 15;
                let y = i >> 8;
                chunk_pos.get_block_at(x, y.wrapping_add(self.min_y), z)
            })
            .collect()
    }

    // JAVA: net.minecraft.world.level.chunk.CarvingMask#toArray
    pub fn to_array(&self) -> Vec<i64> {
        self.mask.to_long_array()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_and_index_masking() {
        let mut mask = CarvingMask::new(16, 0);
        assert!(!mask.get(1, 2, 3));
        mask.set(1, 2, 3);
        assert!(mask.get(1, 2, 3));
        // coordinates are masked by &15 in index calculation
        assert!(mask.get(17, 2, 19));
        assert!(!mask.get(2, 2, 3));
    }

    #[test]
    fn additional_mask_overrides() {
        let mut mask = CarvingMask::new(16, -64);
        mask.set_additional_mask(Box::new(|x: i32, y: i32, z: i32| x == 5 && y == -60 && z == 7));
        assert!(mask.get(5, -60, 7));
        assert!(!mask.get(5, -59, 7));
        mask.set(5, -59, 7);
        assert!(mask.get(5, -59, 7));
    }

    #[test]
    fn stream_produces_chunk_block_positions() {
        let mut mask = CarvingMask::new(16, 0);
        mask.set(0, 1, 0);
        mask.set(15, 2, 15);
        let chunk = ChunkPos { x: 3, z: -2 };
        let mut positions = mask.stream(chunk);
        positions.sort_by_key(|p| (p.y, p.z, p.x));
        assert_eq!(
            positions,
            vec![
                BlockPos { x: 48, y: 1, z: -32 },
                BlockPos { x: 63, y: 2, z: -17 },
            ]
        );
    }

    #[test]
    fn to_array_and_from_long_array_roundtrip() {
        let mut mask = CarvingMask::new(16, 0);
        mask.set(1, 0, 1);
        mask.set(2, 5, 3);
        let arr = mask.to_array();
        let restored = CarvingMask::from_long_array(&arr, 0);
        assert!(restored.get(1, 0, 1));
        assert!(restored.get(2, 5, 3));
        assert!(!restored.get(2, 5, 4));
        assert_eq!(restored.to_array(), arr);
    }

    #[test]
    fn negative_min_y_and_edge_coordinates() {
        let mut mask = CarvingMask::new(384, -64);
        mask.set(-1, -64, -16); // masked to (15, -64, 0)
        assert!(mask.get(15, -64, 0));
        assert!(mask.get(-1, -64, 16));

        // y below min_y gives negative index; should not panic and should be false unless additional mask
        assert!(!mask.get(0, -65, 0));

        let chunk = ChunkPos { x: -1, z: 0 };
        let pos = chunk.get_block_at(15, -64, 0);
        assert_eq!(pos, BlockPos { x: -1, y: -64, z: 0 });
    }

    #[test]
    fn to_long_array_trims_trailing_zeros() {
        let mask = CarvingMask::new(16, 0);
        assert!(mask.to_array().is_empty());
        let mut mask2 = CarvingMask::new(16, 0);
        mask2.set(0, 0, 0);
        let arr = mask2.to_array();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0], 1i64);
    }
}
