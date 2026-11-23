use std::marker::PhantomData;

/// Minimal stub for `com.mojang.brigadier.StringReader`.
#[derive(Debug, Clone)]
pub struct StringReader {
    input: String,
    cursor: usize,
}

impl StringReader {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.input.len());
    }

    pub fn get_string(&self) -> &str {
        &self.input
    }

    pub fn skip_whitespace(&mut self) {
        let bytes = self.input.as_bytes();
        while self.cursor < bytes.len() {
            match bytes[self.cursor] {
                b' ' | b'\t' | b'\n' | b'\r' => self.cursor += 1,
                _ => break,
            }
        }
    }
}

/// Minimal stub for `net.minecraft.util.parsing.packrat.ErrorCollector`.
#[derive(Debug, Clone)]
pub struct ErrorCollector<I> {
    _phantom: PhantomData<I>,
    pub errors: Vec<(i32, String)>,
}

impl<I> ErrorCollector<I> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
            errors: Vec::new(),
        }
    }

    pub fn store(&mut self, pos: i32, err: impl ToString) {
        self.errors.push((pos, err.to_string()));
    }
}

impl<I> Default for ErrorCollector<I> {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal stub for `net.minecraft.util.parsing.packrat.CachedParseState`.
pub struct CachedParseState<I> {
    error_collector: ErrorCollector<I>,
}

impl<I> CachedParseState<I> {
    pub fn new(error_collector: ErrorCollector<I>) -> Self {
        Self { error_collector }
    }

    pub fn error_collector(&mut self) -> &mut ErrorCollector<I> {
        &mut self.error_collector
    }
}

/// Minimal stub for `net.minecraft.util.parsing.packrat.ParseState`.
pub trait ParseState<I> {
    fn input(&mut self) -> &mut I;
    fn mark(&mut self) -> i32;
    fn restore(&mut self, cursor: i32);
}

/// Port of `net.minecraft.util.parsing.packrat.commands.StringReaderParserState`.
pub struct StringReaderParserState {
    base: CachedParseState<StringReader>,
    input: StringReader,
}

impl StringReaderParserState {
    // JAVA: net.minecraft.util.parsing.packrat.commands.StringReaderParserState#<init>
    pub fn new(error_collector: ErrorCollector<StringReader>, input: StringReader) -> Self {
        Self {
            base: CachedParseState::new(error_collector),
            input,
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.StringReaderParserState#input
    pub fn input(&mut self) -> &mut StringReader {
        &mut self.input
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.StringReaderParserState#mark
    pub fn mark(&mut self) -> i32 {
        self.input.get_cursor() as i32
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.StringReaderParserState#restore
    pub fn restore(&mut self, cursor: i32) {
        let clamped = if cursor < 0 { 0usize } else { cursor as usize };
        self.input.set_cursor(clamped);
    }

    pub fn error_collector(&mut self) -> &mut ErrorCollector<StringReader> {
        self.base.error_collector()
    }
}

impl ParseState<StringReader> for StringReaderParserState {
    fn input(&mut self) -> &mut StringReader {
        StringReaderParserState::input(self)
    }

    fn mark(&mut self) -> i32 {
        StringReaderParserState::mark(self)
    }

    fn restore(&mut self, cursor: i32) {
        StringReaderParserState::restore(self, cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_and_input_access() {
        let reader = StringReader::new("hello");
        let mut state = StringReaderParserState::new(ErrorCollector::new(), reader);
        assert_eq!(state.input().get_string(), "hello");
        assert_eq!(state.input().get_cursor(), 0);
    }

    #[test]
    fn mark_and_restore_round_trip() {
        let mut reader = StringReader::new("abcdef");
        reader.set_cursor(3);
        let mut state = StringReaderParserState::new(ErrorCollector::new(), reader);
        let mark = state.mark();
        assert_eq!(mark, 3);
        state.input().set_cursor(5);
        assert_eq!(state.mark(), 5);
        state.restore(mark);
        assert_eq!(state.mark(), 3);
        assert_eq!(state.input().get_cursor(), 3);
    }

    #[test]
    fn restore_negative_clamps_to_zero() {
        let mut reader = StringReader::new("abc");
        reader.set_cursor(2);
        let mut state = StringReaderParserState::new(ErrorCollector::new(), reader);
        state.restore(-1);
        assert_eq!(state.mark(), 0);
        assert_eq!(state.input().get_cursor(), 0);
    }

    #[test]
    fn error_collector_is_accessible() {
        let mut ec = ErrorCollector::new();
        ec.store(1, "preexisting");
        let mut state = StringReaderParserState::new(ec, StringReader::new("x"));
        state.error_collector().store(2, "new");
        assert_eq!(state.error_collector().errors.len(), 2);
        assert_eq!(state.error_collector().errors[0], (1, "preexisting".to_string()));
        assert_eq!(state.error_collector().errors[1], (2, "new".to_string()));
    }

    #[test]
    fn parse_state_trait_methods_delegate() {
        let mut state = StringReaderParserState::new(ErrorCollector::new(), StringReader::new("test"));
        let s: &mut dyn ParseState<StringReader> = &mut state;
        s.input().set_cursor(2);
        let m = s.mark();
        assert_eq!(m, 2);
        s.input().set_cursor(4);
        s.restore(m);
        assert_eq!(s.mark(), 2);
    }
}
