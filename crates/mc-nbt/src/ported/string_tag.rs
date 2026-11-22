use std::fmt;
use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::NbtError;

const SELF_SIZE_IN_BYTES: i32 = 36;
const DOUBLE_QUOTE: char = '"';
const SINGLE_QUOTE: char = '\'';
const ESCAPE: char = '\\';
const NOT_SET: char = '\u{0000}';

/// Minimal NBT size accounter for faithful tag loading/parsing.
#[derive(Debug, Clone)]
pub struct NbtAccounter {
    remaining: u64,
}

impl NbtAccounter {
    pub fn new(quota: u64) -> Self {
        Self { remaining: quota }
    }

    pub fn unlimited_heap() -> Self {
        Self {
            remaining: u64::MAX / 4,
        }
    }

    pub fn account_bytes(&mut self, bytes: u64) -> Result<(), NbtError> {
        if bytes > self.remaining {
            return Err(NbtError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "NBT tag too big",
            )));
        }
        self.remaining = self.remaining.wrapping_sub(bytes);
        Ok(())
    }

    pub fn account_bytes_mul(&mut self, multiplier: u64, count: u64) -> Result<(), NbtError> {
        let bytes = multiplier.checked_mul(count).ok_or_else(|| {
            NbtError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "NBT size overflow",
            ))
        })?;
        self.account_bytes(bytes)
    }
}

/// Minimal tag visitor for `StringTag::accept`.
pub trait TagVisitor {
    fn visit_string(&mut self, tag: &StringTag);
}

/// Minimal stream tag visitor for `StringTag` TYPE parse/accept.
pub trait StreamTagVisitor {
    type ValueResult;
    fn visit_string_value(&mut self, value: &str) -> Self::ValueResult;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StringTagType;

pub static TYPE: StringTagType = StringTagType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringTag {
    value: String,
}

impl StringTag {
    pub fn value(&self) -> &str {
        &self.value
    }

    // JAVA: net.minecraft.nbt.StringTag#valueOf
    pub fn value_of(s: String) -> Self {
        if s.is_empty() {
            Self {
                value: String::new(),
            }
        } else {
            Self { value: s }
        }
    }

    // JAVA: net.minecraft.nbt.StringTag#skipString
    pub fn skip_string<R: Read>(input: &mut R) -> Result<(), NbtError> {
        skip_string(input)
    }

    // JAVA: net.minecraft.nbt.StringTag#write
    pub fn write<W: Write>(&self, output: &mut W) -> Result<(), NbtError> {
        write_modified_utf8(output, &self.value)
    }

    // JAVA: net.minecraft.nbt.StringTag#sizeInBytes
    pub fn size_in_bytes(&self) -> i32 {
        let utf16_len = self.value.encode_utf16().count() as i32;
        SELF_SIZE_IN_BYTES.wrapping_add(2i32.wrapping_mul(utf16_len))
    }

    // JAVA: net.minecraft.nbt.StringTag#getId
    pub fn get_id(&self) -> u8 {
        8
    }

    // JAVA: net.minecraft.nbt.StringTag#getType
    pub fn get_type(&self) -> StringTagType {
        TYPE
    }

    // JAVA: net.minecraft.nbt.StringTag#copy
    pub fn copy(&self) -> Self {
        self.clone()
    }

    // JAVA: net.minecraft.nbt.StringTag#asString
    pub fn as_string(&self) -> Option<String> {
        Some(self.value.clone())
    }

    // JAVA: net.minecraft.nbt.StringTag#accept
    pub fn accept<V: TagVisitor>(&self, visitor: &mut V) {
        visitor.visit_string(self);
    }

    // JAVA: net.minecraft.nbt.StringTag#accept
    pub fn accept_stream<V: StreamTagVisitor>(&self, visitor: &mut V) -> V::ValueResult {
        visitor.visit_string_value(&self.value)
    }
}

impl fmt::Display for StringTag {
    // JAVA: net.minecraft.nbt.StringTag#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        quote_and_escape_to(&self.value, &mut s);
        f.write_str(&s)
    }
}

impl StringTagType {
    // JAVA: net.minecraft.nbt.StringTag#load
    pub fn load<R: Read>(
        &self,
        input: &mut R,
        accounter: &mut NbtAccounter,
    ) -> Result<StringTag, NbtError> {
        Ok(StringTag::value_of(read_accounted(input, accounter)?))
    }

