use bytes::{Buf, BufMut, BytesMut};
use flate2::{Compress, Compression, FlushCompress, Status};

fn write_varint(buf: &mut BytesMut, value: i32) {
    let mut v = value as u32;
    loop {
        if (v & !0x7Fu32) == 0 {
            buf.put_u8(v as u8);
            return;
        }
        buf.put_u8(((v as u8) & 0x7F) | 0x80);
        v >>= 7;
    }
}

pub struct CompressionEncoder {
    encode_buf: [u8; 8192],
    deflater: Compress,
    threshold: i32,
}

impl CompressionEncoder {
    // JAVA: net.minecraft.network.CompressionEncoder#<init>
    pub fn new(threshold: i32) -> Self {
        Self {
            encode_buf: [0u8; 8192],
            deflater: Compress::new(Compression::default(), true),
            threshold,
        }
    }

    // JAVA: net.minecraft.network.CompressionEncoder#encode
    pub fn encode(&mut self, input: &mut BytesMut, output: &mut BytesMut) {
        let len = input.remaining();
        if len > 8_388_608 {
            panic!("Packet too big (is {}, should be less than 8388608)", len);
        }
        if (len as i32) < self.threshold {
            write_varint(output, 0);
            let data = input.split_to(len);
            output.put_slice(&data);
        } else {
            let data = input.split_to(len);
            write_varint(output, len as i32);
            self.deflater.reset();
            loop {
                let total_in = self.deflater.total_in() as usize;
                let before_out = self.deflater.total_out();
                let status = self
                    .deflater
                    .compress(&data[total_in..], &mut self.encode_buf, FlushCompress::Finish)
                    .expect("deflate failed");
                let produced = (self.deflater.total_out() - before_out) as usize;
                if produced > 0 {
                    output.put_slice(&self.encode_buf[..produced]);
                }
                if status == Status::StreamEnd {
                    break;
                }
            }
        }
    }

    // JAVA: net.minecraft.network.CompressionEncoder#getThreshold
    pub fn get_threshold(&self) -> i32 {
        self.threshold
    }

    // JAVA: net.minecraft.network.CompressionEncoder#setThreshold
    pub fn set_threshold(&mut self, threshold: i32) {
        self.threshold = threshold;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    fn read_varint(buf: &mut &[u8]) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = buf[0];
            *buf = &buf[1..];
            result |= ((b & 0x7F) as i32) << shift;
            if (b & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        result
    }

    #[test]
    fn test_uncompressed_below_threshold() {
        let mut enc = CompressionEncoder::new(256);
        let mut input = BytesMut::from(&[1u8, 2, 3, 4][..]);
        let mut output = BytesMut::new();
        enc.encode(&mut input, &mut output);

        let mut slice: &[u8] = &output;
        let uncompressed_len = read_varint(&mut slice);
        assert_eq!(uncompressed_len, 0);
        assert_eq!(slice, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_compressed_above_threshold() {
        let mut enc = CompressionEncoder::new(1);
        let data = vec![42u8; 100];
        let mut input = BytesMut::from(&data[..]);
        let mut output = BytesMut::new();
        enc.encode(&mut input, &mut output);

        let mut slice: &[u8] = &output;
        let uncompressed_len = read_varint(&mut slice);
        assert_eq!(uncompressed_len, 100);

        let mut decoder = ZlibDecoder::new(slice);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_threshold_boundary() {
        let mut enc = CompressionEncoder::new(4);
        let mut input = BytesMut::from(&[1u8, 2, 3, 4][..]);
        let mut output = BytesMut::new();
        enc.encode(&mut input, &mut output);

        let mut slice: &[u8] = &output;
        let uncompressed_len = read_varint(&mut slice);
        assert_eq!(uncompressed_len, 4);
    }

    #[test]
    #[should_panic(expected = "Packet too big")]
    fn test_too_big() {
        let mut enc = CompressionEncoder::new(256);
        let mut input = BytesMut::zeroed(8_388_609);
        let mut output = BytesMut::new();
        enc.encode(&mut input, &mut output);
    }

    #[test]
    fn test_get_set_threshold() {
        let mut enc = CompressionEncoder::new(256);
        assert_eq!(enc.get_threshold(), 256);
        enc.set_threshold(512);
        assert_eq!(enc.get_threshold(), 512);
    }
}
