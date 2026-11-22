use std::error::Error;
use std::fmt;

/// Port of `net.minecraft.nbt.NbtException`.
///
/// Java declares this as `public class NbtException extends RuntimeException`
/// with a single `(String)` constructor. In Rust, this is represented as a
/// concrete error type that preserves the message payload and implements
/// `std::error::Error` + `Display`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtException {
    message: String,
}

impl NbtException {
    // JAVA: net.minecraft.nbt.NbtException#<init>
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the exception message, equivalent to Java `Throwable#getMessage()`.
    ///
    /// This is an ergonomic Rust accessor; it is not a separately declared
    /// method on `net.minecraft.nbt.NbtException` in Java.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    fn assert_error_traits<T: Error + Send + Sync + 'static>() {}

    #[test]
    fn implements_std_error_traits() {
        assert_error_traits::<NbtException>();
    }

    #[test]
    fn new_and_message_round_trip() {
        let e = NbtException::new("bad nbt");
        assert_eq!(e.message(), "bad nbt");
        assert_eq!(e.to_string(), "bad nbt");
    }

    #[test]
    fn new_accepts_string_and_str() {
        let from_str = NbtException::new("str");
        let from_string = NbtException::new(String::from("owned"));
        assert_eq!(from_str.message(), "str");
        assert_eq!(from_string.message(), "owned");
    }

    #[test]
    fn empty_message() {
        let e = NbtException::new("");
        assert_eq!(e.message(), "");
        assert_eq!(e.to_string(), "");
        assert_eq!(format!("{e}"), "");
    }

    #[test]
    fn unicode_message() {
        let msg = "NBT エラー 🧱 \u{202E}rtl\u{202C}";
        let e = NbtException::new(msg);
        assert_eq!(e.message(), msg);
        assert_eq!(e.to_string(), msg);
    }

    #[test]
    fn message_with_interior_nul_and_controls() {
        let msg = "a\0b\nc\td\r";
        let e = NbtException::new(msg);
        assert_eq!(e.message(), msg);
        assert_eq!(e.to_string(), msg);
    }

    #[test]
    fn display_matches_message() {
        let msg = "Unexpected tag type";
        let e = NbtException::new(msg);
        assert_eq!(format!("{}", e), msg);
        assert_eq!(format!("{e}"), msg);
    }

    #[test]
    fn error_source_is_none() {
        let e = NbtException::new("x");
        assert!(e.source().is_none());
    }

    #[test]
    fn as_dyn_std_error() {
        let e: Box<dyn Error> = Box::new(NbtException::new("oops"));
        assert_eq!(e.to_string(), "oops");
        assert!(e.source().is_none());
    }

    #[test]
    fn clone_and_eq() {
        let a = NbtException::new("same");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn different_messages_not_equal() {
        let a = NbtException::new("a");
        let b = NbtException::new("b");
        assert_ne!(a, b);
    }

    #[test]
    fn debug_contains_type_and_message() {
        let e = NbtException::new("detail");
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("NbtException"));
        assert!(dbg.contains("detail"));
    }

    #[test]
    fn long_message() {
        let msg = "x".repeat(100_000);
        let e = NbtException::new(msg.clone());
        assert_eq!(e.message().len(), 100_000);
        assert_eq!(e.message(), msg);
        assert_eq!(e.to_string(), msg);
    }

    #[test]
    fn negative_inputs_in_message() {
        let values: &[i64] = &[
            i64::from(i32::MIN),
            -1,
            -128,
            -32768,
            i64::MIN,
            -9_223_372_036_854_775_807,
        ];
        for &v in values {
            let e = NbtException::new(format!("value={v}"));
            assert_eq!(e.message(), format!("value={v}"));
        }
    }

    #[test]
    fn overflow_integer_semantics_in_message() {
        // Java integer arithmetic wraps; verify wrapped values are preserved faithfully.
        let i32_add = i32::MAX.wrapping_add(1);
        let i32_sub = i32::MIN.wrapping_sub(1);
        let i32_mul = 1_000_000i32.wrapping_mul(3_000);
        assert_eq!(i32_add, i32::MIN);
        assert_eq!(i32_sub, i32::MAX);

        let i64_add = i64::MAX.wrapping_add(1);
        let i64_sub = i64::MIN.wrapping_sub(1);
        let i64_mul = 3_000_000_000i64.wrapping_mul(3_000_000_000i64);
        assert_eq!(i64_add, i64::MIN);
        assert_eq!(i64_sub, i64::MAX);

        let e = NbtException::new(format!(
            "i32_add={i32_add},i32_sub={i32_sub},i32_mul={i32_mul},i64_add={i64_add},i64_sub={i64_sub},i64_mul={i64_mul}"
        ));
        assert_eq!(
            e.message(),
            format!(
                "i32_add={i32_add},i32_sub={i32_sub},i32_mul={i32_mul},i64_add={i64_add},i64_sub={i64_sub},i64_mul={i64_mul}"
            )
        );
    }

