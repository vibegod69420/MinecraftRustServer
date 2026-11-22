use std::error::Error;
use std::fmt;

/// Faithful port of `net.minecraft.util.CryptException`.
///
/// In Minecraft Java Edition, this exception wraps a throwable cause produced
/// by cryptographic operations.
#[derive(Debug)]
pub struct CryptException {
    cause: Box<dyn Error + Send + Sync + 'static>,
}

impl CryptException {
    // JAVA: net.minecraft.util.CryptException#<init>
    pub fn new(cause: impl Into<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        Self {
            cause: cause.into(),
        }
    }

    /// Returns the wrapped cause of this cryptographic exception.
    pub fn cause(&self) -> &(dyn Error + 'static) {
        self.cause.as_ref()
    }
}

impl fmt::Display for CryptException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl Error for CryptException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt;
    use std::io;

    #[derive(Debug)]
    struct SimpleError {
        message: String,
    }

    impl SimpleError {
        fn new(message: &str) -> Self {
            Self {
                message: message.to_string(),
            }
        }
    }

    impl fmt::Display for SimpleError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.message)
        }
    }

    impl Error for SimpleError {}

    #[derive(Debug)]
    struct ChainError {
        message: &'static str,
        source: Option<Box<dyn Error + Send + Sync + 'static>>,
    }

    impl fmt::Display for ChainError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.message)
        }
    }

    impl Error for ChainError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn Error + 'static))
        }
    }

    #[test]
    fn constructor_wraps_cause_display() {
        let err = CryptException::new(SimpleError::new("bad padding"));
        assert_eq!(err.to_string(), "bad padding");
    }

    #[test]
    fn source_returns_original_cause() {
        let err = CryptException::new(SimpleError::new("bad key"));
        let source = err.source().expect("source should exist");
        assert_eq!(source.to_string(), "bad key");
    }

    #[test]
    fn cause_accessor_returns_original_cause() {
        let err = CryptException::new(SimpleError::new("bad signature"));
        assert_eq!(err.cause().to_string(), "bad signature");
    }

    #[test]
    fn debug_contains_type_name_and_cause() {
        let err = CryptException::new(SimpleError::new("debug-cause"));
        let dbg = format!("{:?}", err);
        assert!(dbg.contains("CryptException"));
        assert!(dbg.contains("debug-cause"));
    }

    #[test]
    fn works_with_io_error() {
        let io_err = io::Error::new(io::ErrorKind::InvalidData, "invalid data");
        let err = CryptException::new(io_err);
        assert_eq!(err.to_string(), "invalid data");
        assert!(err.source().is_some());
    }

    #[test]
    fn works_with_boxed_dyn_error() {
        let boxed: Box<dyn Error + Send + Sync + 'static> =
            Box::new(SimpleError::new("boxed cause"));
        let err = CryptException::new(boxed);
        assert_eq!(err.to_string(), "boxed cause");
        assert_eq!(err.cause().to_string(), "boxed cause");
    }

    #[test]
    fn is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CryptException>();
    }

    #[test]
    fn can_be_used_as_dyn_error() {
        let err = CryptException::new(SimpleError::new("dyn error"));
        let dyn_err: &dyn Error = &err;
        assert_eq!(dyn_err.to_string(), "dyn error");
        assert!(dyn_err.source().is_some());
    }

    #[test]
    fn can_be_boxed_as_dyn_error() {
        let err = CryptException::new(SimpleError::new("boxed dyn"));
        let boxed: Box<dyn Error + Send + Sync> = Box::new(err);
        assert_eq!(boxed.to_string(), "boxed dyn");
        assert!(boxed.source().is_some());
    }

    #[test]
    fn nested_crypt_exception_preserves_message() {
        let inner = CryptException::new(SimpleError::new("inner cause"));
        let outer = CryptException::new(inner);
        assert_eq!(outer.to_string(), "inner cause");
        let source = outer.source().expect("outer source should exist");
        assert_eq!(source.to_string(), "inner cause");
    }

    #[test]
    fn preserves_cause_chain() {
        let root = ChainError {
            message: "root",
            source: None,
        };
        let mid = ChainError {
            message: "mid",
            source: Some(Box::new(root)),
        };
        let err = CryptException::new(mid);

        assert_eq!(err.to_string(), "mid");
        let wrapped = err.source().expect("wrapped cause should exist");
        assert_eq!(wrapped.to_string(), "mid");
        let inner = wrapped.source().expect("inner cause should exist");
        assert_eq!(inner.to_string(), "root");
        assert!(inner.source().is_none());
    }

    #[test]
    fn empty_message_cause() {
        let err = CryptException::new(SimpleError::new(""));
        assert_eq!(err.to_string(), "");
        assert_eq!(err.cause().to_string(), "");
        assert!(err.source().is_some());
    }

    #[test]
    fn unicode_message_cause() {
        let msg = "clé invalide 🔐 ошибка";
        let err = CryptException::new(SimpleError::new(msg));
        assert_eq!(err.to_string(), msg);
        assert_eq!(err.cause().to_string(), msg);
    }

    #[test]
    fn long_message_cause() {
        let msg = "x".repeat(10_000);
        let err = CryptException::new(SimpleError::new(&msg));
        assert_eq!(err.to_string(), msg);
        assert_eq!(err.cause().to_string(), msg);
    }

    #[test]
    fn message_with_negative_numbers_and_special_values() {
        // Edge case coverage: negative inputs and numeric formatting in cause messages.
        let msg = format!(
            "overflow={} underflow={} min_i32={} max_i32={} min_i64={}",
            -1_i32,
            i32::wrapping_sub(i32::MIN, 1),
            i32::MIN,
            i32::MAX,
            i64::MIN
        );
        let err = CryptException::new(SimpleError::new(&msg));
        assert_eq!(err.to_string(), msg);
        assert!(err.to_string().contains("-1"));
        assert!(err.to_string().contains(&i32::MIN.to_string()));
    }

    #[test]
    fn message_with_float_bit_patterns() {
        // Edge case coverage: float bit patterns represented in cause messages.
        let nan_bits = f32::NAN.to_bits();
        let neg_zero_bits = (-0.0_f32).to_bits();
        let inf_bits = f32::INFINITY.to_bits();
        let neg_inf_bits = f32::NEG_INFINITY.to_bits();
        let msg = format!(
            "nan_bits=0x{nan_bits:08x} neg_zero_bits=0x{neg_zero_bits:08x} \
             inf_bits=0x{inf_bits:08x} neg_inf_bits=0x{neg_inf_bits:08x}"
        );
        let err = CryptException::new(SimpleError::new(&msg));
        assert_eq!(err.to_string(), msg);
        assert!(err.to_string().contains("0x7fc00000") || err.to_string().contains("nan_bits=0x"));
        assert!(err.to_string().contains("0x80000000"));
    }
}
