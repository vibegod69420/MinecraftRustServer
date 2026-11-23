#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException;

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

    pub fn string(&self) -> &str {
        &self.string
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn can_read(&self) -> bool {
        self.cursor < self.string.len()
    }

    pub fn peek(&self) -> Option<char> {
        self.string[self.cursor..].chars().next()
    }

    pub fn skip(&mut self) {
        if let Some(c) = self.peek() {
            self.cursor += c.len_utf8();
        }
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

pub struct ParseState<S> {
    input: S,
}

impl<S> ParseState<S> {
    pub fn new(input: S) -> Self {
        Self { input }
    }

    // JAVA: net.minecraft.util.parsing.packrat.ParseState#input
    pub fn input(&mut self) -> &mut S {
        &mut self.input
    }
}

pub trait Rule<S, T> {
    // JAVA: net.minecraft.util.parsing.packrat.Rule#parse
    fn parse(&self, state: &mut ParseState<S>) -> Option<T>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    fn is_allowed_in_resource_location(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | ':' | '/')
    }

    fn parse_raw(raw: &str) -> Option<Self> {
        if raw.is_empty() {
            return None;
        }
        let (namespace, path) = match raw.split_once(':') {
            Some((ns, p)) => (ns.to_string(), p.to_string()),
            None => ("minecraft".to_string(), raw.to_string()),
        };
        if namespace.is_empty() || path.is_empty() {
            return None;
        }
        Some(Self { namespace, path })
    }

    // JAVA: net.minecraft.resources.Identifier#readNonEmpty
    pub fn read_non_empty(reader: &mut StringReader) -> Result<Self, CommandSyntaxException> {
        let start = reader.cursor();
        while let Some(c) = reader.peek() {
            if Self::is_allowed_in_resource_location(c) {
                reader.skip();
            } else {
                break;
            }
        }
        let raw = &reader.string()[start..reader.cursor()];
        if raw.is_empty() {
            return Err(CommandSyntaxException);
        }
        Self::parse_raw(raw).ok_or(CommandSyntaxException)
    }
}

pub struct IdentifierParseRule;

impl IdentifierParseRule {
    // JAVA: net.minecraft.util.parsing.packrat.commands.IdentifierParseRule#<init>
    const fn new() -> Self {
        Self
    }
}

// JAVA: net.minecraft.util.parsing.packrat.commands.IdentifierParseRule#INSTANCE
pub static INSTANCE: IdentifierParseRule = IdentifierParseRule::new();

impl Rule<StringReader, Identifier> for IdentifierParseRule {
    // JAVA: net.minecraft.util.parsing.packrat.commands.IdentifierParseRule#parse
    fn parse(&self, state: &mut ParseState<StringReader>) -> Option<Identifier> {
        state.input().skip_whitespace();
        match Identifier::read_non_empty(state.input()) {
            Ok(id) => Some(id),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_identifier() {
        let mut state = ParseState::new(StringReader::new("minecraft:stone"));
        let result = INSTANCE.parse(&mut state);
        assert_eq!(
            result,
            Some(Identifier {
                namespace: "minecraft".into(),
                path: "stone".into(),
            })
        );
    }

    #[test]
    fn parse_skips_whitespace_and_defaults_namespace() {
        let mut state = ParseState::new(StringReader::new("  stone"));
        let result = INSTANCE.parse(&mut state);
        assert_eq!(
            result,
            Some(Identifier {
                namespace: "minecraft".into(),
                path: "stone".into(),
            })
        );
    }

    #[test]
    fn parse_path_with_slash() {
        let mut state = ParseState::new(StringReader::new("foo:bar/baz"));
        let result = INSTANCE.parse(&mut state);
        assert_eq!(
            result,
            Some(Identifier {
                namespace: "foo".into(),
                path: "bar/baz".into(),
            })
        );
    }

    #[test]
    fn parse_empty_returns_none() {
        let mut state = ParseState::new(StringReader::new(""));
        assert_eq!(INSTANCE.parse(&mut state), None);
    }

    #[test]
    fn parse_invalid_returns_none() {
        let mut state = ParseState::new(StringReader::new("!!!"));
        assert_eq!(INSTANCE.parse(&mut state), None);
    }
}
