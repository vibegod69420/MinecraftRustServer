//! Phase A faithful port of `BlockPos`, `SectionPos`, `SimpleBitStorage`, and `ZeroBitStorage`.

use crate::mth::{floor_f64, log2, smallest_encompassing_power_of_two};

const PACKED_HORIZONTAL_LENGTH: i32 = 1 + log2_const_30000000();
const PACKED_Y_LENGTH: i32 = 64 - 2 * PACKED_HORIZONTAL_LENGTH;
const PACKED_X_MASK: i64 = (1_i64 << PACKED_HORIZONTAL_LENGTH) - 1;
const PACKED_Y_MASK: i64 = (1_i64 << PACKED_Y_LENGTH) - 1;
const PACKED_Z_MASK: i64 = (1_i64 << PACKED_HORIZONTAL_LENGTH) - 1;
const Y_OFFSET: i32 = 0;
const Z_OFFSET: i32 = PACKED_Y_LENGTH;
const X_OFFSET: i32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;

const fn log2_const_30000000() -> i32 {
    // Mth.smallestEncompassingPowerOfTwo(30000000) == 33554432 = 2^25 -> log2 == 25 -> 1 + 25 = 26
    25
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#containing
    #[inline]
    pub fn containing(x: f64, y: f64, z: f64) -> Self {
        Self::new(floor_f64(x), floor_f64(y), floor_f64(z))
    }

    // JAVA: net.minecraft.core.BlockPos#asLong(int,int,int)
    #[inline]
    pub fn as_long_coords(x: i32, y: i32, z: i32) -> i64 {
        let mut packed = 0_i64;
        packed |= ((x as i64) & PACKED_X_MASK) << (X_OFFSET & 63);
        packed |= ((y as i64) & PACKED_Y_MASK) << (Y_OFFSET & 63);
        packed | (((z as i64) & PACKED_Z_MASK) << (Z_OFFSET & 63))
    }

    // JAVA: net.minecraft.core.BlockPos#asLong()
    #[inline]
    pub fn as_long(&self) -> i64 {
        Self::as_long_coords(self.x, self.y, self.z)
    }

    // JAVA: net.minecraft.core.BlockPos#getX
    #[inline]
    pub fn get_x(packed: i64) -> i32 {
        let shl = (64 - X_OFFSET - PACKED_HORIZONTAL_LENGTH) & 63;
        let shr = (64 - PACKED_HORIZONTAL_LENGTH) & 63;
        ((packed << shl) >> shr) as i32
    }

    // JAVA: net.minecraft.core.BlockPos#getY
    #[inline]
    pub fn get_y(packed: i64) -> i32 {
        let shl = (64 - PACKED_Y_LENGTH) & 63;
        ((packed << shl) >> shl) as i32
    }

    // JAVA: net.minecraft.core.BlockPos#getZ
    #[inline]
    pub fn get_z(packed: i64) -> i32 {
        let shl = (64 - Z_OFFSET - PACKED_HORIZONTAL_LENGTH) & 63;
        let shr = (64 - PACKED_HORIZONTAL_LENGTH) & 63;
        ((packed << shl) >> shr) as i32
    }

    // JAVA: net.minecraft.core.BlockPos#of
    #[inline]
    pub fn of(packed: i64) -> Self {
        Self::new(Self::get_x(packed), Self::get_y(packed), Self::get_z(packed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SectionPos {
    const SECTION_PACKED_X_MASK: i64 = 4194303;
    const SECTION_PACKED_Y_MASK: i64 = 1048575;
    const SECTION_PACKED_Z_MASK: i64 = 4194303;

    // JAVA: net.minecraft.core.SectionPos#of(int,int,int)
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.SectionPos#blockToSectionCoord(int)
    #[inline]
    pub const fn block_to_section_coord(coord: i32) -> i32 {
        coord >> 4
    }

    // JAVA: net.minecraft.core.SectionPos#asLong(int,int,int)
    #[inline]
    pub const fn as_long_coords(x: i32, y: i32, z: i32) -> i64 {
        let mut packed = 0_i64;
        packed |= ((x as i64) & Self::SECTION_PACKED_X_MASK) << 42;
        packed |= ((y as i64) & Self::SECTION_PACKED_Y_MASK) << 0;
        packed | (((z as i64) & Self::SECTION_PACKED_Z_MASK) << 20)
    }

    // JAVA: net.minecraft.core.SectionPos#x(long)
    #[inline]
    pub const fn get_x(packed: i64) -> i32 {
        ((packed << 0) >> 42) as i32
    }

    // JAVA: net.minecraft.core.SectionPos#y(long)
    #[inline]
    pub const fn get_y(packed: i64) -> i32 {
        ((packed << 44) >> 44) as i32
    }

    // JAVA: net.minecraft.core.SectionPos#z(long)
    #[inline]
    pub const fn get_z(packed: i64) -> i32 {
        ((packed << 22) >> 42) as i32
    }

    // JAVA: net.minecraft.core.SectionPos#of(long)
    #[inline]
    pub const fn of(packed: i64) -> Self {
        Self::new(Self::get_x(packed), Self::get_y(packed), Self::get_z(packed))
    }

    pub fn verify_constants() {
        let _ = (log2(smallest_encompassing_power_of_two(30_000_000)),);
    }
}

const MAGIC: [i32; 192] = [
    -1, -1, 0, i32::MIN, 0, 0, 1431655765, 1431655765, 0, i32::MIN, 0, 1, 858993459, 858993459, 0,
    715827882, 715827882, 0, 613566756, 613566756, 0, i32::MIN, 0, 2, 477218588, 477218588, 0,
    429496729, 429496729, 0, 390451572, 390451572, 0, 357913941, 357913941, 0, 330382099,
    330382099, 0, 306783378, 306783378, 0, 286331153, 286331153, 0, i32::MIN, 0, 3, 252645135,
    252645135, 0, 238609294, 238609294, 0, 226050910, 226050910, 0, 214748364, 214748364, 0,
    204522252, 204522252, 0, 195225786, 195225786, 0, 186737708, 186737708, 0, 178956970,
    178956970, 0, 171798691, 171798691, 0, 165191049, 165191049, 0, 159072862, 159072862, 0,
    153391689, 153391689, 0, 148102320, 148102320, 0, 143165576, 143165576, 0, 138547332,
    138547332, 0, i32::MIN, 0, 4, 130150524, 130150524, 0, 126322567, 126322567, 0, 122713351,
    122713351, 0, 119304647, 119304647, 0, 116080197, 116080197, 0, 113025455, 113025455, 0,
    110127366, 110127366, 0, 107374182, 107374182, 0, 104755299, 104755299, 0, 102261126,
    102261126, 0, 99882960, 99882960, 0, 97612893, 97612893, 0, 95443717, 95443717, 0, 93368854,
    93368854, 0, 91382282, 91382282, 0, 89478485, 89478485, 0, 87652393, 87652393, 0, 85899345,
    85899345, 0, 84215045, 84215045, 0, 82595524, 82595524, 0, 81037118, 81037118, 0, 79536431,
    79536431, 0, 78090314, 78090314, 0, 76695844, 76695844, 0, 75350303, 75350303, 0, 74051160,
    74051160, 0, 72796055, 72796055, 0, 71582788, 71582788, 0, 70409299, 70409299, 0, 69273666,
    69273666, 0, 68174084, 68174084, 0, i32::MIN, 0, 5,
];

#[derive(Debug, Clone)]
pub struct SimpleBitStorage {
    data: Vec<i64>,
    bits: usize,
    mask: i64,
    size: usize,
    values_per_long: usize,
    divide_mul: i32,
    divide_add: i32,
    divide_shift: i32,
}

impl SimpleBitStorage {
    // JAVA: net.minecraft.util.SimpleBitStorage#<init>(int,int,long[])
    pub fn new(bits: usize, size: usize, data: Option<Vec<i64>>) -> Result<Self, String> {
        if !(1..=32).contains(&bits) {
            return Err(format!("bits {bits} out of 1..=32"));
        }
        let mask = (1_i64 << (bits & 63)).wrapping_sub(1);
        let values_per_long = 64 / bits;
        let magic_idx = 3 * (values_per_long - 1);
        let divide_mul = MAGIC[magic_idx];
        let divide_add = MAGIC[magic_idx + 1];
        let divide_shift = MAGIC[magic_idx + 2];
        let expected_len = size.div_ceil(values_per_long);
        let buf = match data {
            Some(v) => {
                if v.len() != expected_len {
                    return Err(format!("Invalid length: got {}, expected {}", v.len(), expected_len));
                }
                v
            }
            None => vec![0_i64; expected_len],
        };
        Ok(Self {
            data: buf,
            bits,
            mask,
            size,
            values_per_long,
            divide_mul,
            divide_add,
            divide_shift,
        })
    }

    // JAVA: net.minecraft.util.SimpleBitStorage#<init>(int,int,int[])
    pub fn from_ints(bits: usize, size: usize, values: &[i32]) -> Result<Self, String> {
        let mut storage = Self::new(bits, size, None)?;
        let mut out_idx = 0;
        let mut i = 0;
        while i + storage.values_per_long <= size {
            let mut word = 0_i64;
            for j in (0..storage.values_per_long).rev() {
                word <<= storage.bits & 63;
                word |= (values[i + j] as i64) & storage.mask;
            }
            storage.data[out_idx] = word;
            out_idx += 1;
            i += storage.values_per_long;
        }
        let rem = size - i;
        if rem > 0 {
            let mut word = 0_i64;
            for j in (0..rem).rev() {
                word <<= storage.bits & 63;
                word |= (values[i + j] as i64) & storage.mask;
            }
            storage.data[out_idx] = word;
        }
        Ok(storage)
    }

    // JAVA: net.minecraft.util.SimpleBitStorage#cellIndex
    #[inline]
    fn cell_index(&self, index: usize) -> usize {
        let mul = (self.divide_mul as u32) as i64;
        let add = (self.divide_add as u32) as i64;
        ((((index as i64).wrapping_mul(mul).wrapping_add(add)) >> 32) >> (self.divide_shift & 63))
            as usize
    }

    // JAVA: net.minecraft.util.SimpleBitStorage#get
    #[inline]
    pub fn get(&self, index: usize) -> i32 {
        assert!(index < self.size);
        let cell = self.cell_index(index);
        let word = self.data[cell];
        let bit = ((index - cell * self.values_per_long) * self.bits) & 63;
        ((word >> bit) & self.mask) as i32
    }

    // JAVA: net.minecraft.util.SimpleBitStorage#set
    #[inline]
    pub fn set(&mut self, index: usize, value: i32) {
        assert!(index < self.size);
        let cell = self.cell_index(index);
        let word = self.data[cell];
        let bit = ((index - cell * self.values_per_long) * self.bits) & 63;
        self.data[cell] = (word & !(self.mask << bit)) | (((value as i64) & self.mask) << bit);
    }

    // JAVA: net.minecraft.util.SimpleBitStorage#getAndSet
    #[inline]
    pub fn get_and_set(&mut self, index: usize, value: i32) -> i32 {
        assert!(index < self.size);
        let cell = self.cell_index(index);
        let word = self.data[cell];
        let bit = ((index - cell * self.values_per_long) * self.bits) & 63;
        let old = ((word >> bit) & self.mask) as i32;
        self.data[cell] = (word & !(self.mask << bit)) | (((value as i64) & self.mask) << bit);
        old
    }

    // JAVA: net.minecraft.util.SimpleBitStorage#getRaw
    #[inline]
    pub fn raw(&self) -> &[i64] {
        &self.data
    }

    // JAVA: net.minecraft.util.SimpleBitStorage#unpack
    pub fn unpack(&self, out: &mut [i32]) {
        assert!(out.len() >= self.size);
        let n = self.data.len();
        let mut idx = 0;
        for &word_init in &self.data[..n.saturating_sub(1)] {
            let mut word = word_init;
            for j in 0..self.values_per_long {
                out[idx + j] = (word & self.mask) as i32;
                word >>= self.bits & 63;
            }
            idx += self.values_per_long;
        }
        let rem = self.size - idx;
        if rem > 0 {
            let mut word = self.data[n - 1];
            for j in 0..rem {
                out[idx + j] = (word & self.mask) as i32;
                word >>= self.bits & 63;
            }
        }
    }
}
