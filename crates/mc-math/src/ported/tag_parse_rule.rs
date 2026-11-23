use std::marker::PhantomData;

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
}

#[derive(Debug, Default, Clone)]
pub struct ErrorCollector {
    pub errors: Vec<(usize, String)>,
}

impl ErrorCollector {
    pub fn store(&mut self, mark: usize, err: impl ToString) {
        self.errors.push((mark, err.to_string()));
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

#[derive(Debug, Clone, PartialEq)]
pub struct Dynamic<T, O: Clone> {
    pub value: T,
    pub ops: O,
}

impl<T, O: Clone> Dynamic<T, O> {
    // JAVA: com.mojang.serialization.Dynamic#<init>
    pub fn new(ops: O, value: T) -> Self {
        Self { value, ops }
    }
}

pub struct TagParser<T, O: Clone> {
    ops: O,
    parse_fn: fn(&mut StringReader) -> Result<T, String>,
    _marker: PhantomData<T>,
}

impl<T, O: Clone> TagParser<T, O> {
    // JAVA: net.minecraft.nbt.TagParser#create
    pub fn create(ops: O) -> Self {
        Self {
            ops,
            parse_fn: |_| Err("no tag parser configured".into()),
            _marker: PhantomData,
        }
    }

    pub fn create_with_parser(ops: O, parse_fn: fn(&mut StringReader) -> Result<T, String>) -> Self {
        Self {
            ops,
            parse_fn,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.nbt.TagParser#getOps
    pub fn get_ops(&self) -> &O {
        &self.ops
    }

    // JAVA: net.minecraft.nbt.TagParser#parseAsArgument
    pub fn parse_as_argument(&self, reader: &mut StringReader) -> Result<T, String> {
        (self.parse_fn)(reader)
    }
}

pub struct TagParseRule<T, O: Clone> {
    parser: TagParser<T, O>,
}

impl<T, O: Clone> TagParseRule<T, O> {
    // JAVA: net.minecraft.util.parsing.packrat.commands.TagParseRule#<init>
    pub fn new(ops: O) -> Self {
        Self {
            parser: TagParser::create(ops),
        }
    }

    pub fn with_parser(ops: O, parse_fn: fn(&mut StringReader) -> Result<T, String>) -> Self {
        Self {
            parser: TagParser::create_with_parser(ops, parse_fn),
        }
    }
}

impl<T, O: Clone> Rule<StringReader, Dynamic<T, O>> for TagParseRule<T, O> {
    // JAVA: net.minecraft.util.parsing.packrat.commands.TagParseRule#parse
    fn parse(&self, state: &mut ParseState) -> Option<Dynamic<T, O>> {
        state.input().skip_whitespace();
        let mark = state.mark();
        match self.parser.parse_as_argument(state.input()) {
            Ok(value) => Some(Dynamic::new(self.parser.get_ops().clone(), value)),
            Err(err) => {
                state.error_collector().store(mark, err);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct DummyOps;

    fn parse_ascii_int(reader: &mut StringReader) -> Result<i32, String> {
        let mut s = String::new();
        while let Some(c) = reader.peek() {
            if c.is_ascii_digit() || (s.is_empty() && c == '-') {
                s.push(c);
                reader.cursor += c.len_utf8();
            } else {
                break;
            }
        }
        if s.is_empty() {
            return Err("not an int".into());
        }
        s.parse::<i32>().map_err(|e| e.to_string())
    }

    #[test]
    fn parse_success_returns_dynamic() {
        let rule = TagParseRule::with_parser(DummyOps, parse_ascii_int);
        let mut state = ParseState::new(StringReader::new("42"));
        let result = rule.parse(&mut state);
        assert_eq!(
            result,
            Some(Dynamic {
                value: 42,
                ops: DummyOps
            })
        );
    }

    #[test]
    fn parse_skips_whitespace() {
        let rule = TagParseRule::with_parser(DummyOps, parse_ascii_int);
        let mut state = ParseState::new(StringReader::new("  -7"));
        let result = rule.parse(&mut state);
        assert_eq!(result.unwrap().value, -7);
    }

    #[test]
    fn parse_error_stores_mark_and_returns_none() {
        let rule = TagParseRule::with_parser(DummyOps, parse_ascii_int);
        let mut state = ParseState::new(StringReader::new("abc"));
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector().errors.len(), 1);
        assert_eq!(state.error_collector().errors[0].0, 0);
    }

    #[test]
    fn parse_error_mark_is_after_whitespace() {
        let rule = TagParseRule::with_parser(DummyOps, parse_ascii_int);
        let mut state = ParseState::new(StringReader::new("  abc"));
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector().errors[0].0, 2);
    }

    #[test]
    fn default_parser_reports_error() {
        let rule: TagParseRule<i32, DummyOps> = TagParseRule::new(DummyOps);
        let mut state = ParseState::new(StringReader::new("1"));
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector().errors.len(), 1);
    }
}
