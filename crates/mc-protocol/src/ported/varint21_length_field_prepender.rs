pub const MAX_VARINT21_BYTES: i32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncoderException {
    pub message: String,
}

impl std::fmt::Display for EncoderException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EncoderException {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteBuf {
    data: Vec<u8>,
    reader_index: usize,
    writer_index: usize,
}

impl Default for ByteBuf {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            reader_index: 0,
            writer_index: 0,
        }
    }
}

impl ByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.to_vec(),
            reader_index: 0,
            writer_index: bytes.len(),
        }
    }

    // JAVA: io.netty.buffer.ByteBuf#readableBytes
    pub fn readable_bytes(&self) -> i32 {
        self.writer_index.wrapping_sub(self.reader_index) as i32
    }

    // JAVA: io.netty.buffer.ByteBuf#readerIndex
    pub fn reader_index(&self) -> i32 {
        self.reader_index as i32
    }

    // JAVA: io.netty.buffer.ByteBuf#ensureWritable
    pub fn ensure_writable(&mut self, additional: i32) {
        let add = if additional < 0 {
            0usize
        } else {
            additional as usize
        };
        let needed = self.writer_index.wrapping_add(add);
        if needed > self.data.len() {
            self.data.resize(needed, 0);
        }
    }

    // JAVA: io.netty.buffer.ByteBuf#writeByte
    pub fn write_byte(&mut self, value: i32) {
        let b = (value & 0xFF) as u8;
        if self.writer_index >= self.data.len() {
            self.data.push(b);
            self.writer_index = self.data.len();
        } else {
            self.data[self.writer_index] = b;
            self.writer_index = self.writer_index.wrapping_add(1);
        }
    }

    // JAVA: io.netty.buffer.ByteBuf#writeBytes
    pub fn write_bytes(&mut self, src: &ByteBuf, index: i32, length: i32) {
        let idx = if index < 0 { 0usize } else { index as usize };
        let len = if length < 0 { 0usize } else { length as usize };
        self.ensure_writable(length);
        for i in 0..len {
            let b = src
                .data
                .get(idx.wrapping_add(i))
                .copied()
                .unwrap_or(0);
            self.write_byte(i32::from(b));
        }
    }

    pub fn written_bytes(&self) -> &[u8] {
        &self.data[..self.writer_index]
    }

    #[cfg(test)]
    fn with_indices(reader_index: usize, writer_index: usize) -> Self {
        Self {
            data: Vec::new(),
            reader_index,
            writer_index,
        }
    }
}

pub struct VarInt;

impl VarInt {
    // JAVA: net.minecraft.network.VarInt#getByteSize
    pub fn get_byte_size(value: i32) -> i32 {
        let mut i: i32 = 1;
        while i < 5 {
            if (value & (-1i32 << (i * 7))) == 0 {
                return i;
            }
            i = i.wrapping_add(1);
        }
        5
    }

    // JAVA: net.minecraft.network.VarInt#write
    pub fn write(buf: &mut ByteBuf, mut value: i32) -> &mut ByteBuf {
        loop {
            if (value & !0x7F) == 0 {
                buf.write_byte(value);
                return buf;
            }
            buf.write_byte((value & 0x7F) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }
}

pub struct Varint21LengthFieldPrepender;

impl Varint21LengthFieldPrepender {
    // JAVA: net.minecraft.network.Varint21LengthFieldPrepender#encode
    pub fn encode<C>(_ctx: &mut C, src: &ByteBuf, dst: &mut ByteBuf) -> Result<(), EncoderException> {
        let len = src.readable_bytes();
        let varint_size = VarInt::get_byte_size(len);
        if varint_size > MAX_VARINT21_BYTES {
            return Err(EncoderException {
                message: format!("Packet too large: size {} is over 8", len),
            });
        }
        dst.ensure_writable(varint_size.wrapping_add(len));
        VarInt::write(dst, len);
        dst.write_bytes(src, src.reader_index(), len);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_small_packet_prepends_varint_length() {
        let src = ByteBuf::from_bytes(&[0x01, 0x02, 0x03]);
        let mut dst = ByteBuf::new();
        Varint21LengthFieldPrepender::encode(&mut (), &src, &mut dst).unwrap();
        assert_eq!(dst.written_bytes(), &[0x03, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn encode_empty_packet() {
        let src = ByteBuf::new();
        let mut dst = ByteBuf::new();
        Varint21LengthFieldPrepender::encode(&mut (), &src, &mut dst).unwrap();
        assert_eq!(dst.written_bytes(), &[0x00]);
    }

    #[test]
    fn encode_two_byte_varint_length() {
        let src = ByteBuf::from_bytes(&vec![0xAA; 128]);
        let mut dst = ByteBuf::new();
        Varint21LengthFieldPrepender::encode(&mut (), &src, &mut dst).unwrap();
        let written = dst.written_bytes();
        assert_eq!(written[0], 0x80);
        assert_eq!(written[1], 0x01);
        assert_eq!(written.len(), 2 + 128);
        assert!(written[2..].iter().all(|&b| b == 0xAA));
    }

    #[test]
    fn encode_rejects_too_large_packet() {
        let src = ByteBuf::from_bytes(&vec![0u8; 2_097_152]);
        let mut dst = ByteBuf::new();
        let err = Varint21LengthFieldPrepender::encode(&mut (), &src, &mut dst).unwrap_err();
        assert_eq!(err.message, "Packet too large: size 2097152 is over 8");
    }

    #[test]
    fn encode_rejects_negative_length() {
        let src = ByteBuf::with_indices(1, 0); // readable_bytes() == -1
        let mut dst = ByteBuf::new();
        let err = Varint21LengthFieldPrepender::encode(&mut (), &src, &mut dst).unwrap_err();
        assert_eq!(err.message, "Packet too large: size -1 is over 8");
    }

    #[test]
    fn varint_get_byte_size_and_write_edge_cases() {
        assert_eq!(VarInt::get_byte_size(0), 1);
        assert_eq!(VarInt::get_byte_size(127), 1);
        assert_eq!(VarInt::get_byte_size(128), 2);
        assert_eq!(VarInt::get_byte_size(2_097_151), 3);
        assert_eq!(VarInt::get_byte_size(2_097_152), 4);
        assert_eq!(VarInt::get_byte_size(-1), 5);
        assert_eq!(VarInt::get_byte_size(i32::MIN), 5);

        let mut buf = ByteBuf::new();
        VarInt::write(&mut buf, -1);
        assert_eq!(buf.written_bytes(), &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }
}