    #[test]
    fn masked_shift_values_in_message() {
        // Java shift counts are masked: & 31 for int, & 63 for long.
        let int_shl = 1i32.wrapping_shl(36u32 & 31); // 36 & 31 == 4
        let int_shr = (-2i32).wrapping_shr(33u32 & 31); // arithmetic shift, 33 & 31 == 1
        let int_shl_neg_count = 1i32.wrapping_shl((-1i32 as u32) & 31); // -1 & 31 == 31

        let long_shl = 1i64.wrapping_shl(68u32 & 63); // 68 & 63 == 4
        let long_shr = (-2i64).wrapping_shr(65u32 & 63); // 65 & 63 == 1
        let long_shl_neg_count = 1i64.wrapping_shl((-1i64 as u32) & 63); // -1 & 63 == 63

        assert_eq!(int_shl, 16);
        assert_eq!(int_shr, -1);
        assert_eq!(int_shl_neg_count, i32::MIN);
        assert_eq!(long_shl, 16);
        assert_eq!(long_shr, -1);
        assert_eq!(long_shl_neg_count, i64::MIN);

        let e = NbtException::new(format!(
            "int_shl={int_shl},int_shr={int_shr},int_shl_neg_count={int_shl_neg_count},long_shl={long_shl},long_shr={long_shr},long_shl_neg_count={long_shl_neg_count}"
        ));
        assert_eq!(
            e.message(),
            "int_shl=16,int_shr=-1,int_shl_neg_count=-2147483648,long_shl=16,long_shr=-1,long_shl_neg_count=-9223372036854775808"
        );
    }

    #[test]
    fn float_bits_in_message() {
        let f32_cases: &[(&str, f32, u32)] = &[
            ("pos_zero", 0.0f32, 0x0000_0000),
            ("neg_zero", -0.0f32, 0x8000_0000),
            ("one", 1.0f32, 0x3f80_0000),
            ("neg_one", -1.0f32, 0xbf80_0000),
            ("inf", f32::INFINITY, 0x7f80_0000),
            ("neg_inf", f32::NEG_INFINITY, 0xff80_0000),
            ("nan", f32::NAN, 0x7fc0_0000),
            ("min_pos_subnormal", f32::from_bits(0x0000_0001), 0x0000_0001),
            ("max_neg_subnormal", f32::from_bits(0x8000_0001), 0x8000_0001),
        ];

        for (name, value, expected_bits) in f32_cases {
            let bits = value.to_bits();
            if *name == "nan" {
                // Rust's f32::NAN bit pattern is not guaranteed to be a specific quiet NaN payload,
                // but it must be a NaN.
                assert!(value.is_nan());
                assert_eq!(bits & 0x7f80_0000, 0x7f80_0000);
                assert_ne!(bits & 0x007f_ffff, 0);
            } else {
                assert_eq!(bits, *expected_bits, "unexpected f32 bits for {name}");
            }
            let e = NbtException::new(format!("{name}:{bits:#010x}"));
            assert_eq!(e.message(), format!("{name}:{bits:#010x}"));
        }

        let f64_cases: &[(&str, f64, u64)] = &[
            ("pos_zero", 0.0f64, 0x0000_0000_0000_0000),
            ("neg_zero", -0.0f64, 0x8000_0000_0000_0000),
            ("one", 1.0f64, 0x3ff0_0000_0000_0000),
            ("neg_one", -1.0f64, 0xbff0_0000_0000_0000),
            ("inf", f64::INFINITY, 0x7ff0_0000_0000_0000),
            ("neg_inf", f64::NEG_INFINITY, 0xfff0_0000_0000_0000),
            ("nan", f64::NAN, 0x7ff8_0000_0000_0000),
            (
                "min_pos_subnormal",
                f64::from_bits(0x0000_0000_0000_0001),
                0x0000_0000_0000_0001,
            ),
            (
                "max_neg_subnormal",
                f64::from_bits(0x8000_0000_0000_0001),
                0x8000_0000_0000_0001,
            ),
        ];

        for (name, value, expected_bits) in f64_cases {
            let bits = value.to_bits();
            if *name == "nan" {
                assert!(value.is_nan());
                assert_eq!(bits & 0x7ff0_0000_0000_0000, 0x7ff0_0000_0000_0000);
                assert_ne!(bits & 0x000f_ffff_ffff_ffff, 0);
            } else {
                assert_eq!(bits, *expected_bits, "unexpected f64 bits for {name}");
            }
            let e = NbtException::new(format!("{name}:{bits:#018x}"));
            assert_eq!(e.message(), format!("{name}:{bits:#018x}"));
        }
    }
}
