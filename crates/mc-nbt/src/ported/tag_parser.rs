use std::fmt;
use std::sync::LazyLock;

use crate::NbtTag;

/// Minimal component stub for self-contained command error messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub key: String,
}

impl Component {
    // JAVA: net.minecraft.network.chat.Component#translatable
    pub fn translatable(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }
}

/// Minimal command syntax exception stub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
    pub input: String,
    pub cursor: usize,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at cursor {} in '{}'",
            self.message, self.cursor, self.input
        )
    }
}

impl std::error::Error for CommandSyntaxException {}

/// Minimal string reader stub (char-cursor based).
#[derive(Debug, Clone)]
pub struct StringReader {
    chars: Vec<char>,
    cursor: usize,
    original: String,
}

impl StringReader {
    // JAVA: com.mojang.brigadier.StringReader#<init>
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            cursor: 0,
            original: input.to_string(),
        }
    }

    // JAVA: com.mojang.brigadier.StringReader#getString
    pub fn get_string(&self) -> &str {
        &self.original
    }

    // JAVA: com.mojang.brigadier.StringReader#canRead
    pub fn can_read(&self) -> bool {
        self.cursor < self.chars.len()
    }

    // JAVA: com.mojang.brigadier.StringReader#canRead
    pub fn can_read_n(&self, n: usize) -> bool {
        self.cursor.wrapping_add(n) <= self.chars.len()
    }

    // JAVA: com.mojang.brigadier.StringReader#peek
    pub fn peek(&self) -> char {
        self.chars[self.cursor]
    }

    // JAVA: com.mojang.brigadier.StringReader#read
    pub fn read(&mut self) -> char {
        let ch = self.chars[self.cursor];
        self.cursor = self.cursor.wrapping_add(1);
        ch
    }

    // JAVA: com.mojang.brigadier.StringReader#skipWhitespace
    pub fn skip_whitespace(&mut self) {
        while self.can_read() && self.peek().is_whitespace() {
            self.cursor = self.cursor.wrapping_add(1);
        }
    }

    // JAVA: com.mojang.brigadier.StringReader#getCursor
    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    // JAVA: com.mojang.brigadier.StringReader#getRemaining
    pub fn get_remaining(&self) -> String {
        self.chars[self.cursor..].iter().collect()
    }
}

/// Minimal simple command exception type stub.
#[derive(Debug, Clone)]
pub struct SimpleCommandExceptionType {
    pub message: Component,
}

impl SimpleCommandExceptionType {
    // JAVA: com.mojang.brigadier.exceptions.SimpleCommandExceptionType#<init>
    pub fn new(message: Component) -> Self {
        Self { message }
    }

    // JAVA: com.mojang.brigadier.exceptions.SimpleCommandExceptionType#createWithContext
    pub fn create_with_context(&self, reader: &StringReader) -> CommandSyntaxException {
        CommandSyntaxException {
            message: self.message.key.clone(),
            input: reader.get_string().to_string(),
            cursor: reader.get_cursor(),
        }
    }
}

// JAVA: net.minecraft.nbt.TagParser#ERROR_TRAILING_DATA
fn make_error_trailing_data() -> SimpleCommandExceptionType {
    SimpleCommandExceptionType::new(Component::translatable("argument.nbt.trailing"))
}

// JAVA: net.minecraft.nbt.TagParser#ERROR_EXPECTED_COMPOUND
fn make_error_expected_compound() -> SimpleCommandExceptionType {
    SimpleCommandExceptionType::new(Component::translatable("argument.nbt.expected.compound"))
}

pub static ERROR_TRAILING_DATA: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(make_error_trailing_data);
pub static ERROR_EXPECTED_COMPOUND: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(make_error_expected_compound);

pub const ELEMENT_SEPARATOR: char = ',';
pub const NAME_VALUE_SEPARATOR: char = ':';

fn syntax_error(reader: &StringReader, message: &str) -> CommandSyntaxException {
    CommandSyntaxException {
        message: message.to_string(),
        input: reader.get_string().to_string(),
        cursor: reader.get_cursor(),
    }
}

fn expect_char(reader: &mut StringReader, expected: char) -> Result<(), CommandSyntaxException> {
    if reader.can_read() && reader.peek() == expected {
        reader.read();
        Ok(())
    } else {
        Err(syntax_error(reader, &format!("Expected '{}'", expected)))
    }
}

fn is_unquoted_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+')
}

fn read_unquoted_string(reader: &mut StringReader) -> String {
    let mut out = String::new();
    while reader.can_read() && is_unquoted_char(reader.peek()) {
        out.push(reader.read());
    }
    out
}

