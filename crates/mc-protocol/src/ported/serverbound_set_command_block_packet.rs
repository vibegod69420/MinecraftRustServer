use bytes::{Buf, BufMut};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }

    pub fn write_byte(&mut self, value: i8) {
        self.data.put_i8(value);
    }

    pub fn read_byte(&mut self) -> i8 {
        assert!(self.remaining() >= 1, "buffer underflow");
        let v = (&self.data[self.read_pos..]).get_i8();
        self.read_pos += 1;
        v
    }

    pub fn write_int(&mut self, value: i32) {
        self.data.put_i32(value);
    }

    pub fn read_int(&mut self) -> i32 {
        assert!(self.remaining() >= 4, "buffer underflow");
        let v = (&self.data[self.read_pos..]).get_i32();
        self.read_pos += 4;
        v
    }

    pub fn write_long(&mut self, value: i64) {
        self.data.put_i64(value);
    }

    pub fn read_long(&mut self) -> i64 {
        assert!(self.remaining() >= 8, "buffer underflow");
        let v = (&self.data[self.read_pos..]).get_i64();
        self.read_pos += 8;
        v
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7F) == 0 {
                self.write_byte(val as i8);
                return;
            }
            self.write_byte(((val & 0x7F) | 0x80) as u8 as i8);
            val >>= 7;
        }
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_byte() as u8;
            result |= ((b & 0x7F) as i32) << shift;
            if (b & 0x80) == 0 {
                return result;
            }
            shift += 7;
            assert!(shift < 32, "VarInt too big");
        }
    }

    pub fn write_utf(&mut self, value: &str) {
        self.write_utf_with_max_len(value, 32767);
    }

    pub fn write_utf_with_max_len(&mut self, value: &str, max_len: usize) {
        let bytes = value.as_bytes();
        assert!(bytes.len() <= max_len, "String too long");
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    pub fn read_utf(&mut self) -> String {
        self.read_utf_with_max_len(32767)
    }

    pub fn read_utf_with_max_len(&mut self, max_len: usize) -> String {
        let len = self.read_var_int();
        assert!(len >= 0, "Negative string length");
        let len = len as usize;
        assert!(len <= max_len.saturating_mul(3), "String too long");
        assert!(self.remaining() >= len, "buffer underflow");
        let bytes = &self.data[self.read_pos..self.read_pos + len];
        let s = std::str::from_utf8(bytes).expect("invalid utf8").to_owned();
        assert!(s.chars().count() <= max_len, "String too long");
        self.read_pos += len;
        s
    }

    pub fn write_block_pos(&mut self, pos: BlockPos) {
        self.write_long(pos.as_long());
    }

    pub fn read_block_pos(&mut self) -> BlockPos {
        BlockPos::from_long(self.read_long())
    }

    pub fn write_enum_ordinal(&mut self, ordinal: i32) {
        self.write_var_int(ordinal);
    }

    pub fn read_enum_ordinal(&mut self) -> i32 {
        self.read_var_int()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    const PACKED_X_LENGTH: u32 = 26;
    const PACKED_Z_LENGTH: u32 = 26;
    const PACKED_Y_LENGTH: u32 = 12;
    const PACKED_X_MASK: i64 = (1_i64 << Self::PACKED_X_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1_i64 << Self::PACKED_Y_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1_i64 << Self::PACKED_Z_LENGTH) - 1;
    const Z_OFFSET: u32 = Self::PACKED_Y_LENGTH;
    const X_OFFSET: u32 = Self::PACKED_Y_LENGTH + Self::PACKED_Z_LENGTH;

    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(self) -> i64 {
        let mut l: i64 = 0;
        l |= (self.x as i64 & Self::PACKED_X_MASK) << Self::X_OFFSET;
        l |= (self.y as i64 & Self::PACKED_Y_MASK);
        l | ((self.z as i64 & Self::PACKED_Z_MASK) << Self::Z_OFFSET)
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn from_long(packed: i64) -> Self {
        let x = (packed << (64 - Self::X_OFFSET - Self::PACKED_X_LENGTH)
            >> (64 - Self::PACKED_X_LENGTH)) as i32;
        let y = (packed << (64 - Self::PACKED_Y_LENGTH) >> (64 - Self::PACKED_Y_LENGTH)) as i32;
        let z = (packed << (64 - Self::Z_OFFSET - Self::PACKED_Z_LENGTH)
            >> (64 - Self::PACKED_Z_LENGTH)) as i32;
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandBlockMode {
    Sequence,
    Auto,
    Redstone,
}

impl CommandBlockMode {
    fn ordinal(self) -> i32 {
        match self {
            CommandBlockMode::Sequence => 0,
            CommandBlockMode::Auto => 1,
            CommandBlockMode::Redstone => 2,
        }
    }

    fn from_ordinal(v: i32) -> Self {
        match v {
            0 => CommandBlockMode::Sequence,
            1 => CommandBlockMode::Auto,
            2 => CommandBlockMode::Redstone,
            _ => panic!("invalid CommandBlockMode ordinal: {v}"),
        }
    }
}

const FLAG_TRACK_OUTPUT: i32 = 1;
const FLAG_CONDITIONAL: i32 = 2;
const FLAG_AUTOMATIC: i32 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundSetCommandBlockPacket {
    pos: BlockPos,
    command: String,
    track_output: bool,
    conditional: bool,
    automatic: bool,
    mode: CommandBlockMode,
}

impl ServerboundSetCommandBlockPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#<init>
    pub fn new(
        pos: BlockPos,
        command: String,
        mode: CommandBlockMode,
        track_output: bool,
        conditional: bool,
        automatic: bool,
    ) -> Self {
        Self {
            pos,
            command,
            track_output,
            conditional,
            automatic,
            mode,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let pos = buf.read_block_pos();
        let command = buf.read_utf();
        let mode = CommandBlockMode::from_ordinal(buf.read_enum_ordinal());
        let flags = buf.read_byte() as i32;
        Self {
            pos,
            command,
            track_output: (flags & FLAG_TRACK_OUTPUT) != 0,
            conditional: (flags & FLAG_CONDITIONAL) != 0,
            automatic: (flags & FLAG_AUTOMATIC) != 0,
            mode,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(self.pos);
        buf.write_utf(&self.command);
        buf.write_enum_ordinal(self.mode.ordinal());
        let mut flags: i32 = 0;
        if self.track_output {
            flags |= FLAG_TRACK_OUTPUT;
        }
        if self.conditional {
            flags |= FLAG_CONDITIONAL;
        }
        if self.automatic {
            flags |= FLAG_AUTOMATIC;
        }
        buf.write_byte(flags as i8);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#getCommand
    pub fn get_command(&self) -> &str {
        &self.command
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#isTrackOutput
    pub fn is_track_output(&self) -> bool {
        self.track_output
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#isConditional
    pub fn is_conditional(&self) -> bool {
        self.conditional
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#isAutomatic
    pub fn is_automatic(&self) -> bool {
        self.automatic
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket#getMode
    pub fn get_mode(&self) -> CommandBlockMode {
        self.mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_roundtrip_preserves_all_fields() {
        let packet = ServerboundSetCommandBlockPacket::new(
            BlockPos { x: 10, y: 64, z: -20 },
            "say hello".to_owned(),
            CommandBlockMode::Auto,
            true,
            false,
            true,
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded = ServerboundSetCommandBlockPacket::read(&mut buf);
        assert_eq!(decoded, packet);
        assert_eq!(buf.remaining(), 0);
    }

    #[test]
    fn flag_byte_encoding_matches_java_layout() {
        let packet = ServerboundSetCommandBlockPacket::new(
            BlockPos { x: 0, y: 0, z: 0 },
            String::new(),
            CommandBlockMode::Sequence,
            false,
            true,
            true,
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let bytes = buf.into_vec();
        // last byte is flags
        assert_eq!(*bytes.last().unwrap(), 0b110);
    }

    #[test]
    fn block_pos_packing_roundtrip_with_negative_coordinates() {
        let positions = [
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: -1, y: -1, z: -1 },
            BlockPos { x: 12345, y: 255, z: -5432 },
            BlockPos { x: -33_554_432, y: -2048, z: 33_554_431 },
        ];
        for pos in positions {
            let packed = pos.as_long();
            let decoded = BlockPos::from_long(packed);
            assert_eq!(decoded, pos);
        }
    }

    #[test]
    fn read_ignores_unknown_high_bits_in_flag_byte() {
        let packet = ServerboundSetCommandBlockPacket::new(
            BlockPos { x: 1, y: 2, z: 3 },
            "cmd".to_owned(),
            CommandBlockMode::Redstone,
            true,
            false,
            true,
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut bytes = buf.into_vec();
        let last = bytes.len() - 1;
        bytes[last] |= 0xF0;
        let mut read_buf = FriendlyByteBuf::from_vec(bytes);
        let decoded = ServerboundSetCommandBlockPacket::read(&mut read_buf);
        assert!(decoded.is_track_output());
        assert!(!decoded.is_conditional());
        assert!(decoded.is_automatic());
        assert_eq!(decoded.get_mode(), CommandBlockMode::Redstone);
    }

    #[test]
    fn command_string_roundtrip_handles_empty_and_unicode() {
        let packet = ServerboundSetCommandBlockPacket::new(
            BlockPos { x: -7, y: 0, z: 7 },
            "say §aこんにちは".to_owned(),
            CommandBlockMode::Sequence,
            true,
            true,
            false,
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded = ServerboundSetCommandBlockPacket::read(&mut buf);
        assert_eq!(decoded.get_command(), "say §aこんにちは");
        assert!(decoded.is_track_output());
        assert!(decoded.is_conditional());
        assert!(!decoded.is_automatic());
    }
}
