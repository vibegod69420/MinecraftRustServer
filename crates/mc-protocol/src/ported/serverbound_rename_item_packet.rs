use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub mod game_packet_types {
    use super::PacketType;

    pub const SERVERBOUND_RENAME_ITEM: PacketType = PacketType("serverbound_rename_item");
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

    pub fn read_utf(&mut self) -> io::Result<String> {
        self.read_utf_with_max_len(32767)
    }

    pub fn read_utf_with_max_len(&mut self, max_len: usize) -> io::Result<String> {
        let len = self.read_var_int()? as usize;
        if len > max_len.wrapping_mul(4).wrapping_add(3) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "String too long (encoded)",
            ));
        }
        if self.remaining() < len {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        let s = String::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
        if s.chars().count() > max_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "String too long (chars)",
            ));
        }
        Ok(s)
    }

    pub fn write_utf(&mut self, value: &str) -> io::Result<()> {
        self.write_utf_with_max_len(value, 32767)
    }

    pub fn write_utf_with_max_len(&mut self, value: &str, max_len: usize) -> io::Result<()> {
        let char_count = value.chars().count();
        if char_count > max_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "String too long",
            ));
        }
        let bytes = value.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
        Ok(())
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
    fn handle_rename_item(&mut self, packet: &ServerboundRenameItemPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundRenameItemPacket {
    name: String,
}

impl ServerboundRenameItemPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundRenameItemPacket#ServerboundRenameItemPacket
    pub fn new(name: String) -> Self {
        Self { name }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRenameItemPacket#ServerboundRenameItemPacket
    fn from_buf(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let name = buf.read_utf()?;
        Ok(Self { name })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRenameItemPacket#write
    fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_utf(&self.name)
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRenameItemPacket#type
    pub fn get_type(&self) -> PacketType {
        game_packet_types::SERVERBOUND_RENAME_ITEM
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRenameItemPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_rename_item(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundRenameItemPacket#getName
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn stream_decode(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Self::from_buf(buf)
    }

    pub fn stream_encode(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        self.write(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen_name: Option<String>,
    }

    impl ServerGamePacketListener for TestListener {
        fn handle_rename_item(&mut self, packet: &ServerboundRenameItemPacket) {
            self.seen_name = Some(packet.get_name().to_string());
        }
    }

    #[test]
    fn packet_accessors_and_type() {
        let packet = ServerboundRenameItemPacket::new("Sword".to_string());
        assert_eq!(packet.get_name(), "Sword");
        assert_eq!(packet.get_type(), game_packet_types::SERVERBOUND_RENAME_ITEM);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ServerboundRenameItemPacket::new("Anvil Name".to_string());
        let mut listener = TestListener { seen_name: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen_name.as_deref(), Some("Anvil Name"));
    }

    #[test]
    fn stream_codec_roundtrip() {
        let packet = ServerboundRenameItemPacket::new("Hello World!".to_string());
        let mut buf = FriendlyByteBuf::writer();
        packet.stream_encode(&mut buf).unwrap();
        let encoded = buf.into_inner();
        let mut read_buf = FriendlyByteBuf::new(encoded);
        let decoded = ServerboundRenameItemPacket::stream_decode(&mut read_buf).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn empty_and_unicode_names_roundtrip() {
        for name in ["", "épée", "日本語", "a\u{0000}b"] {
            let packet = ServerboundRenameItemPacket::new(name.to_string());
            let mut buf = FriendlyByteBuf::writer();
            packet.stream_encode(&mut buf).unwrap();
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ServerboundRenameItemPacket::stream_decode(&mut read_buf).unwrap();
            assert_eq!(decoded.get_name(), name);
        }
    }

    #[test]
    fn write_utf_rejects_too_long_string() {
        let mut buf = FriendlyByteBuf::writer();
        let long = "a".repeat(32768);
        let err = buf.write_utf(&long).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }
}
