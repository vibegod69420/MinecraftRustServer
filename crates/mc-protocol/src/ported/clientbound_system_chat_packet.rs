use bytes::{Buf, BufMut};

/// Minimal self-contained chat component representation.
/// The real network codec uses ComponentSerialization.TRUSTED_STREAM_CODEC (NBT);
/// this module uses a VarInt-prefixed UTF-8 string for self-contained roundtrips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub json: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self {
            json: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSystemChatPacket {
    content: Component,
    overlay: bool,
}

pub trait ClientGamePacketListener {
    fn handle_system_chat(&mut self, packet: &ClientboundSystemChatPacket);
}

impl ClientboundSystemChatPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSystemChatPacket#<init>
    pub fn new(content: Component, overlay: bool) -> Self {
        Self { content, overlay }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSystemChatPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Self {
        let content = read_component(buf);
        let overlay = read_boolean(buf);
        Self { content, overlay }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSystemChatPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_component(buf, &self.content);
        write_boolean(buf, self.overlay);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSystemChatPacket#content
    pub fn content(&self) -> &Component {
        &self.content
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSystemChatPacket#overlay
    pub fn overlay(&self) -> bool {
        self.overlay
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSystemChatPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:system_chat"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSystemChatPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_system_chat(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSystemChatPacket#isSkippable
    pub fn is_skippable(&self) -> bool {
        true
    }
}

fn read_component(buf: &mut &[u8]) -> Component {
    Component {
        json: read_utf(buf),
    }
}

fn write_component(buf: &mut impl BufMut, component: &Component) {
    write_utf(buf, &component.json);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
fn read_utf(buf: &mut &[u8]) -> String {
    let len = read_varint(buf) as usize;
    let mut bytes = vec![0u8; len];
    buf.copy_to_slice(&mut bytes);
    String::from_utf8(bytes).expect("invalid utf8 in component string")
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
fn write_utf(buf: &mut impl BufMut, value: &str) {
    let bytes = value.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_varint(buf: &mut &[u8]) -> i32 {
    let mut result: i32 = 0;
    let mut j: u32 = 0;
    loop {
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl((j.wrapping_mul(7)) & 31);
        j = j.wrapping_add(1);
        if j > 5 {
            panic!("VarInt too big");
        }
        if (b & 0x80) == 0 {
            return result;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_varint(buf: &mut impl BufMut, mut value: i32) {
    loop {
        if (value as u32 & 0xFFFF_FF80) != 0 {
            buf.put_u8(((value as u8) & 0x7F) | 0x80);
            value = ((value as u32) >> 7) as i32;
        } else {
            buf.put_u8(value as u8);
            return;
        }
    }
}

fn read_boolean(buf: &mut &[u8]) -> bool {
    buf.get_u8() != 0
}

fn write_boolean(buf: &mut impl BufMut, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
        overlay: bool,
        content: String,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_system_chat(&mut self, packet: &ClientboundSystemChatPacket) {
            self.called = true;
            self.overlay = packet.overlay();
            self.content = packet.content().json.clone();
        }
    }

    #[test]
    fn roundtrip_overlay_true() {
        let pkt = ClientboundSystemChatPacket::new(Component::literal("hello"), true);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSystemChatPacket::read(&mut slice);
        assert_eq!(decoded, pkt);
        assert!(decoded.overlay());
        assert_eq!(decoded.content().json, "hello");
        assert!(slice.is_empty());
    }

    #[test]
    fn roundtrip_overlay_false_empty_content() {
        let pkt = ClientboundSystemChatPacket::new(Component::literal(""), false);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSystemChatPacket::read(&mut slice);
        assert_eq!(decoded, pkt);
        assert!(!decoded.overlay());
        assert_eq!(decoded.content().json, "");
    }

    #[test]
    fn unicode_content_roundtrip() {
        let text = "{\"text\":\"こんにちは🌍\"}";
        let pkt = ClientboundSystemChatPacket::new(Component::literal(text), false);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSystemChatPacket::read(&mut slice);
        assert_eq!(decoded.content().json, text);
    }

    #[test]
    fn handle_and_flags() {
        let pkt = ClientboundSystemChatPacket::new(Component::literal("msg"), true);
        assert!(pkt.is_skippable());
        assert_eq!(pkt.packet_type(), "minecraft:system_chat");
        let mut listener = MockListener {
            called: false,
            overlay: false,
            content: String::new(),
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert!(listener.overlay);
        assert_eq!(listener.content, "msg");
    }
}