fn parse_quoted_string(reader: &mut StringReader) -> Result<String, CommandSyntaxException> {
    let quote = reader.read();
    if quote != '"' && quote != '\'' {
        return Err(syntax_error(reader, "Expected quote"));
    }
    let mut out = String::new();
    let mut escaped = false;
    while reader.can_read() {
        let ch = reader.read();
        if escaped {
            let unescaped = match ch {
                '"' | '\'' | '\\' => ch,
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => ch,
            };
            out.push(unescaped);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return Ok(out);
        } else {
            out.push(ch);
        }
    }
    Err(syntax_error(reader, "Unterminated string"))
}

fn parse_key(reader: &mut StringReader) -> Result<String, CommandSyntaxException> {
    reader.skip_whitespace();
    if !reader.can_read() {
        return Err(syntax_error(reader, "Expected key"));
    }
    match reader.peek() {
        '"' | '\'' => parse_quoted_string(reader),
        _ => {
            let s = read_unquoted_string(reader);
            if s.is_empty() {
                Err(syntax_error(reader, "Expected key"))
            } else {
                Ok(s)
            }
        }
    }
}

fn parse_number_or_literal(s: &str) -> Option<NbtTag> {
    if s.eq_ignore_ascii_case("true") {
        return Some(NbtTag::Byte(1));
    }
    if s.eq_ignore_ascii_case("false") {
        return Some(NbtTag::Byte(0));
    }

    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let last = bytes[bytes.len().wrapping_sub(1)] as char;
    let has_dot_or_exp = s.contains('.') || s.contains('e') || s.contains('E');

    if last.eq_ignore_ascii_case(&'b') && !has_dot_or_exp {
        let num = &s[..s.len().wrapping_sub(1)];
        if let Ok(v) = num.parse::<i8>() {
            return Some(NbtTag::Byte(v));
        }
    }
    if last.eq_ignore_ascii_case(&'s') && !has_dot_or_exp {
        let num = &s[..s.len().wrapping_sub(1)];
        if let Ok(v) = num.parse::<i16>() {
            return Some(NbtTag::Short(v));
        }
    }
    if last.eq_ignore_ascii_case(&'l') && !has_dot_or_exp {
        let num = &s[..s.len().wrapping_sub(1)];
        if let Ok(v) = num.parse::<i64>() {
            return Some(NbtTag::Long(v));
        }
    }
    if last.eq_ignore_ascii_case(&'f') {
        let num = &s[..s.len().wrapping_sub(1)];
        if let Ok(v) = num.parse::<f32>() {
            return Some(NbtTag::float_value_of(v));
        }
    }
    if last.eq_ignore_ascii_case(&'d') {
        let num = &s[..s.len().wrapping_sub(1)];
        if let Ok(v) = num.parse::<f64>() {
            return Some(NbtTag::double_value_of(v));
        }
    }
    if has_dot_or_exp {
        if let Ok(v) = s.parse::<f64>() {
            return Some(NbtTag::double_value_of(v));
        }
    } else if let Ok(v) = s.parse::<i32>() {
        return Some(NbtTag::Int(v));
    }

    None
}

fn parse_unquoted_or_number(reader: &mut StringReader) -> Result<NbtTag, CommandSyntaxException> {
    let s = read_unquoted_string(reader);
    if s.is_empty() {
        return Err(syntax_error(reader, "Expected value"));
    }
    if let Some(tag) = parse_number_or_literal(&s) {
        Ok(tag)
    } else {
        Ok(NbtTag::String(s))
    }
}

fn parse_compound(reader: &mut StringReader) -> Result<NbtTag, CommandSyntaxException> {
    expect_char(reader, '{')?;
    reader.skip_whitespace();
    let mut entries: Vec<(String, NbtTag)> = Vec::new();
    if reader.can_read() && reader.peek() == '}' {
        reader.read();
        return Ok(NbtTag::Compound(entries));
    }
    loop {
        let key = parse_key(reader)?;
        reader.skip_whitespace();
        expect_char(reader, NAME_VALUE_SEPARATOR)?;
        let value = parse_value(reader)?;
        entries.push((key, value));
        reader.skip_whitespace();
        if !reader.can_read() {
            return Err(syntax_error(reader, "Expected ',' or '}'"));
        }
        let ch = reader.read();
        if ch == ELEMENT_SEPARATOR {
            reader.skip_whitespace();
            if reader.can_read() && reader.peek() == '}' {
                reader.read();
                break;
            }
            continue;
        }
        if ch == '}' {
            break;
        }
        return Err(syntax_error(reader, "Expected ',' or '}' in compound"));
    }
    Ok(NbtTag::Compound(entries))
}

