use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChatFormatting {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}

impl ChatFormatting {
    pub fn is_color(self) -> bool {
        self.get_color().is_some()
    }

    pub fn get_color(self) -> Option<i32> {
        match self {
            Self::Black => Some(0x000000),
            Self::DarkBlue => Some(0x0000AA),
            Self::DarkGreen => Some(0x00AA00),
            Self::DarkAqua => Some(0x00AAAA),
            Self::DarkRed => Some(0xAA0000),
            Self::DarkPurple => Some(0xAA00AA),
            Self::Gold => Some(0xFFAA00),
            Self::Gray => Some(0xAAAAAA),
            Self::DarkGray => Some(0x555555),
            Self::Blue => Some(0x5555FF),
            Self::Green => Some(0x55FF55),
            Self::Aqua => Some(0x55FFFF),
            Self::Red => Some(0xFF5555),
            Self::LightPurple => Some(0xFF55FF),
            Self::Yellow => Some(0xFFFF55),
            Self::White => Some(0xFFFFFF),
            _ => None,
        }
    }

    pub fn get_name(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::DarkBlue => "dark_blue",
            Self::DarkGreen => "dark_green",
            Self::DarkAqua => "dark_aqua",
            Self::DarkRed => "dark_red",
            Self::DarkPurple => "dark_purple",
            Self::Gold => "gold",
            Self::Gray => "gray",
            Self::DarkGray => "dark_gray",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Aqua => "aqua",
            Self::Red => "red",
            Self::LightPurple => "light_purple",
            Self::Yellow => "yellow",
            Self::White => "white",
            Self::Obfuscated => "obfuscated",
            Self::Bold => "bold",
            Self::Strikethrough => "strikethrough",
            Self::Underline => "underline",
            Self::Italic => "italic",
            Self::Reset => "reset",
        }
    }
}

fn named_colors() -> &'static [(ChatFormatting, i32, &'static str)] {
    &[
        (ChatFormatting::Black, 0x000000, "black"),
        (ChatFormatting::DarkBlue, 0x0000AA, "dark_blue"),
        (ChatFormatting::DarkGreen, 0x00AA00, "dark_green"),
        (ChatFormatting::DarkAqua, 0x00AAAA, "dark_aqua"),
        (ChatFormatting::DarkRed, 0xAA0000, "dark_red"),
        (ChatFormatting::DarkPurple, 0xAA00AA, "dark_purple"),
        (ChatFormatting::Gold, 0xFFAA00, "gold"),
        (ChatFormatting::Gray, 0xAAAAAA, "gray"),
        (ChatFormatting::DarkGray, 0x555555, "dark_gray"),
        (ChatFormatting::Blue, 0x5555FF, "blue"),
        (ChatFormatting::Green, 0x55FF55, "green"),
        (ChatFormatting::Aqua, 0x55FFFF, "aqua"),
        (ChatFormatting::Red, 0xFF5555, "red"),
        (ChatFormatting::LightPurple, 0xFF55FF, "light_purple"),
        (ChatFormatting::Yellow, 0xFFFF55, "yellow"),
        (ChatFormatting::White, 0xFFFFFF, "white"),
    ]
}

#[derive(Clone, Debug)]
pub struct TextColor {
    value: i32,
    name: Option<String>,
}

impl TextColor {
    // JAVA: net.minecraft.network.chat.TextColor#<init>
    fn new_named(value: i32, name: String) -> Self {
        Self {
            value: value & 0x00FF_FFFF,
            name: Some(name),
        }
    }

    // JAVA: net.minecraft.network.chat.TextColor#<init>
    fn new_rgb(value: i32) -> Self {
        Self {
            value: value & 0x00FF_FFFF,
            name: None,
        }
    }

    // JAVA: net.minecraft.network.chat.TextColor#getValue
    pub fn get_value(&self) -> i32 {
        self.value
    }

