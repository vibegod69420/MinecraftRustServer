use bytes::{Buf, BufMut, BytesMut};

/// Minimal self-contained buffer for packet encoding/decoding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: BytesMut,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: BytesMut::new(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: BytesMut::from(bytes),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data.to_vec()
    }

    pub fn remaining(&self) -> usize {
        self.data.remaining()
    }

    pub fn read_varint(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.data.get_u8() as i32;
            result |= (b & 0x7f).wrapping_shl(shift & 31);
            if (b & 0x80) == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                return result;
            }
        }
    }

    pub fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut b = ((value as u32) & 0x7f) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                b |= 0x80;
            }
            self.data.put_u8(b);
            if value == 0 {
                break;
            }
        }
    }

    pub fn read_utf(&mut self) -> String {
        self.read_utf_with_max(32_767)
    }

    pub fn read_utf_with_max(&mut self, max_len: usize) -> String {
        let len = self.read_varint() as usize;
        let max_bytes = max_len.saturating_mul(3).saturating_add(3);
        assert!(len <= max_bytes, "String too long (bytes)");
        let bytes = self.data.split_to(len);
        let s = String::from_utf8(bytes.to_vec()).expect("invalid utf8");
        assert!(s.chars().count() <= max_len, "String too long (chars)");
        s
    }

    pub fn write_utf(&mut self, value: &str) {
        self.write_utf_with_max(value, 32_767);
    }

    pub fn write_utf_with_max(&mut self, value: &str, max_len: usize) {
        let char_count = value.chars().count();
        assert!(char_count <= max_len, "String too long");
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.data.put_slice(bytes);
    }
}

pub const PACKET_TYPE_SERVERBOUND_CHAT_COMMAND: &str = "minecraft:serverbound_chat_command";

pub trait ServerGamePacketListener {
    fn handle_chat_command(&mut self, packet: &ServerboundChatCommandPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatCommandPacket {
    command: String,
}

impl ServerboundChatCommandPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandPacket#<init>
    pub fn new(command: String) -> Self {
        Self { command }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandPacket#<init>
    fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        Self::new(buf.read_utf())
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandPacket#write
    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.command);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandPacket#command
    pub fn command(&self) -> &str {
        &self.command
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandPacket#type
    pub fn type_name(&self) -> &'static str {
        PACKET_TYPE_SERVERBOUND_CHAT_COMMAND
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChatCommandPacket#handle
    pub fn handle<L: ServerGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_chat_command(self);
    }

    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        Self::from_buf(buf)
    }

    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        self.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        seen: Option<String>,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_chat_command(&mut self, packet: &ServerboundChatCommandPacket) {
            self.seen = Some(packet.command().to_owned());
        }
    }

    #[test]
    fn roundtrip_command() {
        let packet = ServerboundChatCommandPacket::new("give @p diamond 1".to_owned());
        let mut buf = FriendlyByteBuf::new();
        packet.encode(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(&buf.into_bytes());
        let decoded = ServerboundChatCommandPacket::decode(&mut read_buf);
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn roundtrip_empty_and_unicode() {
        let empty = ServerboundChatCommandPacket::new(String::new());
        let mut buf = FriendlyByteBuf::new();
        empty.encode(&mut buf);
        let decoded = ServerboundChatCommandPacket::decode(&mut FriendlyByteBuf::from_bytes(
            &buf.into_bytes(),
        ));
        assert_eq!(decoded.command(), "");

        let unicode = ServerboundChatCommandPacket::new("say héllo 世界".to_owned());
        let mut buf = FriendlyByteBuf::new();
        unicode.encode(&mut buf);
        let decoded = ServerboundChatCommandPacket::decode(&mut FriendlyByteBuf::from_bytes(
            &buf.into_bytes(),
        ));
        assert_eq!(decoded.command(), "say héllo 世界");
    }

    #[test]
    fn type_and_handle() {
        let packet = ServerboundChatCommandPacket::new("time set day".to_owned());
        assert_eq!(packet.type_name(), "minecraft:serverbound_chat_command");
        let mut listener = MockListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen.as_deref(), Some("time set day"));
    }

    #[test]
    fn varint_negative_and_overflow_edges() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_varint(-1);
        buf.write_varint(i32::MIN);
        buf.write_varint(i32::MAX);
        buf.write_varint(0);
        let bytes = buf.into_bytes();
        let mut r = FriendlyByteBuf::from_bytes(&bytes);
        assert_eq!(r.read_varint(), -1);
        assert_eq!(r.read_varint(), i32::MIN);
        assert_eq!(r.read_varint(), i32::MAX);
        assert_eq!(r.read_varint(), 0);
    }

    #[test]
    fn float_bits_preserved_as_i32_patterns() {
        let neg_inf_bits = f32::NEG_INFINITY.to_bits() as i32;
        let nan_bits = f64::NAN.to_bits();
        let lo = nan_bits as i32;
        let hi = ((nan_bits >> 32) & 0xFFFF_FFFF) as u32 as i32;

        let mut buf = FriendlyByteBuf::new();
        buf.write_varint(neg_inf_bits);
        buf.write_varint(lo);
        buf.write_varint(hi);
        let mut r = FriendlyByteBuf::from_bytes(&buf.into_bytes());

        assert_eq!(r.read_varint() as u32, f32::NEG_INFINITY.to_bits());
        let got_lo = r.read_varint() as u32 as u64;
        let got_hi = r.read_varint() as u32 as u64;
        let got = (got_hi << 32) | got_lo;
        assert_eq!(got, nan_bits);
    }
}
