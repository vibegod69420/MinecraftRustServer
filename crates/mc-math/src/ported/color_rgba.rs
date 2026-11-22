use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorRGBA {
    rgba: i32,
}

impl ColorRGBA {
    // JAVA: net.minecraft.util.ColorRGBA#<init>
    pub fn new(rgba: i32) -> Self {
        Self { rgba }
    }

    // JAVA: net.minecraft.util.ColorRGBA#rgba
    pub fn rgba(&self) -> i32 {
        self.rgba
    }
}

impl fmt::Display for ColorRGBA {
    // JAVA: net.minecraft.util.ColorRGBA#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}", self.rgba as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::ColorRGBA;

    #[test]
    fn to_string_zero() {
        assert_eq!(ColorRGBA::new(0).to_string(), "00000000");
    }

    #[test]
    fn to_string_negative_one() {
        assert_eq!(ColorRGBA::new(-1).to_string(), "ffffffff");
    }

    #[test]
    fn to_string_min_value() {
        assert_eq!(ColorRGBA::new(i32::MIN).to_string(), "80000000");
    }

    #[test]
    fn rgba_accessor_and_hex_format() {
        let c = ColorRGBA::new(0x12AB_CD34u32 as i32);
        assert_eq!(c.rgba(), 0x12AB_CD34u32 as i32);
        assert_eq!(c.to_string(), "12abcd34");
    }
}
