use std::fmt;
use std::io::{self, Read};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonElement {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonElement>),
    Object(Vec<(String, JsonElement)>),
}

impl JsonElement {
    pub fn is_json_null(&self) -> bool {
        matches!(self, JsonElement::Null)
    }
}

#[derive(Debug)]
pub struct JsonSyntaxException {
    pub message: String,
}

impl JsonSyntaxException {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for JsonSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JsonSyntaxException: {}", self.message)
    }
}

impl std::error::Error for JsonSyntaxException {}

#[derive(Debug)]
pub struct JsonIOException {
    pub source: io::Error,
}

impl fmt::Display for JsonIOException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JsonIOException: {}", self.source)
    }
}

impl std::error::Error for JsonIOException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub enum ParseError {
    Syntax(JsonSyntaxException),
    Io(JsonIOException),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Syntax(e) => write!(f, "{e}"),
            ParseError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Syntax(e) => Some(e),
            ParseError::Io(e) => Some(e),
        }
    }
}

impl From<JsonSyntaxException> for ParseError {
    fn from(value: JsonSyntaxException) -> Self {
        ParseError::Syntax(value)
    }
}

impl From<JsonIOException> for ParseError {
    fn from(value: JsonIOException) -> Self {
        ParseError::Io(value)
    }
}

pub struct StrictJsonParser;

impl StrictJsonParser {
    // JAVA: net.minecraft.util.StrictJsonParser#parse
    pub fn parse_reader<R: Read>(mut reader: R) -> Result<JsonElement, ParseError> {
        let mut input = String::new();
        reader
            .read_to_string(&mut input)
            .map_err(|e| ParseError::Io(JsonIOException { source: e }))?;
        Self::parse_str(&input)
    }

    // JAVA: net.minecraft.util.StrictJsonParser#parse
    pub fn parse(input: &str) -> Result<JsonElement, ParseError> {
        Self::parse_str(input)
    }