    // JAVA: net.minecraft.nbt.StringTag#parse
    pub fn parse<R: Read, V: StreamTagVisitor>(
        &self,
        input: &mut R,
        visitor: &mut V,
        accounter: &mut NbtAccounter,
    ) -> Result<V::ValueResult, NbtError> {
        let value = read_accounted(input, accounter)?;
        Ok(visitor.visit_string_value(&value))
    }

    // JAVA: net.minecraft.nbt.StringTag#skip
    pub fn skip<R: Read>(
        &self,
        input: &mut R,
        _accounter: &mut NbtAccounter,
    ) -> Result<(), NbtError> {
        skip_string(input)
    }

    // JAVA: net.minecraft.nbt.StringTag#getName
    pub fn get_name(&self) -> &'static str {
        "STRING"
    }

    // JAVA: net.minecraft.nbt.StringTag#getPrettyName
    pub fn get_pretty_name(&self) -> &'static str {
        "TAG_String"
    }
}

// JAVA: net.minecraft.nbt.StringTag#readAccounted
fn read_accounted<R: Read>(input: &mut R, accounter: &mut NbtAccounter) -> Result<String, NbtError> {
    accounter.account_bytes(36u64)?;
    let s = read_modified_utf8(input)?;
    let utf16_len = s.encode_utf16().count() as u64;
    accounter.account_bytes_mul(2u64, utf16_len)?;
    Ok(s)
}

// JAVA: net.minecraft.nbt.StringTag#skipString
pub fn skip_string<R: Read>(input: &mut R) -> Result<(), NbtError> {
    let len = input.read_u16::<BigEndian>()? as usize;
    let mut buf = vec![0u8; len];
    input.read_exact(&mut buf)?;
    Ok(())
}

// JAVA: net.minecraft.nbt.StringTag#quoteAndEscape
pub fn quote_and_escape(s: &str) -> String {
    let mut builder = String::new();
    quote_and_escape_to(s, &mut builder);
    builder
}

// JAVA: net.minecraft.nbt.StringTag#quoteAndEscape
pub fn quote_and_escape_to(s: &str, builder: &mut String) {
    let start = builder.len();
    builder.push(' ');
    let mut quote: char = NOT_SET;

    for c in s.chars() {
        if c == ESCAPE {
            builder.push_str("\\\\");
        } else if c != DOUBLE_QUOTE && c != SINGLE_QUOTE {
            if let Some(esc) = snbt_grammar_escape_control_characters(c) {
                builder.push(ESCAPE);
                builder.push_str(&esc);
            } else {
                builder.push(c);
            }
        } else {
            if quote == NOT_SET {
                quote = if c == DOUBLE_QUOTE {
                    SINGLE_QUOTE
                } else {
                    DOUBLE_QUOTE
                };
            }

            if quote == c {
                builder.push(ESCAPE);
            }

            builder.push(c);
        }
    }

    if quote == NOT_SET {
        quote = DOUBLE_QUOTE;
    }

    let mut bytes = std::mem::take(builder).into_bytes();
    if start < bytes.len() {
        bytes[start] = quote as u8;
    }
    *builder = String::from_utf8(bytes).expect("valid UTF-8 after ASCII quote replacement");
    builder.push(quote);
}

// JAVA: net.minecraft.nbt.StringTag#escapeWithoutQuotes
pub fn escape_without_quotes(s: &str) -> String {
    let mut builder = String::new();
    escape_without_quotes_to(s, &mut builder);
    builder
}

// JAVA: net.minecraft.nbt.StringTag#escapeWithoutQuotes
pub fn escape_without_quotes_to(s: &str, builder: &mut String) {
    for c in s.chars() {
        match c {
            DOUBLE_QUOTE | SINGLE_QUOTE | ESCAPE => {
                builder.push(ESCAPE);
                builder.push(c);
            }
            _ => {
                if let Some(esc) = snbt_grammar_escape_control_characters(c) {
                    builder.push(ESCAPE);
                    builder.push_str(&esc);
                } else {
                    builder.push(c);
                }
            }
        }
    }
}

