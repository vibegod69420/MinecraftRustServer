use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub mod game_packet_types {
    use super::PacketType;

    pub const SERVERBOUND_SELECT_TRADE: PacketType = PacketType("serverbound_select_trade");
}

/// Minimal self-contained buffer stub for `FriendlyByteBuf`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn writer() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.read_byte()? as i32;
            result |= (current_byte & 127) << position;
            if (current_byte & 128) == 0 {
                break;
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt is too big",
                ));
            }
        }
        Ok(result)
    }

    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & !0x7F) == 0 {
                self.data.push(value as u8);
                return;
            }
            self.data.push(((value & 0x7F) as u8) | 0x80);
            value = ((value as u32) >> 7) as i32;
        }
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }
}

pub trait ServerGamePacketListener {
    fn handle_select_trade(&mut self, packet: &ServerboundSelectTradePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSelectTradePacket {
    item: i32,
}

impl ServerboundSelectTradePacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectTradePacket#ServerboundSelectTradePacket
    pub fn new(item: i32) -> Self {
        Self { item }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectTradePacket#ServerboundSelectTradePacket
    fn from_buf(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let item = buf.read_var_int()?;
        Ok(Self { item })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectTradePacket#write
    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.item);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectTradePacket#type
    pub fn get_type(&self) -> PacketType {
        game_packet_types::SERVERBOUND_SELECT_TRADE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectTradePacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_select_trade(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSelectTradePacket#getItem
    pub fn get_item(&self) -> i32 {
        self.item
    }

    pub fn stream_decode(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Self::from_buf(buf)
    }

    pub fn stream_encode(&self, buf: &mut FriendlyByteBuf) {
        self.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen_item: Option<i32>,
    }

    impl ServerGamePacketListener for TestListener {
        fn handle_select_trade(&mut self, packet: &ServerboundSelectTradePacket) {
            self.seen_item = Some(packet.get_item());
        }
    }

    #[test]
    fn packet_accessors_and_type() {
        let packet = ServerboundSelectTradePacket::new(5);
        assert_eq!(packet.get_item(), 5);
        assert_eq!(packet.get_type(), game_packet_types::SERVERBOUND_SELECT_TRADE);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ServerboundSelectTradePacket::new(12);
        let mut listener = TestListener { seen_item: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen_item, Some(12));
    }

    #[test]
    fn stream_codec_roundtrip() {
        let packet = ServerboundSelectTradePacket::new(42);
        let mut buf = FriendlyByteBuf::writer();
        packet.stream_encode(&mut buf);
        let encoded = buf.into_inner();
        let mut read_buf = FriendlyByteBuf::new(encoded);
        let decoded = ServerboundSelectTradePacket::stream_decode(&mut read_buf).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn negative_and_extreme_varint_values_roundtrip() {
        for item in [0, -1, i32::MIN, i32::MAX, -123456] {
            let packet = ServerboundSelectTradePacket::new(item);
            let mut buf = FriendlyByteBuf::writer();
            packet.stream_encode(&mut buf);
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ServerboundSelectTradePacket::stream_decode(&mut read_buf).unwrap();
            assert_eq!(decoded.get_item(), item);
        }
    }

    #[test]
    fn varint_too_big_returns_error() {
        let data = vec![0x80, 0x80, 0x80, 0x80, 0x80];
        let mut buf = FriendlyByteBuf::new(data);
        let err = ServerboundSelectTradePacket::stream_decode(&mut buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
