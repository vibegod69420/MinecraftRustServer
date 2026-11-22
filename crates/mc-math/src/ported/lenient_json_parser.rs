use std::fmt;
use std::io::Read;

/// Minimal JSON value representation for the LenientJsonParser port.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonElement {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonElement>),
    Object(Vec<(String, JsonElement)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonParseError {
    pub message: String,
}

impl JsonParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for JsonParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for JsonParseError {}

struct Parser<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn err(&self, msg: &str) -> JsonParseError {
        JsonParseError::new(format!("{msg} at byte {}", self.pos))
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
                self.pos += 1;
            }
            if self.pos + 1 < self.chars.len() && self.chars[self.pos] == b'/' {
                if self.chars[self.pos + 1] == b'/' {
                    self.pos += 2;
                    while let Some(c) = self.peek() {
                        self.pos += 1;
                        if c == b'\n' {
                            break;
                        }
                    }
                    continue;
                } else if self.chars[self.pos + 1] == b'*' {
                    self.pos += 2;
                    while self.pos + 1 < self.chars.len() {
                        if self.chars[self.pos] == b'*' && self.chars[self.pos + 1] == b'/' {
                            self.pos += 2;
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }
            }
            break;
        }
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), JsonParseError> {
        for &b in expected {
            if self.next() != Some(b) {
                return Err(self.err("unexpected token"));
            }
        }
        Ok(())
    }

    fn decode_utf8_char(&mut self, first: u8) -> Result<char, JsonParseError> {
        let width = if first < 0x80 {
            1
        } else if first & 0xE0 == 0xC0 {
            2
        } else if first & 0xF0 == 0xE0 {
            3
        } else if first & 0xF8 == 0xF0 {
            4
        } else {
            return Err(self.err("invalid utf-8"));
        };
        let mut buf = [first, 0u8, 0u8, 0u8];
        for i in 1..width {
            buf[i] = self.next().ok_or_else(|| self.err("invalid utf-8"))?;
        }
        std::str::from_utf8(&buf[..width])
            .ok()
            .and_then(|s| s.chars().next())
            .ok_or_else(|| self.err("invalid utf-8"))
    }

    fn parse_string(&mut self) -> Result<String, JsonParseError> {
        if self.next() != Some(b'"') {
            return Err(self.err("expected '\"'"));
        }
        let mut out = String::new();
        loop {
            let c = self.next().ok_or_else(|| self.err("unterminated string"))?;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self
                        .next()
                        .ok_or_else(|| self.err("unterminated escape"))?;
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
                            let mut code: u32 = 0;
                            for _ in 0..4 {
                                let h = self
                                    .next()
                                    .ok_or_else(|| self.err("bad unicode escape"))?;
                                let d = match h {
                                    b'0'..=b'9' => (h - b'0') as u32,
                                    b'a'..=b'f' => (h - b'a' + 10) as u32,
                                    b'A'..=b'F' => (h - b'A' + 10) as u32,
                                    _ => return Err(self.err("bad unicode escape")),
                                };
                                code = (code << 4) | d;
                            }
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                            } else {
                                out.push('\u{FFFD}');
                            }
                        }
                        _ => return Err(self.err("invalid escape")),
                    }
                }
                b if b < 0x20 => return Err(self.err("control character in string")),
                b => out.push(self.decode_utf8_char(b)?),
            }
        }
    }

    fn parse_number(&mut self) -> Result<JsonElement, JsonParseError> {
        let start = self.pos;
        if matches!(self.peek(), Some(b'-')) {
            self.pos += 1;
        }
        let mut has_digit = false;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            has_digit = true;
            self.pos += 1;
        }
        if matches!(self.peek(), Some(b'.')) {
            self.pos += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                has_digit = true;
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            let mut exp_digit = false;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                exp_digit = true;
                self.pos += 1;
            }
            if !exp_digit {
                return Err(self.err("invalid number exponent"));
            }
        }
        if !has_digit {
            return Err(self.err("invalid number"));
        }
        let s = std::str::from_utf8(&self.chars[start..self.pos])
            .map_err(|_| self.err("invalid number"))?;
        let n: f64 = s.parse().map_err(|_| self.err("invalid number"))?;
        Ok(JsonElement::Number(n))
    }

    fn parse_array(&mut self) -> Result<JsonElement, JsonParseError> {
        if self.next() != Some(b'[') {
            return Err(self.err("expected '['"));
        }
        let mut items = Vec::new();
        self.skip_ws_and_comments();
        if matches!(self.peek(), Some(b']')) {
            self.pos += 1;
            return Ok(JsonElement::Array(items));
        }
        loop {
            self.skip_ws_and_comments();
            if matches!(self.peek(), Some(b']')) {
                self.pos += 1;
                break;
            }
            items.push(self.parse_value()?);
            self.skip_ws_and_comments();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    self.skip_ws_and_comments();
                    if matches!(self.peek(), Some(b']')) {
                        self.pos += 1; // lenient trailing comma
                        break;
                    }
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(self.err("expected ',' or ']'")),
            }
        }
        Ok(JsonElement::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonElement, JsonParseError> {
        if self.next() != Some(b'{') {
            return Err(self.err("expected '{'"));
        }
        let mut entries = Vec::new();
        self.skip_ws_and_comments();
        if matches!(self.peek(), Some(b'}')) {
            self.pos += 1;
            return Ok(JsonElement::Object(entries));
        }
        loop {
            self.skip_ws_and_comments();
            if matches!(self.peek(), Some(b'}')) {
                self.pos += 1;
                break;
            }
            let key = self.parse_string()?;
            self.skip_ws_and_comments();
            if self.next() != Some(b':') {
                return Err(self.err("expected ':'"));
            }
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_ws_and_comments();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    self.skip_ws_and_comments();
                    if matches!(self.peek(), Some(b'}')) {
                        self.pos += 1; // lenient trailing comma
                        break;
                    }
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(self.err("expected ',' or '}'")),
            }
        }
        Ok(JsonElement::Object(entries))
    }

    fn parse_value(&mut self) -> Result<JsonElement, JsonParseError> {
        self.skip_ws_and_comments();
        match self.peek() {
            Some(b'"') => self.parse_string().map(JsonElement::String),
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b't') => {
                self.expect_bytes(b"true")?;
                Ok(JsonElement::Bool(true))
            }
            Some(b'f') => {
                self.expect_bytes(b"false")?;
                Ok(JsonElement::Bool(false))
            }
            Some(b'n') => {
                self.expect_bytes(b"null")?;
                Ok(JsonElement::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(_) => Err(self.err("unexpected character")),
            None => Err(self.err("unexpected end of input")),
        }
    }
}

