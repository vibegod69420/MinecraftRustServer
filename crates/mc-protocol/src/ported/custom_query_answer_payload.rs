//! Port of `net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload`.
//!
//! Defines the [`CustomQueryAnswerPayload`] trait for serializing custom login
//! query answers into a [`FriendlyByteBuf`].

/// Minimal self-contained stand-in for `net.minecraft.network.FriendlyByteBuf`.
///
/// The real Minecraft class wraps a Netty `ByteBuf`. This local representation
/// keeps the module self-contained while preserving the write-target role used
/// by [`CustomQueryAnswerPayload`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
}

impl FriendlyByteBuf {
    /// Creates an empty buffer.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates an empty buffer with the specified initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Returns the written bytes as a slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Consumes the buffer and returns the written bytes.
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    /// Returns the number of bytes written.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if no bytes have been written.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Appends raw bytes to the buffer.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
}

/// Port of `net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload`.
///
/// In Minecraft's login protocol, custom query answers serialize their payload
/// data into a friendly byte buffer.
pub trait CustomQueryAnswerPayload {
    // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
    fn write(&self, buf: &mut FriendlyByteBuf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RawPayload {
        bytes: Vec<u8>,
    }

    impl CustomQueryAnswerPayload for RawPayload {
        // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
        fn write(&self, buf: &mut FriendlyByteBuf) {
            buf.write_bytes(&self.bytes);
        }
    }

    #[test]
    fn friendly_byte_buf_defaults_and_accessors() {
        let buf = FriendlyByteBuf::default();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.as_slice(), &[] as &[u8]);

        let buf2 = FriendlyByteBuf::new();
        assert_eq!(buf, buf2);

        let mut buf3 = FriendlyByteBuf::with_capacity(16);
        assert!(buf3.is_empty());
        buf3.write_bytes(&[1, 2, 3]);
        assert_eq!(buf3.len(), 3);
        assert_eq!(buf3.as_slice(), &[1, 2, 3]);
        assert_eq!(buf3.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn write_empty_payload() {
        let payload = RawPayload { bytes: Vec::new() };
        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf);
        assert!(buf.is_empty());
        assert_eq!(buf.as_slice(), &[] as &[u8]);
    }

    #[test]
    fn write_preserves_bytes() {
        let payload = RawPayload {
            bytes: vec![0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF],
        };
        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf);
        assert_eq!(buf.as_slice(), &[0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF]);
    }

    #[test]
    fn write_appends_to_existing_buffer() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_bytes(&[0xAA]);
        let payload = RawPayload {
            bytes: vec![0xBB, 0xCC],
        };
        payload.write(&mut buf);
        assert_eq!(buf.as_slice(), &[0xAA, 0xBB, 0xCC]);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn trait_object_dispatch() {
        let payload = RawPayload {
            bytes: vec![1, 2, 3, 4],
        };
        let obj: &dyn CustomQueryAnswerPayload = &payload;
        let mut buf = FriendlyByteBuf::new();
        obj.write(&mut buf);
        assert_eq!(buf.into_inner(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn multiple_payload_implementations() {
        #[derive(Debug)]
        struct EmptyPayload;

        impl CustomQueryAnswerPayload for EmptyPayload {
            // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
            fn write(&self, _buf: &mut FriendlyByteBuf) {}
        }

        #[derive(Debug)]
        struct SingleBytePayload(u8);

        impl CustomQueryAnswerPayload for SingleBytePayload {
            // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
            fn write(&self, buf: &mut FriendlyByteBuf) {
                buf.write_bytes(&[self.0]);
            }
        }

        let mut buf = FriendlyByteBuf::new();
        EmptyPayload.write(&mut buf);
        assert!(buf.is_empty());

        SingleBytePayload(0x42).write(&mut buf);
        assert_eq!(buf.as_slice(), &[0x42]);

        EmptyPayload.write(&mut buf);
        assert_eq!(buf.as_slice(), &[0x42]);

        SingleBytePayload(0xFF).write(&mut buf);
        assert_eq!(buf.as_slice(), &[0x42, 0xFF]);
    }

    #[test]
    fn write_integer_edge_cases_overflow_and_negative_inputs() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct IntegerPayload {
            values: Vec<i32>,
        }

        impl CustomQueryAnswerPayload for IntegerPayload {
            // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
            fn write(&self, buf: &mut FriendlyByteBuf) {
                for &v in &self.values {
                    buf.write_bytes(&v.to_be_bytes());
                }
            }
        }

        let min = i32::MIN;
        let max = i32::MAX;
        let neg_one: i32 = -1;
        let zero: i32 = 0;

        // Java integer semantics: wrapping arithmetic
        let overflow_add = max.wrapping_add(1);
        assert_eq!(overflow_add, i32::MIN);

        let overflow_sub = min.wrapping_sub(1);
        assert_eq!(overflow_sub, i32::MAX);

        let overflow_mul = max.wrapping_mul(2);
        assert_eq!(overflow_mul, -2);

        let neg_mul = min.wrapping_mul(-1);
        assert_eq!(neg_mul, i32::MIN);

        let payload = IntegerPayload {
            values: vec![
                min,
                max,
                neg_one,
                zero,
                overflow_add,
                overflow_sub,
                overflow_mul,
                neg_mul,
            ],
        };

        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf);

        let bytes = buf.as_slice();
        assert_eq!(bytes.len(), 8 * 4);

        let mut offset = 0usize;
        for &expected in &[
            i32::MIN,
            i32::MAX,
            -1i32,
            0i32,
            i32::MIN, // max.wrapping_add(1)
            i32::MAX, // min.wrapping_sub(1)
            -2i32,    // max.wrapping_mul(2)
            i32::MIN, // min.wrapping_mul(-1)
        ] {
            let actual = i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
            assert_eq!(actual, expected);
            offset += 4;
        }
        assert_eq!(offset, bytes.len());
    }

    #[test]
    fn write_i64_edge_cases_and_java_shift_masks() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct I64Payload {
            values: Vec<i64>,
        }

        impl CustomQueryAnswerPayload for I64Payload {
            // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
            fn write(&self, buf: &mut FriendlyByteBuf) {
                for &v in &self.values {
                    buf.write_bytes(&v.to_be_bytes());
                }
            }
        }

        // Java shift masking: int shifts mask by 31, long shifts mask by 63
        let shift_i32: i32 = 1;
        let shifted_by_32 = shift_i32.wrapping_shl(32u32 & 31);
        assert_eq!(shifted_by_32, 1);

        let shifted_by_neg1_i32 = shift_i32.wrapping_shl((-1i32 as u32) & 31);
        assert_eq!(shifted_by_neg1_i32, i32::MIN);

        let shift_i64: i64 = 1;
        let shifted_by_64 = shift_i64.wrapping_shl((64u32) & 63);
        assert_eq!(shifted_by_64, 1i64);

        let shifted_by_neg1_i64 = shift_i64.wrapping_shl((-1i32 as u32) & 63);
        assert_eq!(shifted_by_neg1_i64, i64::MIN);

        let values = vec![
            i64::MIN,
            i64::MAX,
            -1i64,
            0i64,
            i64::MIN.wrapping_sub(1),
            i64::MAX.wrapping_add(1),
            shifted_by_neg1_i64,
        ];

        let payload = I64Payload {
            values: values.clone(),
        };
        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf);

