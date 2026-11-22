//! Faithful port of `net.minecraft.util.RegistryContextSwapper` (Minecraft Java Edition 1.21.11).
//!
//! This module is self-contained: external DFU/Minecraft types referenced by the Java
//! interface are represented by minimal local traits and enums that preserve the
//! interface contract.

use std::fmt;

/// Minimal self-contained stand-in for `com.mojang.serialization.DataResult<T>`.
///
/// The real DFU `DataResult` carries richer lifecycle and error information.
/// This enum preserves the success/error contract required by
/// [`RegistryContextSwapper`].
#[derive(Clone, PartialEq)]
pub enum DataResult<T> {
    /// Successful swap/operation result.
    Success(T),
    /// Failed swap/operation result with an error message.
    Error(String),
}

impl<T: fmt::Debug> fmt::Debug for DataResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success(value) => f.debug_tuple("Success").field(value).finish(),
            Self::Error(message) => f.debug_tuple("Error").field(message).finish(),
        }
    }
}

/// Minimal self-contained stand-in for `com.mojang.serialization.Codec<T>`.
///
/// The real DFU codec encodes and decodes values. This trait provides the type
/// boundary required by the `RegistryContextSwapper` interface.
pub trait Codec<T> {}

/// Minimal self-contained stand-in for `net.minecraft.core.HolderLookup.Provider`.
///
/// The real provider supplies registry lookup contexts. This trait provides the
/// type boundary required by the `RegistryContextSwapper` interface.
pub trait HolderLookupProvider {}

