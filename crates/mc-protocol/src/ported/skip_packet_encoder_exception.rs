use std::error::Error;
use std::fmt;

/// Marker for `net.minecraft.network.codec.IdDispatchCodec.DontDecorateException`.
pub trait DontDecorateException {}

/// Marker for `net.minecraft.network.SkipPacketException`.
pub trait SkipPacketException {}

#[derive(Debug)]
pub struct SkipPacketEncoderException {
    message: String,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl DontDecorateException for SkipPacketEncoderException {}
impl SkipPacketException for SkipPacketEncoderException {}

impl SkipPacketEncoderException {
    // JAVA: net.minecraft.network.SkipPacketEncoderException#<init>(String)
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
        }
    }

    // JAVA: net.minecraft.network.SkipPacketEncoderException#<init>(Throwable)
    pub fn with_cause(cause: impl Error + Send + Sync + 'static) -> Self {
        let message = cause.to_string();
        Self {
            message,
            cause: Some(Box::new(cause)),
        }
    }
}

impl fmt::Display for SkipPacketEncoderException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for SkipPacketEncoderException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_ref()
            .map(|c| &**c as &(dyn Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct DummyCause(&'static str);

    impl fmt::Display for DummyCause {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl Error for DummyCause {}

    #[test]
    fn new_sets_message_and_no_source() {
        let e = SkipPacketEncoderException::new("encode skip");
        assert_eq!(e.to_string(), "encode skip");
        assert!(e.source().is_none());
    }

    #[test]
    fn with_cause_sets_source_and_message() {
        let e = SkipPacketEncoderException::with_cause(DummyCause("encoder root"));
        assert_eq!(e.to_string(), "encoder root");
        let src = e.source().expect("expected source");
        assert_eq!(src.to_string(), "encoder root");
    }

    #[test]
    fn implements_marker_traits() {
        fn assert_markers<T: DontDecorateException + SkipPacketException>() {}
        assert_markers::<SkipPacketEncoderException>();
    }

    #[test]
    fn new_with_empty_message() {
        let e = SkipPacketEncoderException::new("");
        assert_eq!(e.to_string(), "");
        assert!(e.source().is_none());
    }
}
