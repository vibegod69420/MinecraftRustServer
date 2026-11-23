use bytes::{Buf, BufMut};

/// Minimal entity trait for self-contained compilation.
pub trait EntityLike {
    fn get_id(&self) -> i32;
}

/// Minimal level trait for self-contained compilation.
pub trait LevelLike {
    type Entity;
    fn get_entity(&self, id: i32) -> Option<Self::Entity>;
}

/// Placeholder packet type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_ENTITY_EVENT_TYPE: PacketType =
    PacketType("CLIENTBOUND_ENTITY_EVENT");

/// Minimal listener trait for self-contained compilation.
pub trait ClientGamePacketListener {
    fn handle_entity_event(&mut self, packet: &ClientboundEntityEventPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityEventPacket {
    entity_id: i32,
    event_id: i8,
}

impl ClientboundEntityEventPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityEventPacket#<init>
    pub fn new(entity: &dyn EntityLike, event_id: i8) -> Self {
        Self {
            entity_id: entity.get_id(),
            event_id,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityEventPacket#<init>
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let entity_id = buf.get_i32();
        let event_id = buf.get_i8();
        Self { entity_id, event_id }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityEventPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_i32(self.entity_id);
        buf.put_i8(self.event_id);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityEventPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_ENTITY_EVENT_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityEventPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_entity_event(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityEventPacket#getEntity
    pub fn get_entity<L: LevelLike>(&self, level: &L) -> Option<L::Entity> {
        level.get_entity(self.entity_id)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityEventPacket#getEventId
    pub fn get_event_id(&self) -> i8 {
        self.event_id
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

    #[derive(Debug, PartialEq, Eq)]
    struct DummyEntity(i32);

    struct MockLevel;
    impl LevelLike for MockLevel {
        type Entity = DummyEntity;
        fn get_entity(&self, id: i32) -> Option<Self::Entity> {
            if id == 42 {
                Some(DummyEntity(42))
            } else {
                None
            }
        }
    }

    #[derive(Default)]
    struct MockListener {
        entity_id: i32,
        event_id: i8,
    }
    impl ClientGamePacketListener for MockListener {
        fn handle_entity_event(&mut self, packet: &ClientboundEntityEventPacket) {
            self.entity_id = packet.entity_id;
            self.event_id = packet.event_id;
        }
    }

    #[test]
    fn test_new_and_get_event_id() {
        let pkt = ClientboundEntityEventPacket::new(&MockEntity(10), 3);
        assert_eq!(pkt.get_event_id(), 3);
    }

    #[test]
    fn test_write_read_roundtrip() {
        let pkt = ClientboundEntityEventPacket::new(&MockEntity(-100), -1);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundEntityEventPacket::read(&mut reader);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_negative_and_extreme_values() {
        let pkt = ClientboundEntityEventPacket::new(&MockEntity(i32::MIN), i8::MIN);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundEntityEventPacket::read(&mut reader);
        assert_eq!(decoded.entity_id, i32::MIN);
        assert_eq!(decoded.get_event_id(), i8::MIN);

        let pkt2 = ClientboundEntityEventPacket::new(&MockEntity(i32::MAX), i8::MAX);
        let mut buf2 = BytesMut::new();
        pkt2.write(&mut buf2);
        let mut reader2 = buf2.freeze();
        let decoded2 = ClientboundEntityEventPacket::read(&mut reader2);
        assert_eq!(decoded2.entity_id, i32::MAX);
        assert_eq!(decoded2.get_event_id(), i8::MAX);
    }

    #[test]
    fn test_get_entity() {
        let pkt = ClientboundEntityEventPacket::new(&MockEntity(42), 1);
        let level = MockLevel;
        assert_eq!(pkt.get_entity(&level), Some(DummyEntity(42)));

        let missing = ClientboundEntityEventPacket::new(&MockEntity(99), 1);
        assert_eq!(missing.get_entity(&level), None);
    }

    #[test]
    fn test_handle_and_type() {
        let pkt = ClientboundEntityEventPacket::new(&MockEntity(7), 35);
        assert_eq!(pkt.get_type(), CLIENTBOUND_ENTITY_EVENT_TYPE);
        let mut listener = MockListener::default();
        pkt.handle(&mut listener);
        assert_eq!(listener.entity_id, 7);
        assert_eq!(listener.event_id, 35);
    }
}