// JAVA: net.minecraft.nbt.SnbtGrammar#escapeControlCharacters
fn snbt_grammar_escape_control_characters(c: char) -> Option<String> {
    match c {
        '\x08' => Some("b".to_owned()),
        '\t' => Some("t".to_owned()),
        '\n' => Some("n".to_owned()),
        '\x0C' => Some("f".to_owned()),
        '\r' => Some("r".to_owned()),
        _ if (c as u32) < 0x20 => Some(format!("u{:04X}", c as u32)),
        _ => None,
    }
}

fn modified_utf8_len(s: &str) -> usize {
    let mut utflen: usize = 0;
    for c in s.chars() {
        let cp = c as u32;
        if cp == 0 {
            utflen = utflen.wrapping_add(2);
        } else if cp <= 0x7F {
            utflen = utflen.wrapping_add(1);
        } else if cp <= 0x7FF {
            utflen = utflen.wrapping_add(2);
        } else if cp <= 0xFFFF {
            utflen = utflen.wrapping_add(3);
        } else {
            utflen = utflen.wrapping_add(6);
        }
    }
    utflen
}

fn write_modified_utf8<W: Write>(output: &mut W, s: &str) -> Result<(), NbtError> {
    let utflen = modified_utf8_len(s);
    if utflen > u16::MAX as usize {
        return Err(NbtError::StringTooLong(utflen));
    }
    output.write_u16::<BigEndian>(utflen as u16)?;
    for c in s.chars() {
        let cp = c as u32;
        if cp >= 0x0001 && cp <= 0x007F {
            output.write_u8(cp as u8)?;
        } else if cp == 0 || cp <= 0x07FF {
            output.write_u8((0xC0u32 | ((cp >> 6) & 0x1F)) as u8)?;
            output.write_u8((0x80u32 | (cp & 0x3F)) as u8)?;
        } else if cp <= 0xFFFF {
            output.write_u8((0xE0u32 | ((cp >> 12) & 0x0F)) as u8)?;
            output.write_u8((0x80u32 | ((cp >> 6) & 0x3F)) as u8)?;
            output.write_u8((0x80u32 | (cp & 0x3F)) as u8)?;
        } else {
            let u = cp.wrapping_sub(0x10000);
            let hi = 0xD800u32.wrapping_add((u >> 10) & 0x3FF);
            let lo = 0xDC00u32.wrapping_add(u & 0x3FF);
            for surr in [hi, lo] {
                output.write_u8((0xE0u32 | ((surr >> 12) & 0x0F)) as u8)?;
                output.write_u8((0x80u32 | ((surr >> 6) & 0x3F)) as u8)?;
                output.write_u8((0x80u32 | (surr & 0x3F)) as u8)?;
            }
        }
    }
    Ok(())
}

fn read_modified_utf8<R: Read>(input: &mut R) -> Result<String, NbtError> {
    let utflen = input.read_u16::<BigEndian>()? as usize;
    let mut bytearr = vec![0u8; utflen];
    input.read_exact(&mut bytearr)?;
    decode_modified_utf8(&bytearr)
}

