use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub &'static str);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const SERVERBOUND_CHAT_ACK: PacketType = PacketType("SERVERBOUND_CHAT_ACK");
}

/// Placeholder for `net.minecraft.network.protocol.game.ServerGamePacketListener`.
pub trait ServerGamePacketListener {
    fn handle_chat_ack(&mut self, packet: &ServerboundChatAckPacket);
}

/// Minimal `FriendlyByteBuf`-like buffer for VarInt encoding/decoding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut shift: i32 = 0;
        loop {
            if self.pos >= self.data.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "VarInt too short",
                ));
            }
            let b = self.data[self.pos];
            self.pos += 1;
            value |= ((b & 0x7F) as i32) << shift;
            if (b & 0x80) == 0 {
                return Ok(value);
            }
            shift += 7;
            if shift >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }
    }

    pub fn write_var_int(&mut self, value: i32) -> &mut Self {
        let mut v = value;
        while (v & -128i32) != 0 {
            self.data.push(((v & 127) as u8) | 128);
            v = ((v as u32) >> 7) as i32;
        }
        self.data.push(v as u8);
        self
    }
}

/// Port of `net.minecraft.network.protocol.game.ServerboundChatAckPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServerboundChatAckPacket {
    offset: i32,
}

impl ServerboundChatAckPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundChatAckPacket#<init>
    pub fn new(offset: i32) -> Self {
        Self { offset }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatAckPacket#<init>(FriendlyByteBuf)
    fn from_friendly_byte_buf(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let offset = buf.read_var_int()?;
        Ok(Self::new(offset))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatAckPacket#write
    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.offset);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatAckPacket#offset
    pub fn offset(&self) -> i32 {
        self.offset
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatAckPacket#type
    pub fn type_(&self) -> PacketType {
        GamePacketTypes::SERVERBOUND_CHAT_ACK
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatAckPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_chat_ack(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        handled: bool,
        offset: i32,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_chat_ack(&mut self, packet: &ServerboundChatAckPacket) {
            self.handled = true;
            self.offset = packet.offset();
        }
    }

    #[test]
    fn new_and_offset_accessor() {
        let packet = ServerboundChatAckPacket::new(42);
        assert_eq!(packet.offset(), 42);
    }

    #[test]
    fn packet_type_is_chat_ack() {
        let packet = ServerboundChatAckPacket::new(0);
        assert_eq!(packet.type_(), GamePacketTypes::SERVERBOUND_CHAT_ACK);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ServerboundChatAckPacket::new(-123);
        let mut listener = MockListener {
            handled: false,
            offset: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.handled);
        assert_eq!(listener.offset, -123);
    }

    #[test]
    fn write_and_read_round_trip_edge_values() {
        for offset in [0, 1, 127, 128, 255, -1, -128, i32::MIN, i32::MAX] {
            let packet = ServerboundChatAckPacket::new(offset);
            let mut buf = FriendlyByteBuf::default();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded =
                ServerboundChatAckPacket::from_friendly_byte_buf(&mut read_buf).unwrap();
            assert_eq!(decoded.offset(), offset);
            assert_eq!(read_buf.remaining(), 0);
        }
    }

    #[test]
    fn varint_encoding_known_values() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(0);
        assert_eq!(buf.into_inner(), vec![0x00]);

        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(300);
        assert_eq!(buf.into_inner(), vec![0xAC, 0x02]);

        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(-1);
        assert_eq!(buf.into_inner(), vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }

    #[test]
    fn read_var_int_rejects_short_buffer() {
        let mut buf = FriendlyByteBuf::new(vec![0x80]);
        assert!(buf.read_var_int().is_err());
    }
}
