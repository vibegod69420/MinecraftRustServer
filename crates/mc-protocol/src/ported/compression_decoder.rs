use bytes::{Buf, BytesMut};
use flate2::Decompress;

pub const MAXIMUM_COMPRESSED_LENGTH: i32 = 2_097_152;
pub const MAXIMUM_UNCOMPRESSED_LENGTH: i32 = 8_388_608;

#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("{0}")]
    Decoder(String),
    #[error("decompression error: {0}")]
    Decompress(#[from] flate2::DecompressError),
}

// JAVA: net.minecraft.network.VarInt#read
fn varint_read_buf<B: Buf>(buf: &mut B) -> Result<i32, CompressionError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    for _ in 0..5 {
        if !buf.has_remaining() {
            return Err(CompressionError::Decoder("VarInt too short".to_string()));
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl(shift & 31);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
    }
    Err(CompressionError::Decoder("VarInt too big".to_string()))
}

pub struct CompressionDecoder {
    inflater: Decompress,
    threshold: i32,
    validate_decompressed: bool,
    input_buf: BytesMut,
}

impl CompressionDecoder {
    // JAVA: net.minecraft.network.CompressionDecoder#<init>
    pub fn new(threshold: i32, validate_decompressed: bool) -> Self {
        Self {
            inflater: Decompress::new(false),
            threshold,
            validate_decompressed,
            input_buf: BytesMut::new(),
        }
    }

    // JAVA: net.minecraft.network.CompressionDecoder#decode
    pub fn decode(
        &mut self,
        input: &mut BytesMut,
        out: &mut Vec<BytesMut>,
    ) -> Result<(), CompressionError> {
        let n = varint_read_buf(input)?;
        if n == 0 {
            let remaining = input.split_to(input.remaining());
            out.push(remaining);
        } else {
            if self.validate_decompressed {
                if n < self.threshold {
                    return Err(CompressionError::Decoder(format!(
                        "Badly compressed packet - size of {} is below server threshold of {}",
                        n, self.threshold
                    )));
                }
                if n > MAXIMUM_UNCOMPRESSED_LENGTH {
                    return Err(CompressionError::Decoder(format!(
                        "Badly compressed packet - size of {} is larger than protocol maximum of {}",
                        n, MAXIMUM_UNCOMPRESSED_LENGTH
                    )));
                }
            }

            self.setup_inflater_input(input);
            let decompressed = self.inflate(n as usize)?;
            self.inflater.reset(false);
            out.push(decompressed);
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.CompressionDecoder#setupInflaterInput
    fn setup_inflater_input(&mut self, input: &mut BytesMut) {
        self.input_buf = input.split_to(input.remaining());
    }

    // JAVA: net.minecraft.network.CompressionDecoder#inflate
    fn inflate(&mut self, size: usize) -> Result<BytesMut, CompressionError> {
        let mut output = vec![0_u8; size];
        let before_out = self.inflater.total_out();
        let _status = self.inflater.decompress(
            &self.input_buf,
            &mut output,
            flate2::FlushDecompress::Finish,
        )?;
        let produced = (self.inflater.total_out().wrapping_sub(before_out)) as usize;
        if produced != size {
            return Err(CompressionError::Decoder(format!(
                "Badly compressed packet - actual length of uncompressed payload {} is does not match declared size {}",
                produced, size
            )));
        }
        Ok(BytesMut::from(&output[..]))
    }

    // JAVA: net.minecraft.network.CompressionDecoder#setThreshold
    pub fn set_threshold(&mut self, threshold: i32, validate_decompressed: bool) {
        self.threshold = threshold;
        self.validate_decompressed = validate_decompressed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BufMut;
    use flate2::{Compress, Compression, FlushCompress};

    fn write_varint(buf: &mut BytesMut, mut val: i32) {
        let mut v = val as u32;
        loop {
            if (v & !0x7F) == 0 {
                buf.put_u8(v as u8);
                return;
            }
            buf.put_u8(((v as u8) & 0x7F) | 0x80);
            v >>= 7;
        }
        let _ = val;
    }

    fn zlib_compress(data: &[u8]) -> Vec<u8> {
        let mut c = Compress::new(Compression::default(), false);
        let mut out = vec![0_u8; data.len() + 64];
        let mut in_pos = 0;
        let mut out_pos = 0;
        loop {
            let before_in = c.total_in();
            let before_out = c.total_out();
            let status = c
                .compress(&data[in_pos..], &mut out[out_pos..], FlushCompress::Finish)
                .unwrap();
            in_pos += (c.total_in() - before_in) as usize;
            out_pos += (c.total_out() - before_out) as usize;
            if status == flate2::Status::StreamEnd {
                break;
            }
            if out_pos == out.len() {
                out.resize(out.len() * 2, 0);
            }
        }
        out.truncate(out_pos);
        out
    }

    #[test]
    fn decode_uncompressed_passthrough() {
        let mut dec = CompressionDecoder::new(256, true);
        let mut input = BytesMut::new();
        write_varint(&mut input, 0);
        input.extend_from_slice(&[1, 2, 3, 4, 5]);
        let mut out = Vec::new();
        dec.decode(&mut input, &mut out).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(&out[0][..], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn decode_compressed_round_trip() {
        let payload = vec![42_u8; 512];
        let compressed = zlib_compress(&payload);
        let mut dec = CompressionDecoder::new(256, true);
        let mut input = BytesMut::new();
        write_varint(&mut input, payload.len() as i32);
        input.extend_from_slice(&compressed);
        let mut out = Vec::new();
        dec.decode(&mut input, &mut out).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(&out[0][..], &payload[..]);
    }

    #[test]
    fn reject_below_threshold() {
        let payload = vec![1_u8; 10];
        let compressed = zlib_compress(&payload);
        let mut dec = CompressionDecoder::new(256, true);
        let mut input = BytesMut::new();
        write_varint(&mut input, payload.len() as i32);
        input.extend_from_slice(&compressed);
        let err = dec.decode(&mut input, &mut Vec::new()).unwrap_err();
        assert!(err.to_string().contains("below server threshold"));
    }

    #[test]
    fn reject_size_mismatch() {
        let payload = vec![7_u8; 512];
        let compressed = zlib_compress(&payload);
        let mut dec = CompressionDecoder::new(256, true);
        let mut input = BytesMut::new();
        write_varint(&mut input, 513);
        input.extend_from_slice(&compressed);
        let err = dec.decode(&mut input, &mut Vec::new()).unwrap_err();
        assert!(err.to_string().contains("does not match declared size"));
    }

    #[test]
    fn reject_above_maximum_and_set_threshold() {
        let mut dec = CompressionDecoder::new(0, true);
        let mut input = BytesMut::new();
        write_varint(&mut input, MAXIMUM_UNCOMPRESSED_LENGTH + 1);
        input.extend_from_slice(&[0]);
        let err = dec.decode(&mut input, &mut Vec::new()).unwrap_err();
        assert!(err.to_string().contains("larger than protocol maximum"));

        dec.set_threshold(1024, false);
        let payload = vec![9_u8; 10];
        let compressed = zlib_compress(&payload);
        let mut input2 = BytesMut::new();
        write_varint(&mut input2, payload.len() as i32);
        input2.extend_from_slice(&compressed);
        let mut out = Vec::new();
        dec.decode(&mut input2, &mut out).unwrap();
        assert_eq!(&out[0][..], &payload[..]);
    }
}
