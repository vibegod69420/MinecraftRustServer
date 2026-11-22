use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub u32);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_SET_CURSOR_ITEM: PacketType = PacketType(0x33);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item_id: i32,
    pub count: i32,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#<init>
    pub fn new(item_id: i32, count: i32) -> Self {
        Self { item_id, count }
    }

    // JAVA: net.minecraft.world.item.ItemStack#EMPTY
    pub fn empty() -> Self {
        Self {
            item_id: 0,
            count: 0,
        }
    }

    // JAVA: net.minecraft.world.item.ItemStack#isEmpty
    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }

    // JAVA: net.minecraft.world.item.ItemStack#OPTIONAL_STREAM_CODEC
    pub fn encode_optional<B: BufMut>(&self, buf: &mut B) {
        if self.is_empty() {
            buf.put_i32(0);
        } else {
            buf.put_i32(self.count);
            buf.put_i32(self.item_id);
        }
    }

    // JAVA: net.minecraft.world.item.ItemStack#OPTIONAL_STREAM_CODEC
    pub fn decode_optional<B: Buf>(buf: &mut B) -> Self {
        let count = buf.get_i32();
        if count <= 0 {
            return Self::empty();
        }
        let item_id = buf.get_i32();
        Self::new(item_id, count)
    }
}

pub trait ClientGamePacketListener {
    fn handle_set_cursor_item(&mut self, packet: &ClientboundSetCursorItemPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetCursorItemPacket {
    contents: ItemStack,
}

impl ClientboundSetCursorItemPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket#<init>
    pub fn new(contents: ItemStack) -> Self {
        Self { contents }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket#contents
    pub fn contents(&self) -> &ItemStack {
        &self.contents
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket#type
    pub fn packet_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_SET_CURSOR_ITEM
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_set_cursor_item(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket#STREAM_CODEC
    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        self.contents.encode_optional(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket#STREAM_CODEC
    pub fn decode<B: Buf>(buf: &mut B) -> Self {
        Self::new(ItemStack::decode_optional(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener {
        seen: Option<ItemStack>,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_set_cursor_item(&mut self, packet: &ClientboundSetCursorItemPacket) {
            self.seen = Some(packet.contents().clone());
        }
    }

    #[test]
    fn roundtrip_non_empty_item_stack() {
        let pkt = ClientboundSetCursorItemPacket::new(ItemStack::new(42, 64));
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let decoded = ClientboundSetCursorItemPacket::decode(&mut buf.freeze());
        assert_eq!(decoded, pkt);
        assert_eq!(decoded.packet_type(), GamePacketTypes::CLIENTBOUND_SET_CURSOR_ITEM);
    }

    #[test]
    fn roundtrip_empty_item_stack() {
        let pkt = ClientboundSetCursorItemPacket::new(ItemStack::empty());
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let decoded = ClientboundSetCursorItemPacket::decode(&mut buf.freeze());
        assert!(decoded.contents().is_empty());
    }

    #[test]
    fn negative_count_decodes_as_empty() {
        let mut buf = BytesMut::new();
        buf.put_i32(-1);
        buf.put_i32(123);
        let decoded = ItemStack::decode_optional(&mut buf.freeze());
        assert!(decoded.is_empty());
        assert_eq!(decoded, ItemStack::empty());
    }

    #[test]
    fn overflow_and_edge_integer_values() {
        let stack = ItemStack::new(i32::MAX, i32::MAX);
        let mut buf = BytesMut::new();
        stack.encode_optional(&mut buf);
        let decoded = ItemStack::decode_optional(&mut buf.freeze());
        assert_eq!(decoded.item_id, i32::MAX);
        assert_eq!(decoded.count, i32::MAX);

        let min_stack = ItemStack::new(i32::MIN, 1);
        let mut buf2 = BytesMut::new();
        min_stack.encode_optional(&mut buf2);
        let decoded2 = ItemStack::decode_optional(&mut buf2.freeze());
        assert_eq!(decoded2.item_id, i32::MIN);
        assert_eq!(decoded2.count, 1);
    }

    #[test]
    fn float_bits_and_handle_dispatch() {
        let id = f32::NAN.to_bits() as i32;
        let count = f32::INFINITY.to_bits() as i32;
        let pkt = ClientboundSetCursorItemPacket::new(ItemStack::new(id, count));
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let decoded = ClientboundSetCursorItemPacket::decode(&mut buf.freeze());
        assert_eq!(decoded.contents().item_id, id);
        assert_eq!(decoded.contents().count, count);

        let mut listener = TestListener { seen: None };
        decoded.handle(&mut listener);
        assert_eq!(listener.seen.unwrap().item_id, id);
    }
}