    fn parse_str(input: &str) -> Result<JsonElement, ParseError> {
        let mut parser = Parser::new(input.as_bytes());
        let value = parser.parse_value().map_err(ParseError::Syntax)?;
        parser.skip_ws();
        if !value.is_json_null() && parser.peek().is_some() {
            return Err(ParseError::Syntax(JsonSyntaxException::new(
                "Did not consume the entire document.",
            )));
        }
        Ok(value)
    }
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn parse_value(&mut self) -> Result<JsonElement, JsonSyntaxException> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => self.parse_string().map(JsonElement::String),
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b't') => self.parse_literal(b"true", JsonElement::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", JsonElement::Bool(false)),
            Some(b'n') => self.parse_literal(b"null", JsonElement::Null),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(c) => Err(JsonSyntaxException::new(format!(
                "Unexpected character '{}'",
                c as char
            ))),
            None => Err(JsonSyntaxException::new("Unexpected end of input")),
        }
    }

    fn parse_literal(
        &mut self,
        literal: &[u8],
        value: JsonElement,
    ) -> Result<JsonElement, JsonSyntaxException> {
        if self.input.get(self.pos..self.pos + literal.len()) == Some(literal) {
            self.pos += literal.len();
            Ok(value)
        } else {
            Err(JsonSyntaxException::new("Invalid literal"))
        }
    }

    fn parse_string(&mut self) -> Result<String, JsonSyntaxException> {
        if self.next() != Some(b'"') {
            return Err(JsonSyntaxException::new("Expected '\"'"));
        }
        let mut out = String::new();
        loop {
            let b = self
                .next()
                .ok_or_else(|| JsonSyntaxException::new("Unterminated string"))?;
            match b {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self
                        .next()
                        .ok_or_else(|| JsonSyntaxException::new("Unterminated escape sequence"))?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let cp = self.parse_unicode_escape()?;
                            if (0xD800..=0xDBFF).contains(&cp) {
                                // high surrogate, expect \uXXXX low surrogate
                                if self.next() != Some(b'\\') || self.next() != Some(b'u') {
                                    return Err(JsonSyntaxException::new(
                                        "Invalid unicode surrogate pair",
                                    ));
                                }
                                let low = self.parse_unicode_escape()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err(JsonSyntaxException::new(
                                        "Invalid unicode surrogate pair",
                                    ));
                                }
                                let code = 0x10000 + (((cp - 0xD800) << 10) | (low - 0xDC00));
                                let ch = char::from_u32(code).ok_or_else(|| {
                                    JsonSyntaxException::new("Invalid unicode codepoint")
                                })?;
                                out.push(ch);
                            } else if (0xDC00..=0xDFFF).contains(&cp) {
                                return Err(JsonSyntaxException::new(
                                    "Invalid unicode surrogate pair",
                                ));
                            } else {
                                let ch = char::from_u32(cp).ok_or_else(|| {
                                    JsonSyntaxException::new("Invalid unicode codepoint")
                                })?;
                                out.push(ch);
                            }
                        }
                        _ => {
                            return Err(JsonSyntaxException::new(format!(
                                "Invalid escape character '{}'",
                                esc as char
                            )))
                        }
                    }
                }
                0x00..=0x1F => {
                    return Err(JsonSyntaxException::new(
                        "Unescaped control character in string",
                    ))
                }
                _ => {
                    // UTF-8 byte sequence; reconstruct char from input bytes
                    self.pos -= 1;
                    let ch = self.parse_utf8_char()?;
                    out.push(ch);
                }
            }
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<u32, JsonSyntaxException> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            let b = self
                .next()
                .ok_or_else(|| JsonSyntaxException::new("Incomplete unicode escape"))?;
            let digit = match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'a'..=b'f' => (b - b'a' + 10) as u32,
                b'A'..=b'F' => (b - b'A' + 10) as u32,
                _ => return Err(JsonSyntaxException::new("Invalid unicode escape")),
            };
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn parse_utf8_char(&mut self) -> Result<char, JsonSyntaxException> {
        let b0 = self
            .next()
            .ok_or_else(|| JsonSyntaxException::new("Unexpected end of input"))?;
        let width = match b0 {
            0x00..=0x7F => 1,
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF4 => 4,
            _ => return Err(JsonSyntaxException::new("Invalid UTF-8 sequence")),
        };
        let mut code: u32 = if width == 1 {
            b0 as u32
        } else {
            (b0 & (0x7F >> width)) as u32
        };
        for _ in 1..width {
            let b = self
                .next()
                .ok_or_else(|| JsonSyntaxException::new("Invalid UTF-8 sequence"))?;
            if (b & 0xC0) != 0x80 {
                return Err(JsonSyntaxException::new("Invalid UTF-8 sequence"));
            }
            code = (code << 6) | (b & 0x3F) as u32;
        }
        char::from_u32(code).ok_or_else(|| JsonSyntaxException::new("Invalid unicode codepoint"))
    }

    fn parse_number(&mut self) -> Result<JsonElement, JsonSyntaxException> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(JsonSyntaxException::new("Leading zeros are not allowed"));
                }
            }
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(JsonSyntaxException::new("Invalid number")),
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonSyntaxException::new("Invalid number fraction"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonSyntaxException::new("Invalid number exponent"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let s = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| JsonSyntaxException::new("Invalid number encoding"))?;
        let n: f64 = s
            .parse()
            .map_err(|e| JsonSyntaxException::new(format!("Invalid number: {e}")))?;
        Ok(JsonElement::Number(n))
    }

    fn parse_array(&mut self) -> Result<JsonElement, JsonSyntaxException> {
        if self.next() != Some(b'[') {
            return Err(JsonSyntaxException::new("Expected '['"));
        }
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonElement::Array(items));
        }
        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_ws();
            match self.next() {
                Some(b',') => {
                    self.skip_ws();
                    if self.peek() == Some(b']') {
                        return Err(JsonSyntaxException::new("Trailing comma in array"));
                    }
                }
                Some(b']') => return Ok(JsonElement::Array(items)),
                _ => return Err(JsonSyntaxException::new("Expected ',' or ']' in array")),
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonElement, JsonSyntaxException> {
        if self.next() != Some(b'{') {
            return Err(JsonSyntaxException::new("Expected '{'"));
        }
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonElement::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(JsonSyntaxException::new("Object key must be a string"));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.next() != Some(b':') {
                return Err(JsonSyntaxException::new("Expected ':' after object key"));
            }
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_ws();
            match self.next() {
                Some(b',') => {
                    self.skip_ws();
                    if self.peek() == Some(b'}') {
                        return Err(JsonSyntaxException::new("Trailing comma in object"));
                    }
                }
                Some(b'}') => return Ok(JsonElement::Object(entries)),
                _ => return Err(JsonSyntaxException::new("Expected ',' or '}' in object")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_simple_object_and_array() {
        let v = StrictJsonParser::parse(r#"{"a": [1, true, null, "x"]}"#).unwrap();
        match v {
            JsonElement::Object(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "a");
                match &entries[0].1 {
                    JsonElement::Array(items) => {
                        assert_eq!(items.len(), 4);
                        assert_eq!(items[0], JsonElement::Number(1.0));
                        assert_eq!(items[1], JsonElement::Bool(true));
                        assert_eq!(items[2], JsonElement::Null);
                        assert_eq!(items[3], JsonElement::String("x".to_string()));
                    }
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn parse_reader_and_string_equivalence() {
        let input = r#"{"k":"v"}"#;
        let from_str = StrictJsonParser::parse(input).unwrap();
        let from_reader = StrictJsonParser::parse_reader(Cursor::new(input.as_bytes())).unwrap();
        assert_eq!(from_str, from_reader);
    }

    #[test]
    fn rejects_trailing_data_for_non_null() {
        let err = StrictJsonParser::parse("1 2").unwrap_err();
        match err {
            ParseError::Syntax(e) => {
                assert!(e.message.contains("Did not consume the entire document."));
            }
            _ => panic!("expected syntax error"),
        }
    }

    #[test]
    fn null_allows_trailing_check_behavior_matches_java_condition() {
        // Java: if (!isJsonNull() && peek != END_DOCUMENT) throw; so null does not trigger this check.
        // Strict parsing may still fail on trailing tokens depending on reader state, but our faithful
        // condition preserves the exact guard.
        let v = StrictJsonParser::parse("null").unwrap();
        assert!(v.is_json_null());
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(matches!(
            StrictJsonParser::parse("{\"a\": 01}"),
            Err(ParseError::Syntax(_))
        ));
        assert!(matches!(
            StrictJsonParser::parse("{a:1}"),
            Err(ParseError::Syntax(_))
        ));
        assert!(matches!(
            StrictJsonParser::parse("[1,]"),
            Err(ParseError::Syntax(_))
        ));
    }

    #[test]
    fn io_error_is_mapped() {
        struct BadReader;
        impl Read for BadReader {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Other, "boom"))
            }
        }
        let err = StrictJsonParser::parse_reader(BadReader).unwrap_err();
        assert!(matches!(err, ParseError::Io(_)));
    }
}
