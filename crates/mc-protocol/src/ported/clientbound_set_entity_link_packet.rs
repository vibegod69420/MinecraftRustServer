use bytes::{Buf, BufMut};

/// Minimal entity trait for self-contained compilation.
pub trait EntityLike {
    fn get_id(&self) -> i32;
}

/// Placeholder packet type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_SET_ENTITY_LINK_TYPE: PacketType =
    PacketType("CLIENTBOUND_SET_ENTITY_LINK");

/// Minimal listener trait for self-contained compilation.
pub trait ClientGamePacketListener {
    fn handle_entity_link_packet(&mut self, packet: &ClientboundSetEntityLinkPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetEntityLinkPacket {
    source_id: i32,
    dest_id: i32,
}

impl ClientboundSetEntityLinkPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket#<init>
    pub fn new(source: &dyn EntityLike, dest: Option<&dyn EntityLike>) -> Self {
        let source_id = source.get_id();
        let dest_id = match dest {
            Some(entity) => entity.get_id(),
            None => 0,
        };
        Self { source_id, dest_id }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket#<init>
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let source_id = buf.get_i32();
        let dest_id = buf.get_i32();
        Self { source_id, dest_id }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_i32(self.source_id);
        buf.put_i32(self.dest_id);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_SET_ENTITY_LINK_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_entity_link_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket#getSourceId
    pub fn get_source_id(&self) -> i32 {
        self.source_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket#getDestId
    pub fn get_dest_id(&self) -> i32 {
        self.dest_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockEntity(i32);
    impl EntityLike for MockEntity {
        fn get_id(&self) -> i32 {
            self.0
        }
    }

    #[derive(Default)]
    struct MockListener {
        source: i32,
        dest: i32,
    }
    impl ClientGamePacketListener for MockListener {
        fn handle_entity_link_packet(&mut self, packet: &ClientboundSetEntityLinkPacket) {
            self.source = packet.get_source_id();
            self.dest = packet.get_dest_id();
        }
    }

    #[test]
    fn test_new_with_and_without_dest() {
        let src = MockEntity(42);
        let dst = MockEntity(99);
        let with_dest = ClientboundSetEntityLinkPacket::new(&src, Some(&dst));
        assert_eq!(with_dest.get_source_id(), 42);
        assert_eq!(with_dest.get_dest_id(), 99);

        let without_dest = ClientboundSetEntityLinkPacket::new(&src, None);
        assert_eq!(without_dest.get_source_id(), 42);
        assert_eq!(without_dest.get_dest_id(), 0);
    }

    #[test]
    fn test_write_read_roundtrip() {
        let pkt = ClientboundSetEntityLinkPacket::new(&MockEntity(-1), Some(&MockEntity(123)));
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSetEntityLinkPacket::read(&mut reader);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_negative_and_extreme_ids() {
        let pkt =
            ClientboundSetEntityLinkPacket::new(&MockEntity(i32::MIN), Some(&MockEntity(i32::MAX)));
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSetEntityLinkPacket::read(&mut reader);
        assert_eq!(decoded.get_source_id(), i32::MIN);
        assert_eq!(decoded.get_dest_id(), i32::MAX);
    }

    #[test]
    fn test_handle_and_type() {
        let pkt = ClientboundSetEntityLinkPacket::new(&MockEntity(10), Some(&MockEntity(20)));
        assert_eq!(pkt.get_type(), CLIENTBOUND_SET_ENTITY_LINK_TYPE);
        let mut listener = MockListener::default();
        pkt.handle(&mut listener);
        assert_eq!(listener.source, 10);
        assert_eq!(listener.dest, 20);
    }
}