fn parse_list_or_array(reader: &mut StringReader) -> Result<NbtTag, CommandSyntaxException> {
    expect_char(reader, '[')?;
    reader.skip_whitespace();
    if !reader.can_read() {
        return Err(syntax_error(reader, "Expected list or array"));
    }

    if reader.can_read_n(2) {
        let c0 = reader.peek();
        let c1 = reader.chars[reader.cursor.wrapping_add(1)];
        if (c0 == 'B' || c0 == 'I' || c0 == 'L') && c1 == ';' {
            let kind = reader.read();
            reader.read(); // ';'
            return parse_numeric_array(reader, kind);
        }
    }

    let mut items = Vec::new();
    reader.skip_whitespace();
    if reader.can_read() && reader.peek() == ']' {
        reader.read();
        return Ok(NbtTag::List { elem_type: 0, items });
    }
    loop {
        let value = parse_value(reader)?;
        items.push(value);
        reader.skip_whitespace();
        if !reader.can_read() {
            return Err(syntax_error(reader, "Expected ',' or ']' in list"));
        }
        let ch = reader.read();
        if ch == ELEMENT_SEPARATOR {
            reader.skip_whitespace();
            if reader.can_read() && reader.peek() == ']' {
                reader.read();
                break;
            }
            continue;
        }
        if ch == ']' {
            break;
        }
        return Err(syntax_error(reader, "Expected ',' or ']' in list"));
    }
    let elem_type = items.first().map(|t| t.id()).unwrap_or(0);
    Ok(NbtTag::List { elem_type, items })
}

fn parse_numeric_array(reader: &mut StringReader, kind: char) -> Result<NbtTag, CommandSyntaxException> {
    reader.skip_whitespace();
    if reader.can_read() && reader.peek() == ']' {
        reader.read();
        return Ok(match kind {
            'B' => NbtTag::ByteArray(Vec::new()),
            'I' => NbtTag::IntArray(Vec::new()),
            'L' => NbtTag::LongArray(Vec::new()),
            _ => return Err(syntax_error(reader, "Unknown array type")),
        });
    }

    let mut byte_vals: Vec<i8> = Vec::new();
    let mut int_vals: Vec<i32> = Vec::new();
    let mut long_vals: Vec<i64> = Vec::new();

    loop {
        let value = parse_value(reader)?;
        match kind {
            'B' => match value {
                NbtTag::Byte(v) => byte_vals.push(v),
                _ => return Err(syntax_error(reader, "Expected byte in byte array")),
            },
            'I' => match value {
                NbtTag::Int(v) => int_vals.push(v),
                _ => return Err(syntax_error(reader, "Expected int in int array")),
            },
            'L' => match value {
                NbtTag::Long(v) => long_vals.push(v),
                _ => return Err(syntax_error(reader, "Expected long in long array")),
            },
            _ => return Err(syntax_error(reader, "Unknown array type")),
        }

        reader.skip_whitespace();
        if !reader.can_read() {
            return Err(syntax_error(reader, "Expected ',' or ']' in array"));
        }
        let ch = reader.read();
        if ch == ELEMENT_SEPARATOR {
            reader.skip_whitespace();
            if reader.can_read() && reader.peek() == ']' {
                reader.read();
                break;
            }
            continue;
        }
        if ch == ']' {
            break;
        }
        return Err(syntax_error(reader, "Expected ',' or ']' in array"));
    }

    Ok(match kind {
        'B' => NbtTag::ByteArray(byte_vals),
        'I' => NbtTag::IntArray(int_vals),
        'L' => NbtTag::LongArray(long_vals),
        _ => return Err(syntax_error(reader, "Unknown array type")),
    })
}

fn parse_value(reader: &mut StringReader) -> Result<NbtTag, CommandSyntaxException> {
    reader.skip_whitespace();
    if !reader.can_read() {
        return Err(syntax_error(reader, "Expected value"));
    }
    match reader.peek() {
        '{' => parse_compound(reader),
        '[' => parse_list_or_array(reader),
        '"' | '\'' => parse_quoted_string(reader).map(NbtTag::String),
        _ => parse_unquoted_or_number(reader),
    }
}

// JAVA: net.minecraft.nbt.TagParser#castToCompoundOrThrow
fn cast_to_compound_or_throw(
    reader: &StringReader,
    tag: NbtTag,
) -> Result<NbtTag, CommandSyntaxException> {
    match tag {
        NbtTag::Compound(_) => Ok(tag),
        _ => Err(ERROR_EXPECTED_COMPOUND.create_with_context(reader)),
    }
}

/// Port of `net.minecraft.nbt.TagParser<T>`.
/// This self-contained port parses SNBT into `crate::NbtTag`.
#[derive(Debug, Clone, Default)]
pub struct TagParser {
    _private: (),
}

impl TagParser {
    // JAVA: net.minecraft.nbt.TagParser#create
    pub fn create() -> Self {
        Self { _private: () }
    }

