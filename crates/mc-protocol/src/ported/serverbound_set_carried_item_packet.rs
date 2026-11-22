use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    buf: BytesMut,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            buf: BytesMut::from(bytes),
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#asByteBuf
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeShort
    pub fn write_short(&mut self, value: i32) {
        self.buf.put_i16(value as i16);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readShort
    pub fn read_short(&mut self) -> i16 {
        self.buf.get_i16()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const SERVERBOUND_SET_CARRIED_ITEM: PacketType =
    PacketType("minecraft:serverbound_set_carried_item");

pub trait ServerGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerGamePacketListener#handleSetCarriedItem
    fn handle_set_carried_item(&mut self, packet: &ServerboundSetCarriedItemPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCarriedItemPacket {
    slot: i32,
}

impl ServerboundSetCarriedItemPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket#<init>
    pub fn new(slot: i32) -> Self {
        Self { slot }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        let slot = buf.read_short() as i32;
        Self::new(slot)
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_short(self.slot);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket#type
    pub fn packet_type(&self) -> PacketType {
        SERVERBOUND_SET_CARRIED_ITEM
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_carried_item(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket#getSlot
    pub fn get_slot(&self) -> i32 {
        self.slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_slot() {
        let pkt = ServerboundSetCarriedItemPacket::new(5);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ServerboundSetCarriedItemPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_slot(), 5);
        assert_eq!(decoded.packet_type(), SERVERBOUND_SET_CARRIED_ITEM);
    }

    #[test]
    fn negative_slot_round_trip() {
        let pkt = ServerboundSetCarriedItemPacket::new(-1);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ServerboundSetCarriedItemPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_slot(), -1);
    }

    #[test]
    fn write_truncates_to_short_like_java() {
        let pkt = ServerboundSetCarriedItemPacket::new(0x0001_0001);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ServerboundSetCarriedItemPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_slot(), 1);
    }

    #[test]
    fn i16_bounds_round_trip() {
        for slot in [i16::MIN as i32, i16::MAX as i32, 0] {
            let pkt = ServerboundSetCarriedItemPacket::new(slot);
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
            let decoded = ServerboundSetCarriedItemPacket::from_buf(&mut read_buf);
            assert_eq!(decoded.get_slot(), slot);
        }
    }

    #[test]
    fn handle_calls_listener() {
        struct L {
            called: bool,
            slot: i32,
        }
        impl ServerGamePacketListener for L {
            fn handle_set_carried_item(&mut self, packet: &ServerboundSetCarriedItemPacket) {
                self.called = true;
                self.slot = packet.get_slot();
            }
        }
        let pkt = ServerboundSetCarriedItemPacket::new(3);
        let mut l = L {
            called: false,
            slot: -1,
        };
        pkt.handle(&mut l);
        assert!(l.called);
        assert_eq!(l.slot, 3);
    }
}
