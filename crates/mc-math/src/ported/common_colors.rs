// JAVA: net.minecraft.util.CommonColors#class
/// Port of `net.minecraft.util.CommonColors`.
///
/// ARGB color constants represented as Java `int` values (`i32` in Rust).
pub const WHITE: i32 = -1;
pub const BLACK: i32 = -16_777_216;
pub const GRAY: i32 = -8_355_712;
pub const DARK_GRAY: i32 = -12_566_464;
pub const LIGHT_GRAY: i32 = -6_250_336;
pub const LIGHTER_GRAY: i32 = -4_539_718;
pub const RED: i32 = -65_536;
pub const SOFT_RED: i32 = -2_142_128;
pub const GREEN: i32 = -16_711_936;
pub const BLUE: i32 = -16_776_961;
pub const YELLOW: i32 = -256;
pub const SOFT_YELLOW: i32 = -171;
pub const DARK_PURPLE: i32 = -11_534_256;
pub const HIGH_CONTRAST_DIAMOND: i32 = -11_010_079;
pub const COSMOS_PINK: i32 = -13_108;
pub const TEXT_GRAY: i32 = -2_039_584;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_and_black_match_java_int_bits() {
        assert_eq!(WHITE, -1);
        assert_eq!(WHITE as u32, 0xFFFF_FFFF);
        assert_eq!(BLACK, -16_777_216);
        assert_eq!(BLACK as u32, 0xFF00_0000);
    }

    #[test]
    fn primary_colors_match() {
        assert_eq!(RED, -65_536);
        assert_eq!(GREEN, -16_711_936);
        assert_eq!(BLUE, -16_776_961);
        assert_eq!(RED as u32, 0xFFFF_0000);
        assert_eq!(GREEN as u32, 0xFF00_FF00);
        assert_eq!(BLUE as u32, 0xFF00_00FF);
    }

    #[test]
    fn gray_and_accent_values_match() {
        assert_eq!(GRAY, -8_355_712);
        assert_eq!(DARK_GRAY, -12_566_464);
        assert_eq!(LIGHT_GRAY, -6_250_336);
        assert_eq!(LIGHTER_GRAY, -4_539_718);
        assert_eq!(SOFT_YELLOW, -171);
        assert_eq!(COSMOS_PINK, -13_108);
    }

    #[test]
    fn all_constants_are_negative_or_minus_one() {
        let all = [
            WHITE,
            BLACK,
            GRAY,
            DARK_GRAY,
            LIGHT_GRAY,
            LIGHTER_GRAY,
            RED,
            SOFT_RED,
            GREEN,
            BLUE,
            YELLOW,
            SOFT_YELLOW,
            DARK_PURPLE,
            HIGH_CONTRAST_DIAMOND,
            COSMOS_PINK,
            TEXT_GRAY,
        ];
        for c in all {
            assert!(c < 0, "color constant should be negative i32: {c}");
        }
    }
}
