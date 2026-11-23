#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Brightness {
    pub block: i32,
    pub sky: i32,
}

impl Brightness {
    pub const FULL_BRIGHT: Brightness = Brightness { block: 15, sky: 15 };

    // JAVA: net.minecraft.util.Brightness#pack
    pub fn pack_coords(block: i32, sky: i32) -> i32 {
        block.wrapping_shl(4) | sky.wrapping_shl(20)
    }

    // JAVA: net.minecraft.util.Brightness#pack
    pub fn pack(&self) -> i32 {
        Self::pack_coords(self.block, self.sky)
    }

    // JAVA: net.minecraft.util.Brightness#block
    pub fn block(packed: i32) -> i32 {
        packed.wrapping_shr(4) & 0xFFFF
    }

    // JAVA: net.minecraft.util.Brightness#sky
    pub fn sky(packed: i32) -> i32 {
        packed.wrapping_shr(20) & 0xFFFF
    }

    // JAVA: net.minecraft.util.Brightness#unpack
    pub fn unpack(packed: i32) -> Self {
        Self {
            block: Self::block(packed),
            sky: Self::sky(packed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_bright_and_roundtrip() {
        assert_eq!(Brightness::FULL_BRIGHT, Brightness { block: 15, sky: 15 });
        let b = Brightness { block: 7, sky: 13 };
        let packed = b.pack();
        assert_eq!(Brightness::unpack(packed), b);
    }

    #[test]
    fn pack_layout_matches_java_shifts() {
        assert_eq!(Brightness::pack_coords(0xF, 0), 0xF0);
        assert_eq!(Brightness::pack_coords(0, 0xF), 0x00F0_0000);
        let packed = Brightness::pack_coords(15, 15);
        assert_eq!(Brightness::block(packed), 15);
        assert_eq!(Brightness::sky(packed), 15);
    }

    #[test]
    fn negative_and_overflowing_values_are_masked() {
        let packed = Brightness::pack_coords(-1, -1);
        assert_eq!(Brightness::block(packed), 0xFFFF);
        assert_eq!(Brightness::sky(packed), 0xFFFF);

        let packed_min = i32::MIN;
        assert_eq!(
            Brightness::block(packed_min),
            packed_min.wrapping_shr(4) & 0xFFFF
        );
        assert_eq!(
            Brightness::sky(packed_min),
            packed_min.wrapping_shr(20) & 0xFFFF
        );
    }

    #[test]
    fn unpack_reads_independent_fields() {
        let packed = Brightness::pack_coords(1, 2) | Brightness::pack_coords(3, 4);
        let unpacked = Brightness::unpack(packed);
        assert_eq!(unpacked.block, Brightness::block(packed));
        assert_eq!(unpacked.sky, Brightness::sky(packed));
    }
}
