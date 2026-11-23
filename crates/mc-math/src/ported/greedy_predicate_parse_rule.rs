use std::fmt;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct CommandSyntaxException {
    pub message: String,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandSyntaxException {}

#[derive(Clone, Debug)]
pub struct DelayedException<E> {
    message: String,
    _phantom: PhantomData<E>,
}

impl<E> DelayedException<E> {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ErrorCollector {
    errors: Vec<(usize, String)>,
}

impl ErrorCollector {
    pub fn store<E>(&mut self, mark: usize, err: &DelayedException<E>) {
        self.errors.push((mark, err.message.clone()));
    }

    pub fn errors(&self) -> &[(usize, String)] {
        &self.errors
    }
}

#[derive(Clone, Debug)]
pub struct StringReader {
    chars: Vec<char>,
    cursor: usize,
}

impl StringReader {
    pub fn new(s: &str) -> Self {
        Self {
            chars: s.chars().collect(),
            cursor: 0,
        }
    }

    pub fn get_string(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    pub fn chars(&self) -> &[char] {
        &self.chars
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

    pub fn input(&mut self) -> &mut StringReader {
        &mut self.input
    }

    pub fn mark(&self) -> usize {
        self.input.get_cursor()
    }

    pub fn error_collector(&mut self) -> &mut ErrorCollector {
        &mut self.error_collector
    }
}

pub struct GreedyPredicateParseRule<F>
where
    F: Fn(char) -> bool,
{
    min_size: i32,
    max_size: i32,
    error: DelayedException<CommandSyntaxException>,
    predicate: F,
}

impl<F: Fn(char) -> bool> GreedyPredicateParseRule<F> {
    // JAVA: net.minecraft.util.parsing.packrat.commands.GreedyPredicateParseRule#<init>
    pub fn new(
        min_size: i32,
        error: DelayedException<CommandSyntaxException>,
        predicate: F,
    ) -> Self {
        Self::with_max(min_size, i32::MAX, error, predicate)
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.GreedyPredicateParseRule#<init>
    pub fn with_max(
        min_size: i32,
        max_size: i32,
        error: DelayedException<CommandSyntaxException>,
        predicate: F,
    ) -> Self {
        Self {
            min_size,
            max_size,
            error,
            predicate,
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.GreedyPredicateParseRule#parse
    pub fn parse(&self, state: &mut ParseState) -> Option<String> {
        let (chars, start) = {
            let input = state.input();
            (input.chars().to_vec(), input.get_cursor())
        };
        let mut end = start;

        while end < chars.len()
            && self.is_accepted(chars[end])
            && (end as i32).wrapping_sub(start as i32) < self.max_size
        {
            end += 1;
        }

        let len = (end as i32).wrapping_sub(start as i32);
        if len < self.min_size {
            let mark = state.mark();
            state.error_collector().store(mark, &self.error);
            None
        } else {
            state.input().set_cursor(end);
            Some(chars[start..end].iter().collect())
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.GreedyPredicateParseRule#isAccepted
    fn is_accepted(&self, ch: char) -> bool {
        (self.predicate)(ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(min: i32, max: i32) -> GreedyPredicateParseRule<impl Fn(char) -> bool> {
        GreedyPredicateParseRule::with_max(min, max, DelayedException::new("err"), |c: char| {
            c.is_ascii_alphabetic()
        })
    }

    #[test]
    fn parses_greedy_match() {
        let r = rule(1, i32::MAX);
        let mut state = ParseState::new(StringReader::new("abc123"));
        let out = r.parse(&mut state);
        assert_eq!(out.as_deref(), Some("abc"));
        assert_eq!(state.input().get_cursor(), 3);
    }

    #[test]
    fn fails_when_below_min_size() {
        let r = rule(5, i32::MAX);
        let mut state = ParseState::new(StringReader::new("abc"));
        let out = r.parse(&mut state);
        assert_eq!(out, None);
        assert_eq!(state.input().get_cursor(), 0);
        assert_eq!(state.error_collector().errors().len(), 1);
    }

    #[test]
    fn respects_max_size() {
        let r = rule(1, 2);
        let mut state = ParseState::new(StringReader::new("abcd"));
        let out = r.parse(&mut state);
        assert_eq!(out.as_deref(), Some("ab"));
        assert_eq!(state.input().get_cursor(), 2);
    }

    #[test]
    fn accepts_empty_when_min_is_zero() {
        let r = rule(0, i32::MAX);
        let mut state = ParseState::new(StringReader::new("123"));
        let out = r.parse(&mut state);
        assert_eq!(out.as_deref(), Some(""));
        assert_eq!(state.input().get_cursor(), 0);
    }

    #[test]
    fn handles_negative_min_size() {
        let r = rule(-1, i32::MAX);
        let mut state = ParseState::new(StringReader::new("123"));
        let out = r.parse(&mut state);
        assert_eq!(out.as_deref(), Some(""));
        assert!(state.error_collector().errors().is_empty());
    }

    #[test]
    fn max_size_zero_returns_empty_if_allowed() {
        let r = rule(0, 0);
        let mut state = ParseState::new(StringReader::new("abc"));
        let out = r.parse(&mut state);
        assert_eq!(out.as_deref(), Some(""));
        assert_eq!(state.input().get_cursor(), 0);
    }
}