// JAVA: net.minecraft.util.LenientJsonParser#parse
pub fn parse_reader<R: Read>(mut reader: R) -> Result<JsonElement, JsonParseError> {
    let mut buf = String::new();
    reader
        .read_to_string(&mut buf)
        .map_err(|e| JsonParseError::new(e.to_string()))?;
    parse(&buf)
}

// JAVA: net.minecraft.util.LenientJsonParser#parse
pub fn parse(input: &str) -> Result<JsonElement, JsonParseError> {
    let mut p = Parser::new(input);
    let value = p.parse_value()?;
    p.skip_ws_and_comments();
    if p.pos != p.chars.len() {
        return Err(JsonParseError::new(format!("trailing data at byte {}", p.pos)));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_string_and_reader() {
        let v1 = parse("\"hello\"").unwrap();
        assert_eq!(v1, JsonElement::String("hello".to_string()));

        let v2 = parse_reader(Cursor::new(b"123")).unwrap();
        assert_eq!(v2, JsonElement::Number(123.0));
    }

    #[test]
    fn parse_object_array_and_literals() {
        let v = parse(r#"{"a": [1, true, false, null], "b": "x"}"#).unwrap();
        let JsonElement::Object(entries) = v else {
            panic!("expected object");
        };
        assert_eq!(entries[0].0, "a");
        assert_eq!(
            entries[0].1,
            JsonElement::Array(vec![
                JsonElement::Number(1.0),
                JsonElement::Bool(true),
                JsonElement::Bool(false),
                JsonElement::Null
            ])
        );
        assert_eq!(entries[1], ("b".to_string(), JsonElement::String("x".to_string())));
    }

    #[test]
    fn parse_negative_and_exponent_numbers() {
        assert_eq!(parse("-42").unwrap(), JsonElement::Number(-42.0));
        assert_eq!(parse("3.5e2").unwrap(), JsonElement::Number(350.0));
        assert_eq!(parse("-1.25E-2").unwrap(), JsonElement::Number(-0.0125));
    }

    #[test]
    fn lenient_comments_and_trailing_commas() {
        let input = r#"
            {
                // line comment
                "a": 1,
                /* block comment */
                "b": [2, 3,],
            }
        "#;
        let v = parse(input).unwrap();
        let JsonElement::Object(entries) = v else {
            panic!("expected object");
        };
        assert_eq!(entries[0], ("a".to_string(), JsonElement::Number(1.0)));
        assert_eq!(
            entries[1],
            (
                "b".to_string(),
                JsonElement::Array(vec![JsonElement::Number(2.0), JsonElement::Number(3.0)])
            )
        );
    }

    #[test]
    fn parse_errors_on_invalid_input() {
        assert!(parse("{").is_err());
        assert!(parse("\"unterminated").is_err());
        assert!(parse("1 2").is_err());
        assert!(parse("tru").is_err());
    }
}
