use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    fn ensure_word(&mut self, word_idx: usize) {
        if word_idx >= self.words.len() {
            self.words.resize(word_idx + 1, 0u64);
        }
    }

    fn set_range(&mut self, from: usize, to: usize) {
        if from >= to {
            return;
        }
        let start_word = from / 64;
        let end_word = (to - 1) / 64;
        self.ensure_word(end_word);

        let start_mask = !0u64 << (from % 64);
        let end_bit = (to - 1) % 64;
        let end_mask = if end_bit == 63 {
            !0u64
        } else {
            (1u64 << (end_bit + 1)) - 1
        };

        if start_word == end_word {
            self.words[start_word] |= start_mask & end_mask;
        } else {
            self.words[start_word] |= start_mask;
            for w in (start_word + 1)..end_word {
                self.words[w] = !0u64;
            }
            self.words[end_word] |= end_mask;
        }
    }

    fn clear_range(&mut self, from: usize, to: usize) {
        if from >= to || self.words.is_empty() {
            return;
        }
        let max_bit = self.words.len() * 64;
        if from >= max_bit {
            return;
        }
        let to_clamped = to.min(max_bit);
        if from >= to_clamped {
            return;
        }

        let start_word = from / 64;
        let end_word = (to_clamped - 1) / 64;

        let start_mask = !0u64 << (from % 64);
        let end_bit = (to_clamped - 1) % 64;
        let end_mask = if end_bit == 63 {
            !0u64
        } else {
            (1u64 << (end_bit + 1)) - 1
        };

        if start_word == end_word {
            self.words[start_word] &= !(start_mask & end_mask);
        } else {
            self.words[start_word] &= !start_mask;
            for w in (start_word + 1)..end_word {
                self.words[w] = 0u64;
            }
            self.words[end_word] &= !end_mask;
        }
    }

    fn next_set_bit(&self, from: usize) -> i32 {
        let mut word_idx = from / 64;
        if word_idx >= self.words.len() {
            return -1;
        }
        let bit_idx = from % 64;
        let mut word = self.words[word_idx] & (!0u64 << bit_idx);
        loop {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let idx = word_idx * 64 + bit;
                return i32::try_from(idx).unwrap_or(-1);
            }
            word_idx += 1;
            if word_idx >= self.words.len() {
                return -1;
            }
            word = self.words[word_idx];
        }
    }

    fn next_clear_bit(&self, from: usize) -> i32 {
        let mut word_idx = from / 64;
        let bit_idx = from % 64;
        if word_idx >= self.words.len() {
            return i32::try_from(from).unwrap_or(i32::MAX);
        }
        let mut word = (!self.words[word_idx]) & (!0u64 << bit_idx);
        loop {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let idx = word_idx * 64 + bit;
                return i32::try_from(idx).unwrap_or(i32::MAX);
            }
            word_idx += 1;
            let idx_base = word_idx * 64;
            if word_idx >= self.words.len() {
                return i32::try_from(idx_base).unwrap_or(i32::MAX);
            }
            word = !self.words[word_idx];
        }
    }

    fn set_range_i32(&mut self, from: i32, to: i32) {
        if from < 0 || to < 0 || from > to {
            panic!("IndexOutOfBoundsException: from={from}, to={to}");
        }
        self.set_range(from as usize, to as usize);
    }

    fn clear_range_i32(&mut self, from: i32, to: i32) {
        if from < 0 || to < 0 || from > to {
            panic!("IndexOutOfBoundsException: from={from}, to={to}");
        }
        self.clear_range(from as usize, to as usize);
    }

    fn next_set_bit_i32(&self, from: i32) -> i32 {
        if from < 0 {
            panic!("IndexOutOfBoundsException: from={from}");
        }
        self.next_set_bit(from as usize)
    }

    fn next_clear_bit_i32(&self, from: i32) -> i32 {
        if from < 0 {
            panic!("IndexOutOfBoundsException: from={from}");
        }
        self.next_clear_bit(from as usize)
    }

    fn used_bits(&self) -> BTreeSet<i32> {
        let mut result = BTreeSet::new();
        for (word_idx, &word) in self.words.iter().enumerate() {
            let mut w = word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                let idx = word_idx * 64 + bit;
                if let Ok(v) = i32::try_from(idx) {
                    result.insert(v);
                }
                w &= w - 1;
            }
        }
        result
    }
}

/// Port of `net.minecraft.world.level.chunk.storage.RegionBitmap`.
#[derive(Debug, Clone, Default)]
pub struct RegionBitmap {
    used: BitSet,
}

impl RegionBitmap {
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.world.level.chunk.storage.RegionBitmap#force
    pub fn force(&mut self, from: i32, length: i32) {
        let to = from.wrapping_add(length);
        self.used.set_range_i32(from, to);
    }

    // JAVA: net.minecraft.world.level.chunk.storage.RegionBitmap#free
    pub fn free(&mut self, from: i32, length: i32) {
        let to = from.wrapping_add(length);
        self.used.clear_range_i32(from, to);
    }

    // JAVA: net.minecraft.world.level.chunk.storage.RegionBitmap#allocate
    pub fn allocate(&mut self, size: i32) -> i32 {
        let mut cursor: i32 = 0;
        loop {
            let start = self.used.next_clear_bit_i32(cursor);
            let end = self.used.next_set_bit_i32(start);
            if end == -1 || end.wrapping_sub(start) >= size {
                self.force(start, size);
                return start;
            }
            cursor = end;
        }
    }

    // JAVA: net.minecraft.world.level.chunk.storage.RegionBitmap#getUsed
    pub fn get_used(&self) -> BTreeSet<i32> {
        self.used.used_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_and_get_used_track_bits() {
        let mut bm = RegionBitmap::new();
        bm.force(0, 2);
        bm.force(4, 1);
        let used = bm.get_used();
        assert_eq!(used, BTreeSet::from([0, 1, 4]));
    }

    #[test]
    fn free_clears_range() {
        let mut bm = RegionBitmap::new();
        bm.force(0, 5);
        bm.free(1, 3); // clears 1,2,3
        assert_eq!(bm.get_used(), BTreeSet::from([0, 4]));
    }

    #[test]
    fn allocate_finds_first_fit_gap() {
        let mut bm = RegionBitmap::new();
        assert_eq!(bm.allocate(2), 0);
        assert_eq!(bm.allocate(3), 2);
        bm.free(1, 2); // free bits 1,2
        assert_eq!(bm.allocate(2), 1);
        assert_eq!(bm.get_used(), BTreeSet::from([0, 1, 2, 3, 4]));
    }

    #[test]
    fn allocate_uses_gap_after_used_region() {
        let mut bm = RegionBitmap::new();
        bm.force(0, 4);
        bm.force(6, 2);
        // Gap at 4..6 has length 2
        assert_eq!(bm.allocate(2), 4);
        assert_eq!(bm.allocate(1), 8);
    }

    #[test]
    fn allocate_with_zero_size_does_not_change_bits() {
        let mut bm = RegionBitmap::new();
        bm.force(0, 3);
        let before = bm.get_used();
        let pos = bm.allocate(0);
        assert_eq!(pos, 3); // first clear bit; gap to end is "infinite",  -1 branch
        assert_eq!(bm.get_used(), before);
    }

    #[test]
    fn negative_from_panics_like_java_index_out_of_bounds() {
        let result = std::panic::catch_unwind(|| {
            let mut bm = RegionBitmap::new();
            bm.force(-1, 1);
        });
        assert!(result.is_err());
    }
}
