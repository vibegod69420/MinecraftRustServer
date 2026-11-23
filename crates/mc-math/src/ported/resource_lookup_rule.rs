use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub const ERROR_INVALID: &'static str = "invalid identifier";

    pub fn new(namespace: String, path: String) -> Self {
        Self { namespace, path }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

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

pub trait ResourceSuggestion {
    fn possible_resources(&self) -> Vec<Identifier> {
        Vec::new()
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

pub trait NamedRule {
    fn parse_id(&self, state: &mut ParseState) -> Option<Identifier>;
}

#[derive(Debug, Clone)]
pub struct ErrorEntry {
    pub mark: i32,
    pub message: String,
}

#[derive(Default)]
pub struct ErrorCollector {
    pub entries: Vec<ErrorEntry>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.ErrorCollector#store
    pub fn store(
        &mut self,
        mark: i32,
        _suggestion: &dyn ResourceSuggestion,
        message: String,
    ) {
        self.entries.push(ErrorEntry { mark, message });
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
            error_collector: ErrorCollector::new(),
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#input
    pub fn input(&mut self) -> &mut StringReader {
        &mut self.input
    }

    pub fn input_ref(&self) -> &StringReader {
        &self.input
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#mark
    pub fn mark(&self) -> i32 {
        self.input.cursor
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#parse
    pub fn parse_id(&mut self, rule: &dyn NamedRule) -> Option<Identifier> {
        rule.parse_id(self)
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#errorCollector
    pub fn error_collector(&mut self) -> &mut ErrorCollector {
        &mut self.error_collector
    }

    pub fn error_collector_ref(&self) -> &ErrorCollector {
        &self.error_collector
    }
}

pub struct ResourceLookupRule<C, V> {
    id_parser: Box<dyn NamedRule>,
    pub context: C,
    error: DelayedException<CommandSyntaxException>,
    validator: Box<dyn Fn(&C, &StringReader, &Identifier) -> Result<V, String>>,
}

impl<C, V> ResourceSuggestion for ResourceLookupRule<C, V> {}

impl<C, V> ResourceLookupRule<C, V> {
    // JAVA: net.minecraft.util.parsing.packrat.commands.ResourceLookupRule#<init>
    pub fn new(
        id_parser: Box<dyn NamedRule>,
        context: C,
        validator: Box<dyn Fn(&C, &StringReader, &Identifier) -> Result<V, String>>,
    ) -> Self {
        let error = DelayedException::new(|| CommandSyntaxException {
            message: Identifier::ERROR_INVALID.to_string(),
        });
        Self {
            id_parser,
            context,
            error,
            validator,
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.ResourceLookupRule#parse
    pub fn parse(&self, state: &mut ParseState) -> Option<V> {
        state.input().skip_whitespace();
        let mark = state.mark();
        let id = state.parse_id(self.id_parser.as_ref());
        if let Some(id) = id {
            let reader_snapshot = state.input_ref().clone();
            match (self.validator)(&self.context, &reader_snapshot, &id) {
                Ok(v) => Some(v),
                Err(e) => {
                    state.error_collector().store(mark, self, e);
                    None
                }
            }
        } else {
            let msg = self.error.create().message;
            state.error_collector().store(mark, self, msg);
            None
        }
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.ResourceLookupRule#validateElement
    pub fn validate_element(
        &self,
        reader: &StringReader,
        id: &Identifier,
    ) -> Result<V, String> {
        (self.validator)(&self.context, reader, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedIdRule {
        id: Option<Identifier>,
        advance: i32,
    }

    impl NamedRule for FixedIdRule {
        fn parse_id(&self, state: &mut ParseState) -> Option<Identifier> {
            if let Some(ref id) = self.id {
                let cur = state.input().get_cursor();
                state.input().set_cursor(cur.wrapping_add(self.advance));
                Some(id.clone())
            } else {
                None
            }
        }
    }

    fn make_rule(
        id: Option<Identifier>,
        valid: bool,
    ) -> ResourceLookupRule<i32, String> {
        ResourceLookupRule::new(
            Box::new(FixedIdRule { id, advance: 3 }),
            42i32,
            Box::new(move |ctx, _reader, ident| {
                if valid {
                    Ok(format!("{}@{}", ident, ctx))
                } else {
                    Err("not found".to_string())
                }
            }),
        )
    }

    #[test]
    fn parse_success_returns_validated_value() {
        let id = Identifier::new("minecraft".to_string(), "stone".to_string());
        let rule = make_rule(Some(id), true);
        let mut state = ParseState::new(StringReader::new("minecraft:stone".to_string()));
        let result = rule.parse(&mut state);
        assert_eq!(result.as_deref(), Some("minecraft:stone@42"));
        assert!(state.error_collector_ref().entries.is_empty());
    }

    #[test]
    fn parse_missing_id_stores_delayed_error() {
        let rule = make_rule(None, true);
        let mut state = ParseState::new(StringReader::new("bad".to_string()));
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector_ref().entries.len(), 1);
        assert_eq!(
            state.error_collector_ref().entries[0].message,
            Identifier::ERROR_INVALID
        );
    }

    #[test]
    fn parse_validation_failure_stores_exception() {
        let id = Identifier::new("minecraft".to_string(), "missing".to_string());
        let rule = make_rule(Some(id), false);
        let mut state = ParseState::new(StringReader::new("minecraft:missing".to_string()));
        let result = rule.parse(&mut state);
        assert!(result.is_none());
        assert_eq!(state.error_collector_ref().entries.len(), 1);
        assert_eq!(state.error_collector_ref().entries[0].message, "not found");
    }

    #[test]
    fn parse_skips_whitespace_before_mark() {
        let id = Identifier::new("a".to_string(), "b".to_string());
        let rule = ResourceLookupRule::new(
            Box::new(FixedIdRule {
                id: Some(id),
                advance: 1,
            }),
            0i32,
            Box::new(|_, reader, ident| Ok(format!("{}:{}", ident, reader.get_cursor()))),
        );
        let mut state = ParseState::new(StringReader::new("  a:b".to_string()));
        let result = rule.parse(&mut state);
        assert!(result.is_some());
        // mark was taken after skip_whitespace, so cursor was 2 at mark time
        assert_eq!(state.error_collector_ref().entries.len(), 0);
    }

    #[test]
    fn validate_element_delegates_to_validator() {
        let rule = make_rule(Some(Identifier::new("x".to_string(), "y".to_string())), true);
        let reader = StringReader::new("x:y".to_string());
        let id = Identifier::new("x".to_string(), "y".to_string());
        let v = rule.validate_element(&reader, &id).unwrap();
        assert_eq!(v, "x:y@42");
    }
}