fn decode_modified_utf8(bytearr: &[u8]) -> Result<String, NbtError> {
    let mut chars: Vec<u16> = Vec::with_capacity(bytearr.len());
    let mut i: usize = 0;
    let utflen = bytearr.len();

    while i < utflen {
        let c = bytearr[i] as u32;
        if c > 127 {
            break;
        }
        i = i.wrapping_add(1);
        chars.push(c as u16);
    }

    while i < utflen {
        let c = bytearr[i] as u32;
        match c >> 4 {
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => {
                i = i.wrapping_add(1);
                chars.push(c as u16);
            }
            12 | 13 => {
                i = i.wrapping_add(2);
                if i > utflen {
                    return Err(NbtError::InvalidMutf8);
                }
                let c2 = bytearr[i.wrapping_sub(1)] as u32;
                if (c2 & 0xC0) != 0x80 {
                    return Err(NbtError::InvalidMutf8);
                }
                chars.push((((c & 0x1F) << 6) | (c2 & 0x3F)) as u16);
            }
            14 => {
                i = i.wrapping_add(3);
                if i > utflen {
                    return Err(NbtError::InvalidMutf8);
                }
                let c2 = bytearr[i.wrapping_sub(2)] as u32;
                let c3 = bytearr[i.wrapping_sub(1)] as u32;
                if (c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80 {
                    return Err(NbtError::InvalidMutf8);
                }
                chars.push((((c & 0x0F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F)) as u16);
            }
            _ => return Err(NbtError::InvalidMutf8),
        }
    }

    String::from_utf16(&chars).map_err(|_| NbtError::InvalidMutf8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct TestStreamVisitor;
    impl StreamTagVisitor for TestStreamVisitor {
        type ValueResult = String;
        fn visit_string_value(&mut self, value: &str) -> Self::ValueResult {
            value.to_owned()
        }
    }

    struct TestTagVisitor {
        seen: Option<String>,
    }
    impl TagVisitor for TestTagVisitor {
        fn visit_string(&mut self, tag: &StringTag) {
            self.seen = Some(tag.value().to_owned());
        }
    }

    #[test]
    fn value_of_and_basic_properties() {
        let empty = StringTag::value_of(String::new());
        let tag = StringTag::value_of("abc".to_owned());
        assert_eq!(empty.value(), "");
        assert_eq!(tag.get_id(), 8);
        assert_eq!(tag.as_string(), Some("abc".to_owned()));
        assert_eq!(tag.copy().value(), "abc");
        assert_eq!(empty.size_in_bytes(), 36);
        assert_eq!(tag.size_in_bytes(), 36i32.wrapping_add(2i32.wrapping_mul(3)));

        let mut visitor = TestTagVisitor { seen: None };
        tag.accept(&mut visitor);
        assert_eq!(visitor.seen.as_deref(), Some("abc"));
    }

    #[test]
    fn write_read_roundtrip_modified_utf8() {
        let original = StringTag::value_of("hello\0world\u{0800}\u{1F600}".to_owned());
        let mut buf = Vec::new();
        original.write(&mut buf).unwrap();

        let mut cursor = Cursor::new(&buf);
        let mut acc = NbtAccounter::unlimited_heap();
        let loaded = TYPE.load(&mut cursor, &mut acc).unwrap();
        assert_eq!(loaded.value(), original.value());
        assert_eq!(cursor.position() as usize, buf.len());

        let mut cursor2 = Cursor::new(&buf);
        let mut acc2 = NbtAccounter::unlimited_heap();
        let mut sv = TestStreamVisitor;
        let parsed = TYPE.parse(&mut cursor2, &mut sv, &mut acc2).unwrap();
        assert_eq!(parsed, original.value());
    }

    #[test]
    fn skip_and_accounter_limits() {
        let tag = StringTag::value_of("test".to_owned());
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();

        let mut cursor = Cursor::new(&buf);
        let mut acc = NbtAccounter::unlimited_heap();
        TYPE.skip(&mut cursor, &mut acc).unwrap();
        assert_eq!(cursor.position() as usize, buf.len());

        let mut cursor2 = Cursor::new(&buf);
        let mut small_acc = NbtAccounter::new(10);
        let err = TYPE.load(&mut cursor2, &mut small_acc);
        assert!(err.is_err());
    }

    #[test]
    fn quote_and_escape_cases() {
        assert_eq!(quote_and_escape("hello"), "\"hello\"");
        assert_eq!(quote_and_escape("he\"llo"), "'he\"llo'");
        assert_eq!(quote_and_escape("it's"), "\"it's\"");
        assert_eq!(quote_and_escape("a\"b'c"), "'a\"b\\'c'");
        assert_eq!(quote_and_escape("line\nbreak"), "\"line\\nbreak\"");
        assert_eq!(StringTag::value_of("x".to_owned()).to_string(), "\"x\"");
    }

    #[test]
    fn escape_without_quotes_and_size_wrapping() {
        assert_eq!(escape_without_quotes("a\"b'c\\d"), "a\\\"b\\'c\\\\d");
        assert_eq!(escape_without_quotes("a\n\tb"), "a\\n\\tb");

        // Java int wrapping semantics for sizeInBytes: 36 + 2 * len
        let big_len: i32 = i32::MAX;
        let wrapped = 36i32.wrapping_add(2i32.wrapping_mul(big_len));
        assert_eq!(wrapped, 34i32);

        let negative_len: i32 = -1;
        let wrapped_neg = 36i32.wrapping_add(2i32.wrapping_mul(negative_len));
        assert_eq!(wrapped_neg, 34i32);
    }
}
