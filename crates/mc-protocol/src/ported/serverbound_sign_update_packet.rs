use std::fmt;

pub const MAX_STRING_LENGTH: usize = 384;

const PACKED_X_LENGTH: i32 = 26;
const PACKED_Z_LENGTH: i32 = 26;
const PACKED_Y_LENGTH: i32 = 12;
const PACKED_X_MASK: i64 = (1_i64 << PACKED_X_LENGTH) - 1_i64;
const PACKED_Y_MASK: i64 = (1_i64 << PACKED_Y_LENGTH) - 1_i64;
const PACKED_Z_MASK: i64 = (1_i64 << PACKED_Z_LENGTH) - 1_i64;
const Z_OFFSET: i32 = PACKED_Y_LENGTH;
const X_OFFSET: i32 = PACKED_Y_LENGTH + PACKED_Z_LENGTH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    NotEnoughBytes,
    VarIntTooBig,
    StringTooLong,
    InvalidUtf8,
    NegativeStringLength,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::NotEnoughBytes => write!(f, "not enough bytes"),
            DecodeError::VarIntTooBig => write!(f, "varint too big"),
            DecodeError::StringTooLong => write!(f, "string too long"),
            DecodeError::InvalidUtf8 => write!(f, "invalid utf8"),
            DecodeError::NegativeStringLength => write!(f, "negative string length"),
        }
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn as_long(&self) -> i64 {
        let mut l: i64 = 0_i64;
        l |= ((self.x as i64) & PACKED_X_MASK) << (X_OFFSET & 63);
        l |= ((self.y as i64) & PACKED_Y_MASK) << 0_i32;
        l | (((self.z as i64) & PACKED_Z_MASK) << (Z_OFFSET & 63))
    }

    pub fn from_long(packed_pos: i64) -> Self {
        let x = ((packed_pos << ((64_i32 - X_OFFSET - PACKED_X_LENGTH) & 63))
            >> ((64_i32 - PACKED_X_LENGTH) & 63)) as i32;
        let y = ((packed_pos << ((64_i32 - PACKED_Y_LENGTH) & 63))
            >> ((64_i32 - PACKED_Y_LENGTH) & 63)) as i32;
        let z = ((packed_pos << ((64_i32 - Z_OFFSET - PACKED_Z_LENGTH) & 63))
            >> ((64_i32 - PACKED_Z_LENGTH) & 63)) as i32;
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    rpos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            rpos: 0,
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, rpos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.rpos)
    }

    fn read_byte(&mut self) -> Result<u8, DecodeError> {
        if self.rpos >= self.data.len() {
            return Err(DecodeError::NotEnoughBytes);
        }
        let b = self.data[self.rpos];
        self.rpos = self.rpos.wrapping_add(1);
        Ok(b)
    }

    pub fn read_long(&mut self) -> Result<i64, DecodeError> {
        if self.remaining() < 8 {
            return Err(DecodeError::NotEnoughBytes);
        }
        let mut bytes = [0_u8; 8];
        for i in 0..8 {
            bytes[i] = self.data[self.rpos.wrapping_add(i)];
        }
        self.rpos = self.rpos.wrapping_add(8);
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn write_long(&mut self, v: i64) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    pub fn read_block_pos(&mut self) -> Result<BlockPos, DecodeError> {
        Ok(BlockPos::from_long(self.read_long()?))
    }

    pub fn write_block_pos(&mut self, pos: BlockPos) {
        self.write_long(pos.as_long());
    }

    pub fn read_boolean(&mut self) -> Result<bool, DecodeError> {
        Ok(self.read_byte()? != 0)
    }

    pub fn write_boolean(&mut self, v: bool) {
        self.data.push(if v { 1_u8 } else { 0_u8 });
    }

    pub fn read_var_int(&mut self) -> Result<i32, DecodeError> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;
        loop {
            let byte = self.read_byte()? as i32;
            value |= (byte & 0x7F) << (position & 31);
            if (byte & 0x80) == 0 {
                return Ok(value);
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(DecodeError::VarIntTooBig);
            }
        }
    }

    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let mut temp = (value as u32 & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                temp |= 0x80;
            }
            self.data.push(temp);
            if value == 0 {
                return;
            }
        }
    }

    pub fn read_utf(&mut self, max_length: usize) -> Result<String, DecodeError> {
        let max_bytes = max_length.wrapping_mul(3);
        let len_i32 = self.read_var_int()?;
        if len_i32 < 0 {
            return Err(DecodeError::NegativeStringLength);
        }
        let len = len_i32 as usize;
        if len > max_bytes {
            return Err(DecodeError::StringTooLong);
        }
        if self.remaining() < len {
            return Err(DecodeError::NotEnoughBytes);
        }
        let start = self.rpos;
        let end = self.rpos.wrapping_add(len);
        let bytes = self.data[start..end].to_vec();
        self.rpos = end;
        let s = String::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)?;
        let utf16_len: usize = s.chars().map(|c| c.len_utf16()).sum();
        if utf16_len > max_length {
            return Err(DecodeError::StringTooLong);
        }
        Ok(s)
    }

    pub fn write_utf(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    ServerboundSignUpdate,
}

