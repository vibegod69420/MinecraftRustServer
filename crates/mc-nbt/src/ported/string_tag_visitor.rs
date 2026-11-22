use crate::NbtTag;

/// Faithful port of `net.minecraft.nbt.StringTagVisitor`.
pub struct StringTagVisitor {
    builder: String,
}

impl StringTagVisitor {
    // JAVA: net.minecraft.nbt.StringTagVisitor#<init>
    pub fn new() -> Self {
        Self {
            builder: String::new(),
        }
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#build
    pub fn build(&self) -> String {
        self.builder.clone()
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitString
    pub fn visit_string(&mut self, value: &str) {
        let quoted = quote_and_escape(value);
        self.builder.push_str(&quoted);
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitByte
    pub fn visit_byte(&mut self, value: i8) {
        self.builder.push_str(&value.to_string());
        self.builder.push('b');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitShort
    pub fn visit_short(&mut self, value: i16) {
        self.builder.push_str(&value.to_string());
        self.builder.push('s');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitInt
    pub fn visit_int(&mut self, value: i32) {
        self.builder.push_str(&value.to_string());
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitLong
    pub fn visit_long(&mut self, value: i64) {
        self.builder.push_str(&value.to_string());
        self.builder.push('L');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitFloat
    pub fn visit_float(&mut self, value: f32) {
        self.builder.push_str(&java_f32_to_string(value));
        self.builder.push('f');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitDouble
    pub fn visit_double(&mut self, value: f64) {
        self.builder.push_str(&java_f64_to_string(value));
        self.builder.push('d');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitByteArray
    pub fn visit_byte_array(&mut self, value: &[i8]) {
        self.builder.push_str("[B;");
        for i in 0..value.len() {
            if i != 0 {
                self.builder.push(',');
            }
            self.builder.push_str(&value[i].to_string());
            self.builder.push('B');
        }
        self.builder.push(']');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitIntArray
    pub fn visit_int_array(&mut self, value: &[i32]) {
        self.builder.push_str("[I;");
        for i in 0..value.len() {
            if i != 0 {
                self.builder.push(',');
            }
            self.builder.push_str(&value[i].to_string());
        }
        self.builder.push(']');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitLongArray
    pub fn visit_long_array(&mut self, value: &[i64]) {
        self.builder.push_str("[L;");
        for i in 0..value.len() {
            if i != 0 {
                self.builder.push(',');
            }
            self.builder.push_str(&value[i].to_string());
            self.builder.push('L');
        }
        self.builder.push(']');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitList
    pub fn visit_list(&mut self, items: &[NbtTag]) {
        self.builder.push('[');
        for i in 0..items.len() {
            if i != 0 {
                self.builder.push(',');
            }
            self.visit_tag(&items[i]);
        }
        self.builder.push(']');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitCompound
    pub fn visit_compound(&mut self, entries: &[(String, NbtTag)]) {
        self.builder.push('{');
        let mut sorted: Vec<&(String, NbtTag)> = entries.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        for i in 0..sorted.len() {
            let (key, value) = sorted[i];
            if i != 0 {
                self.builder.push(',');
            }
            self.handle_key_escape(key);
            self.builder.push(':');
            self.visit_tag(value);
        }
        self.builder.push('}');
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#handleKeyEscape
    fn handle_key_escape(&mut self, key: &str) {
        if !key.eq_ignore_ascii_case("true")
            && !key.eq_ignore_ascii_case("false")
            && is_unquoted_key_match(key)
        {
            self.builder.push_str(key);
        } else {
            quote_and_escape_to(key, &mut self.builder);
        }
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitEnd
    pub fn visit_end(&mut self) {
        self.builder.push_str("END");
    }

    // JAVA: net.minecraft.nbt.StringTagVisitor#visitTag
    pub fn visit_tag(&mut self, tag: &NbtTag) {
        match tag {
            NbtTag::Byte(v) => self.visit_byte(*v),
            NbtTag::Short(v) => self.visit_short(*v),
            NbtTag::Int(v) => self.visit_int(*v),
            NbtTag::Long(v) => self.visit_long(*v),
            NbtTag::Float(v) => self.visit_float(*v),
            NbtTag::Double(v) => self.visit_double(*v),
            NbtTag::ByteArray(v) => self.visit_byte_array(v),
            NbtTag::String(v) => self.visit_string(v),
            NbtTag::List { items, .. } => self.visit_list(items),
            NbtTag::Compound(entries) => self.visit_compound(entries),
            NbtTag::IntArray(v) => self.visit_int_array(v),
            NbtTag::LongArray(v) => self.visit_long_array(v),
        }
    }
}

// JAVA: net.minecraft.nbt.StringTagVisitor#UNQUOTED_KEY_MATCH
fn is_unquoted_key_match(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '.' || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '+' || c == '-')
}

// JAVA: net.minecraft.nbt.StringTagVisitor#quoteAndEscape
pub fn quote_and_escape(value: &str) -> String {
    let mut output = String::new();
    quote_and_escape_to(value, &mut output);
    output
}

// JAVA: net.minecraft.nbt.StringTagVisitor#quoteAndEscapeTo
pub fn quote_and_escape_to(value: &str, output: &mut String) {
    let i = output.len();
    output.push(' ');
    let mut quote: char = '\0';
    for c in value.chars() {
        if c == '\\' {
            output.push('\\');
        } else if c == '"' || c == '\'' {
            if quote == '\0' {
                quote = if c == '"' { '\'' } else { '"' };
            }
            if quote == c {
                output.push('\\');
            }
        }
        output.push(c);
    }
    if quote == '\0' {
        quote = '"';
    }
    let quote_str = if quote == '"' { "\"" } else { "'" };
    output.replace_range(i..i + 1, quote_str);
    output.push(quote);
}

// JAVA: net.minecraft.nbt.StringTagVisitor#javaF32ToString
fn java_f32_to_string(v: f32) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        };
    }
    if v.to_bits() == 0x8000_0000 {
        return "-0.0".to_string();
    }
    if v == 0.0f32 {
        return "0.0".to_string();
    }
    let abs = v.abs();
    let s = format!("{}", v);
    if abs >= 1e-3f32 && abs < 1e7f32 {
        if s.contains('e') || s.contains('E') {
            scientific_to_decimal_java(&s)
        } else {
            ensure_decimal_point(&s)
        }
    } else if s.contains('e') || s.contains('E') {
        normalize_scientific_java(&s)
    } else {
        decimal_to_java_scientific(&s)
    }
}

// JAVA: net.minecraft.nbt.StringTagVisitor#javaF64ToString
fn java_f64_to_string(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        };
    }
    if v.to_bits() == 0x8000_0000_0000_0000 {
        return "-0.0".to_string();
    }
    if v == 0.0f64 {
        return "0.0".to_string();
    }
    let abs = v.abs();
    let s = format!("{}", v);
    if abs >= 1e-3f64 && abs < 1e7f64 {
        if s.contains('e') || s.contains('E') {
            scientific_to_decimal_java(&s)
        } else {
            ensure_decimal_point(&s)
        }
    } else if s.contains('e') || s.contains('E') {
        normalize_scientific_java(&s)
    } else {
        decimal_to_java_scientific(&s)
    }
}

// JAVA: net.minecraft.nbt.StringTagVisitor#ensureDecimalPoint
fn ensure_decimal_point(s: &str) -> String {
    if s.contains('.') {
        s.to_string()
    } else {
        format!("{}.0", s)
    }
}

// JAVA: net.minecraft.nbt.StringTagVisitor#normalizeScientificJava
fn normalize_scientific_java(s: &str) -> String {
    let (sign, rest) = if let Some(r) = s.strip_prefix('-') {
        ("-", r)
    } else {
        ("", s)
    };
    let Some(e_pos) = rest.find(|c| c == 'e' || c == 'E') else {
        return s.to_string();
    };
    let mantissa = &rest[..e_pos];
    let exp = &rest[e_pos + 1..];
    let exp = exp.strip_prefix('+').unwrap_or(exp);
    let mantissa = ensure_decimal_point(mantissa);
    format!("{}{}E{}", sign, mantissa, exp)
}

// JAVA: net.minecraft.nbt.StringTagVisitor#decimalToJavaScientific
fn decimal_to_java_scientific(s: &str) -> String {
    let (sign, rest) = if let Some(r) = s.strip_prefix('-') {
        ("-", r)
    } else {
        ("", s)
    };
    let dot_pos = rest.find('.').unwrap_or(rest.len());
    let digits: String = rest.chars().filter(|&c| c != '.').collect();
    let digits_trimmed = digits.trim_start_matches('0');
    if digits_trimmed.is_empty() {
        return format!("{}0.0E0", sign);
    }
    let first_sig = rest
        .find(|c: char| c != '0' && c != '.')
        .unwrap_or(0);
    let exp: i32 = if dot_pos > first_sig {
        (dot_pos as i32) - (first_sig as i32) - 1
    } else {
        -((first_sig as i32) - (dot_pos as i32))
    };
    let mut sig = digits_trimmed.trim_end_matches('0').to_string();
    if sig.is_empty() {
        sig.push('0');
    }
    let mantissa = if sig.len() == 1 {
        format!("{}.0", &sig[0..1])
    } else {
        format!("{}.{}", &sig[0..1], &sig[1..])
    };
    format!("{}{}E{}", sign, mantissa, exp)
}

// JAVA: net.minecraft.nbt.StringTagVisitor#scientificToDecimalJava
fn scientific_to_decimal_java(s: &str) -> String {
    let (sign, rest) = if let Some(r) = s.strip_prefix('-') {
        ("-", r)
    } else {
        ("", s)
    };
    let Some(e_pos) = rest.find(|c| c == 'e' || c == 'E') else {
        return ensure_decimal_point(s);
    };
    let mantissa_str = &rest[..e_pos];
    let exp: i32 = rest[e_pos + 1..].parse().unwrap_or(0);
    let digits: String = mantissa_str.chars().filter(|&c| c != '.').collect();
    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let mantissa_dot = mantissa_str.find('.').unwrap_or(mantissa_str.len());
    let new_dot = (mantissa_dot as i32).wrapping_add(exp);

    let abs_result = if new_dot <= 0 {
        let zeros = ((-new_dot) as usize).min(512);
        format!("0.{}{}", "0".repeat(zeros), digits)
    } else if (new_dot as usize) >= digits.len() {
        let trailing = (new_dot as usize).wrapping_sub(digits.len());
        format!("{}{}", digits, "0".repeat(trailing.min(512)))
    } else {
        let split = new_dot as usize;
        format!("{}.{}", &digits[..split], &digits[split..])
    };
    format!("{}{}", sign, ensure_decimal_point(&abs_result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NbtTag;

    fn visit(tag: &NbtTag) -> String {
        let mut v = StringTagVisitor::new();
        v.visit_tag(tag);
        v.build()
    }

    #[test]
    fn primitives_negative_and_overflow() {
        assert_eq!(visit(&NbtTag::Byte(i8::MIN)), "-128b");
        assert_eq!(visit(&NbtTag::Short(i16::MIN)), "-32768s");
        assert_eq!(visit(&NbtTag::Int(i32::MIN)), "-2147483648");
        assert_eq!(visit(&NbtTag::Long(i64::MIN)), "-9223372036854775808L");
        assert_eq!(visit(&NbtTag::Int(-1)), "-1");
        assert_eq!(visit(&NbtTag::Long(-2)), "-2L");
    }

    #[test]
    fn float_double_special_and_bits() {
        assert_eq!(visit(&NbtTag::Float(f32::NAN)), "NaNf");
        assert_eq!(visit(&NbtTag::Float(f32::INFINITY)), "Infinityf");
        assert_eq!(visit(&NbtTag::Float(f32::NEG_INFINITY)), "-Infinityf");
        assert_eq!(
            visit(&NbtTag::Float(f32::from_bits(0x8000_0000))),
            "-0.0f"
        );
        assert_eq!(visit(&NbtTag::Float(0.0f32)), "0.0f");
        // NaN with custom payload: Java Float.toString is "NaN"
        assert_eq!(
            visit(&NbtTag::Float(f32::from_bits(0x7FC0_0001))),
            "NaNf"
        );
        assert_eq!(visit(&NbtTag::Float(10_000_000.0f32)), "1.0E7f");

        assert_eq!(visit(&NbtTag::Double(f64::NAN)), "NaNd");
        assert_eq!(
            visit(&NbtTag::Double(f64::from_bits(0x8000_0000_0000_0000))),
            "-0.0d"
        );
        assert_eq!(visit(&NbtTag::Double(1.5f64)), "1.5d");
        assert_eq!(visit(&NbtTag::Double(0.0001f64)), "1.0E-4d");
    }

    #[test]
    fn arrays_and_empty() {
        assert_eq!(
            visit(&NbtTag::ByteArray(vec![1i8, -2, 127])),
            "[B;1B,-2B,127B]"
        );
        assert_eq!(visit(&NbtTag::ByteArray(vec![])), "[B;]");
        assert_eq!(visit(&NbtTag::IntArray(vec![])), "[I;]");
        assert_eq!(
            visit(&NbtTag::IntArray(vec![i32::MIN, 0, 1])),
            "[I;-2147483648,0,1]"
        );
        assert_eq!(visit(&NbtTag::LongArray(vec![-1i64, 2])), "[L;-1L,2L]");
    }

    #[test]
    fn compound_sorting_and_key_escape() {
        let tag = NbtTag::Compound(vec![
            ("b".to_string(), NbtTag::Int(2)),
            ("a".to_string(), NbtTag::Int(1)),
            ("1invalid".to_string(), NbtTag::Int(3)),
            ("true".to_string(), NbtTag::Int(4)),
            ("false".to_string(), NbtTag::Int(5)),
        ]);
        // Sorted by key: "1invalid", "a", "b", "false", "true"
        assert_eq!(
            visit(&tag),
            "{\"1invalid\":3,a:1,b:2,\"false\":5,\"true\":4}"
        );

        let spaced = NbtTag::Compound(vec![(
            "has space".to_string(),
            NbtTag::String("hello".to_string()),
        )]);
        assert_eq!(visit(&spaced), "{\"has space\":\"hello\"}");
    }

    #[test]
    fn list_nested_and_end() {
        let list = NbtTag::List {
            elem_type: 3,
            items: vec![NbtTag::Int(1), NbtTag::Int(-2), NbtTag::Int(3)],
        };
        assert_eq!(visit(&list), "[1,-2,3]");

        let nested = NbtTag::Compound(vec![(
            "k".to_string(),
            NbtTag::List {
                elem_type: 10,
                items: vec![NbtTag::Compound(vec![(
                    "x".to_string(),
                    NbtTag::Byte(1),
                )])],
            },
        )]);
        assert_eq!(visit(&nested), "{k:[{x:1b}]}");

        let mut v = StringTagVisitor::new();
        v.visit_end();
        assert_eq!(v.build(), "END");
    }
}