    // JAVA: net.minecraft.network.chat.TextColor#serialize
    pub fn serialize(&self) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => self.format_value(),
        }
    }

    // JAVA: net.minecraft.network.chat.TextColor#formatValue
    fn format_value(&self) -> String {
        format!("#{:06X}", self.value)
    }

    // JAVA: net.minecraft.network.chat.TextColor#fromLegacyFormat
    pub fn from_legacy_format(formatting: ChatFormatting) -> Option<TextColor> {
        for &(fmt, color, name) in named_colors() {
            if fmt == formatting {
                return Some(TextColor::new_named(color, name.to_string()));
            }
        }
        None
    }

    // JAVA: net.minecraft.network.chat.TextColor#fromRgb
    pub fn from_rgb(color: i32) -> TextColor {
        TextColor::new_rgb(color)
    }

    // JAVA: net.minecraft.network.chat.TextColor#parseColor
    pub fn parse_color(input: &str) -> Result<TextColor, String> {
        if let Some(hex) = input.strip_prefix('#') {
            match parse_java_int_hex(hex) {
                Ok(n) if (0..=0x00FF_FFFF).contains(&n) => Ok(TextColor::from_rgb(n)),
                Ok(_) => Err(format!("Color value out of range: {}", input)),
                Err(_) => Err(format!("Invalid color value: {}", input)),
            }
        } else {
            for &(_, color, name) in named_colors() {
                if name == input {
                    return Ok(TextColor::new_named(color, name.to_string()));
                }
            }
            Err(format!("Invalid color name: {}", input))
        }
    }
}

fn parse_java_int_hex(s: &str) -> Result<i32, ()> {
    let (negative, digits) = if let Some(rest) = s.strip_prefix('-') {
        (true, rest)
    } else {
        (false, s)
    };
    if digits.is_empty() {
        return Err(());
    }
    let mut result: i64 = 0;
    for &b in digits.as_bytes() {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as i64,
            b'a'..=b'f' => (b - b'a' + 10) as i64,
            b'A'..=b'F' => (b - b'A' + 10) as i64,
            _ => return Err(()),
        };
        result = result.wrapping_mul(16).wrapping_add(digit);
        if result > (i32::MAX as i64) + 1 {
            return Err(());
        }
    }
    let result = if negative { -result } else { result };
    if result < i32::MIN as i64 || result > i32::MAX as i64 {
        return Err(());
    }
    Ok(result as i32)
}

impl PartialEq for TextColor {
    // JAVA: net.minecraft.network.chat.TextColor#equals
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for TextColor {}

impl Hash for TextColor {
    // JAVA: net.minecraft.network.chat.TextColor#hashCode
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.name.hash(state);
    }
}

impl fmt::Display for TextColor {
    // JAVA: net.minecraft.network.chat.TextColor#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.serialize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color() {
        let c = TextColor::parse_color("#FF0000").unwrap();
        assert_eq!(c.get_value(), 0xFF0000);
        assert_eq!(c.serialize(), "#FF0000");
    }

    #[test]
    fn parse_named_color() {
        let c = TextColor::parse_color("red").unwrap();
        assert_eq!(c.get_value(), 0xFF5555);
        assert_eq!(c.serialize(), "red");
    }

    #[test]
    fn parse_invalid_and_out_of_range() {
        assert!(TextColor::parse_color("not_a_color").is_err());
        assert!(TextColor::parse_color("#GGGGGG").is_err());
        assert!(TextColor::parse_color("#1000000").is_err());
        assert_eq!(
            TextColor::parse_color("#-1").unwrap_err(),
            "Color value out of range: #-1"
        );
    }

    #[test]
    fn from_legacy_format_and_non_color() {
        let c = TextColor::from_legacy_format(ChatFormatting::DarkBlue).unwrap();
        assert_eq!(c.get_value(), 0x0000AA);
        assert_eq!(c.serialize(), "dark_blue");
        assert!(TextColor::from_legacy_format(ChatFormatting::Bold).is_none());
    }

    #[test]
    fn from_rgb_masks_and_negative_input() {
        let c = TextColor::from_rgb(0x01FF_FFFF_i32);
        assert_eq!(c.get_value(), 0x00FF_FFFF);
        let c2 = TextColor::from_rgb(-1);
        assert_eq!(c2.get_value(), 0x00FF_FFFF);
        assert_eq!(c2.serialize(), "#FFFFFF");
    }

    #[test]
    fn equality_ignores_name_but_serialize_preserves() {
        let named = TextColor::parse_color("red").unwrap();
        let rgb = TextColor::from_rgb(0x00FF_5555);
        assert_eq!(named, rgb);
        assert_eq!(named.serialize(), "red");
        assert_eq!(rgb.serialize(), "#FF5555");
        assert_eq!(named.to_string(), "red");
    }
}
