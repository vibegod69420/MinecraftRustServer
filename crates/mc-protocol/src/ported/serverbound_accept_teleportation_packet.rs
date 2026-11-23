use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundAcceptTeleportationPacket {
    id: i32,
}

pub trait ServerGamePacketListener {
    fn handle_accept_teleport_packet(&mut self, packet: &ServerboundAcceptTeleportationPacket);
}

impl ServerboundAcceptTeleportationPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket#<init>
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Self {
        let id = read_varint(buf);
        Self { id }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_varint(buf, self.id);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:accept_teleportation"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_accept_teleport_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
        id: i32,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_accept_teleport_packet(&mut self, packet: &ServerboundAcceptTeleportationPacket) {
            self.called = true;
            self.id = packet.get_id();
        }
    }

    #[test]
    fn roundtrip_zero_and_positive() {
        for id in [0, 1, 127, 128, 255, 2_147_483_647_i32] {
            let pkt = ServerboundAcceptTeleportationPacket::new(id);
            let mut buf = BytesMut::new();
            pkt.write(&mut buf);
            let mut slice: &[u8] = &buf;
            let decoded = ServerboundAcceptTeleportationPacket::read(&mut slice);
            assert_eq!(decoded.get_id(), id);
            assert!(slice.is_empty());
        }
    }

    #[test]
    fn roundtrip_negative_and_min() {
        for id in [-1, -128, -129, i32::MIN] {
            let pkt = ServerboundAcceptTeleportationPacket::new(id);
            let mut buf = BytesMut::new();
            pkt.write(&mut buf);
            let mut slice: &[u8] = &buf;
            let decoded = ServerboundAcceptTeleportationPacket::read(&mut slice);
            assert_eq!(decoded.get_id(), id);
        }
    }

    #[test]
    fn varint_negative_one_encoding() {
        let pkt = ServerboundAcceptTeleportationPacket::new(-1);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        assert_eq!(&buf[..], &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }

    #[test]
    fn handle_and_type() {
        let pkt = ServerboundAcceptTeleportationPacket::new(42);
        assert_eq!(pkt.packet_type(), "minecraft:accept_teleportation");
        let mut listener = MockListener { called: false, id: 0 };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.id, 42);
    }
}
