use bytes::{Buf, BytesMut};
use std::fmt;

const MAX_VARINT21_BYTES: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorruptedFrameException {
    pub message: String,
}

impl fmt::Display for CorruptedFrameException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CorruptedFrameException {}

pub trait BandwidthDebugMonitor {
    fn on_receive(&mut self, bytes: i32);
}

pub struct Varint21FrameDecoder<M> {
    helper_buf: [u8; MAX_VARINT21_BYTES],
    monitor: Option<M>,
}

// JAVA: net.minecraft.network.VarInt#hasContinuationBit
fn varint_has_continuation_bit(b: u8) -> bool {
    (b & 0x80) != 0
}

// JAVA: net.minecraft.network.VarInt#read
fn varint_read(buf: &[u8]) -> i32 {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    for &b in buf {
        result |= ((b & 0x7F) as i32) << shift;
        if !varint_has_continuation_bit(b) {
            return result;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            break;
        }
    }
    result
}

// JAVA: net.minecraft.network.VarInt#getByteSize
fn varint_get_byte_size(value: i32) -> i32 {
    let mut j: i32 = 1;
    while j < 5 {
        let mask = (-1i32).wrapping_shl((j * 7) as u32);
        if (value & mask) == 0 {
            return j;
        }
        j += 1;
    }
    5
}

impl<M: BandwidthDebugMonitor> Varint21FrameDecoder<M> {
    // JAVA: net.minecraft.network.Varint21FrameDecoder#<init>
    pub fn new(monitor: Option<M>) -> Self {
        Self {
            helper_buf: [0u8; MAX_VARINT21_BYTES],
            monitor,
        }
    }

    // JAVA: net.minecraft.network.Varint21FrameDecoder#handlerRemoved0
    pub fn handler_removed0(&mut self) {
        // helperBuf.release() - no-op in Rust (stack-allocated helper buffer)
        self.helper_buf = [0u8; MAX_VARINT21_BYTES];
    }

    // JAVA: net.minecraft.network.Varint21FrameDecoder#copyVarint
    fn copy_varint(
        src: &[u8],
        helper_buf: &mut [u8; MAX_VARINT21_BYTES],
    ) -> Result<Option<usize>, CorruptedFrameException> {
        let mut i: usize = 0;
        while i < MAX_VARINT21_BYTES {
            if i >= src.len() {
                return Ok(None);
            }
            let b = src[i];
            helper_buf[i] = b;
            if !varint_has_continuation_bit(b) {
                return Ok(Some(i + 1));
            }
            i += 1;
        }
        Err(CorruptedFrameException {
            message: "length wider than 21-bit".to_string(),
        })
    }

    // JAVA: net.minecraft.network.Varint21FrameDecoder#decode
    pub fn decode(
        &mut self,
        src: &mut BytesMut,
        out: &mut Vec<BytesMut>,
    ) -> Result<(), CorruptedFrameException> {
        let varint_len = match Self::copy_varint(&src[..], &mut self.helper_buf)? {
            Some(n) => n,
            None => return Ok(()),
        };

        let length = varint_read(&self.helper_buf[..varint_len]);
        if length == 0 {
            return Err(CorruptedFrameException {
                message: "Frame length cannot be zero".to_string(),
            });
        }

        let length_usize = length as usize;
        let total_needed = varint_len.saturating_add(length_usize);
        if src.remaining() < total_needed {
            return Ok(());
        }

        if let Some(monitor) = self.monitor.as_mut() {
            monitor.on_receive(length.wrapping_add(varint_get_byte_size(length)));
        }

        src.advance(varint_len);
        let frame = src.split_to(length_usize);
        out.push(frame);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestMonitor {
        received: Vec<i32>,
    }

    impl BandwidthDebugMonitor for TestMonitor {
        fn on_receive(&mut self, bytes: i32) {
            self.received.push(bytes);
        }
    }

    #[test]
    fn decode_single_frame() {
        let mut decoder = Varint21FrameDecoder::<TestMonitor>::new(None);
        let mut src = BytesMut::from(&[0x01, 0x42][..]);
        let mut out = Vec::new();

        decoder.decode(&mut src, &mut out).unwrap();

        assert_eq!(out.len(), 1);
        assert_eq!(&out[0][..], &[0x42]);
        assert!(src.is_empty());
    }

    #[test]
    fn incomplete_varint_no_progress() {
        let mut decoder = Varint21FrameDecoder::<TestMonitor>::new(None);
        let mut src = BytesMut::from(&[0x80][..]); // continuation bit set, missing next byte
        let mut out = Vec::new();

        decoder.decode(&mut src, &mut out).unwrap();

        assert!(out.is_empty());
        assert_eq!(&src[..], &[0x80]);
    }

    #[test]
    fn incomplete_frame_no_progress() {
        let mut decoder = Varint21FrameDecoder::<TestMonitor>::new(None);
        let mut src = BytesMut::from(&[0x03, 0x01][..]); // length 3, only 1 payload byte
        let mut out = Vec::new();

        decoder.decode(&mut src, &mut out).unwrap();

        assert!(out.is_empty());
        assert_eq!(&src[..], &[0x03, 0x01]);
    }

    #[test]
    fn zero_length_throws() {
        let mut decoder = Varint21FrameDecoder::<TestMonitor>::new(None);
        let mut src = BytesMut::from(&[0x00][..]);
        let mut out = Vec::new();

        let err = decoder.decode(&mut src, &mut out).unwrap_err();
        assert_eq!(err.message, "Frame length cannot be zero");
    }

    #[test]
    fn varint_too_wide_throws() {
        let mut decoder = Varint21FrameDecoder::<TestMonitor>::new(None);
        let mut src = BytesMut::from(&[0x80, 0x80, 0x80][..]);
        let mut out = Vec::new();

        let err = decoder.decode(&mut src, &mut out).unwrap_err();
        assert_eq!(err.message, "length wider than 21-bit");
    }

    #[test]
    fn monitor_receives_total_bytes() {
        let monitor = TestMonitor {
            received: Vec::new(),
        };
        let mut decoder = Varint21FrameDecoder::new(Some(monitor));
        let mut src = BytesMut::from(&[0x02, 0xAA, 0xBB][..]);
        let mut out = Vec::new();

        decoder.decode(&mut src, &mut out).unwrap();

        assert_eq!(out.len(), 1);
        assert_eq!(&out[0][..], &[0xAA, 0xBB]);
        let received = &decoder.monitor.as_ref().unwrap().received;
        // length=2, VarInt byte size for 2 is 1 => total 3
        assert_eq!(received, &vec![3]);
    }
}
