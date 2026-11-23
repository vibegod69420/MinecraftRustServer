//! Faithful port of `net.minecraft.util.StringUtil`.

use crate::mth;

fn is_color_code_char(c: char) -> bool {
    matches!(
        c,
        '0'..='9' | 'a'..='f' | 'k'..='o' | 'r' | 'A'..='F' | 'K'..='O' | 'R'
    )
}

// JAVA: net.minecraft.util.StringUtil#formatTickDuration
pub fn format_tick_duration(ticks: i32, ticks_per_second: f32) -> String {
    let total_seconds = mth::floor_f32((ticks as f32) / ticks_per_second);
    let mut seconds = total_seconds;
    let mut minutes = seconds / 60;
    seconds %= 60;
    let hours = minutes / 60;
    minutes %= 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

// JAVA: net.minecraft.util.StringUtil#stripColor
pub fn strip_color(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\u{00A7}' && i + 1 < chars.len() && is_color_code_char(chars[i + 1]) {
            i += 2;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// JAVA: net.minecraft.util.StringUtil#isNullOrEmpty
pub fn is_null_or_empty(s: Option<&str>) -> bool {
    match s {
        None => true,
        Some(s) => s.is_empty(),
    }
}

// JAVA: net.minecraft.util.StringUtil#truncateStringIfNecessary
pub fn truncate_string_if_necessary(s: &str, max_len: usize, add_ellipsis: bool) -> String {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    if utf16.len() <= max_len {
        return s.to_owned();
    }
    if add_ellipsis && max_len > 3 {
        let truncated: String = char::decode_utf16(utf16[..max_len - 3].iter().copied())
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .collect();
        format!("{truncated}...")
    } else {
        char::decode_utf16(utf16[..max_len].iter().copied())
            .map(|r| r.unwrap_or('\u{FFFD}'))
            .collect()
    }
}

// JAVA: net.minecraft.util.StringUtil#lineCount
pub fn line_count(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }
    let chars: Vec<char> = s.chars().collect();
    let mut count: usize = 1;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\r' && i + 1 < chars.len() && chars[i + 1] == '\n' {
            count += 1;
            i += 2;
        } else if chars[i] == '\u{000B}' {
            count += 1;
            i += 1;
        } else {
            i += 1;
        }
    }
    count
}

// JAVA: net.minecraft.util.StringUtil#endsWithNewLine
pub fn ends_with_new_line(s: &str) -> bool {
    s.ends_with('\u{000B}') || s.ends_with("\r\n")
}

// JAVA: net.minecraft.util.StringUtil#trimChatMessage
pub fn trim_chat_message(s: &str) -> String {
    truncate_string_if_necessary(s, 256, false)
}

// JAVA: net.minecraft.util.StringUtil#isAllowedChatCharacter
pub fn is_allowed_chat_character(c: i32) -> bool {
    c != 167 && c >= 32 && c != 127
}

// JAVA: net.minecraft.util.StringUtil#isValidPlayerName
pub fn is_valid_player_name(s: &str) -> bool {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    if utf16.len() > 16 {
        return false;
    }
    !utf16
        .iter()
        .any(|&c| (c as i32) <= 32 || (c as i32) >= 127)
}

// JAVA: net.minecraft.util.StringUtil#filterText
pub fn filter_text(s: &str) -> String {
    filter_text_allow_line_breaks(s, false)
}

// JAVA: net.minecraft.util.StringUtil#filterText
pub fn filter_text_allow_line_breaks(s: &str, allow_line_breaks: bool) -> String {
    let mut out = String::new();
    for c in s.chars() {
        let cp = c as i32;
        if is_allowed_chat_character(cp) {
            out.push(c);
        } else if allow_line_breaks && c == '\n' {
            out.push(c);
        }
    }
    out
}

// JAVA: net.minecraft.util.StringUtil#isWhitespace
pub fn is_whitespace(c: i32) -> bool {
    // Java: Character.isWhitespace(c) || Character.isSpaceChar(c)
    matches!(
        c,
        0x0009..=0x000D
            | 0x001C..=0x001F
            | 0x0020
            | 0x1680
            | 0x2000..=0x2006
            | 0x2008..=0x200A
            | 0x2028
            | 0x2029
            | 0x205F
            | 0x3000
    )
}

// JAVA: net.minecraft.util.StringUtil#isBlank
pub fn is_blank(s: Option<&str>) -> bool {
    match s {
        None => true,
        Some(s) if s.is_empty() => true,
        Some(s) => s.chars().all(|c| is_whitespace(c as i32)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tick_duration_formats_minutes_and_hours() {
        assert_eq!(format_tick_duration(20 * 65, 20.0), "01:05");
        assert_eq!(format_tick_duration(20 * 3665, 20.0), "01:01:05");
        assert_eq!(format_tick_duration(0, 20.0), "00:00");
    }

    #[test]
    fn strip_color_removes_section_codes_case_insensitively() {
        assert_eq!(strip_color("§aGreen§RReset"), "GreenReset");
        assert_eq!(strip_color("no codes"), "no codes");
        assert_eq!(strip_color("§xnot"), "§xnot");
    }

    #[test]
    fn truncate_uses_utf16_units_and_ellipsis() {
        assert_eq!(truncate_string_if_necessary("abcdef", 4, false), "abcd");
        assert_eq!(truncate_string_if_necessary("abcdef", 4, true), "a...");
        assert_eq!(truncate_string_if_necessary("abc", 4, true), "abc");
        // 😀 is a surrogate pair in UTF-16 (2 code units)
        assert_eq!(truncate_string_if_necessary("A😀B", 3, false), "A😀");
    }

    #[test]
    fn line_count_and_endings_match_java_patterns() {
        assert_eq!(line_count(""), 0);
        assert_eq!(line_count("a\nb"), 1); // \n alone is not matched by Java pattern
        assert_eq!(line_count("a\r\nb\u{000B}c"), 3);
        assert!(ends_with_new_line("hello\r\n"));
        assert!(ends_with_new_line("hello\u{000B}"));
        assert!(!ends_with_new_line("hello\n"));
    }

    #[test]
    fn chat_and_player_name_validation() {
        assert!(!is_allowed_chat_character(167)); // §
        assert!(!is_allowed_chat_character(127));
        assert!(is_allowed_chat_character(32));
        assert!(is_valid_player_name("Player_123"));
        assert!(!is_valid_player_name("ThisNameIsWayTooLong"));
        assert!(!is_valid_player_name("bad name"));
        assert_eq!(filter_text("a§b\nc"), "abc");
        assert_eq!(filter_text_allow_line_breaks("a§b\nc", true), "ab\nc");
    }

    #[test]
    fn blank_and_whitespace_checks() {
        assert!(is_null_or_empty(None));
        assert!(is_null_or_empty(Some("")));
        assert!(!is_null_or_empty(Some("a")));
        assert!(is_whitespace(' ' as i32));
        assert!(is_whitespace('\n' as i32));
        assert!(is_whitespace(0x2003));
        assert!(!is_whitespace('A' as i32));
        assert!(is_blank(None));
        assert!(is_blank(Some(" \t\n")));
        assert!(!is_blank(Some(" x ")));
    }
}
