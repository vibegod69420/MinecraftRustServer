use std::fmt;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum DecodeError {
    UnexpectedEof,
    VarIntTooBig,
    Utf8(FromUtf8Error),
    StringTooLong,
    InvalidEnumId(i32),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of buffer"),
            Self::VarIntTooBig => write!(f, "VarInt too big"),
            Self::Utf8(e) => write!(f, "invalid utf8: {e}"),
            Self::StringTooLong => write!(f, "string too long"),
            Self::InvalidEnumId(id) => write!(f, "invalid enum id: {id}"),
        }
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            pos: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn read_byte(&mut self) -> Result<u8, DecodeError> {
        if self.pos >= self.data.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value as u32 & !0x7Fu32) == 0 {
                self.data.push(value as u8);
                return self;
            }
            self.data.push(((value as u32 & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> Result<i32, DecodeError> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let byte = self.read_byte()? as i32;
            value |= (byte & 0x7F).wrapping_shl(position & 31);
            if (byte & 0x80) == 0 {
                return Ok(value);
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(DecodeError::VarIntTooBig);
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, value: i64) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> Result<i64, DecodeError> {
        if self.remaining() < 8 {
            return Err(DecodeError::UnexpectedEof);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf(&mut self, value: &str, max_length: usize) -> Result<&mut Self, DecodeError> {
        let utf8_bytes = value.as_bytes();
        let char_count = value.chars().count();
        if char_count > max_length {
            return Err(DecodeError::StringTooLong);
        }
        let max_bytes = max_length.saturating_mul(3);
        if utf8_bytes.len() > max_bytes {
            return Err(DecodeError::StringTooLong);
        }
        self.write_var_int(utf8_bytes.len() as i32);
        self.data.extend_from_slice(utf8_bytes);
        Ok(self)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf(&mut self, max_length: usize) -> Result<String, DecodeError> {
        let max_bytes = max_length.saturating_mul(3);
        let byte_len = self.read_var_int()? as usize;
        if byte_len > max_bytes {
            return Err(DecodeError::StringTooLong);
        }
        if self.remaining() < byte_len {
            return Err(DecodeError::UnexpectedEof);
        }
        let bytes = self.data[self.pos..self.pos + byte_len].to_vec();
        self.pos += byte_len;
        let s = String::from_utf8(bytes).map_err(DecodeError::Utf8)?;
        if s.chars().count() > max_length {
            return Err(DecodeError::StringTooLong);
        }
        Ok(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    const PACKED_X_LEN: i32 = 26;
    const PACKED_Z_LEN: i32 = 26;
    const PACKED_Y_LEN: i32 = 12;
    const PACKED_X_MASK: i64 = (1i64 << Self::PACKED_X_LEN) - 1;
    const PACKED_Y_MASK: i64 = (1i64 << Self::PACKED_Y_LEN) - 1;
    const PACKED_Z_MASK: i64 = (1i64 << Self::PACKED_Z_LEN) - 1;
    const Z_OFFSET: i32 = Self::PACKED_Y_LEN;
    const X_OFFSET: i32 = Self::PACKED_Y_LEN + Self::PACKED_Z_LEN;

    // JAVA: net.minecraft.core.BlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(&self) -> i64 {
        let mut l: i64 = 0i64;
        l |= ((self.x as i64) & Self::PACKED_X_MASK) << Self::X_OFFSET;
        l |= (self.y as i64) & Self::PACKED_Y_MASK;
        l | (((self.z as i64) & Self::PACKED_Z_MASK) << Self::Z_OFFSET)
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn of(packed: i64) -> Self {
        let x = (packed << 0i32 >> Self::X_OFFSET) as i32;
        let y = (packed << (64 - Self::PACKED_Y_LEN) >> (64 - Self::PACKED_Y_LEN)) as i32;
        let z = (packed << (64 - Self::X_OFFSET) >> (64 - Self::PACKED_Z_LEN)) as i32;
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

impl TestBlockMode {
    // JAVA: net.minecraft.world.level.block.state.properties.TestBlockMode#getId
    pub fn get_id(&self) -> i32 {
        match self {
            Self::Start => 0,
            Self::Log => 1,
            Self::Fail => 2,
            Self::Accept => 3,
        }
    }

    // JAVA: net.minecraft.world.level.block.state.properties.TestBlockMode#byId
    pub fn by_id(id: i32) -> Result<Self, DecodeError> {
        match id {
            0 => Ok(Self::Start),
            1 => Ok(Self::Log),
            2 => Ok(Self::Fail),
            3 => Ok(Self::Accept),
            _ => Err(DecodeError::InvalidEnumId(id)),
        }
    }
}

pub trait ServerGamePacketListener {
    fn handle_set_test_block(&mut self, packet: &ServerboundSetTestBlockPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetTestBlockPacket {
    pub position: BlockPos,
    pub mode: TestBlockMode,
    pub message: String,
}

impl ServerboundSetTestBlockPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSetTestBlockPacket#<init>
    pub fn new(position: BlockPos, mode: TestBlockMode, message: String) -> Self {
        Self {
            position,
            mode,
            message,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetTestBlockPacket#STREAM_CODEC
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> Result<(), DecodeError> {
        buf.write_long(self.position.as_long());
        buf.write_var_int(self.mode.get_id());
        buf.write_utf(&self.message, 32767)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetTestBlockPacket#STREAM_CODEC
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, DecodeError> {
        let position = BlockPos::of(buf.read_long()?);
        let mode = TestBlockMode::by_id(buf.read_var_int()?)?;
        let message = buf.read_utf(32767)?;
        Ok(Self::new(position, mode, message))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetTestBlockPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:set_test_block"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetTestBlockPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_test_block(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_pos_round_trip_and_negative() {
        let positions = [
            BlockPos::new(0, 0, 0),
            BlockPos::new(123, 64, -456),
            BlockPos::new(-33_554_432, -2048, -33_554_432),
            BlockPos::new(33_554_431, 2047, 33_554_431),
        ];
        for pos in positions {
            let packed = pos.as_long();
            let unpacked = BlockPos::of(packed);
            assert_eq!(pos, unpacked);
        }
    }

    #[test]
    fn block_pos_masks_overflow_bits() {
        let pos = BlockPos::new(i32::MAX, i32::MAX, i32::MIN);
        let unpacked = BlockPos::of(pos.as_long());
        assert_eq!(unpacked.x, (i32::MAX << 6 >> 6));
        assert_eq!(unpacked.y, (i32::MAX << 20 >> 20));
        assert_eq!(unpacked.z, (i32::MIN << 6 >> 6));
    }

    #[test]
    fn test_block_mode_round_trip_and_invalid() {
        for id in 0..=3 {
            let mode = TestBlockMode::by_id(id).unwrap();
            assert_eq!(mode.get_id(), id);
        }
        assert!(matches!(
            TestBlockMode::by_id(4),
            Err(DecodeError::InvalidEnumId(4))
        ));
        assert!(matches!(
            TestBlockMode::by_id(-1),
            Err(DecodeError::InvalidEnumId(-1))
        ));
    }

    #[test]
    fn packet_round_trip() {
        let packet = ServerboundSetTestBlockPacket::new(
            BlockPos::new(-10, 5, 42),
            TestBlockMode::Accept,
            "hello".to_string(),
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ServerboundSetTestBlockPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn packet_empty_message_and_handle() {
        struct Listener {
            called: bool,
            pos: Option<BlockPos>,
        }
        impl ServerGamePacketListener for Listener {
            fn handle_set_test_block(&mut self, packet: &ServerboundSetTestBlockPacket) {
                self.called = true;
                self.pos = Some(packet.position);
            }
        }

        let packet = ServerboundSetTestBlockPacket::new(
            BlockPos::new(0, -1, 0),
            TestBlockMode::Start,
            String::new(),
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf).unwrap();
        let decoded =
            ServerboundSetTestBlockPacket::read(&mut FriendlyByteBuf::from_bytes(buf.into_inner()))
                .unwrap();
        assert_eq!(decoded.message, "");

        let mut listener = Listener {
            called: false,
            pos: None,
        };
        decoded.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.pos, Some(BlockPos::new(0, -1, 0)));
        assert_eq!(decoded.packet_type(), "minecraft:set_test_block");
    }
}