/// Faithful port of `net.minecraft.util.RegistryContextSwapper`.
///
/// A registry context swapper transforms a value by reinterpreting it against a
/// different registry lookup context.
pub trait RegistryContextSwapper {
    // JAVA: net.minecraft.util.RegistryContextSwapper#swapTo
    fn swap_to<T>(
        &self,
        codec: &impl Codec<T>,
        value: T,
        registries: &impl HolderLookupProvider,
    ) -> DataResult<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, Default)]
    struct NoopCodec;

    impl<T> Codec<T> for NoopCodec {}

    #[derive(Debug, Clone, Copy, Default)]
    struct NoopProvider;

    impl HolderLookupProvider for NoopProvider {}

    #[derive(Debug, Clone, Copy, Default)]
    struct IdentitySwapper;

    impl RegistryContextSwapper for IdentitySwapper {
        fn swap_to<T>(
            &self,
            _codec: &impl Codec<T>,
            value: T,
            _registries: &impl HolderLookupProvider,
        ) -> DataResult<T> {
            DataResult::Success(value)
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct FailingSwapper;

    impl RegistryContextSwapper for FailingSwapper {
        fn swap_to<T>(
            &self,
            _codec: &impl Codec<T>,
            _value: T,
            _registries: &impl HolderLookupProvider,
        ) -> DataResult<T> {
            DataResult::Error("swap failed".to_owned())
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct AlternateCodec;

    impl<T> Codec<T> for AlternateCodec {}

    #[derive(Debug, Clone, Copy, Default)]
    struct AlternateProvider;

    impl HolderLookupProvider for AlternateProvider {}

    #[test]
    fn swap_i32_edge_cases() {
        let swapper = IdentitySwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        for value in [i32::MIN, -1, 0, 1, i32::MAX] {
            let result = swapper.swap_to(&codec, value, &registries);
            assert_eq!(result, DataResult::Success(value));
        }
    }

    #[test]
    fn swap_i32_wrapping_arithmetic_values() {
        let swapper = IdentitySwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        let values = [
            i32::MAX.wrapping_add(1),
            i32::MIN.wrapping_sub(1),
            (-1i32).wrapping_mul(i32::MIN),
            0i32.wrapping_sub(1),
        ];

        for value in values {
            let result = swapper.swap_to(&codec, value, &registries);
            assert_eq!(result, DataResult::Success(value));
        }
    }

    #[test]
    fn swap_i64_edge_cases() {
        let swapper = IdentitySwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        for value in [i64::MIN, -1, 0, 1, i64::MAX] {
            let result = swapper.swap_to(&codec, value, &registries);
            assert_eq!(result, DataResult::Success(value));
        }
    }

    #[test]
    fn swap_u32_edge_and_wrapping_values() {
        let swapper = IdentitySwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        let values = [
            0u32,
            1u32,
            u32::MAX,
            u32::MAX.wrapping_add(1),
            0u32.wrapping_sub(1),
        ];

        for value in values {
            let result = swapper.swap_to(&codec, value, &registries);
            assert_eq!(result, DataResult::Success(value));
        }
    }

    #[test]
    fn swap_u64_edge_and_wrapping_values() {
        let swapper = IdentitySwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        let values = [
            0u64,
            1u64,
            u64::MAX,
            u64::MAX.wrapping_add(1),
            1u64.wrapping_sub(2),
            (u64::MAX).wrapping_mul(2),
        ];

        for value in values {
            let result = swapper.swap_to(&codec, value, &registries);
            assert_eq!(result, DataResult::Success(value));
        }
    }

    #[test]
    fn swap_f32_bit_patterns() {
        let swapper = IdentitySwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        let patterns: [u32; 12] = [
            0x0000_0000, // +0.0
            0x8000_0000, // -0.0
            0x7F80_0000, // +inf
            0xFF80_0000, // -inf
            0x7FC0_0000, // quiet NaN
            0xFFC0_0000, // negative quiet NaN
            0x3F80_0000, // 1.0
            0xBF80_0000, // -1.0
            0x0000_0001, // smallest positive subnormal
            0x8000_0001, // smallest negative subnormal
            0x007F_FFFF, // largest subnormal
            0x807F_FFFF, // largest negative subnormal
        ];

        for bits in patterns {
            let value = f32::from_bits(bits);
            let result = swapper.swap_to(&codec, value, &registries);
            match result {
                DataResult::Success(out) => assert_eq!(out.to_bits(), bits),
                DataResult::Error(message) => panic!("unexpected error: {message}"),
            }
        }
    }

    #[test]
    fn swap_f64_bit_patterns() {
        let swapper = IdentitySwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        let patterns: [u64; 12] = [
            0x0000_0000_0000_0000, // +0.0
            0x8000_0000_0000_0000, // -0.0
            0x7FF0_0000_0000_0000, // +inf
            0xFFF0_0000_0000_0000, // -inf
            0x7FF8_0000_0000_0000, // quiet NaN
            0xFFF8_0000_0000_0000, // negative quiet NaN
            0x3FF0_0000_0000_0000, // 1.0
            0xBFF0_0000_0000_0000, // -1.0
            0x0000_0000_0000_0001, // smallest positive subnormal
            0x8000_0000_0000_0001, // smallest negative subnormal
            0x000F_FFFF_FFFF_FFFF, // largest subnormal
            0x800F_FFFF_FFFF_FFFF, // largest negative subnormal
        ];

        for bits in patterns {
            let value = f64::from_bits(bits);
            let result = swapper.swap_to(&codec, value, &registries);
            match result {
                DataResult::Success(out) => assert_eq!(out.to_bits(), bits),
                DataResult::Error(message) => panic!("unexpected error: {message}"),
            }
        }
    }

    #[test]
    fn swap_error_path_preserves_message() {
        let swapper = FailingSwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        let result = swapper.swap_to(&codec, -42i32, &registries);
        assert_eq!(result, DataResult::Error("swap failed".to_owned()));
    }

    #[test]
    fn swap_with_alternate_codec_and_provider_types() {
        let swapper = IdentitySwapper;
        let codec = AlternateCodec;
        let registries = AlternateProvider;

        let result = swapper.swap_to(&codec, i32::MIN, &registries);
        assert_eq!(result, DataResult::Success(i32::MIN));
    }

    #[test]
    fn data_result_debug_clone_and_equality() {
        let success: DataResult<i64> = DataResult::Success(i64::MIN);
        let error: DataResult<i64> = DataResult::Error("boom".to_owned());

        let success_clone = success.clone();
        let error_clone = error.clone();
        assert_eq!(success, success_clone);
        assert_eq!(error, error_clone);
        assert_ne!(success, DataResult::Success(0i64));
        assert_ne!(error, DataResult::Error("other".to_owned()));

        let success_debug = format!("{success:?}");
        let error_debug = format!("{error:?}");
        assert!(success_debug.contains("Success"));
        assert!(success_debug.contains(&i64::MIN.to_string()));
        assert!(error_debug.contains("Error"));
        assert!(error_debug.contains("boom"));
    }

    #[test]
    fn swap_bool_and_negative_float_values() {
        let swapper = IdentitySwapper;
        let codec = NoopCodec;
        let registries = NoopProvider;

        assert_eq!(
            swapper.swap_to(&codec, true, &registries),
            DataResult::Success(true)
        );
        assert_eq!(
            swapper.swap_to(&codec, false, &registries),
            DataResult::Success(false)
        );

        let neg_f32 = -123.456f32;
        let neg_f64 = -987_654_321.125f64;

        match swapper.swap_to(&codec, neg_f32, &registries) {
            DataResult::Success(out) => assert_eq!(out.to_bits(), neg_f32.to_bits()),
            DataResult::Error(message) => panic!("unexpected error: {message}"),
        }

        match swapper.swap_to(&codec, neg_f64, &registries) {
            DataResult::Success(out) => assert_eq!(out.to_bits(), neg_f64.to_bits()),
            DataResult::Error(message) => panic!("unexpected error: {message}"),
        }
    }
}
