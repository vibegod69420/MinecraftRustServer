use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::NbtTag;

static KEY_ORDER: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "{}".to_string(),
        vec![
            "DataVersion".to_string(),
            "author".to_string(),
            "size".to_string(),
            "data".to_string(),
            "entities".to_string(),
            "palette".to_string(),
            "palettes".to_string(),
        ],
    );
    m.insert(
        "{}.data.[].{}".to_string(),
        vec!["pos".to_string(), "state".to_string(), "nbt".to_string()],
    );
    m.insert(
        "{}.entities.[].{}".to_string(),
        vec!["blockPos".to_string(), "pos".to_string()],
    );
    m
});

static NO_INDENTATION: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut s = HashSet::new();
    s.insert("{}.size.[]".to_string());
    s.insert("{}.data.[].{}".to_string());
    s.insert("{}.palette.[].{}".to_string());
    s.insert("{}.entities.[].{}".to_string());
    s
});

// JAVA: net.minecraft.nbt.StringTag#quoteAndEscape
pub fn quote_and_escape(s: &str) -> String {
    let mut body = String::with_capacity(s.len());
    let mut quote: char = '\0';
    for c in s.chars() {
        if c == '\\' {
            body.push('\\');
        } else if c == '"' || c == '\'' {
            if quote == '\0' {
                quote = if c == '"' { '\'' } else { '"' };
            }
            if quote == c {
                body.push('\\');
            }
        }
        body.push(c);
    }
    if quote == '\0' {
        quote = '"';
    }
    let mut result = String::with_capacity(body.len() + 2);
    result.push(quote);
    result.push_str(&body);
    result.push(quote);
    result
}

fn simple_value_matches(s: &str) -> bool {
    !s.is_empty()
        && s.bytes().all(|b| {
            b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'+' || b == b'-'
        })
}

// JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#handleEscapePretty
fn handle_escape_pretty(s: &str) -> String {
    if simple_value_matches(s) {
        s.to_string()
    } else {
        quote_and_escape(s)
    }
}

fn compound_get<'a>(entries: &'a [(String, NbtTag)], key: &str) -> Option<&'a NbtTag> {
    entries.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn parse_decimal_repr(s: &str) -> (String, i32) {
    let (mantissa, e_exp) = if let Some(pos) = s.find('e').or_else(|| s.find('E')) {
        let e: i32 = s[pos + 1..].parse().unwrap_or(0);
        (&s[..pos], e)
    } else {
        (s, 0i32)
    };

    let (raw_digits, decimal_places) = if let Some(dot) = mantissa.find('.') {
        let int_p = &mantissa[..dot];
        let frac_p = &mantissa[dot + 1..];
        let mut d = String::with_capacity(int_p.len() + frac_p.len());
        d.push_str(int_p);
        d.push_str(frac_p);
        (d, frac_p.len() as i32)
    } else {
        (mantissa.to_string(), 0i32)
    };

    let trimmed = raw_digits.trim_start_matches('0');
    let mut digits = if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    };
    let mut exp10 = e_exp.wrapping_sub(decimal_places);
    if digits != "0" {
        let t_len = digits.trim_end_matches('0').len();
        let stripped = digits.len() - t_len;
        if stripped > 0 {
            digits.truncate(t_len);
            exp10 = exp10.wrapping_add(stripped as i32);
        }
    }
    (digits, exp10)
}

fn format_decimal_notation(digits: &str, exp10: i32) -> String {
    if digits == "0" {
        return "0.0".to_string();
    }
    let digit_count = digits.len() as i32;
    let decimal_pos = digit_count.wrapping_add(exp10);

    if decimal_pos <= 0 {
        let zeros = (-decimal_pos) as usize;
        let mut s = String::with_capacity(2 + zeros + digits.len());
        s.push_str("0.");
        for _ in 0..zeros {
            s.push('0');
        }
        s.push_str(digits);
        s
    } else if decimal_pos >= digit_count {
        let trailing_zeros = (decimal_pos - digit_count) as usize;
        let mut s = String::with_capacity(digits.len() + trailing_zeros + 2);
        s.push_str(digits);
        for _ in 0..trailing_zeros {
            s.push('0');
        }
        s.push_str(".0");
        s
    } else {
        let pos = decimal_pos as usize;
        let mut s = String::with_capacity(digits.len() + 1);
        s.push_str(&digits[..pos]);
        s.push('.');
        s.push_str(&digits[pos..]);
        s
    }
}

