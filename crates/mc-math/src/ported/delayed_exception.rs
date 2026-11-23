//! Faithful port of `net.minecraft.util.parsing.packrat.DelayedException`.

use std::fmt;

/// Minimal self-contained stub for Brigadier's `CommandSyntaxException`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException {
    pub message: String,
    pub input: String,
    pub cursor: i32,
}

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}: {}", self.message, self.cursor, self.input)
    }
}

impl std::error::Error for CommandSyntaxException {}

/// Minimal self-contained stub for `StringReader`/`StringReaderTerms` reader context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringReader {
    pub input: String,
    pub cursor: i32,
}

/// Minimal self-contained stub for `net.minecraft.util.parsing.packrat.commands.StringReaderTerms`.
pub struct StringReaderTerms;

impl StringReaderTerms {
    // JAVA: net.minecraft.util.parsing.packrat.commands.StringReaderTerms#createReader
    pub fn create_reader(input: &str, cursor: i32) -> StringReader {
        StringReader {
            input: input.to_string(),
            cursor,
        }
    }
}

/// Minimal self-contained stub for Brigadier's `SimpleCommandExceptionType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCommandExceptionType {
    pub message: String,
}

impl SimpleCommandExceptionType {
    // JAVA: com.mojang.brigadier.exceptions.SimpleCommandExceptionType#createWithContext
    pub fn create_with_context(&self, reader: StringReader) -> CommandSyntaxException {
        CommandSyntaxException {
            message: self.message.clone(),
            input: reader.input,
            cursor: reader.cursor,
        }
    }
}

/// Minimal self-contained stub for Brigadier's `DynamicCommandExceptionType`.
#[derive(Debug, Clone)]
pub struct DynamicCommandExceptionType {
    pub message_prefix: String,
}

impl DynamicCommandExceptionType {
    // JAVA: com.mojang.brigadier.exceptions.DynamicCommandExceptionType#createWithContext
    pub fn create_with_context(&self, reader: StringReader, arg: &str) -> CommandSyntaxException {
        CommandSyntaxException {
            message: format!("{}{}", self.message_prefix, arg),
            input: reader.input,
            cursor: reader.cursor,
        }
    }
}

/// Port of `net.minecraft.util.parsing.packrat.DelayedException`.
pub trait DelayedException<T> {
    // JAVA: net.minecraft.util.parsing.packrat.DelayedException#create
    fn create(&self, input: &str, cursor: i32) -> T;
}

impl<F, T> DelayedException<T> for F
where
    F: Fn(&str, i32) -> T,
{
    // JAVA: net.minecraft.util.parsing.packrat.DelayedException#create
    fn create(&self, input: &str, cursor: i32) -> T {
        self(input, cursor)
    }
}

// JAVA: net.minecraft.util.parsing.packrat.DelayedException#create
pub fn create_simple(
    exception_type: SimpleCommandExceptionType,
) -> impl DelayedException<CommandSyntaxException> {
    move |input: &str, cursor: i32| {
        exception_type.create_with_context(StringReaderTerms::create_reader(input, cursor))
    }
}

// JAVA: net.minecraft.util.parsing.packrat.DelayedException#create
pub fn create_dynamic(
    exception_type: DynamicCommandExceptionType,
    arg: String,
) -> impl DelayedException<CommandSyntaxException> {
    move |input: &str, cursor: i32| {
        exception_type.create_with_context(StringReaderTerms::create_reader(input, cursor), &arg)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        create_dynamic, create_simple, CommandSyntaxException, DelayedException,
        DynamicCommandExceptionType, SimpleCommandExceptionType, StringReaderTerms,
    };

    #[test]
    fn closure_impl_creates_exception() {
        let delayed = |input: &str, cursor: i32| CommandSyntaxException {
            message: "err".to_string(),
            input: input.to_string(),
            cursor,
        };
        let ex = delayed.create("abc", 2);
        assert_eq!(ex.message, "err");
        assert_eq!(ex.input, "abc");
        assert_eq!(ex.cursor, 2);
    }

    #[test]
    fn simple_factory_uses_reader_context() {
        let delayed = create_simple(SimpleCommandExceptionType {
            message: "simple".to_string(),
        });
        let ex = delayed.create("hello world", 6);
        assert_eq!(ex.message, "simple");
        assert_eq!(ex.input, "hello world");
        assert_eq!(ex.cursor, 6);
    }

    #[test]
    fn dynamic_factory_includes_arg() {
        let delayed = create_dynamic(
            DynamicCommandExceptionType {
                message_prefix: "bad: ".to_string(),
            },
            "token".to_string(),
        );
        let ex = delayed.create("input", 0);
        assert_eq!(ex.message, "bad: token");
        assert_eq!(ex.input, "input");
        assert_eq!(ex.cursor, 0);
    }

    #[test]
    fn handles_negative_cursor_and_empty_input() {
        let delayed = create_simple(SimpleCommandExceptionType {
            message: "neg".to_string(),
        });
        let ex = delayed.create("", -1);
        assert_eq!(ex.input, "");
        assert_eq!(ex.cursor, -1);
        assert_eq!(ex.message, "neg");
    }

    #[test]
    fn string_reader_terms_create_reader_preserves_values() {
        let reader = StringReaderTerms::create_reader("test", i32::MIN);
        assert_eq!(reader.input, "test");
        assert_eq!(reader.cursor, i32::MIN);
    }

    #[test]
    fn display_and_error_impl() {
        let ex = CommandSyntaxException {
            message: "m".to_string(),
            input: "in".to_string(),
            cursor: 3,
        };
        let s = ex.to_string();
        assert!(s.contains("m"));
        assert!(s.contains('3'));
        let _e: &dyn std::error::Error = &ex;
    }
}
