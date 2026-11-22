//! Port of `net.minecraft.nbt.NbtFormatException`.
//!
//! This module is self-contained: it includes a minimal local `NbtException` base
//! type to mirror the Java inheritance relationship without depending on unported
//! external Minecraft classes.

use std::error::Error;
use std::fmt;

/// Minimal local representation of `net.minecraft.nbt.NbtException`.
///
/// Defined locally so this module compiles cleanly on its own. When a shared
/// NBT exception type is ported, this can be unified with that implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtException {
    message: String,
}

impl NbtException {
    /// Creates a new `NbtException` with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the exception message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for NbtException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for NbtException {}

/// Port of `net.minecraft.nbt.NbtFormatException`.
///
/// In Java, this extends `NbtException`. In Rust, the inheritance relationship is
/// represented by composition plus conversions/accessors to the base type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtFormatException {
    base: NbtException,
}

impl NbtFormatException {
    // JAVA: net.minecraft.nbt.NbtFormatException#<init>
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            base: NbtException::new(message),
        }
    }

    /// Returns the exception message.
    pub fn message(&self) -> &str {
        self.base.message()
    }

    /// Returns a reference to the underlying base `NbtException`.
    pub fn as_base(&self) -> &NbtException {
        &self.base
    }

    /// Consumes this exception and returns the underlying base `NbtException`.
    pub fn into_base(self) -> NbtException {
        self.base
    }
}

impl fmt::Display for NbtFormatException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.base, f)
    }
}

impl Error for NbtFormatException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // NbtFormatException extends NbtException in Java; it is not caused by
        // another error.
        None
    }
}

impl From<NbtFormatException> for NbtException {
    fn from(value: NbtFormatException) -> Self {
        value.base
    }
}

impl AsRef<NbtException> for NbtFormatException {
    fn as_ref(&self) -> &NbtException {
        &self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn new_preserves_message() {
        let e = NbtFormatException::new("bad nbt format");
        assert_eq!(e.message(), "bad nbt format");
    }

    #[test]
    fn empty_message() {
        let e = NbtFormatException::new("");
        assert_eq!(e.message(), "");
        assert_eq!(e.to_string(), "");
    }

    #[test]
    fn unicode_and_control_char_message() {
        let msg = "Ошибка NBT: 日本語 \u{1F4A5}\n\t\r\u{0000} end";
        let e = NbtFormatException::new(msg);
        assert_eq!(e.message(), msg);
        assert_eq!(e.to_string(), msg);
    }

    #[test]
    fn long_message_is_preserved() {
        let msg = "x".repeat(100_000);
        let e = NbtFormatException::new(msg.clone());
        assert_eq!(e.message(), msg);
        assert_eq!(e.to_string().len(), msg.len());
    }

    #[test]
    fn display_matches_message() {
        let e = NbtFormatException::new("format error at offset 42");
        assert_eq!(format!("{}", e), "format error at offset 42");
    }

    #[test]
    fn error_source_is_none() {
        let e = NbtFormatException::new("oops");
        let src = e.source();
        assert!(src.is_none());
    }

    #[test]
    fn as_dyn_error() {
        let e = NbtFormatException::new("dyn error");
        let err: &dyn Error = &e;
        assert_eq!(err.to_string(), "dyn error");
        assert!(err.source().is_none());
    }

    #[test]
    fn clone_and_equality() {
        let a = NbtFormatException::new("same");
        let b = a.clone();
        assert_eq!(a, b);

        let c = NbtFormatException::new("different");
        assert_ne!(a, c);
    }

    #[test]
    fn debug_contains_type_and_message() {
        let e = NbtFormatException::new("debug msg");
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("NbtFormatException"));
        assert!(dbg.contains("debug msg"));
    }

    #[test]
    fn accepts_string_and_str() {
        let from_str = NbtFormatException::new("from str");
        let from_string = NbtFormatException::new(String::from("from string"));
        assert_eq!(from_str.message(), "from str");
        assert_eq!(from_string.message(), "from string");
    }

    #[test]
    fn converts_to_base_exception() {
        let e = NbtFormatException::new("base msg");
        let as_ref_base: &NbtException = e.as_ref();
        assert_eq!(as_ref_base.message(), "base msg");
        assert_eq!(e.as_base().message(), "base msg");

        let cloned = e.clone();
        let base: NbtException = cloned.into();
        assert_eq!(base.message(), "base msg");

        let consumed_base = e.into_base();
        assert_eq!(consumed_base.message(), "base msg");
    }

    #[test]
    fn base_exception_behaves_as_error() {
        let base = NbtException::new("base");
        assert_eq!(base.message(), "base");
        assert_eq!(base.to_string(), "base");
        assert!(base.source().is_none());

        let dbg = format!("{:?}", base);
        assert!(dbg.contains("NbtException"));
        assert!(dbg.contains("base"));
    }

    #[test]
    fn base_exception_clone_and_equality() {
        let a = NbtException::new("x");
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, NbtException::new("y"));
    }

    #[test]
    fn message_preserves_negative_and_overflow_integer_text() {
        let neg_i32: i32 = -1;
        let wrapped_i32: i32 = neg_i32.wrapping_add(i32::MIN);
        let neg_i64: i64 = -1;
        let wrapped_i64: i64 = neg_i64.wrapping_sub(i64::MAX);
        let wrapped_u64: u64 = u64::MAX.wrapping_add(1);

        let msg = format!(
            "i32_min={} i32_max={} wrapped_i32={} i64_min={} i64_max={} wrapped_i64={} u64_max={} wrapped_u64={}",
            i32::MIN,
            i32::MAX,
            wrapped_i32,
            i64::MIN,
            i64::MAX,
            wrapped_i64,
            u64::MAX,
            wrapped_u64
        );

        let e = NbtFormatException::new(msg.clone());
        assert_eq!(e.message(), msg);
        assert_eq!(wrapped_i32, i32::MAX);
        assert_eq!(wrapped_i64, i64::MIN);
        assert_eq!(wrapped_u64, 0u64);
        assert!(e.message().contains("-2147483648"));
        assert!(e.message().contains("9223372036854775807"));
    }

    #[test]
    fn message_preserves_float_bit_patterns() {
        let f32_nan_bits = f32::NAN.to_bits();
        let f32_neg_zero_bits = (-0.0f32).to_bits();
        let f32_pos_inf_bits = f32::INFINITY.to_bits();
        let f32_neg_inf_bits = f32::NEG_INFINITY.to_bits();

        let f64_nan_bits = f64::NAN.to_bits();
        let f64_neg_zero_bits = (-0.0f64).to_bits();
        let f64_pos_inf_bits = f64::INFINITY.to_bits();
        let f64_neg_inf_bits = f64::NEG_INFINITY.to_bits();

        let msg = format!(
            "f32_nan=0x{f32_nan_bits:08X} f32_neg_zero=0x{f32_neg_zero_bits:08X} f32_pos_inf=0x{f32_pos_inf_bits:08X} f32_neg_inf=0x{f32_neg_inf_bits:08X} \
             f64_nan=0x{f64_nan_bits:016X} f64_neg_zero=0x{f64_neg_zero_bits:016X} f64_pos_inf=0x{f64_pos_inf_bits:016X} f64_neg_inf=0x{f64_neg_inf_bits:016X}"
        );

        let e = NbtFormatException::new(msg.clone());
        assert_eq!(e.message(), msg);
        assert!(e.message().contains("0x7FC00000"));
        assert!(e.message().contains("0x80000000"));
        assert!(e.message().contains("0x8000000000000000"));
        assert!(e.message().contains("0x7FF0000000000000"));
    }
}
