use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DelayedException<E> {
    pub message: String,
    _marker: PhantomData<E>,
}

impl<E> DelayedException<E> {
    // JAVA: net.minecraft.util.parsing.packrat.DelayedException#<init>
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StringReader {
    string: String,
    cursor: usize,
}

impl StringReader {
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            string: s.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn peek(&self) -> Option<char> {
        self.string[self.cursor..].chars().next()
    }

    // JAVA: com.mojang.brigadier.StringReader#skipWhitespace
    pub fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.cursor += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn is_allowed_in_unquoted_string(c: char) -> bool {
        c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+')
    }

    // JAVA: com.mojang.brigadier.StringReader#readUnquotedString
    pub fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while let Some(c) = self.peek() {
            if Self::is_allowed_in_unquoted_string(c) {
                self.cursor += c.len_utf8();
            } else {
                break;
            }
        }
        self.string[start..self.cursor].to_string()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ErrorCollector {
    pub errors: Vec<(usize, String)>,
}

impl ErrorCollector {
    pub fn store<E>(&mut self, mark: usize, err: &DelayedException<E>) {
        self.errors.push((mark, err.message.clone()));
    }
}

pub struct ParseState {
    input: StringReader,
    error_collector: ErrorCollector,
}

impl ParseState {
    pub fn new(input: StringReader) -> Self {
        Self {
            input,
            error_collector: ErrorCollector::default(),
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#input
    pub fn input(&mut self) -> &mut StringReader {
        &mut self.input
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#mark
    pub fn mark(&self) -> usize {
        self.input.cursor()
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#errorCollector
    pub fn error_collector(&mut self) -> &mut ErrorCollector {
        &mut self.error_collector
    }
}

pub trait Rule<S, T> {
    // JAVA: net.minecraft.util.parsing.packrat.Rule#parse
    fn parse(&self, state: &mut ParseState) -> Option<T>;
}

pub struct UnquotedStringParseRule {
    min_size: i32,
    error: DelayedException<CommandSyntaxException>,
}

impl UnquotedStringParseRule {
    // JAVA: net.minecraft.util.parsing.packrat.commands.UnquotedStringParseRule#<init>
    pub fn new(min_size: i32, error: DelayedException<CommandSyntaxException>) -> Self {
        Self { min_size, error }
    }
}

impl Rule<StringReader, String> for UnquotedStringParseRule {
    // JAVA: net.minecraft.util.parsing.packrat.commands.UnquotedStringParseRule#parse
    fn parse(&self, state: &mut ParseState) -> Option<String> {
        state.input().skip_whitespace();
        let mark = state.mark();
        let s = state.input().read_unquoted_string();
        if (s.encode_utf16().count() as i32) < self.min_size {
            state.error_collector().store(mark, &self.error);
            None
        } else {
            Some(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_unquoted_string() {
        let rule = UnquotedStringParseRule::new(1, DelayedException::new("too short"));
        let mut state = ParseState::new(StringReader::new("hello"));
        assert_eq!(rule.parse(&mut state), Some("hello".into()));
    }

    #[test]
    fn parse_skips_leading_whitespace() {
        let rule = UnquotedStringParseRule::new(1, DelayedException::new("too short"));
        let mut state = ParseState::new(StringReader::new("  world"));
        assert_eq!(rule.parse(&mut state), Some("world".into()));
    }

    #[test]
    fn parse_too_short_stores_error_and_returns_none() {
        let rule = UnquotedStringParseRule::new(5, DelayedException::new("too short"));
        let mut state = ParseState::new(StringReader::new("hi"));
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector().errors.len(), 1);
        assert_eq!(state.error_collector().errors[0].1, "too short");
    }

    #[test]
    fn parse_error_mark_is_after_whitespace() {
        let rule = UnquotedStringParseRule::new(5, DelayedException::new("too short"));
        let mut state = ParseState::new(StringReader::new("  ab"));
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector().errors[0].0, 2);
    }

    #[test]
    fn parse_min_size_zero_accepts_empty() {
        let rule = UnquotedStringParseRule::new(0, DelayedException::new("too short"));
        let mut state = ParseState::new(StringReader::new(""));
        assert_eq!(rule.parse(&mut state), Some("".into()));
    }
}
