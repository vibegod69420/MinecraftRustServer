//! Faithful port of `net.minecraft.core.QuartPos`.

pub const BITS: i32 = 2;
pub const SIZE: i32 = 4;
pub const MASK: i32 = 3;
const SECTION_TO_QUARTS_BITS: i32 = 2;

// JAVA: net.minecraft.core.QuartPos#fromBlock
pub fn from_block(v: i32) -> i32 {
    v >> (BITS & 31)
}

// JAVA: net.minecraft.core.QuartPos#quartLocal
pub fn quart_local(v: i32) -> i32 {
    v & MASK
}

// JAVA: net.minecraft.core.QuartPos#toBlock
pub fn to_block(v: i32) -> i32 {
    v.wrapping_shl((BITS as u32) & 31)
}

// JAVA: net.minecraft.core.QuartPos#fromSection
pub fn from_section(v: i32) -> i32 {
    v.wrapping_shl((SECTION_TO_QUARTS_BITS as u32) & 31)
}

// JAVA: net.minecraft.core.QuartPos#toSection
pub fn to_section(v: i32) -> i32 {
    v >> (SECTION_TO_QUARTS_BITS & 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_java() {
        assert_eq!(BITS, 2);
        assert_eq!(SIZE, 4);
        assert_eq!(MASK, 3);
    }

    #[test]
    fn from_block_positive_and_zero() {
        assert_eq!(from_block(0), 0);
        assert_eq!(from_block(4), 1);
        assert_eq!(from_block(15), 3);
        assert_eq!(from_block(16), 4);
    }

    #[test]
    fn from_block_negative_arithmetic_shift() {
        assert_eq!(from_block(-1), -1);
        assert_eq!(from_block(-4), -1);
        assert_eq!(from_block(-5), -2);
        assert_eq!(from_block(i32::MIN), i32::MIN >> 2);
    }

    #[test]
    fn quart_local_masks_low_bits() {
        assert_eq!(quart_local(0), 0);
        assert_eq!(quart_local(1), 1);
        assert_eq!(quart_local(3), 3);
        assert_eq!(quart_local(4), 0);
        assert_eq!(quart_local(-1), 3);
    }

    #[test]
    fn to_block_and_from_section_shift_left() {
        assert_eq!(to_block(0), 0);
        assert_eq!(to_block(1), 4);
        assert_eq!(to_block(-1), -4);
        assert_eq!(from_section(3), 12);
        assert_eq!(from_section(-2), -8);
    }

    #[test]
    fn to_section_matches_from_block_semantics() {
        assert_eq!(to_section(0), 0);
        assert_eq!(to_section(4), 1);
        assert_eq!(to_section(-1), -1);
        assert_eq!(to_section(-4), -1);
        assert_eq!(to_section(-5), -2);
    }
}
