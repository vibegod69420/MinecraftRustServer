//! Port of `net.minecraft.network.codec.StreamMemberEncoder` (Minecraft Java Edition 1.21.11).
//!
//! Java package: `net.minecraft.network.codec`
//! Java declaration:
//! `@FunctionalInterface public interface StreamMemberEncoder<O, T> { void encode(T var1, O var2); }`

/// A functional encoder that writes a value of type `T` into an output of type `O`.
///
/// This is the Rust port of Java's `StreamMemberEncoder<O, T>`.
/// The Java method signature is `void encode(T object, O output)`, where the
/// object/value is encoded into the mutable output.
pub trait StreamMemberEncoder<O, T: ?Sized> {
    // JAVA: net.minecraft.network.codec.StreamMemberEncoder#encode
    fn encode(&self, object: &T, output: &mut O);
}

impl<O, T: ?Sized, F> StreamMemberEncoder<O, T> for F
where
    F: Fn(&T, &mut O),
{
    // JAVA: net.minecraft.network.codec.StreamMemberEncoder#encode
    fn encode(&self, object: &T, output: &mut O) {
        (self)(object, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct TestOutput {
        bytes: Vec<u8>,
    }

    struct I32BeEncoder;

    impl StreamMemberEncoder<TestOutput, i32> for I32BeEncoder {
        // JAVA: net.minecraft.network.codec.StreamMemberEncoder#encode
        fn encode(&self, object: &i32, output: &mut TestOutput) {
            output.bytes.extend_from_slice(&object.to_be_bytes());
        }
    }

    fn encode_with_bound<E>(encoder: &E, value: i32, out: &mut Vec<u8>)
    where
        E: StreamMemberEncoder<Vec<u8>, i32> + ?Sized,
    {
        encoder.encode(&value, out);
    }

    #[test]
    fn trait_impl_encodes_i32_big_endian() {
        let mut out = TestOutput::default();
        let encoder = I32BeEncoder;
        encoder.encode(&42_i32, &mut out);
        assert_eq!(out.bytes, 42_i32.to_be_bytes());
    }

    #[test]
    fn closure_impl_encodes_value() {
        let mut out = TestOutput::default();
        let encoder = |object: &u8, output: &mut TestOutput| {
            output.bytes.push(*object);
        };
        encoder.encode(&0xFF_u8, &mut out);
        assert_eq!(out.bytes, vec![0xFF_u8]);
    }

    #[test]
    fn trait_as_generic_bound() {
        let mut out = Vec::new();
        let encoder = |object: &i32, output: &mut Vec<u8>| {
            output.extend_from_slice(&object.to_be_bytes());
        };
        encode_with_bound(&encoder, -12_345_i32, &mut out);
        assert_eq!(out, (-12_345_i32).to_be_bytes());
    }

    #[test]
    fn trait_is_object_safe() {
        let encoder: Box<dyn StreamMemberEncoder<Vec<u8>, i32>> =
            Box::new(|object: &i32, output: &mut Vec<u8>| {
                output.extend_from_slice(&object.to_be_bytes());
            });
        let mut out = Vec::new();
        encoder.encode(&7_i32, &mut out);
        assert_eq!(out, 7_i32.to_be_bytes());
    }

    #[test]
    fn encode_parameter_order_object_then_output() {
        let mut output: Vec<String> = Vec::new();
        let object = String::from("hello");
        let encoder = |obj: &String, out: &mut Vec<String>| {
            out.push(obj.clone());
        };
        // Java signature: encode(T object, O output)
        encoder.encode(&object, &mut output);
        assert_eq!(output, vec![String::from("hello")]);
    }

    #[test]
    fn encode_negative_inputs() {
        let mut out = TestOutput::default();
        let encoder = |object: &i64, output: &mut TestOutput| {
            output.bytes.extend_from_slice(&object.to_be_bytes());
        };

        encoder.encode(&-1_i64, &mut out);
        assert_eq!(out.bytes, (-1_i64).to_be_bytes());

        out.bytes.clear();
        encoder.encode(&i64::MIN, &mut out);
        assert_eq!(out.bytes, i64::MIN.to_be_bytes());

        out.bytes.clear();
        encoder.encode(&i64::MAX, &mut out);
        assert_eq!(out.bytes, i64::MAX.to_be_bytes());
    }

    #[test]
    fn encode_boundary_integers() {
        let mut out: Vec<u8> = Vec::new();
        let encoder = |object: &i32, output: &mut Vec<u8>| {
            output.extend_from_slice(&object.to_be_bytes());
        };

        for &value in &[i32::MIN, -1, 0, 1, i32::MAX] {
            out.clear();
            encoder.encode(&value, &mut out);
            let decoded = i32::from_be_bytes(out.as_slice().try_into().expect("4 bytes"));
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn encode_with_wrapping_integer_semantics() {
        let mut out: Vec<i32> = Vec::new();
        let encoder = |object: &i32, output: &mut Vec<i32>| {
            // Java int arithmetic uses wrapping semantics.
            let encoded = object.wrapping_add(i32::MAX);
            output.push(encoded);
        };

        encoder.encode(&1_i32, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], 1_i32.wrapping_add(i32::MAX));
        assert_eq!(out[0], i32::MIN);

        out.clear();
        encoder.encode(&i32::MIN, &mut out);
        assert_eq!(out[0], i32::MIN.wrapping_add(i32::MAX));
        assert_eq!(out[0], -1_i32);
    }

    #[test]
    fn encode_with_wrapping_mul_sub_and_masked_shift_semantics() {
        let mut out: Vec<i32> = Vec::new();
        let encoder = |object: &i32, output: &mut Vec<i32>| {
            // Java int ops: wrapping add/sub/mul, shifts masked by 31.
            let a = object.wrapping_mul(31_i32);
            let b = a.wrapping_sub(7_i32);
            let shift = ((*object as u32) & 31_u32) as u32;
            let c = b.wrapping_shl(shift);
            output.push(a);
            output.push(b);
            output.push(c);
        };

        encoder.encode(&i32::MAX, &mut out);
        let expected_a = i32::MAX.wrapping_mul(31_i32);
        let expected_b = expected_a.wrapping_sub(7_i32);
        let expected_shift = ((i32::MAX as u32) & 31_u32) as u32;
        let expected_c = expected_b.wrapping_shl(expected_shift);
        assert_eq!(out, vec![expected_a, expected_b, expected_c]);

        out.clear();
        encoder.encode(&-123_456_i32, &mut out);
        let expected_a = (-123_456_i32).wrapping_mul(31_i32);
        let expected_b = expected_a.wrapping_sub(7_i32);
        let expected_shift = (((-123_456_i32) as u32) & 31_u32) as u32;
        let expected_c = expected_b.wrapping_shl(expected_shift);
        assert_eq!(out, vec![expected_a, expected_b, expected_c]);
    }

    #[test]
    fn encode_with_u64_masked_shift_semantics() {
        let mut out: Vec<u64> = Vec::new();
        let encoder = |object: &u64, output: &mut Vec<u64>| {
            // Java long shift distance is masked by 63.
            let shift = ((*object) & 63_u64) as u32;
            let shifted = object.wrapping_shl(shift);
            output.push(shifted);
        };

        encoder.encode(&65_u64, &mut out); // 65 & 63 == 1
        assert_eq!(out[0], 65_u64.wrapping_shl(1_u32));

        out.clear();
        encoder.encode(&u64::MAX, &mut out);
        let shift = ((u64::MAX) & 63_u64) as u32;
        assert_eq!(out[0], u64::MAX.wrapping_shl(shift));
    }

    #[test]
    fn encode_negative_with_wrapping_arithmetic() {
        let mut out: Vec<i32> = Vec::new();
        let encoder = |object: &i32, output: &mut Vec<i32>| {
            let value = object.wrapping_mul(-3_i32).wrapping_add(i32::MIN);
            output.push(value);
        };

        encoder.encode(&i32::MIN, &mut out);
        assert_eq!(
            out[0],
            i32::MIN.wrapping_mul(-3_i32).wrapping_add(i32::MIN)
        );

        out.clear();
        encoder.encode(&-1_i32, &mut out);
        assert_eq!(out[0], (-1_i32).wrapping_mul(-3_i32).wrapping_add(i32::MIN));
    }

    #[test]
    fn encode_f32_bits_preserved() {
        let mut out = TestOutput::default();
        let encoder = |object: &f32, output: &mut TestOutput| {
            output
                .bytes
                .extend_from_slice(&object.to_bits().to_be_bytes());
        };

        let values = [
            0.0_f32,
            -0.0_f32,
            1.0_f32,
            -1.5_f32,
            f32::MIN_POSITIVE,
            f32::MAX,
            f32::MIN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
        ];

        for value in values {
            out.bytes.clear();
            encoder.encode(&value, &mut out);
            let bits = u32::from_be_bytes(out.bytes.as_slice().try_into().expect("4 bytes"));
            assert_eq!(bits, value.to_bits(), "f32 bit mismatch for {value:?}");
        }
    }

    #[test]
    fn encode_f32_nan_payload_bits_preserved() {
        let mut out = TestOutput::default();
        let encoder = |object: &f32, output: &mut TestOutput| {
            output
                .bytes
                .extend_from_slice(&object.to_bits().to_be_bytes());
        };

        let nan_with_payload = f32::from_bits(0x7FC0_1234_u32);
        assert!(nan_with_payload.is_nan());
        encoder.encode(&nan_with_payload, &mut out);
        let bits = u32::from_be_bytes(out.bytes.as_slice().try_into().expect("4 bytes"));
        assert_eq!(bits, 0x7FC0_1234_u32);
    }

    #[test]
    fn encode_f64_bits_preserved() {
        let mut out = TestOutput::default();
        let encoder = |object: &f64, output: &mut TestOutput| {
            output
                .bytes
                .extend_from_slice(&object.to_bits().to_be_bytes());
        };

        let values = [
            0.0_f64,
            -0.0_f64,
            1.0_f64,
            -123.456_f64,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::MIN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ];

        for value in values {
            out.bytes.clear();
            encoder.encode(&value, &mut out);
            let bits = u64::from_be_bytes(out.bytes.as_slice().try_into().expect("8 bytes"));
            assert_eq!(bits, value.to_bits(), "f64 bit mismatch for {value:?}");
        }
    }

    #[test]
    fn encode_f64_nan_payload_bits_preserved() {
        let mut out = TestOutput::default();
        let encoder = |object: &f64, output: &mut TestOutput| {
            output
                .bytes
                .extend_from_slice(&object.to_bits().to_be_bytes());
        };

        let nan_with_payload = f64::from_bits(0x7FF8_0000_0000_1234_u64);
        assert!(nan_with_payload.is_nan());
        encoder.encode(&nan_with_payload, &mut out);
        let bits = u64::from_be_bytes(out.bytes.as_slice().try_into().expect("8 bytes"));
        assert_eq!(bits, 0x7FF8_0000_0000_1234_u64);
    }

    #[test]
    fn encode_f32_operation_order_preserved() {
        #[derive(Debug, PartialEq)]
        struct FloatOut {
            bits: u32,
        }

        let encoder = |object: &(f32, f32, f32), output: &mut FloatOut| {
            // Preserve Java-like evaluation order: (a + b) * c
            let result = (object.0 + object.1) * object.2;
            output.bits = result.to_bits();
        };

        let mut out = FloatOut { bits: 0 };
        let input = (1.0e20_f32, -1.0e20_f32, 3.0_f32);
        encoder.encode(&input, &mut out);
        let expected = ((input.0 + input.1) * input.2).to_bits();
        assert_eq!(out.bits, expected);
    }

    #[test]
    fn encode_f64_operation_order_preserved() {
        #[derive(Debug, PartialEq)]
        struct FloatOut {
            bits: u64,
        }

        let encoder = |object: &(f64, f64, f64), output: &mut FloatOut| {
            // Preserve Java-like evaluation order: (a + b) * c
            let result = (object.0 + object.1) * object.2;
            output.bits = result.to_bits();
        };

        let mut out = FloatOut { bits: 0 };
        let input = (1.0e308_f64, -1.0e308_f64, 7.0_f64);
        encoder.encode(&input, &mut out);
        let expected = ((input.0 + input.1) * input.2).to_bits();
        assert_eq!(out.bits, expected);
    }

    #[test]
    fn encode_unsized_str_payload() {
        let mut out = TestOutput::default();
        let encoder = |object: &str, output: &mut TestOutput| {
            output.bytes.extend_from_slice(object.as_bytes());
        };

        let value: &str = "minecraft:overworld";
        encoder.encode(value, &mut out);
        assert_eq!(out.bytes, b"minecraft:overworld");
    }
}
