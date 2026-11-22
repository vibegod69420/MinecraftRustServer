use bytes::{Buf, BufMut, BytesMut};

/// Minimal self-contained buffer for packet encoding/decoding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf {
    data: BytesMut,
}

impl RegistryFriendlyByteBuf {
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

    fn read_varint(&mut self) -> i32 {
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

    fn write_varint(&mut self, mut value: i32) {
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

    fn read_utf(&mut self, max_len: usize) -> String {
        let len = self.read_varint() as usize;
        let max_bytes = max_len.saturating_mul(3).saturating_add(3);
        assert!(len <= max_bytes, "String too long (bytes)");
        let bytes = self.data.split_to(len);
        let s = String::from_utf8(bytes.to_vec()).expect("invalid utf8");
        assert!(s.chars().count() <= max_len, "String too long (chars)");
        s
    }

    fn write_utf(&mut self, value: &str, max_len: usize) {
        let char_count = value.chars().count();
        assert!(char_count <= max_len, "String too long");
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.data.put_slice(bytes);
    }
}

/// Minimal self-contained Component representation.
/// In Java this is serialized via `ComponentSerialization.TRUSTED_STREAM_CODEC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub value: String,
}

impl Component {
    pub fn literal(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    fn read(buf: &mut RegistryFriendlyByteBuf) -> Self {
        Self {
            value: buf.read_utf(262_144),
        }
    }

    fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_utf(&self.value, 262_144);
    }
}

pub const PACKET_TYPE_CLIENTBOUND_SET_TITLE_TEXT: &str =
    "minecraft:clientbound_set_title_text";

pub trait ClientGamePacketListener {
    fn set_title_text(&mut self, packet: &ClientboundSetTitleTextPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetTitleTextPacket {
    text: Component,
}

impl ClientboundSetTitleTextPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket#<init>
    pub fn new(text: Component) -> Self {
        Self { text }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket#text
    pub fn text(&self) -> &Component {
        &self.text
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket#type
    pub fn type_name(&self) -> &'static str {
        PACKET_TYPE_CLIENTBOUND_SET_TITLE_TEXT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.set_title_text(self);
    }

    pub fn decode(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let text = Component::read(buf);
        Self::new(text)
    }

    pub fn encode(&self, buf: &mut RegistryFriendlyByteBuf) {
        self.text.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        seen: Option<ClientboundSetTitleTextPacket>,
    }

    impl ClientGamePacketListener for MockListener {
        fn set_title_text(&mut self, packet: &ClientboundSetTitleTextPacket) {
            self.seen = Some(packet.clone());
        }
    }

    #[test]
    fn roundtrip_simple_text() {
        let packet = ClientboundSetTitleTextPacket::new(Component::literal("Hello"));
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.encode(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(&buf.into_bytes());
        let decoded = ClientboundSetTitleTextPacket::decode(&mut read_buf);
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn roundtrip_empty_and_unicode() {
        let packet = ClientboundSetTitleTextPacket::new(Component::literal(""));
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.encode(&mut buf);
        let decoded = ClientboundSetTitleTextPacket::decode(
            &mut RegistryFriendlyByteBuf::from_bytes(&buf.into_bytes()),
        );
        assert_eq!(decoded.text().value, "");

        let unicode = "§aHello 世界 🌍";
        let packet = ClientboundSetTitleTextPacket::new(Component::literal(unicode));
        let mut buf = RegistryFriendlyByteBuf::new();
        packet.encode(&mut buf);
        let decoded = ClientboundSetTitleTextPacket::decode(
            &mut RegistryFriendlyByteBuf::from_bytes(&buf.into_bytes()),
        );
        assert_eq!(decoded.text().value, unicode);
    }

    #[test]
    fn type_and_handle() {
        let packet = ClientboundSetTitleTextPacket::new(Component::literal("title"));
        assert_eq!(
            packet.type_name(),
            "minecraft:clientbound_set_title_text"
        );
        let mut listener = MockListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen, Some(packet));
    }

    #[test]
    fn varint_negative_overflow_and_float_bits() {
        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_varint(-1);
        buf.write_varint(i32::MIN);
        buf.write_varint(i32::MAX);
        let nan_bits = f32::NAN.to_bits() as i32;
        buf.write_varint(nan_bits);

        let bytes = buf.into_bytes();
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(&bytes);
        assert_eq!(read_buf.read_varint(), -1);
        assert_eq!(read_buf.read_varint(), i32::MIN);
        assert_eq!(read_buf.read_varint(), i32::MAX);
        assert_eq!(read_buf.read_varint() as u32, f32::NAN.to_bits());
    }
}