fn format_scientific_notation(digits: &str, sci_exp: i32) -> String {
    if digits == "0" {
        return "0.0E0".to_string();
    }
    let first = &digits[0..1];
    let rest = &digits[1..];
    let mut s = String::new();
    s.push_str(first);
    s.push('.');
    if rest.is_empty() {
        s.push('0');
    } else {
        s.push_str(rest);
    }
    s.push('E');
    s.push_str(&sci_exp.to_string());
    s
}

fn java_float_to_string(v: f32) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    if v.to_bits() == 0x8000_0000u32 {
        return "-0.0".to_string();
    }
    if v == 0.0f32 {
        return "0.0".to_string();
    }
    let negative = v.is_sign_negative();
    let abs = v.abs();
    let rust_str = format!("{}", abs);
    let (digits, exp10) = parse_decimal_repr(&rust_str);
    let sci_exp = exp10.wrapping_add(digits.len() as i32).wrapping_sub(1);
    let sign = if negative { "-" } else { "" };
    if abs >= 1e-3f32 && abs < 1e7f32 {
        format!("{}{}", sign, format_decimal_notation(&digits, exp10))
    } else {
        format!("{}{}", sign, format_scientific_notation(&digits, sci_exp))
    }
}

fn java_double_to_string(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    if v.to_bits() == 0x8000_0000_0000_0000u64 {
        return "-0.0".to_string();
    }
    if v == 0.0f64 {
        return "0.0".to_string();
    }
    let negative = v.is_sign_negative();
    let abs = v.abs();
    let rust_str = format!("{}", abs);
    let (digits, exp10) = parse_decimal_repr(&rust_str);
    let sci_exp = exp10.wrapping_add(digits.len() as i32).wrapping_sub(1);
    let sign = if negative { "-" } else { "" };
    if abs >= 1e-3f64 && abs < 1e7f64 {
        format!("{}{}", sign, format_decimal_notation(&digits, exp10))
    } else {
        format!("{}{}", sign, format_scientific_notation(&digits, sci_exp))
    }
}

pub struct SnbtPrinterTagVisitor {
    indentation: String,
    depth: i32,
    path: Vec<String>,
    result: String,
}

