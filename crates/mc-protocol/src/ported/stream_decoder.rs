//! Port of `net.minecraft.network.codec.StreamDecoder`.

/// A decoder that reads a value of type `T` from input `I`.
///
/// Port of `net.minecraft.network.codec.StreamDecoder`.
pub trait StreamDecoder<I, T> {
    // JAVA: net.minecraft.network.codec.StreamDecoder#decode
    fn decode(&self, input: I) -> T;
}

impl<I, T, F> StreamDecoder<I, T> for F
where
    F: Fn(I) -> T,
{
    // JAVA: net.minecraft.network.codec.StreamDecoder#decode
    fn decode(&self, input: I) -> T {
        (self)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::StreamDecoder;

    #[test]
    fn decode_i32_wrapping_overflow() {
        let decoder = |input: i32| input.wrapping_add(i32::MAX);
        assert_eq!(decoder.decode(1i32), i32::MIN);
        assert_eq!(decoder.decode(0i32), i32::MAX);
        assert_eq!(decoder.decode(-1i32), i32::MAX - 1);
    }

    #[test]
    fn decode_i32_negative_and_mul_wrap() {
        let decoder = |input: i32| input.wrapping_mul(-2);
        assert_eq!(decoder.decode(-5i32), 10);
        assert_eq!(decoder.decode(i32::MIN), 0);
        // i32::MAX * -2 = -4294967294, truncated to i32 = 2
        assert_eq!(decoder.decode(i32::MAX), 2);
    }

    #[test]
    fn decode_shift_masking() {
        // Java masks int shift counts with & 31.
        let decoder = |shift: i32| 1i32.wrapping_shl((shift as u32) & 31);
        assert_eq!(decoder.decode(0), 1);
        assert_eq!(decoder.decode(31), i32::MIN);
        assert_eq!(decoder.decode(32), 1); // 32 & 31 == 0
        assert_eq!(decoder.decode(-1), i32::MIN); // -1 & 31 == 31
    }

    #[test]
    fn decode_f32_bits() {
        let decoder = |bits: u32| f32::from_bits(bits);
        assert!(decoder.decode(0x7FC0_0000u32).is_nan());
        assert_eq!(decoder.decode(1.0f32.to_bits()).to_bits(), 1.0f32.to_bits());
        assert_eq!(
            decoder.decode((-1.0f32).to_bits()).to_bits(),
            (-1.0f32).to_bits()
        );
        assert_eq!(
            decoder.decode((-0.0f32).to_bits()).to_bits(),
            (-0.0f32).to_bits()
        );
    }

    #[test]
    fn decode_f64_bits_and_operation_order() {
        let from_bits = |bits: u64| f64::from_bits(bits);
        assert!(from_bits.decode(f64::NAN.to_bits()).is_nan());
        assert_eq!(
            from_bits.decode((-0.0f64).to_bits()).to_bits(),
            (-0.0f64).to_bits()
        );
        assert_eq!(from_bits.decode(f64::INFINITY.to_bits()), f64::INFINITY);

        // Java f64 operation order: (a * b) + c
        let mul_add = |(a, b, c): (f64, f64, f64)| (a * b) + c;
        assert!(mul_add.decode((1e308, 1e308, f64::NEG_INFINITY)).is_nan());
    }

    #[test]
    fn decode_struct_impl() {
        struct AddDecoder(i32);

        impl StreamDecoder<i32, i32> for AddDecoder {
            // JAVA: net.minecraft.network.codec.StreamDecoder#decode
            fn decode(&self, input: i32) -> i32 {
                input.wrapping_add(self.0)
            }
        }

        let decoder = AddDecoder(10);
        assert_eq!(decoder.decode(5), 15);
        assert_eq!(decoder.decode(i32::MAX), i32::MIN + 9);
    }
}
