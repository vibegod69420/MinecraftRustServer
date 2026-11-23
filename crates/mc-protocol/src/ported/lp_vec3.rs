use bytes::{Buf, BufMut};

/// Minimal 3D vector for LpVec3 encoding/decoding.
/// Faithful to the fields used by `net.minecraft.world.phys.Vec3`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// Minimal VarInt codec for self-contained LpVec3 continuation encoding.
/// Faithful to Minecraft VarInt wire format used by `LpVec3`.
pub struct VarInt;

impl VarInt {
    // JAVA: net.minecraft.network.VarInt#read
    pub fn read(buf: &mut impl Buf) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = buf.get_u8() as i32;
            result |= (b & 0x7F) << shift;
            if (b & 0x80) == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                panic!("VarInt too big");
            }
        }
    }

    // JAVA: net.minecraft.network.VarInt#write
    pub fn write(buf: &mut impl BufMut, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                buf.put_u8(value as u8);
                return;
            }
            buf.put_u8(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }
}

// JAVA: net.minecraft.network.LpVec3
pub struct LpVec3;

impl LpVec3 {
    const DATA_BITS: u32 = 15;
    const DATA_BITS_MASK: i64 = 32767;
    const MAX_QUANTIZED_VALUE: f64 = 32766.0;
    const SCALE_BITS: u32 = 2;
    const SCALE_BITS_MASK: i32 = 3;
    const CONTINUATION_FLAG: i32 = 4;
    const X_OFFSET: u32 = 3;
    const Y_OFFSET: u32 = 18;
    const Z_OFFSET: u32 = 33;
    pub const ABS_MAX_VALUE: f64 = 1.7179869183E10;
    pub const ABS_MIN_VALUE: f64 = 3.051944088384301E-5;

    // JAVA: net.minecraft.network.LpVec3#hasContinuationBit
    pub fn has_continuation_bit(value: i32) -> bool {
        (value & Self::CONTINUATION_FLAG) == Self::CONTINUATION_FLAG
    }

    // JAVA: net.minecraft.network.LpVec3#read
    pub fn read(buf: &mut impl Buf) -> Vec3 {
        let b0 = buf.get_u8() as i32;
        if b0 == 0 {
            return Vec3::ZERO;
        }

        let b1 = buf.get_u8() as i32;
        let u32_part = buf.get_u32() as i64;
        let packed = (u32_part << 16) | ((b1 as i64) << 8) | (b0 as i64);

        let mut scale = (b0 & Self::SCALE_BITS_MASK) as i64;
        if Self::has_continuation_bit(b0) {
            let extra = (VarInt::read(buf) as u32 as i64) & 0xFFFF_FFFF;
            scale |= extra << Self::SCALE_BITS;
        }

        let scale_f = scale as f64;
        Vec3::new(
            Self::unpack(packed >> Self::X_OFFSET) * scale_f,
            Self::unpack(packed >> Self::Y_OFFSET) * scale_f,
            Self::unpack(packed >> Self::Z_OFFSET) * scale_f,
        )
    }

    // JAVA: net.minecraft.network.LpVec3#write
    pub fn write(buf: &mut impl BufMut, vec: Vec3) {
        let x = Self::sanitize(vec.x);
        let y = Self::sanitize(vec.y);
        let z = Self::sanitize(vec.z);
        let max_abs = Self::abs_max(x, Self::abs_max(y, z));
        if max_abs < Self::ABS_MIN_VALUE {
            buf.put_u8(0);
            return;
        }

        let scale = Self::ceil_long(max_abs);
        let needs_continuation = (scale & 3_i64) != scale;
        let scale_low = if needs_continuation {
            (scale & 3_i64) | 4_i64
        } else {
            scale
        };

        let scale_f = scale as f64;
        let x_packed = Self::pack(x / scale_f) << Self::X_OFFSET;
        let y_packed = Self::pack(y / scale_f) << Self::Y_OFFSET;
        let z_packed = Self::pack(z / scale_f) << Self::Z_OFFSET;
        let payload = scale_low | x_packed | y_packed | z_packed;

        buf.put_u8(payload as u8);
        buf.put_u8((payload >> 8) as u8);
        buf.put_i32((payload >> 16) as i32);

        if needs_continuation {
            VarInt::write(buf, (scale >> Self::SCALE_BITS) as i32);
        }
    }

