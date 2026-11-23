use bytes::{Buf, BufMut};

pub struct VarLong;

impl VarLong {
    // JAVA: net.minecraft.network.VarLong#getByteSize
    pub fn get_byte_size(value: i64) -> i32 {
        for i in 1..10i32 {
            let shift = (i as u32).wrapping_mul(7);
            let mask = (-1i64).wrapping_shl(shift);
            if (value & mask) == 0 {
                return i;
            }
        }
        10
    }

    // JAVA: net.minecraft.network.VarLong#hasContinuationBit
    pub fn has_continuation_bit(b: u8) -> bool {
        (b & 128) == 128
    }

    // JAVA: net.minecraft.network.VarLong#read
    pub fn read<B: Buf>(buf: &mut B) -> i64 {
        let mut result: i64 = 0;
        let mut count: i32 = 0;
        loop {
            let b = buf.get_u8();
            let shift = count.wrapping_mul(7) as u32;
            result |= ((b & 127) as i64).wrapping_shl(shift);
            count = count.wrapping_add(1);
            if count > 10 {
                panic!("VarLong too big");
            }
            if !Self::has_continuation_bit(b) {
                break;
            }
        }
        result
    }

    // JAVA: net.minecraft.network.VarLong#write
    pub fn write<B: BufMut>(buf: &mut B, value: i64) -> &mut B {
        let mut v = value as u64;
        while (v & 0xFFFF_FFFF_FFFF_FF80u64) != 0 {
            buf.put_u8(((v as u8) & 127) | 128);
            v >>= 7;
        }
        buf.put_u8(v as u8);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_get_byte_size() {
        assert_eq!(VarLong::get_byte_size(0), 1);
        assert_eq!(VarLong::get_byte_size(127), 1);
        assert_eq!(VarLong::get_byte_size(128), 2);
        assert_eq!(VarLong::get_byte_size(-1), 10);
        assert_eq!(VarLong::get_byte_size(i64::MAX), 9);
        assert_eq!(VarLong::get_byte_size(i64::MIN), 10);
    }

    #[test]
    fn test_round_trip() {
        let values = [
            0i64,
            1,
            127,
            128,
            255,
            i32::MAX as i64,
            i64::MAX,
            -1,
            -128,
            i64::MIN,
        ];
        for &v in &values {
            let mut buf = BytesMut::new();
            VarLong::write(&mut buf, v);
            let read_back = VarLong::read(&mut buf);
            assert_eq!(read_back, v, "round trip failed for {v}");
        }
    }

    #[test]
    fn test_has_continuation_bit() {
        assert!(VarLong::has_continuation_bit(0x80));
        assert!(VarLong::has_continuation_bit(0xFF));
        assert!(!VarLong::has_continuation_bit(0x7F));
        assert!(!VarLong::has_continuation_bit(0x00));
    }

    #[test]
    fn test_write_encoding() {
        let mut buf = BytesMut::new();
        VarLong::write(&mut buf, 0);
        assert_eq!(&buf[..], &[0x00]);

        let mut buf = BytesMut::new();
        VarLong::write(&mut buf, 128);
        assert_eq!(&buf[..], &[0x80, 0x01]);
    }

    #[test]
    #[should_panic(expected = "VarLong too big")]
    fn test_read_too_big() {
        let mut buf = BytesMut::from(&[0x80u8; 11][..]);
        VarLong::read(&mut buf);
    }
}
