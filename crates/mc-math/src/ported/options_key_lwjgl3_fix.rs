// JAVA: net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix
pub const KEY_UNKNOWN: &str = "key.unknown";

// JAVA: net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix#MAP
fn key_code_name(code: i32) -> &'static str {
    match code {
        0 => "key.unknown",
        11 => "key.0",
        2 => "key.1",
        3 => "key.2",
        4 => "key.3",
        5 => "key.4",
        6 => "key.5",
        7 => "key.6",
        8 => "key.7",
        9 => "key.8",
        10 => "key.9",
        30 => "key.a",
        40 => "key.apostrophe",
        48 => "key.b",
        43 => "key.backslash",
        14 => "key.backspace",
        46 => "key.c",
        58 => "key.caps.lock",
        51 => "key.comma",
        32 => "key.d",
        211 => "key.delete",
        208 => "key.down",
        18 => "key.e",
        207 => "key.end",
        28 => "key.enter",
        13 => "key.equal",
        1 => "key.escape",
        33 => "key.f",
        59 => "key.f1",
        68 => "key.f10",
        87 => "key.f11",
        88 => "key.f12",
        100 => "key.f13",
        101 => "key.f14",
        102 => "key.f15",
        103 => "key.f16",
        104 => "key.f17",
        105 => "key.f18",
        113 => "key.f19",
        60 => "key.f2",
        61 => "key.f3",
        62 => "key.f4",
        63 => "key.f5",
        64 => "key.f6",
        65 => "key.f7",
        66 => "key.f8",
        67 => "key.f9",
        34 => "key.g",
        41 => "key.grave.accent",
        35 => "key.h",
        199 => "key.home",
        23 => "key.i",
        210 => "key.insert",
        36 => "key.j",
        37 => "key.k",
        82 => "key.keypad.0",
        79 => "key.keypad.1",
        80 => "key.keypad.2",
        81 => "key.keypad.3",
        75 => "key.keypad.4",
        76 => "key.keypad.5",
        77 => "key.keypad.6",
        71 => "key.keypad.7",
        72 => "key.keypad.8",
        73 => "key.keypad.9",
        78 => "key.keypad.add",
        83 => "key.keypad.decimal",
        181 => "key.keypad.divide",
        156 => "key.keypad.enter",
        141 => "key.keypad.equal",
        55 => "key.keypad.multiply",
        74 => "key.keypad.subtract",
        38 => "key.l",
        203 => "key.left",
        56 => "key.left.alt",
        26 => "key.left.bracket",
        29 => "key.left.control",
        42 => "key.left.shift",
        219 => "key.left.win",
        50 => "key.m",
        12 => "key.minus",
        49 => "key.n",
        69 => "key.num.lock",
        24 => "key.o",
        25 => "key.p",
        209 => "key.page.down",
        201 => "key.page.up",
        197 => "key.pause",
        52 => "key.period",
        183 => "key.print.screen",
        16 => "key.q",
        19 => "key.r",
        205 => "key.right",
        184 => "key.right.alt",
        27 => "key.right.bracket",
        157 => "key.right.control",
        54 => "key.right.shift",
        220 => "key.right.win",
        31 => "key.s",
        70 => "key.scroll.lock",
        39 => "key.semicolon",
        53 => "key.slash",
        57 => "key.space",
        20 => "key.t",
        15 => "key.tab",
        22 => "key.u",
        200 => "key.up",
        47 => "key.v",
        17 => "key.w",
        45 => "key.x",
        21 => "key.y",
        44 => "key.z",
        _ => KEY_UNKNOWN,
    }
}

// JAVA: net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix#makeRule (value conversion)
pub fn key_code_to_name(code: i32) -> String {
    if code < 0 {
        let mouse = code.wrapping_add(100);
        if mouse == 0 {
            "key.mouse.left".to_string()
        } else if mouse == 1 {
            "key.mouse.right".to_string()
        } else if mouse == 2 {
            "key.mouse.middle".to_string()
        } else {
            format!("key.mouse.{}", mouse.wrapping_add(1))
        }
    } else {
        key_code_name(code).to_string()
    }
}

// JAVA: net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix#makeRule (entry conversion)
pub fn fix_key_value(key: &str, value: &str) -> String {
    if !key.starts_with("key_") {
        return value.to_string();
    }

    match value.parse::<i32>() {
        Ok(code) => key_code_to_name(code),
        Err(_) => value.to_string(),
    }
}

// JAVA: net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix#makeRule (map conversion)
pub fn fix_options_entries(entries: &[(String, String)]) -> Vec<(String, String)> {
    entries
        .iter()
        .map(|(key, value)| (key.clone(), fix_key_value(key, value)))
        .collect()
}

pub struct OptionsKeyLwjgl3Fix;

impl OptionsKeyLwjgl3Fix {
    // JAVA: net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix#makeRule
    pub fn key_code_to_name(code: i32) -> String {
        key_code_to_name(code)
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix#makeRule
    pub fn fix_key_value(key: &str, value: &str) -> String {
        fix_key_value(key, value)
    }

    // JAVA: net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix#makeRule
    pub fn fix_options_entries(entries: &[(String, String)]) -> Vec<(String, String)> {
        fix_options_entries(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_key_code() {
        assert_eq!(key_code_to_name(30), "key.a");
        assert_eq!(key_code_to_name(57), "key.space");
    }

    #[test]
    fn unknown_key_code() {
        assert_eq!(key_code_to_name(9999), KEY_UNKNOWN);
    }

    #[test]
    fn mouse_buttons() {
        assert_eq!(key_code_to_name(-100), "key.mouse.left");
        assert_eq!(key_code_to_name(-99), "key.mouse.right");
        assert_eq!(key_code_to_name(-98), "key.mouse.middle");
        assert_eq!(key_code_to_name(-97), "key.mouse.4");
    }

    #[test]
    fn non_key_entry_unchanged() {
        assert_eq!(fix_key_value("difficulty", "2"), "2");
    }

    #[test]
    fn key_entry_converted() {
        assert_eq!(fix_key_value("key_attack", "30"), "key.a");
        assert_eq!(fix_key_value("key_use", "-100"), "key.mouse.left");
    }

    #[test]
    fn invalid_value_unchanged() {
        assert_eq!(fix_key_value("key_attack", "not-a-number"), "not-a-number");
    }
}