    // JAVA: net.minecraft.network.LpVec3#sanitize
    fn sanitize(value: f64) -> f64 {
        if value.is_nan() {
            0.0
        } else {
            value.clamp(-Self::ABS_MAX_VALUE, Self::ABS_MAX_VALUE)
        }
    }

    // JAVA: net.minecraft.network.LpVec3#pack
    fn pack(value: f64) -> i64 {
        Self::java_round_f64_to_i64((value * 0.5 + 0.5) * Self::MAX_QUANTIZED_VALUE)
    }

    // JAVA: net.minecraft.network.LpVec3#unpack
    fn unpack(value: i64) -> f64 {
        let quantized = ((value & Self::DATA_BITS_MASK) as f64).min(Self::MAX_QUANTIZED_VALUE);
        quantized * 2.0 / Self::MAX_QUANTIZED_VALUE - 1.0
    }

    fn abs_max(a: f64, b: f64) -> f64 {
        if a.abs() >= b.abs() {
            a.abs()
        } else {
            b.abs()
        }
    }

    fn ceil_long(value: f64) -> i64 {
        value.ceil() as i64
    }

    fn java_round_f64_to_i64(value: f64) -> i64 {
        // Java Math.round(double): floor(value + 0.5)
        (value + 0.5).floor() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    fn assert_vec_close(a: Vec3, b: Vec3, eps: f64) {
        assert!(
            (a.x - b.x).abs() <= eps && (a.y - b.y).abs() <= eps && (a.z - b.z).abs() <= eps,
            "vectors not close: {:?} vs {:?}, eps={}",
            a,
            b,
            eps
        );
    }

    #[test]
    fn zero_vector_encodes_as_single_zero_byte() {
        let mut buf = BytesMut::new();
        LpVec3::write(&mut buf, Vec3::ZERO);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], 0);
        let decoded = LpVec3::read(&mut buf);
        assert_eq!(decoded, Vec3::ZERO);
    }

    #[test]
    fn has_continuation_bit_matches_java_mask() {
        assert!(!LpVec3::has_continuation_bit(0));
        assert!(!LpVec3::has_continuation_bit(3));
        assert!(LpVec3::has_continuation_bit(4));
        assert!(LpVec3::has_continuation_bit(7));
    }

    #[test]
    fn round_trip_small_values() {
        let mut buf = BytesMut::new();
        let v = Vec3::new(1.0, -2.0, 0.5);
        LpVec3::write(&mut buf, v);
        let decoded = LpVec3::read(&mut buf);
        // Quantization error is bounded by scale / MAX_QUANTIZED_VALUE.
        assert_vec_close(v, decoded, 2.0 / 32766.0 + 1e-9);
    }

    #[test]
    fn round_trip_negative_and_mixed_values() {
        let mut buf = BytesMut::new();
        let v = Vec3::new(-0.125, 0.75, -3.5);
        LpVec3::write(&mut buf, v);
        let decoded = LpVec3::read(&mut buf);
        let scale = 4.0_f64; // ceil(max abs) = 4
        let eps = scale / 32766.0 + 1e-9;
        assert_vec_close(v, decoded, eps);
    }

    #[test]
    fn nan_is_sanitized_to_zero() {
        let mut buf = BytesMut::new();
        let v = Vec3::new(f64::NAN, 1.0, f64::NAN);
        LpVec3::write(&mut buf, v);
        let decoded = LpVec3::read(&mut buf);
        assert!(decoded.x.abs() <= 1.0 / 32766.0 + 1e-9);
        assert!((decoded.y - 1.0).abs() <= 1.0 / 32766.0 + 1e-9);
        assert!(decoded.z.abs() <= 1.0 / 32766.0 + 1e-9);
    }

    #[test]
    fn large_values_use_continuation_and_round_trip() {
        let mut buf = BytesMut::new();
        let v = Vec3::new(100_000.0, -200_000.0, 50_000.0);
        LpVec3::write(&mut buf, v);
        assert!(buf.len() > 6, "expected continuation varint bytes");
        let decoded = LpVec3::read(&mut buf);
        let scale = 200_000.0_f64;
        let eps = scale / 32766.0 + 1e-6;
        assert_vec_close(v, decoded, eps);
    }
}