impl SnbtPrinterTagVisitor {
    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#<init>
    pub fn new() -> Self {
        Self::with_params("    ".to_string(), 0, Vec::new())
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#<init>
    pub fn with_params(indentation: String, depth: i32, path: Vec<String>) -> Self {
        Self {
            indentation,
            depth,
            path,
            result: String::new(),
        }
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visit
    pub fn visit(&mut self, tag: &NbtTag) -> String {
        self.visit_tag(tag);
        self.result.clone()
    }

    fn visit_tag(&mut self, tag: &NbtTag) {
        match tag {
            NbtTag::String(v) => self.visit_string(v),
            NbtTag::Byte(v) => self.visit_byte(*v),
            NbtTag::Short(v) => self.visit_short(*v),
            NbtTag::Int(v) => self.visit_int(*v),
            NbtTag::Long(v) => self.visit_long(*v),
            NbtTag::Float(v) => self.visit_float(*v),
            NbtTag::Double(v) => self.visit_double(*v),
            NbtTag::ByteArray(v) => self.visit_byte_array(v),
            NbtTag::IntArray(v) => self.visit_int_array(v),
            NbtTag::LongArray(v) => self.visit_long_array(v),
            NbtTag::List { items, .. } => self.visit_list(items),
            NbtTag::Compound(entries) => self.visit_compound(entries),
        }
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitString
    pub fn visit_string(&mut self, value: &str) {
        self.result = quote_and_escape(value);
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitByte
    pub fn visit_byte(&mut self, value: i8) {
        self.result = format!("{}b", value);
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitShort
    pub fn visit_short(&mut self, value: i16) {
        self.result = format!("{}s", value);
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitInt
    pub fn visit_int(&mut self, value: i32) {
        self.result = value.to_string();
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitLong
    pub fn visit_long(&mut self, value: i64) {
        self.result = format!("{}L", value);
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitFloat
    pub fn visit_float(&mut self, value: f32) {
        self.result = format!("{}f", java_float_to_string(value));
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitDouble
    pub fn visit_double(&mut self, value: f64) {
        self.result = format!("{}d", java_double_to_string(value));
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitByteArray
    pub fn visit_byte_array(&mut self, value: &[i8]) {
        let mut builder = String::from("[B;");
        for (i, &b) in value.iter().enumerate() {
            builder.push_str(&format!(" {}B", b));
            if i + 1 != value.len() {
                builder.push(',');
            }
        }
        builder.push(']');
        self.result = builder;
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitIntArray
    pub fn visit_int_array(&mut self, value: &[i32]) {
        let mut builder = String::from("[I;");
        for (i, &v) in value.iter().enumerate() {
            builder.push_str(&format!(" {}", v));
            if i + 1 != value.len() {
                builder.push(',');
            }
        }
        builder.push(']');
        self.result = builder;
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitLongArray
    pub fn visit_long_array(&mut self, value: &[i64]) {
        let mut builder = String::from("[L;");
        for (i, &v) in value.iter().enumerate() {
            builder.push_str(&format!(" {}L", v));
            if i + 1 != value.len() {
                builder.push(',');
            }
        }
        builder.push(']');
        self.result = builder;
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitList
    pub fn visit_list(&mut self, items: &[NbtTag]) {
        if items.is_empty() {
            self.result = "[]".to_string();
            return;
        }
        let mut builder = String::from("[");
        self.push_path("[]".to_string());
        let no_indent = NO_INDENTATION.contains(&self.path_string());
        let indent_str: String = if no_indent {
            String::new()
        } else {
            self.indentation.clone()
        };
        if !indent_str.is_empty() {
            builder.push('\n');
        }

        for i in 0..items.len() {
            for _ in 0..(self.depth.wrapping_add(1)) {
                builder.push_str(&indent_str);
            }
            let child_result = {
                let mut child = SnbtPrinterTagVisitor::with_params(
                    indent_str.clone(),
                    self.depth.wrapping_add(1),
                    std::mem::take(&mut self.path),
                );
                let r = child.visit(&items[i]);
                self.path = child.path;
                r
            };
            builder.push_str(&child_result);
            if i.wrapping_add(1) != items.len() {
                builder.push(',');
                if indent_str.is_empty() {
                    builder.push(' ');
                } else {
                    builder.push('\n');
                }
            }
        }

        if !indent_str.is_empty() {
            builder.push('\n');
            for _ in 0..self.depth {
                builder.push_str(&indent_str);
            }
        }

        builder.push(']');
        self.result = builder;
        self.pop_path();
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitCompound
    pub fn visit_compound(&mut self, entries: &[(String, NbtTag)]) {
        if entries.is_empty() {
            self.result = "{}".to_string();
            return;
        }
        let mut builder = String::from("{");
        self.push_path("{}".to_string());
        let no_indent = NO_INDENTATION.contains(&self.path_string());
        let indent_str: String = if no_indent {
            String::new()
        } else {
            self.indentation.clone()
        };
        if !indent_str.is_empty() {
            builder.push('\n');
        }

        let keys = self.get_keys(entries);
        let mut it = keys.iter().peekable();
        while let Some(key) = it.next() {
            let Some(tag) = compound_get(entries, key) else {
                continue;
            };
            self.push_path(key.clone());
            for _ in 0..(self.depth.wrapping_add(1)) {
                builder.push_str(&indent_str);
            }
            builder.push_str(&handle_escape_pretty(key));
            builder.push(':');
            builder.push(' ');
            let child_result = {
                let mut child = SnbtPrinterTagVisitor::with_params(
                    indent_str.clone(),
                    self.depth.wrapping_add(1),
                    std::mem::take(&mut self.path),
                );
                let r = child.visit(tag);
                self.path = child.path;
                r
            };
            builder.push_str(&child_result);
            self.pop_path();
            if it.peek().is_some() {
                builder.push(',');
                if indent_str.is_empty() {
                    builder.push(' ');
                } else {
                    builder.push('\n');
                }
            }
        }

        if !indent_str.is_empty() {
            builder.push('\n');
            for _ in 0..self.depth {
                builder.push_str(&indent_str);
            }
        }

        builder.push('}');
        self.result = builder;
        self.pop_path();
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#popPath
    fn pop_path(&mut self) {
        self.path.pop();
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#pushPath
    fn push_path(&mut self, path: String) {
        self.path.push(path);
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#getKeys
    fn get_keys(&self, entries: &[(String, NbtTag)]) -> Vec<String> {
        let mut key_set: HashSet<String> = HashSet::new();
        for (k, _) in entries {
            key_set.insert(k.clone());
        }
        let mut result: Vec<String> = Vec::new();
        let path = self.path_string();
        if let Some(order) = KEY_ORDER.get(&path) {
            for k in order {
                if key_set.remove(k) {
                    result.push(k.clone());
                }
            }
            if !key_set.is_empty() {
                let mut remaining: Vec<String> = key_set.into_iter().collect();
                remaining.sort();
                result.extend(remaining);
            }
        } else {
            result.extend(key_set);
            result.sort();
        }
        result
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#pathString
    pub fn path_string(&self) -> String {
        self.path.join(".")
    }

    // JAVA: net.minecraft.nbt.SnbtPrinterTagVisitor#visitEnd
    pub fn visit_end(&mut self) {}
}

impl Default for SnbtPrinterTagVisitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NbtTag;

    #[test]
    fn test_primitives_and_negatives() {
        let mut v = SnbtPrinterTagVisitor::new();
        assert_eq!(v.visit(&NbtTag::Byte(-1)), "-1b");
        assert_eq!(v.visit(&NbtTag::Short(i16::MIN)), "-32768s");
        assert_eq!(v.visit(&NbtTag::Int(-42)), "-42");
        assert_eq!(
            v.visit(&NbtTag::Long(i64::MIN)),
            "-9223372036854775808L"
        );
        assert_eq!(v.visit(&NbtTag::String("hello".to_string())), "\"hello\"");
    }

    #[test]
    fn test_float_double_special_and_format() {
        let mut v = SnbtPrinterTagVisitor::new();
        assert_eq!(v.visit(&NbtTag::Float(0.0)), "0.0f");
        assert_eq!(v.visit(&NbtTag::Float(-0.0)), "-0.0f");
        assert_eq!(v.visit(&NbtTag::Float(f32::NAN)), "NaNf");
        assert_eq!(v.visit(&NbtTag::Float(f32::INFINITY)), "Infinityf");
        assert_eq!(v.visit(&NbtTag::Float(f32::NEG_INFINITY)), "-Infinityf");
        assert_eq!(v.visit(&NbtTag::Double(1.0e7)), "1.0E7d");
        assert_eq!(v.visit(&NbtTag::Double(1.0e-4)), "1.0E-4d");
        assert_eq!(v.visit(&NbtTag::Double(-0.0)), "-0.0d");
    }

    #[test]
    fn test_arrays() {
        let mut v = SnbtPrinterTagVisitor::new();
        assert_eq!(
            v.visit(&NbtTag::ByteArray(vec![1i8, -2, 3])),
            "[B; 1B, -2B, 3B]"
        );
        assert_eq!(v.visit(&NbtTag::IntArray(vec![])), "[I;]");
        assert_eq!(v.visit(&NbtTag::IntArray(vec![10, 20])), "[I; 10, 20]");
        assert_eq!(
            v.visit(&NbtTag::LongArray(vec![i64::MIN, 0])),
            "[L; -9223372036854775808L, 0L]"
        );
    }

    #[test]
    fn test_compound_ordering_and_escaping() {
        let mut v = SnbtPrinterTagVisitor::new();
        let tag = NbtTag::Compound(vec![
            ("b".to_string(), NbtTag::Int(2)),
            ("a key".to_string(), NbtTag::Int(3)),
            ("a".to_string(), NbtTag::Int(1)),
        ]);
        let expected = "{\n    a: 1,\n    \"a key\": 3,\n    b: 2\n}";
        assert_eq!(v.visit(&tag), expected);
    }

    #[test]
    fn test_list_and_empty() {
        let mut v = SnbtPrinterTagVisitor::new();
        let list = NbtTag::List {
            elem_type: 3,
            items: vec![NbtTag::Int(1), NbtTag::Int(2)],
        };
        assert_eq!(v.visit(&list), "[\n    1,\n    2\n]");
        assert_eq!(
            v.visit(&NbtTag::List {
                elem_type: 0,
                items: vec![],
            }),
            "[]"
        );
        assert_eq!(v.visit(&NbtTag::Compound(vec![])), "{}");
    }

    #[test]
    fn test_key_order_no_indentation_and_quote_escape() {
        let mut v = SnbtPrinterTagVisitor::new();
        let tag = NbtTag::Compound(vec![
            (
                "data".to_string(),
                NbtTag::List {
                    elem_type: 10,
                    items: vec![NbtTag::Compound(vec![
                        ("state".to_string(), NbtTag::Int(2)),
                        ("pos".to_string(), NbtTag::Int(1)),
                    ])],
                },
            ),
            ("DataVersion".to_string(), NbtTag::Int(99)),
        ]);
        let out = v.visit(&tag);
        let expected =
            "{\n    DataVersion: 99,\n    data: [\n        {pos: 1, state: 2}\n    ]\n}";
        assert_eq!(out, expected);

        assert_eq!(quote_and_escape("hello"), "\"hello\"");
        assert_eq!(quote_and_escape("say \"hi\""), "'say \"hi\"'");
        assert_eq!(quote_and_escape("it's"), "\"it's\"");
    }
}
