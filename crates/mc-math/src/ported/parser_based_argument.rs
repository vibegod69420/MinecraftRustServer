use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException(pub String);

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

    pub fn remaining(&self) -> &str {
        &self.string[self.cursor..]
    }

    pub fn read_remaining(&mut self) -> String {
        let rem = self.remaining().to_string();
        self.cursor = self.string.len();
        rem
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Suggestions;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SuggestionsBuilder;

#[derive(Debug, Clone, Default)]
pub struct CommandContext<S> {
    _marker: PhantomData<S>,
}

impl<S> CommandContext<S> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletableFuture<T>(pub T);

impl<T> CompletableFuture<T> {
    pub fn completed_future(value: T) -> Self {
        Self(value)
    }
}

pub trait CommandArgumentParser<T> {
    // JAVA: net.minecraft.util.parsing.packrat.commands.CommandArgumentParser#parseForCommands
    fn parse_for_commands(&self, reader: &mut StringReader) -> Result<T, CommandSyntaxException>;

    // JAVA: net.minecraft.util.parsing.packrat.commands.CommandArgumentParser#parseForSuggestions
    fn parse_for_suggestions(&self, builder: SuggestionsBuilder) -> CompletableFuture<Suggestions>;
}

pub trait ArgumentType<T> {
    fn parse(&mut self, reader: &mut StringReader) -> Result<T, CommandSyntaxException>;
    fn list_suggestions<S>(
        &self,
        context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> CompletableFuture<Suggestions>;
}

pub struct ParserBasedArgument<T, P: CommandArgumentParser<T>> {
    parser: P,
    _marker: PhantomData<T>,
}

impl<T, P: CommandArgumentParser<T>> ParserBasedArgument<T, P> {
    // JAVA: net.minecraft.util.parsing.packrat.commands.ParserBasedArgument#<init>
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _marker: PhantomData,
        }
    }
}

impl<T, P: CommandArgumentParser<T>> ArgumentType<T> for ParserBasedArgument<T, P> {
    // JAVA: net.minecraft.util.parsing.packrat.commands.ParserBasedArgument#parse
    fn parse(&mut self, reader: &mut StringReader) -> Result<T, CommandSyntaxException> {
        self.parser.parse_for_commands(reader)
    }

    // JAVA: net.minecraft.util.parsing.packrat.commands.ParserBasedArgument#listSuggestions
    fn list_suggestions<S>(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> CompletableFuture<Suggestions> {
        self.parser.parse_for_suggestions(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IntParser;

    impl CommandArgumentParser<i32> for IntParser {
        fn parse_for_commands(
            &self,
            reader: &mut StringReader,
        ) -> Result<i32, CommandSyntaxException> {
            let s = reader.read_remaining();
            s.trim()
                .parse::<i32>()
                .map_err(|_| CommandSyntaxException("bad int".into()))
        }

        fn parse_for_suggestions(
            &self,
            _builder: SuggestionsBuilder,
        ) -> CompletableFuture<Suggestions> {
            CompletableFuture::completed_future(Suggestions)
        }
    }

    #[test]
    fn parse_delegates_to_parser() {
        let mut arg = ParserBasedArgument::new(IntParser);
        let mut reader = StringReader::new("42");
        assert_eq!(arg.parse(&mut reader).unwrap(), 42);
    }

    #[test]
    fn parse_propagates_error() {
        let mut arg = ParserBasedArgument::new(IntParser);
        let mut reader = StringReader::new("nope");
        assert_eq!(
            arg.parse(&mut reader).unwrap_err(),
            CommandSyntaxException("bad int".into())
        );
    }

    #[test]
    fn list_suggestions_delegates_to_parser() {
        let arg = ParserBasedArgument::new(IntParser);
        let ctx: CommandContext<()> = CommandContext::new();
        let fut = arg.list_suggestions(&ctx, SuggestionsBuilder);
        assert_eq!(fut, CompletableFuture(Suggestions));
    }

    #[test]
    fn parse_negative_integer() {
        let mut arg = ParserBasedArgument::new(IntParser);
        let mut reader = StringReader::new("-123");
        assert_eq!(arg.parse(&mut reader).unwrap(), -123);
    }
}