        let bytes = buf.as_slice();
        assert_eq!(bytes.len(), values.len() * 8);
        for (i, &expected) in values.iter().enumerate() {
            let start = i * 8;
            let actual = i64::from_be_bytes(bytes[start..start + 8].try_into().unwrap());
            assert_eq!(actual, expected);
        }

        // Ensure masked-shift results used above match Java semantics exactly.
        assert_eq!(shifted_by_32, 1);
        assert_eq!(shifted_by_64, 1i64);
    }

    #[test]
    fn write_float_bit_patterns() {
        #[derive(Debug, Clone)]
        struct FloatPayload {
            f32_values: Vec<f32>,
            f64_values: Vec<f64>,
        }

        impl CustomQueryAnswerPayload for FloatPayload {
            // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
            fn write(&self, buf: &mut FriendlyByteBuf) {
                for &v in &self.f32_values {
                    buf.write_bytes(&v.to_bits().to_be_bytes());
                }
                for &v in &self.f64_values {
                    buf.write_bytes(&v.to_bits().to_be_bytes());
                }
            }
        }

        assert_ne!((-0.0f32).to_bits(), 0.0f32.to_bits());
        assert_ne!((-0.0f64).to_bits(), 0.0f64.to_bits());

        let f32_values = vec![
            0.0f32,
            -0.0f32,
            1.0f32,
            -1.0f32,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            f32::MIN,
            f32::MAX,
            f32::MIN_POSITIVE,
            f32::EPSILON,
            f32::from_bits(0x0000_0001), // smallest positive subnormal
            f32::from_bits(0x007F_FFFF), // largest subnormal
            f32::from_bits(0x8000_0001), // smallest negative subnormal
            f32::from_bits(0x7FC0_0001), // quiet NaN payload variant
            f32::from_bits(0xFFC0_0001), // negative quiet NaN payload variant
        ];

        let f64_values = vec![
            0.0f64,
            -0.0f64,
            1.0f64,
            -1.0f64,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::MIN,
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::from_bits(0x0000_0000_0000_0001), // smallest positive subnormal
            f64::from_bits(0x000F_FFFF_FFFF_FFFF), // largest subnormal
            f64::from_bits(0x8000_0000_0000_0001), // smallest negative subnormal
            f64::from_bits(0x7FF8_0000_0000_0001), // quiet NaN payload variant
            f64::from_bits(0xFFF8_0000_0000_0001), // negative quiet NaN payload variant
        ];

        let payload = FloatPayload {
            f32_values: f32_values.clone(),
            f64_values: f64_values.clone(),
        };
        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf);

        let bytes = buf.as_slice();
        let mut offset = 0usize;

        for &v in &f32_values {
            let expected_bits = v.to_bits();
            let expected_bytes = expected_bits.to_be_bytes();
            assert_eq!(
                &bytes[offset..offset + 4],
                &expected_bytes,
                "f32 bit pattern mismatch for value {v:?} (bits {expected_bits:#010X})"
            );
            let read_bits = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
            assert_eq!(read_bits, expected_bits);
            offset += 4;
        }

        for &v in &f64_values {
            let expected_bits = v.to_bits();
            let expected_bytes = expected_bits.to_be_bytes();
            assert_eq!(
                &bytes[offset..offset + 8],
                &expected_bytes,
                "f64 bit pattern mismatch for value {v:?} (bits {expected_bits:#018X})"
            );
            let read_bits = u64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap());
            assert_eq!(read_bits, expected_bits);
            offset += 8;
        }

        assert_eq!(offset, buf.len());
        assert_eq!(offset, f32_values.len() * 4 + f64_values.len() * 8);
    }
}