pub trait ServerGamePacketListener {
    fn handle_sign_update(&mut self, packet: &ServerboundSignUpdatePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSignUpdatePacket {
    pos: BlockPos,
    lines: [String; 4],
    is_front_text: bool,
}

impl ServerboundSignUpdatePacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSignUpdatePacket#<init>
    pub fn new(
        pos: BlockPos,
        is_front_text: bool,
        line0: String,
        line1: String,
        line2: String,
        line3: String,
    ) -> Self {
        Self {
            pos,
            is_front_text,
            lines: [line0, line1, line2, line3],
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSignUpdatePacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, DecodeError> {
        let pos = buf.read_block_pos()?;
        let is_front_text = buf.read_boolean()?;
        let mut lines: [String; 4] = [
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ];
        let mut i: i32 = 0;
        while i < 4 {
            let idx = i as usize;
            lines[idx] = buf.read_utf(MAX_STRING_LENGTH)?;
            i = i.wrapping_add(1);
        }
        Ok(Self {
            pos,
            lines,
            is_front_text,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSignUpdatePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(self.pos);
        buf.write_boolean(self.is_front_text);
        let mut i: i32 = 0;
        while i < 4 {
            let idx = i as usize;
            buf.write_utf(&self.lines[idx]);
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSignUpdatePacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ServerboundSignUpdate
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSignUpdatePacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_sign_update(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSignUpdatePacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSignUpdatePacket#isFrontText
    pub fn is_front_text(&self) -> bool {
        self.is_front_text
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSignUpdatePacket#getLines
    pub fn get_lines(&self) -> &[String; 4] {
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        called: bool,
        pos: BlockPos,
        front: bool,
        lines: [String; 4],
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_sign_update(&mut self, packet: &ServerboundSignUpdatePacket) {
            self.called = true;
            self.pos = packet.get_pos();
            self.front = packet.is_front_text();
            self.lines = packet.get_lines().clone();
        }
    }

    #[test]
    fn round_trip_basic_and_handle() {
        let packet = ServerboundSignUpdatePacket::new(
            BlockPos::new(10, 64, -20),
            true,
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
            "line4".to_string(),
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_inner());
        let decoded = ServerboundSignUpdatePacket::read(&mut read_buf).expect("decode");
        assert_eq!(decoded, packet);
        assert_eq!(decoded.packet_type(), PacketType::ServerboundSignUpdate);

        let mut listener = MockListener::default();
        decoded.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.pos, BlockPos::new(10, 64, -20));
        assert!(listener.front);
        assert_eq!(listener.lines[0], "line1");
        assert_eq!(listener.lines[3], "line4");
    }

    #[test]
    fn negative_coordinates_and_block_pos_packing() {
        let cases = [
            BlockPos::new(-1, -1, -1),
            BlockPos::new(-33_554_432, -2048, -33_554_432),
            BlockPos::new(33_554_431, 2047, 33_554_431),
            BlockPos::new(i32::MIN, i32::MIN, i32::MIN),
        ];
        for pos in cases {
            let packed = pos.as_long();
            let unpacked = BlockPos::from_long(packed);
            let expected_x = ((pos.x as i64) << 38 >> 38) as i32;
            let expected_y = ((pos.y as i64) << 52 >> 52) as i32;
            let expected_z = ((pos.z as i64) << 38 >> 38) as i32;
            assert_eq!(unpacked.x, expected_x);
            assert_eq!(unpacked.y, expected_y);
            assert_eq!(unpacked.z, expected_z);
        }

        let packet = ServerboundSignUpdatePacket::new(
            BlockPos::new(-123, -5, 4567),
            false,
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded = ServerboundSignUpdatePacket::read(&mut FriendlyByteBuf::from_vec(buf.into_inner()))
            .expect("decode");
        assert_eq!(decoded.get_pos(), BlockPos::new(-123, -5, 4567));
        assert!(!decoded.is_front_text());
    }

    #[test]
    fn max_length_and_empty_strings() {
        let max_line = "a".repeat(MAX_STRING_LENGTH);
        let packet = ServerboundSignUpdatePacket::new(
            BlockPos::new(0, 0, 0),
            true,
            max_line.clone(),
            String::new(),
            "§".repeat(MAX_STRING_LENGTH), // 2-byte UTF-8, 1 UTF-16 unit each
            "末".repeat(MAX_STRING_LENGTH), // 3-byte UTF-8
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded = ServerboundSignUpdatePacket::read(&mut FriendlyByteBuf::from_vec(buf.into_inner()))
            .expect("decode");
        assert_eq!(decoded.get_lines()[0], max_line);
        assert_eq!(decoded.get_lines()[1], "");
        assert_eq!(decoded.get_lines()[2].chars().count(), MAX_STRING_LENGTH);
        assert_eq!(decoded.get_lines()[3].chars().count(), MAX_STRING_LENGTH);
    }

    #[test]
    fn string_length_overflow_and_decode_errors() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_block_pos(BlockPos::new(1, 2, 3));
        buf.write_boolean(false);
        buf.write_utf(&"b".repeat(MAX_STRING_LENGTH.wrapping_add(1)));
        let err = ServerboundSignUpdatePacket::read(&mut FriendlyByteBuf::from_vec(buf.into_inner()))
            .expect_err("should fail");
        assert_eq!(err, DecodeError::StringTooLong);

        let mut bad_len = FriendlyByteBuf::new();
        bad_len.write_var_int(-1);
        assert_eq!(
            bad_len.read_utf(MAX_STRING_LENGTH).expect_err("negative len"),
            DecodeError::NegativeStringLength
        );
    }

    #[test]
    fn varint_and_boolean_edge_cases() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(0);
        buf.write_var_int(-1);
        buf.write_var_int(i32::MAX);
        buf.write_var_int(i32::MIN);
        buf.write_boolean(false);
        buf.write_boolean(true);
        buf.write_boolean(2_u8 != 0); // mimic non-zero byte truthiness source
        let mut r = FriendlyByteBuf::from_vec(buf.into_inner());
        assert_eq!(r.read_var_int().unwrap(), 0);
        assert_eq!(r.read_var_int().unwrap(), -1);
        assert_eq!(r.read_var_int().unwrap(), i32::MAX);
        assert_eq!(r.read_var_int().unwrap(), i32::MIN);
        assert!(!r.read_boolean().unwrap());
        assert!(r.read_boolean().unwrap());
        assert!(r.read_boolean().unwrap());

        // Truncated buffer
        let mut truncated = FriendlyByteBuf::from_vec(vec![0x80, 0x80]);
        assert_eq!(truncated.read_var_int().expect_err("truncated"), DecodeError::NotEnoughBytes);
    }

    #[test]
    fn block_pos_long_bit_layout() {
        let pos = BlockPos::new(1, 2, 3);
        let packed = pos.as_long();
        let expected = ((1_i64 & PACKED_X_MASK) << 38) | ((3_i64 & PACKED_Z_MASK) << 12) | (2_i64 & PACKED_Y_MASK);
        assert_eq!(packed, expected);
        assert_eq!(BlockPos::from_long(expected), pos);

        let neg = BlockPos::new(-1, -1, -1);
        assert_eq!(neg.as_long(), -1_i64);
        assert_eq!(BlockPos::from_long(-1_i64), BlockPos::new(-1, -1, -1));
    }
}
