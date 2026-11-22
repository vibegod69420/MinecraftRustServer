//! Port of `net.minecraft.network.codec.StreamEncoder`.
//!
//! A `@FunctionalInterface` for encoding values into an output stream/buffer.
//! In Minecraft Java Edition, this is used throughout the protocol codec system
//! to write typed values into a `FriendlyByteBuf` or similar output.

use std::io;

use thiserror::Error;

/// Errors that can occur during stream encoding.
#[derive(Debug, Error)]
pub enum StreamEncodeError {
    /// An underlying I/O error.
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    /// A custom encoding error with a descriptive message.
    #[error("{0}")]
    Custom(String),
}

impl StreamEncodeError {
    /// Creates a custom encoding error from a displayable message.
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

/// Port of `net.minecraft.network.codec.StreamEncoder<O, T>`.
///
/// Encodes a value of type `T` into an output of type `O`.
/// This is the Rust equivalent of Java's `@FunctionalInterface`
/// `StreamEncoder<O, T>` with a single `encode` method.
///
/// The output is taken as `&mut O` to match Java's mutable buffer semantics.
/// The value is taken as `&T` to allow encoding both owned and borrowed values
/// without unnecessary moves.
pub trait StreamEncoder<O, T: ?Sized> {
    /// Encodes `value` into `output`.
    // JAVA: net.minecraft.network.codec.StreamEncoder#encode
    fn encode(&self, output: &mut O, value: &T) -> Result<(), StreamEncodeError>;
}

impl<O, T: ?Sized, F> StreamEncoder<O, T> for F
where
    F: Fn(&mut O, &T) -> Result<(), StreamEncodeError>,
{
    // JAVA: net.minecraft.network.codec.StreamEncoder#encode
    fn encode(&self, output: &mut O, value: &T) -> Result<(), StreamEncodeError> {
        (self)(output, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn encode_u8_with_closure() {
        let encoder = |out: &mut Vec<u8>, value: &u8| -> Result<(), StreamEncodeError> {
            out.push(*value);
            Ok(())
        };

        let mut buf = Vec::new();
        StreamEncoder::encode(&encoder, &mut buf, &42u8).unwrap();
        assert_eq!(buf, vec![42u8]);
    }

    #[test]
    fn encode_multiple_values_append() {
        let encoder = |out: &mut Vec<u8>, value: &u8| -> Result<(), StreamEncodeError> {
            out.push(*value);
            Ok(())
        };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &1u8).unwrap();
        encoder.encode(&mut buf, &2u8).unwrap();
        encoder.encode(&mut buf, &255u8).unwrap();
        assert_eq!(buf, vec![1, 2, 255]);
    }

    #[test]
    fn encode_negative_i32_be_bytes() {
        let encoder = |out: &mut Vec<u8>, value: &i32| -> Result<(), StreamEncodeError> {
            out.extend_from_slice(&value.to_be_bytes());
            Ok(())
        };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &-1i32).unwrap();
        assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &i32::MIN).unwrap();
        assert_eq!(buf, i32::MIN.to_be_bytes());

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &-123456789i32).unwrap();
        assert_eq!(buf, (-123456789i32).to_be_bytes());
    }

    #[test]
    fn encode_i64_extremes() {
        let encoder = |out: &mut Vec<u8>, value: &i64| -> Result<(), StreamEncodeError> {
            out.extend_from_slice(&value.to_be_bytes());
            Ok(())
        };

        for &v in &[i64::MIN, i64::MAX, -1i64, 0i64, 1i64] {
            let mut buf = Vec::new();
            encoder.encode(&mut buf, &v).unwrap();
            assert_eq!(buf, v.to_be_bytes());
        }
    }

    #[test]
    fn encode_negative_i8_bit_patterns() {
        let encoder = |out: &mut Vec<u8>, value: &i8| -> Result<(), StreamEncodeError> {
            out.push(*value as u8);
            Ok(())
        };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(-1i8)).unwrap();
        assert_eq!(buf, [0xFF]);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &i8::MIN).unwrap();
        assert_eq!(buf, [0x80]);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &i8::MAX).unwrap();
        assert_eq!(buf, [0x7F]);
    }

    #[test]
    fn encode_f32_bits_nan_and_negative_zero() {
        let encoder = |out: &mut Vec<u8>, value: &f32| -> Result<(), StreamEncodeError> {
            out.extend_from_slice(&value.to_bits().to_be_bytes());
            Ok(())
        };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &f32::NAN).unwrap();
        let bits = u32::from_be_bytes(buf.try_into().unwrap());
        assert!(f32::from_bits(bits).is_nan());

        let mut buf_neg_zero = Vec::new();
        encoder
            .encode(&mut buf_neg_zero, &(-0.0f32))
            .unwrap();
        let neg_zero_bits = u32::from_be_bytes(buf_neg_zero.try_into().unwrap());
        assert_eq!(neg_zero_bits, (-0.0f32).to_bits());
        assert_ne!(neg_zero_bits, 0.0f32.to_bits());

        let mut buf_pos_zero = Vec::new();
        encoder.encode(&mut buf_pos_zero, &0.0f32).unwrap();
        let pos_zero_bits = u32::from_be_bytes(buf_pos_zero.try_into().unwrap());
        assert_eq!(pos_zero_bits, 0.0f32.to_bits());
    }

    #[test]
    fn encode_f64_bits_infinity_nan_and_negative_zero() {
        let encoder = |out: &mut Vec<u8>, value: &f64| -> Result<(), StreamEncodeError> {
            out.extend_from_slice(&value.to_bits().to_be_bytes());
            Ok(())
        };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &f64::NEG_INFINITY).unwrap();
        assert_eq!(buf, f64::NEG_INFINITY.to_bits().to_be_bytes());

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &f64::INFINITY).unwrap();
        assert_eq!(buf, f64::INFINITY.to_bits().to_be_bytes());

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &f64::NAN).unwrap();
        let bits = u64::from_be_bytes(buf.try_into().unwrap());
        assert!(f64::from_bits(bits).is_nan());

        let mut buf_neg_zero = Vec::new();
        encoder
            .encode(&mut buf_neg_zero, &(-0.0f64))
            .unwrap();
        let neg_zero_bits = u64::from_be_bytes(buf_neg_zero.try_into().unwrap());
        assert_eq!(neg_zero_bits, (-0.0f64).to_bits());
        assert_ne!(neg_zero_bits, 0.0f64.to_bits());
    }

    #[test]
    fn encode_wrapping_add_overflow() {
        // Java int addition wraps on overflow.
        let encoder =
            |out: &mut Vec<u8>, value: &(i32, i32)| -> Result<(), StreamEncodeError> {
                let sum = value.0.wrapping_add(value.1);
                out.extend_from_slice(&sum.to_be_bytes());
                Ok(())
            };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(i32::MAX, 1)).unwrap();
        let result = i32::from_be_bytes(buf.try_into().unwrap());
        assert_eq!(result, i32::MIN);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(i32::MIN, -1)).unwrap();
        let result = i32::from_be_bytes(buf.try_into().unwrap());
        assert_eq!(result, i32::MAX);
    }

    #[test]
    fn encode_wrapping_sub_overflow() {
        // Java int subtraction wraps on overflow.
        let encoder =
            |out: &mut Vec<u8>, value: &(i32, i32)| -> Result<(), StreamEncodeError> {
                let diff = value.0.wrapping_sub(value.1);
                out.extend_from_slice(&diff.to_be_bytes());
                Ok(())
            };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(i32::MIN, 1)).unwrap();
        let result = i32::from_be_bytes(buf.try_into().unwrap());
        assert_eq!(result, i32::MAX);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(i32::MAX, -1)).unwrap();
        let result = i32::from_be_bytes(buf.try_into().unwrap());
        assert_eq!(result, i32::MIN);
    }

    #[test]
    fn encode_wrapping_mul_overflow() {
        // Java int multiplication wraps on overflow.
        let encoder =
            |out: &mut Vec<u8>, value: &(i32, i32)| -> Result<(), StreamEncodeError> {
                let product = value.0.wrapping_mul(value.1);
                out.extend_from_slice(&product.to_be_bytes());
                Ok(())
            };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(i32::MAX, 2)).unwrap();
        let result = i32::from_be_bytes(buf.try_into().unwrap());
        assert_eq!(result, i32::MAX.wrapping_mul(2));
        assert_eq!(result, -2i32);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(i32::MIN, -1)).unwrap();
        let result = i32::from_be_bytes(buf.try_into().unwrap());
        assert_eq!(result, i32::MIN.wrapping_mul(-1));
    }

    #[test]
    fn encode_masked_shift_left_i32() {
        // Java masks int shift count with & 31.
        let encoder =
            |out: &mut Vec<u8>, value: &(i32, u32)| -> Result<(), StreamEncodeError> {
                let (val, shift) = *value;
                let shifted = val.wrapping_shl(shift & 31);
                out.extend_from_slice(&shifted.to_be_bytes());
                Ok(())
            };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(1i32, 32)).unwrap();
        assert_eq!(i32::from_be_bytes(buf.try_into().unwrap()), 1); // 32 & 31 == 0

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(1i32, 33)).unwrap();
        assert_eq!(i32::from_be_bytes(buf.try_into().unwrap()), 2); // 33 & 31 == 1

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(-1i32, 1)).unwrap();
        assert_eq!(i32::from_be_bytes(buf.try_into().unwrap()), -2);
    }

    #[test]
    fn encode_masked_shift_right_i64() {
        // Java masks long shift count with & 63.
        let encoder =
            |out: &mut Vec<u8>, value: &(i64, u32)| -> Result<(), StreamEncodeError> {
                let (val, shift) = *value;
                let shifted = val.wrapping_shr(shift & 63); // arithmetic shift right for i64
                out.extend_from_slice(&shifted.to_be_bytes());
                Ok(())
            };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(-1i64, 64)).unwrap();
        assert_eq!(i64::from_be_bytes(buf.try_into().unwrap()), -1i64); // 64 & 63 == 0

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(-1i64, 65)).unwrap();
        assert_eq!(i64::from_be_bytes(buf.try_into().unwrap()), -1i64); // 65 & 63 == 1

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(i64::MIN, 63)).unwrap();
        assert_eq!(i64::from_be_bytes(buf.try_into().unwrap()), -1i64);
    }

    #[test]
    fn encode_custom_error() {
        let encoder =
            |_out: &mut Vec<u8>, _value: &u8| -> Result<(), StreamEncodeError> {
                Err(StreamEncodeError::custom("encode failed"))
            };

        let mut buf = Vec::new();
        let err = encoder.encode(&mut buf, &1u8).unwrap_err();
        assert!(matches!(err, StreamEncodeError::Custom(_)));
        assert_eq!(err.to_string(), "encode failed");
    }

    #[test]
    fn encode_io_error_from_conversion() {
        let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "eof");
        let err = StreamEncodeError::from(io_err);
        assert!(matches!(err, StreamEncodeError::Io(_)));
        assert!(err.to_string().contains("io error:"));
        assert!(err.to_string().contains("eof"));
    }

    #[test]
    fn closure_can_propagate_io_error_with_question_mark() {
        let encoder =
            |out: &mut Vec<u8>, value: &[u8]| -> Result<(), StreamEncodeError> {
                let checked: Result<(), io::Error> = if value.is_empty() {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "empty"))
                } else {
                    Ok(())
                };
                checked?;
                out.extend_from_slice(value);
                Ok(())
            };

        let mut buf = Vec::new();
        let err = encoder.encode(&mut buf, b"" as &[u8]).unwrap_err();
        assert!(matches!(err, StreamEncodeError::Io(_)));

        encoder.encode(&mut buf, b"ok" as &[u8]).unwrap();
        assert_eq!(buf, b"ok");
    }

    #[test]
    fn encode_error_leaves_buffer_as_written() {
        let encoder = |out: &mut Vec<u8>, value: &u8| -> Result<(), StreamEncodeError> {
            out.push(*value);
            if *value == 0 {
                return Err(StreamEncodeError::custom("zero"));
            }
            Ok(())
        };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &1u8).unwrap();
        let err = encoder.encode(&mut buf, &0u8).unwrap_err();
        assert!(matches!(err, StreamEncodeError::Custom(_)));
        assert_eq!(buf, vec![1, 0]); // value was pushed before the error
    }

    #[test]
    fn encode_str_unsized() {
        let encoder = |out: &mut Vec<u8>, value: &str| -> Result<(), StreamEncodeError> {
            let bytes = value.as_bytes();
            out.extend_from_slice(&(bytes.len() as i32).to_be_bytes());
            out.extend_from_slice(bytes);
            Ok(())
        };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, "hello").unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&5i32.to_be_bytes());
        expected.extend_from_slice(b"hello");
        assert_eq!(buf, expected);
    }

    #[test]
    fn encode_bytes_slice_unsized() {
        let encoder = |out: &mut Vec<u8>, value: &[u8]| -> Result<(), StreamEncodeError> {
            out.extend_from_slice(value);
            Ok(())
        };

        let mut buf = Vec::new();
        let data: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF];
        encoder.encode(&mut buf, data).unwrap();
        assert_eq!(buf, data);
    }

    #[test]
    fn encode_empty_value() {
        let encoder =
            |_out: &mut Vec<u8>, _value: &()| -> Result<(), StreamEncodeError> { Ok(()) };

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &()).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn trait_object_safety() {
        let encoder: Box<dyn StreamEncoder<Vec<u8>, u8>> = Box::new(
            |out: &mut Vec<u8>, value: &u8| -> Result<(), StreamEncodeError> {
                out.push(*value);
                Ok(())
            },
        );

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &7u8).unwrap();
        assert_eq!(buf, vec![7u8]);
    }

    fn encode_u8_fn(out: &mut Vec<u8>, value: &u8) -> Result<(), StreamEncodeError> {
        out.push(*value);
        Ok(())
    }

    #[test]
    fn fn_pointer_encoder() {
        let mut buf = Vec::new();
        let f: fn(&mut Vec<u8>, &u8) -> Result<(), StreamEncodeError> = encode_u8_fn;
        f.encode(&mut buf, &99u8).unwrap();
        assert_eq!(buf, vec![99u8]);
    }

    #[test]
    fn encode_into_bytes_mut() {
        use bytes::BufMut;

        let encoder =
            |out: &mut bytes::BytesMut, value: &u16| -> Result<(), StreamEncodeError> {
                out.put_u16(*value);
                Ok(())
            };

        let mut buf = bytes::BytesMut::new();
        encoder.encode(&mut buf, &0x1234u16).unwrap();
        assert_eq!(&buf[..], &[0x12, 0x34]);
    }

    fn encode_twice<O, T, E>(
        encoder: &E,
        output: &mut O,
        value: &T,
    ) -> Result<(), StreamEncodeError>
    where
        E: StreamEncoder<O, T>,
        T: ?Sized,
    {
        encoder.encode(output, value)?;
        encoder.encode(output, value)
    }

    #[test]
    fn generic_bound_usage() {
        let encoder = |out: &mut Vec<u8>, value: &u8| -> Result<(), StreamEncodeError> {
            out.push(*value);
            Ok(())
        };

        let mut buf = Vec::new();
        encode_twice(&encoder, &mut buf, &5u8).unwrap();
        assert_eq!(buf, vec![5, 5]);
    }

    struct VarIntEncoder;

    impl StreamEncoder<Vec<u8>, i32> for VarIntEncoder {
        // JAVA: net.minecraft.network.codec.StreamEncoder#encode
        fn encode(&self, output: &mut Vec<u8>, value: &i32) -> Result<(), StreamEncodeError> {
            // VarInt encoding uses the raw 32-bit pattern, with 7-bit groups.
            let mut val = *value as u32;
            loop {
                let mut byte = (val & 0x7F) as u8;
                val >>= 7;
                if val != 0 {
                    byte |= 0x80;
                }
                output.push(byte);
                if val == 0 {
                    break;
                }
            }
            Ok(())
        }
    }

    #[test]
    fn struct_encoder_varint_edge_cases() {
        let encoder = VarIntEncoder;

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &0i32).unwrap();
        assert_eq!(buf, [0x00]);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &127i32).unwrap();
        assert_eq!(buf, [0x7f]);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &128i32).unwrap();
        assert_eq!(buf, [0x80, 0x01]);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &(-1i32)).unwrap();
        assert_eq!(buf, [0xff, 0xff, 0xff, 0xff, 0x0f]);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &i32::MAX).unwrap();
        assert_eq!(buf, [0xff, 0xff, 0xff, 0xff, 0x07]);

        let mut buf = Vec::new();
        encoder.encode(&mut buf, &i32::MIN).unwrap();
        assert_eq!(buf, [0x80, 0x80, 0x80, 0x80, 0x08]);
    }
}
