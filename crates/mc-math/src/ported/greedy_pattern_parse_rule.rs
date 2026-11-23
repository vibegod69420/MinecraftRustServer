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
}

fn char_count(s: &str) -> i32 {
    s.chars().count() as i32
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

/// Minimal self-contained pattern abstraction (no external regex crate).
/// Returns the exclusive end char index of a match anchored at `region_start`, or `None`.
pub struct Pattern {
    finder: Box<dyn Fn(&str, i32, i32) -> Option<i32>>,
}

impl Pattern {
    pub fn new<F: Fn(&str, i32, i32) -> Option<i32> + 'static>(f: F) -> Self {
        Self {
            finder: Box::new(f),
        }
    }

    fn looking_at_end(&self, input: &str, region_start: i32, region_end: i32) -> Option<i32> {
        (self.finder)(input, region_start, region_end)
    }
}

pub struct GreedyPatternParseRule {
    pattern: Pattern,
    error: DelayedException<CommandSyntaxException>,
}

impl GreedyPatternParseRule {
    // JAVA: net.minecraft.util.parsing.packrat.commands.GreedyPatternParseRule#<init>
    pub fn new(pattern: Pattern, error: DelayedException<CommandSyntaxException>) -> Self {
        Self { pattern, error }
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.GreedyPatternParseRule#parse
    pub fn parse(&self, state: &mut ParseState) -> Option<String> {
        let cursor = state.input().get_cursor();
        let s = state.input().get_string().to_string();
        let len = char_count(&s);
        let match_end = self.pattern.looking_at_end(&s, cursor, len);
        if match_end.is_none() {
            let mark = state.mark();
            state.error_collector().store(mark, &self.error);
            return None;
        }
        let end = match_end.unwrap();
        state.input().set_cursor(end);
        let start_byte = byte_offset(&s, cursor);
        let end_byte = byte_offset(&s, end);
        Some(s[start_byte..end_byte].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefix_pattern(prefix: &str) -> Pattern {
        let prefix: Vec<char> = prefix.chars().collect();
        Pattern::new(move |input: &str, start: i32, end: i32| {
            let chars: Vec<char> = input.chars().collect();
            let s = if start < 0 { 0usize } else { start as usize };
            let e = if end < 0 { 0usize } else { (end as usize).min(chars.len()) };
            if s.wrapping_add(prefix.len()) > e {
                return None;
            }
            if chars[s..].starts_with(&prefix) {
                Some((s + prefix.len()) as i32)
            } else {
                None
            }
        })
    }

    fn err() -> DelayedException<CommandSyntaxException> {
        DelayedException::new(|| CommandSyntaxException {
            message: "bad".to_string(),
        })
    }

    #[test]
    fn parse_match_advances_cursor() {
        let rule = GreedyPatternParseRule::new(prefix_pattern("hello"), err());
        let mut state = ParseState::new(StringReader::new("hello world".to_string()), 0);
        let result = rule.parse(&mut state);
        assert_eq!(result.as_deref(), Some("hello"));
        assert_eq!(state.input_ref().get_cursor(), 5);
        assert!(state.error_collector_ref().stored_marks.is_empty());
    }

    #[test]
    fn parse_no_match_stores_error_at_mark() {
        let rule = GreedyPatternParseRule::new(prefix_pattern("xyz"), err());
        let mut state = ParseState::new(StringReader::new("hello".to_string()), 2);
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector_ref().stored_marks, vec![2]);
        assert_eq!(state.input_ref().get_cursor(), 0);
    }

    #[test]
    fn parse_at_end_of_string_fails() {
        let rule = GreedyPatternParseRule::new(prefix_pattern("a"), err());
        let mut state = ParseState::new(StringReader::new("abc".to_string()), 0);
        state.input.set_cursor(3);
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector_ref().stored_marks, vec![0]);
    }

    #[test]
    fn parse_with_offset_cursor() {
        let rule = GreedyPatternParseRule::new(prefix_pattern("cd"), err());
        let mut state = ParseState::new(StringReader::new("abcd".to_string()), 1);
        state.input.set_cursor(2);
        let result = rule.parse(&mut state);
        assert_eq!(result.as_deref(), Some("cd"));
        assert_eq!(state.input_ref().get_cursor(), 4);
    }

    #[test]
    fn parse_negative_mark_and_cursor_edge() {
        let rule = GreedyPatternParseRule::new(prefix_pattern("ab"), err());
        let mut state = ParseState::new(StringReader::new("ab".to_string()), -1);
        state.input.set_cursor(-5);
        let result = rule.parse(&mut state);
        // cursor clamped to 0 by byte_offset/char logic; pattern sees start -5 but chars indexing handles it
        // The match should succeed from effective start 0 because finder checks chars[s..]
        assert_eq!(result.as_deref(), Some("ab"));
        assert_eq!(state.input_ref().get_cursor(), 2);
    }
}
