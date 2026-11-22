use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct TranslatableFormatException {
    message: String,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl TranslatableFormatException {
    // JAVA: net.minecraft.network.chat.contents.TranslatableFormatException#<init>
    pub fn new_parse_error(contents: &impl fmt::Display, detail: &str) -> Self {
        Self {
            message: format!("Error parsing: {}: {}", contents, detail),
            cause: None,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableFormatException#<init>
    pub fn new_invalid_index(contents: &impl fmt::Display, index: i32) -> Self {
        Self {
            message: format!("Invalid index {} requested for {}", index, contents),
            cause: None,
        }
    }

    // JAVA: net.minecraft.network.chat.contents.TranslatableFormatException#<init>
    pub fn new_with_cause(
        contents: &impl fmt::Display,
        cause: Box<dyn Error + Send + Sync>,
    ) -> Self {
        Self {
            message: format!("Error while parsing: {}", contents),
            cause: Some(cause),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn cause(&self) -> Option<&(dyn Error + Send + Sync + 'static)> {
        self.cause.as_deref()
    }
}

impl fmt::Display for TranslatableFormatException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for TranslatableFormatException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|e| e as &(dyn Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct DummyContents(&'static str);

    impl fmt::Display for DummyContents {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    #[derive(Debug)]
    struct SimpleError(&'static str);

    impl fmt::Display for SimpleError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl Error for SimpleError {}

    #[test]
    fn parse_error_formats_message() {
        let err = TranslatableFormatException::new_parse_error(
            &DummyContents("translation.key"),
            "bad format",
        );
        assert_eq!(err.message(), "Error parsing: translation.key: bad format");
        assert_eq!(err.to_string(), "Error parsing: translation.key: bad format");
        assert!(err.source().is_none());
        assert!(err.cause().is_none());
    }

    #[test]
    fn invalid_index_formats_message() {
        let err = TranslatableFormatException::new_invalid_index(&DummyContents("key"), 5);
        assert_eq!(err.message(), "Invalid index 5 requested for key");
        assert!(err.source().is_none());
    }

    #[test]
    fn invalid_index_handles_negative_and_extreme_values() {
        let neg = TranslatableFormatException::new_invalid_index(&DummyContents("k"), -1);
        assert_eq!(neg.message(), "Invalid index -1 requested for k");

        let min = TranslatableFormatException::new_invalid_index(&DummyContents("k"), i32::MIN);
        assert_eq!(
            min.message(),
            format!("Invalid index {} requested for k", i32::MIN)
        );

        let max = TranslatableFormatException::new_invalid_index(&DummyContents("k"), i32::MAX);
        assert_eq!(
            max.message(),
            format!("Invalid index {} requested for k", i32::MAX)
        );
    }

    #[test]
    fn with_cause_formats_message_and_preserves_source() {
        let cause = SimpleError("inner");
        let err = TranslatableFormatException::new_with_cause(
            &DummyContents("translation.key"),
            Box::new(cause),
        );
        assert_eq!(err.message(), "Error while parsing: translation.key");
        let source = err.source().expect("source should be present");
        assert_eq!(source.to_string(), "inner");
        let direct_cause = err.cause().expect("cause should be present");
        assert_eq!(direct_cause.to_string(), "inner");
    }
}
