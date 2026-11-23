//! Faithful port of `net.minecraft.util.StringDecomposer` (Minecraft 1.21.11).

const REPLACEMENT_CHAR: i32 = 0xFFFD;

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
    pub fn get_by_code(code: char) -> Option<Self> {
        match code.to_ascii_lowercase() {
            '0' => Some(Self::Black),
            '1' => Some(Self::DarkBlue),
            '2' => Some(Self::DarkGreen),
            '3' => Some(Self::DarkAqua),
            '4' => Some(Self::DarkRed),
            '5' => Some(Self::DarkPurple),
            '6' => Some(Self::Gold),
            '7' => Some(Self::Gray),
            '8' => Some(Self::DarkGray),
            '9' => Some(Self::Blue),
            'a' => Some(Self::Green),
            'b' => Some(Self::Aqua),
            'c' => Some(Self::Red),
            'd' => Some(Self::LightPurple),
            'e' => Some(Self::Yellow),
            'f' => Some(Self::White),
            'k' => Some(Self::Obfuscated),
            'l' => Some(Self::Bold),
            'm' => Some(Self::Strikethrough),
            'n' => Some(Self::Underline),
            'o' => Some(Self::Italic),
            'r' => Some(Self::Reset),
            _ => None,
        }
    }

    #[inline]
    pub fn is_color(self) -> bool {
        matches!(
            self,
            Self::Black
                | Self::DarkBlue
                | Self::DarkGreen
                | Self::DarkAqua
                | Self::DarkRed
                | Self::DarkPurple
                | Self::Gold
                | Self::Gray
                | Self::DarkGray
                | Self::Blue
                | Self::Green
                | Self::Aqua
                | Self::Red
                | Self::LightPurple
                | Self::Yellow
                | Self::White
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: Option<ChatFormatting>,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub strikethrough: bool,
    pub obfuscated: bool,
}

impl Style {
    pub const EMPTY: Self = Self {
        color: None,
        bold: false,
        italic: false,
        underlined: false,
        strikethrough: false,
        obfuscated: false,
    };

    pub fn apply_legacy_format(self, formatting: ChatFormatting) -> Self {
        if formatting.is_color() {
            return Self {
                color: Some(formatting),
                bold: false,
                italic: false,
                underlined: false,
                strikethrough: false,
                obfuscated: false,
            };
        }
        match formatting {
            ChatFormatting::Obfuscated => Self {
                obfuscated: true,
                ..self
            },
            ChatFormatting::Bold => Self { bold: true, ..self },
            ChatFormatting::Strikethrough => Self {
                strikethrough: true,
                ..self
            },
            ChatFormatting::Underline => Self {
                underlined: true,
                ..self
            },
            ChatFormatting::Italic => Self {
                italic: true,
                ..self
            },
            ChatFormatting::Reset => Self::EMPTY,
            _ => self,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::EMPTY
    }
}

pub trait FormattedCharSink {
    fn accept(&mut self, index: i32, style: &Style, code_point: i32) -> bool;
}

impl<F> FormattedCharSink for F
where
    F: for<'a> FnMut(i32, &'a Style, i32) -> bool,
{
    fn accept(&mut self, index: i32, style: &Style, code_point: i32) -> bool {
        self(index, style, code_point)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unit;

pub trait FormattedText {
    fn visit<T, F>(&self, acceptor: F, style: Style) -> Option<T>
    where
        F: FnMut(Style, &str) -> Option<T>;
}

#[inline]
fn is_high_surrogate(c: u16) -> bool {
    (0xD800..=0xDBFF).contains(&c)
}

#[inline]
fn is_low_surrogate(c: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&c)
}

#[inline]
fn is_surrogate(c: u16) -> bool {
    (0xD800..=0xDFFF).contains(&c)
}

#[inline]
fn to_code_point(high: u16, low: u16) -> i32 {
    let h = (high as i32).wrapping_sub(0xD800);
    let l = (low as i32).wrapping_sub(0xDC00);
    h.wrapping_shl(10)
        .wrapping_add(l)
        .wrapping_add(0x1_0000)
}

// JAVA: net.minecraft.util.StringDecomposer#feedChar
fn feed_char(style: &Style, sink: &mut impl FormattedCharSink, index: i32, c: u16) -> bool {
    if is_surrogate(c) {
        sink.accept(index, style, REPLACEMENT_CHAR)
    } else {
        sink.accept(index, style, c as i32)
    }
}

// JAVA: net.minecraft.util.StringDecomposer#iterate
pub fn iterate(text: &str, style: &Style, sink: &mut impl FormattedCharSink) -> bool {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    iterate_utf16(&utf16, style, sink)
}

pub fn iterate_utf16(text: &[u16], style: &Style, sink: &mut impl FormattedCharSink) -> bool {
    let len = text.len() as i32;
    let mut i: i32 = 0;
    while i < len {
        let c = text[i as usize];
        if is_high_surrogate(c) {
            let next = i.wrapping_add(1);
            if next >= len {
                if !sink.accept(i, style, REPLACEMENT_CHAR) {
                    return false;
                }
                break;
            }
            let c2 = text[next as usize];
            if is_low_surrogate(c2) {
                if !sink.accept(i, style, to_code_point(c, c2)) {
                    return false;
                }
                i = i.wrapping_add(1);
            } else if !sink.accept(i, style, REPLACEMENT_CHAR) {
                return false;
            }
        } else if !feed_char(style, sink, i, c) {
            return false;
        }
        i = i.wrapping_add(1);
    }
    true
}

// JAVA: net.minecraft.util.StringDecomposer#iterateBackwards
pub fn iterate_backwards(text: &str, style: &Style, sink: &mut impl FormattedCharSink) -> bool {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    iterate_backwards_utf16(&utf16, style, sink)
}

pub fn iterate_backwards_utf16(
    text: &[u16],
    style: &Style,
    sink: &mut impl FormattedCharSink,
) -> bool {
    let len = text.len() as i32;
    let mut i: i32 = len.wrapping_sub(1);
    while i >= 0 {
        let c = text[i as usize];
        if is_low_surrogate(c) {
            let prev = i.wrapping_sub(1);
            if prev < 0 {
                if !sink.accept(0, style, REPLACEMENT_CHAR) {
                    return false;
                }
                break;
            }
            let c2 = text[prev as usize];
            if is_high_surrogate(c2) {
                i = prev;
                if !sink.accept(i, style, to_code_point(c2, c)) {
                    return false;
                }
            } else if !sink.accept(i, style, REPLACEMENT_CHAR) {
                return false;
            }
        } else if !feed_char(style, sink, i, c) {
            return false;
        }
        i = i.wrapping_sub(1);
    }
    true
}

// JAVA: net.minecraft.util.StringDecomposer#iterateFormatted
pub fn iterate_formatted(text: &str, style: &Style, sink: &mut impl FormattedCharSink) -> bool {
    iterate_formatted_from(text, 0, style, sink)
}

// JAVA: net.minecraft.util.StringDecomposer#iterateFormatted
pub fn iterate_formatted_from(
    text: &str,
    start: i32,
    style: &Style,
    sink: &mut impl FormattedCharSink,
) -> bool {
    iterate_formatted_from_with_default(text, start, style, style, sink)
}

// JAVA: net.minecraft.util.StringDecomposer#iterateFormatted
pub fn iterate_formatted_from_with_default(
    text: &str,
    start: i32,
    style: &Style,
    default_style: &Style,
    sink: &mut impl FormattedCharSink,
) -> bool {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    iterate_formatted_utf16_from_with_default(&utf16, start, style, default_style, sink)
}

pub fn iterate_formatted_utf16_from_with_default(
    text: &[u16],
    start: i32,
    style: &Style,
    default_style: &Style,
    sink: &mut impl FormattedCharSink,
) -> bool {
    let len = text.len() as i32;
    let mut current_style = *style;
    let mut i = start;
    while i < len {
        let c = text[i as usize];
        if c == 167 {
            let next = i.wrapping_add(1);
            if next >= len {
                break;
            }
            let code_char = text[next as usize];
            if let Some(ch) = char::from_u32(code_char as u32) {
                if let Some(fmt) = ChatFormatting::get_by_code(ch) {
                    current_style = if fmt == ChatFormatting::Reset {
                        *default_style
                    } else {
                        current_style.apply_legacy_format(fmt)
                    };
                }
            }
            i = i.wrapping_add(1);
        } else if is_high_surrogate(c) {
            let next = i.wrapping_add(1);
            if next >= len {
                if !sink.accept(i, &current_style, REPLACEMENT_CHAR) {
                    return false;
                }
                break;
            }
            let c2 = text[next as usize];
            if is_low_surrogate(c2) {
                if !sink.accept(i, &current_style, to_code_point(c, c2)) {
                    return false;
                }
                i = i.wrapping_add(1);
            } else if !sink.accept(i, &current_style, REPLACEMENT_CHAR) {
                return false;
            }
        } else if !feed_char(&current_style, sink, i, c) {
            return false;
        }
        i = i.wrapping_add(1);
    }
    true
}

// JAVA: net.minecraft.util.StringDecomposer#iterateFormatted
pub fn iterate_formatted_text(
    text: &impl FormattedText,
    style: &Style,
    sink: &mut impl FormattedCharSink,
) -> bool {
    text.visit(
        |part_style: Style, content: &str| {
            if iterate_formatted_from_with_default(content, 0, &part_style, &part_style, sink) {
                None::<Unit>
            } else {
                Some(Unit)
            }
        },
        *style,
    )
    .is_none()
}

// JAVA: net.minecraft.util.StringDecomposer#filterBrokenSurrogates
pub fn filter_broken_surrogates(text: &str) -> String {
    let mut out = String::new();
    iterate(
        text,
        &Style::EMPTY,
        &mut |_index: i32, _style: &Style, cp: i32| {
            if let Some(ch) = char::from_u32(cp as u32) {
                out.push(ch);
            } else {
                out.push('\u{FFFD}');
            }
            true
        },
    );
    out
}

// JAVA: net.minecraft.util.StringDecomposer#getPlainText
pub fn get_plain_text(text: &impl FormattedText) -> String {
    let mut out = String::new();
    iterate_formatted_text(
        text,
        &Style::EMPTY,
        &mut |_index: i32, _style: &Style, cp: i32| {
            if let Some(ch) = char::from_u32(cp as u32) {
                out.push(ch);
            } else {
                out.push('\u{FFFD}');
            }
            true
        },
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterate_ascii_and_surrogate_pairs() {
        let mut out: Vec<(i32, i32)> = Vec::new();
        let ok = iterate(
            "A\u{10000}B",
            &Style::EMPTY,
            &mut |idx: i32, _style: &Style, cp: i32| {
                out.push((idx, cp));
                true
            },
        );
        assert!(ok);
        assert_eq!(out, vec![(0, 'A' as i32), (1, 0x10000), (3, 'B' as i32)]);

        let mut cps: Vec<i32> = Vec::new();
        iterate_utf16(
            &[0xDBFF, 0xDFFF],
            &Style::EMPTY,
            &mut |_i: i32, _s: &Style, cp: i32| {
                cps.push(cp);
                true
            },
        );
        assert_eq!(cps, vec![0x10FFFF]);
    }

    #[test]
    fn iterate_broken_surrogates_and_backwards() {
        let mut out: Vec<i32> = Vec::new();
        iterate_utf16(
            &[0xD800],
            &Style::EMPTY,
            &mut |_idx: i32, _style: &Style, cp: i32| {
                out.push(cp);
                true
            },
        );
        iterate_utf16(
            &[0xD800, 0x0041],
            &Style::EMPTY,
            &mut |_idx: i32, _style: &Style, cp: i32| {
                out.push(cp);
                true
            },
        );
        iterate_utf16(
            &[0xDC00],
            &Style::EMPTY,
            &mut |_idx: i32, _style: &Style, cp: i32| {
                out.push(cp);
                true
            },
        );
        assert_eq!(out, vec![0xFFFD, 0xFFFD, 'A' as i32, 0xFFFD]);

        let mut back: Vec<(i32, i32)> = Vec::new();
        let ok = iterate_backwards(
            "A\u{10000}B",
            &Style::EMPTY,
            &mut |idx: i32, _style: &Style, cp: i32| {
                back.push((idx, cp));
                true
            },
        );
        assert!(ok);
        assert_eq!(back, vec![(3, 'B' as i32), (1, 0x10000), (0, 'A' as i32)]);

        let mut lone_low: Vec<(i32, i32)> = Vec::new();
        iterate_backwards_utf16(
            &[0xDC00],
            &Style::EMPTY,
            &mut |idx: i32, _style: &Style, cp: i32| {
                lone_low.push((idx, cp));
                true
            },
        );
        assert_eq!(lone_low, vec![(0, 0xFFFD)]);
    }

    #[test]
    fn iterate_formatted_codes_reset_and_trailing() {
        let mut seen: Vec<(i32, char, bool)> = Vec::new();
        let ok = iterate_formatted(
            "a§lb§rc",
            &Style::EMPTY,
            &mut |idx: i32, style: &Style, cp: i32| {
                seen.push((idx, char::from_u32(cp as u32).unwrap(), style.bold));
                true
            },
        );
        assert!(ok);
        assert_eq!(
            seen,
            vec![(0, 'a', false), (3, 'b', true), (6, 'c', false)]
        );

        let mut chars: Vec<char> = Vec::new();
        let ok2 = iterate_formatted(
            "§xA§",
            &Style::EMPTY,
            &mut |_idx: i32, _style: &Style, cp: i32| {
                chars.push(char::from_u32(cp as u32).unwrap());
                true
            },
        );
        assert!(ok2);
        assert_eq!(chars, vec!['A']);
    }

    #[test]
    fn early_stop_and_plain_text() {
        let mut count = 0i32;
        let ok = iterate(
            "abcd",
            &Style::EMPTY,
            &mut |_idx: i32, _style: &Style, _cp: i32| {
                count = count.wrapping_add(1);
                count < 2
            },
        );
        assert!(!ok);
        assert_eq!(count, 2);

        assert_eq!(filter_broken_surrogates("A\u{10000}B"), "A\u{10000}B");

        struct Plain(String);
        impl FormattedText for Plain {
            fn visit<T, F>(&self, mut acceptor: F, style: Style) -> Option<T>
            where
                F: FnMut(Style, &str) -> Option<T>,
            {
                acceptor(style, &self.0)
            }
        }

        assert_eq!(get_plain_text(&Plain("§aX".to_string())), "X");
    }

    #[test]
    fn formatted_start_and_boundaries() {
        let mut calls = 0i32;
        let ok = iterate_formatted_from(
            "ABC",
            3,
            &Style::EMPTY,
            &mut |_idx: i32, _style: &Style, _cp: i32| {
                calls = calls.wrapping_add(1);
                true
            },
        );
        assert!(ok);
        assert_eq!(calls, 0);

        let ok2 = iterate_formatted_from(
            "ABC",
            i32::MAX,
            &Style::EMPTY,
            &mut |_idx: i32, _style: &Style, _cp: i32| {
                calls = calls.wrapping_add(1);
                true
            },
        );
        assert!(ok2);
        assert_eq!(calls, 0);

        let mut out: Vec<i32> = Vec::new();
        iterate_formatted_from(
            "A\u{10000}B",
            1,
            &Style::EMPTY,
            &mut |_idx: i32, _style: &Style, cp: i32| {
                out.push(cp);
                true
            },
        );
        assert_eq!(out, vec![0x10000, 'B' as i32]);
    }
}
