use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

pub struct DelayedException<E> {
    factory: Rc<dyn Fn() -> E>,
}

impl<E> DelayedException<E> {
    // JAVA: net.minecraft.util.parsing.packrat.DelayedException#create
    pub fn new<F: Fn() -> E + 'static>(f: F) -> Self {
        Self {
            factory: Rc::new(f),
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.DelayedException#create
    pub fn create(&self) -> E {
        (self.factory)()
    }
}

#[derive(Debug, Clone)]
pub struct StringReader {
    string: String,
    cursor: i32,
}

impl StringReader {
    pub fn new(string: String) -> Self {
        Self { string, cursor: 0 }
    }

    pub fn get_string(&self) -> &str {
        &self.string
    }

    pub fn get_cursor(&self) -> i32 {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: i32) {
        self.cursor = cursor;
    }

    // JAVA: com.mojang.brigadier.StringReader#skipWhitespace
    pub fn skip_whitespace(&mut self) {
        let chars: Vec<char> = self.string.chars().collect();
        let mut i = if self.cursor < 0 { 0usize } else { self.cursor as usize };
        while i < chars.len() && chars[i].is_whitespace() {
            i = i.wrapping_add(1);
        }
        self.cursor = i as i32;
    }
}

fn byte_offset(s: &str, char_idx: i32) -> usize {
    if char_idx <= 0 {
        return 0;
    }
    let idx = char_idx as usize;
    s.char_indices().nth(idx).map(|(i, _)| i).unwrap_or(s.len())
}

#[derive(Default)]
pub struct ErrorCollector {
    pub stored_marks: Vec<i32>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            stored_marks: Vec::new(),
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.ErrorCollector#store
    pub fn store(&mut self, mark: i32, _error: &DelayedException<CommandSyntaxException>) {
        self.stored_marks.push(mark);
    }
}

pub struct ParseState {
    input: StringReader,
    error_collector: ErrorCollector,
    mark: i32,
}

impl ParseState {
    pub fn new(input: StringReader, mark: i32) -> Self {
        Self {
            input,
            error_collector: ErrorCollector::new(),
            mark,
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#input
    pub fn input(&mut self) -> &mut StringReader {
        &mut self.input
    }

    pub fn input_ref(&self) -> &StringReader {
        &self.input
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#errorCollector
    pub fn error_collector(&mut self) -> &mut ErrorCollector {
        &mut self.error_collector
    }

    pub fn error_collector_ref(&self) -> &ErrorCollector {
        &self.error_collector
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#mark
    pub fn mark(&self) -> i32 {
        self.mark
    }
}

pub struct NumberRunParseRule {
    no_value_error: DelayedException<CommandSyntaxException>,
    underscore_not_allowed_error: DelayedException<CommandSyntaxException>,
    is_accepted: Box<dyn Fn(char) -> bool>,
}

impl NumberRunParseRule {
    // JAVA: net.minecraft.util.parsing.packrat.commands.NumberRunParseRule#<init>
    pub fn new(
        no_value_error: DelayedException<CommandSyntaxException>,
        underscore_not_allowed_error: DelayedException<CommandSyntaxException>,
        is_accepted: Box<dyn Fn(char) -> bool>,
    ) -> Self {
        Self {
            no_value_error,
            underscore_not_allowed_error,
            is_accepted,
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.NumberRunParseRule#parse
    pub fn parse(&self, state: &mut ParseState) -> Option<String> {
        state.input().skip_whitespace();
        let s = state.input().get_string().to_string();
        let chars: Vec<char> = s.chars().collect();
        let start = state.input().get_cursor();
        let mut end = start;
        let len = chars.len() as i32;

        while end < len && (self.is_accepted)(chars[end as usize]) {
            end = end.wrapping_add(1);
        }

        let run_len = end.wrapping_sub(start);
        if run_len == 0 {
            let mark = state.mark();
            state.error_collector().store(mark, &self.no_value_error);
            return None;
        }

        let first = chars[start as usize];
        let last = chars[(end.wrapping_sub(1)) as usize];
        if first != '_' && last != '_' {
            state.input().set_cursor(end);
            let start_byte = byte_offset(&s, start);
            let end_byte = byte_offset(&s, end);
            return Some(s[start_byte..end_byte].to_string());
        }

        let mark = state.mark();
        state
            .error_collector()
            .store(mark, &self.underscore_not_allowed_error);
        None
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.NumberRunParseRule#isAccepted
    pub fn is_accepted(&self, c: char) -> bool {
        (self.is_accepted)(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digit_rule() -> NumberRunParseRule {
        NumberRunParseRule::new(
            DelayedException::new(|| CommandSyntaxException {
                message: "no value".to_string(),
            }),
            DelayedException::new(|| CommandSyntaxException {
                message: "underscore".to_string(),
            }),
            Box::new(|c: char| c.is_ascii_digit() || c == '_'),
        )
    }

    #[test]
    fn parse_simple_number() {
        let rule = digit_rule();
        let mut state = ParseState::new(StringReader::new("123abc".to_string()), 0);
        let result = rule.parse(&mut state);
        assert_eq!(result.as_deref(), Some("123"));
        assert_eq!(state.input_ref().get_cursor(), 3);
    }

    #[test]
    fn parse_skips_whitespace() {
        let rule = digit_rule();
        let mut state = ParseState::new(StringReader::new("  42".to_string()), 1);
        let result = rule.parse(&mut state);
        assert_eq!(result.as_deref(), Some("42"));
        assert_eq!(state.input_ref().get_cursor(), 4);
    }

    #[test]
    fn parse_no_value_stores_error() {
        let rule = digit_rule();
        let mut state = ParseState::new(StringReader::new("abc".to_string()), 7);
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector_ref().stored_marks, vec![7]);
    }

    #[test]
    fn parse_leading_underscore_rejected() {
        let rule = digit_rule();
        let mut state = ParseState::new(StringReader::new("_12".to_string()), 3);
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector_ref().stored_marks, vec![3]);
        assert_eq!(state.input_ref().get_cursor(), 0);
    }

    #[test]
    fn parse_trailing_underscore_rejected() {
        let rule = digit_rule();
        let mut state = ParseState::new(StringReader::new("12_".to_string()), 5);
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector_ref().stored_marks, vec![5]);
    }

    #[test]
    fn parse_negative_mark_edge() {
        let rule = digit_rule();
        let mut state = ParseState::new(StringReader::new("!".to_string()), -10);
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector_ref().stored_marks, vec![-10]);
    }
}
