use bytes::{Buf, BufMut};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoderException {
    pub message: String,
}

impl fmt::Display for DecoderException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecoderException {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncoderException {
    pub message: String,
}

impl fmt::Display for EncoderException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EncoderException {}

// JAVA: io.netty.buffer.ByteBufUtil#utf8MaxBytes
fn utf8_max_bytes(seq_len: i32) -> i32 {
    seq_len.wrapping_mul(3)
}

// JAVA: net.minecraft.network.VarInt#read
fn varint_read<B: Buf>(buf: &mut B) -> Result<i32, DecoderException> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(DecoderException {
                message: "Not enough bytes in buffer for VarInt".to_string(),
            });
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32) << shift;
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return Err(DecoderException {
                message: "VarInt too big".to_string(),
            });
        }
    }
}

// JAVA: net.minecraft.network.VarInt#write
fn varint_write<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value & !0x7F) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

// JAVA: net.minecraft.network.Utf8String#read
pub fn read<B: Buf>(buf: &mut B, max_length: i32) -> Result<String, DecoderException> {
    let max_bytes = utf8_max_bytes(max_length);
    let byte_len = varint_read(buf)?;
    if byte_len > max_bytes {
        return Err(DecoderException {
            message: format!(
                "The received encoded string buffer length is longer than maximum allowed ({} > {})",
                byte_len, max_bytes
            ),
        });
    } else if byte_len < 0 {
        return Err(DecoderException {
            message: "The received encoded string buffer length is less than zero! Weird string!"
                .to_string(),
        });
    } else {
        let readable = buf.remaining() as i32;
        if byte_len > readable {
            return Err(DecoderException {
                message: format!(
                    "Not enough bytes in buffer, expected {}, but got {}",
                    byte_len, readable
                ),
            });
        } else {
            let byte_len_usize = byte_len as usize;
            let mut bytes = vec![0u8; byte_len_usize];
            buf.copy_to_slice(&mut bytes);
            let s = String::from_utf8_lossy(&bytes).into_owned();
            let str_len = s.encode_utf16().count() as i32;
            if str_len > max_length {
                return Err(DecoderException {
                    message: format!(
                        "The received string length is longer than maximum allowed ({} > {})",
                        str_len, max_length
                    ),
                });
            } else {
                return Ok(s);
            }
        }
    }
}

// JAVA: net.minecraft.network.Utf8String#write
pub fn write<B: BufMut>(buf: &mut B, value: &str, max_length: i32) -> Result<(), EncoderException> {
    let char_len = value.encode_utf16().count() as i32;
    if char_len > max_length {
        return Err(EncoderException {
            message: format!(
                "String too big (was {} characters, max {})",
                char_len, max_length
            ),
        });
    }

    let utf8_bytes = value.as_bytes();
    let actual_bytes = utf8_bytes.len() as i32;
    let max_bytes_for_limit = utf8_max_bytes(max_length);
    if actual_bytes > max_bytes_for_limit {
        return Err(EncoderException {
            message: format!(
                "String too big (was {} bytes encoded, max {})",
                actual_bytes, max_bytes_for_limit
            ),
        });
    }

    varint_write(buf, actual_bytes);
    buf.put_slice(utf8_bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BufMut, BytesMut};

    #[test]
    fn round_trip_ascii() {
        let mut buf = BytesMut::new();
        write(&mut buf, "hello", 10).unwrap();
        let mut reader = buf.freeze();
        let s = read(&mut reader, 10).unwrap();
        assert_eq!(s, "hello");
        assert!(!reader.has_remaining());
    }

    #[test]
    fn write_rejects_too_many_chars() {
        let mut buf = BytesMut::new();
        let err = write(&mut buf, "hello", 3).unwrap_err();
        assert_eq!(
            err.message,
            "String too big (was 5 characters, max 3)"
        );
    }

    #[test]
    fn read_rejects_too_long_encoded_length() {
        let mut buf = BytesMut::new();
        varint_write(&mut buf, 16); // max bytes for max_length=5 is 15
        buf.put_slice(&[b'a'; 16]);
        let mut reader = buf.freeze();
        let err = read(&mut reader, 5).unwrap_err();
        assert_eq!(
            err.message,
            "The received encoded string buffer length is longer than maximum allowed (16 > 15)"
        );
    }

    #[test]
    fn read_rejects_negative_length() {
        let mut buf = BytesMut::new();
        varint_write(&mut buf, -1);
        let mut reader = buf.freeze();
        let err = read(&mut reader, 100).unwrap_err();
        assert_eq!(
            err.message,
            "The received encoded string buffer length is less than zero! Weird string!"
        );
    }

    #[test]
    fn read_rejects_not_enough_bytes() {
        let mut buf = BytesMut::new();
        varint_write(&mut buf, 5);
        buf.put_slice(&[b'a', b'b']);
        let mut reader = buf.freeze();
        let err = read(&mut reader, 10).unwrap_err();
        assert_eq!(
            err.message,
            "Not enough bytes in buffer, expected 5, but got 2"
        );
    }

    #[test]
    fn read_rejects_decoded_string_too_long() {
        let mut buf = BytesMut::new();
        varint_write(&mut buf, 2);
        buf.put_slice(b"ab");
        let mut reader = buf.freeze();
        let err = read(&mut reader, 1).unwrap_err();
        assert_eq!(
            err.message,
            "The received string length is longer than maximum allowed (2 > 1)"
        );
    }
}