    // JAVA: net.minecraft.nbt.TagParser#parseCompoundFully
    pub fn parse_compound_fully(input: &str) -> Result<NbtTag, CommandSyntaxException> {
        let mut reader = StringReader::new(input);
        let parser = Self::create();
        let tag = parser.parse_fully_reader(&mut reader)?;
        cast_to_compound_or_throw(&reader, tag)
    }

    // JAVA: net.minecraft.nbt.TagParser#parseFully
    pub fn parse_fully_str(&self, input: &str) -> Result<NbtTag, CommandSyntaxException> {
        let mut reader = StringReader::new(input);
        self.parse_fully_reader(&mut reader)
    }

    // JAVA: net.minecraft.nbt.TagParser#parseFully
    pub fn parse_fully_reader(
        &self,
        reader: &mut StringReader,
    ) -> Result<NbtTag, CommandSyntaxException> {
        let tag = self.parse_as_argument(reader)?;
        reader.skip_whitespace();
        if reader.can_read() {
            return Err(ERROR_TRAILING_DATA.create_with_context(reader));
        }
        Ok(tag)
    }

    // JAVA: net.minecraft.nbt.TagParser#parseAsArgument
    pub fn parse_as_argument(
        &self,
        reader: &mut StringReader,
    ) -> Result<NbtTag, CommandSyntaxException> {
        parse_value(reader)
    }

    // JAVA: net.minecraft.nbt.TagParser#parseCompoundAsArgument
    pub fn parse_compound_as_argument(
        reader: &mut StringReader,
    ) -> Result<NbtTag, CommandSyntaxException> {
        let tag = parse_value(reader)?;
        cast_to_compound_or_throw(reader, tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_compound_fully_and_access() {
        let tag = TagParser::parse_compound_fully("{a:1b,b:\"hi\",c:[I;1,2]}").unwrap();
        match &tag {
            NbtTag::Compound(entries) => {
                assert_eq!(entries.len(), 3);
            }
            _ => panic!("expected compound"),
        }
        assert_eq!(tag.compound_get("a"), Some(&NbtTag::Byte(1)));
        assert_eq!(
            tag.compound_get("b"),
            Some(&NbtTag::String("hi".to_string()))
        );
        assert_eq!(
            tag.compound_get("c"),
            Some(&NbtTag::IntArray(vec![1, 2]))
        );
    }

    #[test]
    fn parse_fully_trailing_data_error() {
        let parser = TagParser::create();
        let err = parser.parse_fully_str("{a:1b} trailing").unwrap_err();
        assert_eq!(err.message, "argument.nbt.trailing");
    }

    #[test]
    fn parse_compound_fully_expected_compound_error() {
        let err = TagParser::parse_compound_fully("1b").unwrap_err();
        assert_eq!(err.message, "argument.nbt.expected.compound");
    }

    #[test]
    fn parse_negative_and_float_bits() {
        let parser = TagParser::create();
        let tag = parser.parse_fully_str("{b:-1b,f:-1.5f,d:0.0d}").unwrap();
        assert_eq!(tag.compound_get("b"), Some(&NbtTag::Byte(-1)));
        match tag.compound_get("f") {
            Some(NbtTag::Float(v)) => assert_eq!(v.to_bits(), (-1.5f32).to_bits()),
            other => panic!("expected float, got {:?}", other),
        }
        match tag.compound_get("d") {
            Some(NbtTag::Double(v)) => assert_eq!(v.to_bits(), 0.0f64.to_bits()),
            other => panic!("expected double, got {:?}", other),
        }
    }

    #[test]
    fn parse_as_argument_and_compound_as_argument() {
        let parser = TagParser::create();
        let mut reader = StringReader::new("{x:1} rest");
        let tag = parser.parse_as_argument(&mut reader).unwrap();
        assert!(matches!(tag, NbtTag::Compound(_)));
        reader.skip_whitespace();
        assert_eq!(reader.get_remaining(), "rest");

        let mut reader2 = StringReader::new("{y:2b}");
        let tag2 = TagParser::parse_compound_as_argument(&mut reader2).unwrap();
        assert_eq!(tag2.compound_get("y"), Some(&NbtTag::Byte(2)));
    }

    #[test]
    fn parse_arrays_and_numeric_edge_types() {
        let parser = TagParser::create();
        let tag = parser
            .parse_fully_str("{ba:[B;-1b,127b],la:[L;-1L,0L],s:-32768s,i:-2147483648}")
            .unwrap();
        assert_eq!(tag.compound_get("ba"), Some(&NbtTag::ByteArray(vec![-1, 127])));
        assert_eq!(tag.compound_get("la"), Some(&NbtTag::LongArray(vec![-1i64, 0i64])));
        assert_eq!(tag.compound_get("s"), Some(&NbtTag::Short(i16::MIN)));
        assert_eq!(tag.compound_get("i"), Some(&NbtTag::Int(i32::MIN)));
    }
}
