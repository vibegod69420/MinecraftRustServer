use bytes::{Buf, BufMut};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub const SERVERBOUND_TELEPORT_TO_ENTITY: PacketType = PacketType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundTeleportToEntityPacket {
    uuid: Uuid,
}

pub trait ServerGamePacketListener {
    fn handle_teleport_to_entity_packet(&mut self, packet: &ServerboundTeleportToEntityPacket);
}

pub trait ServerLevelEntityLookup {
    type Entity;
    fn get_entity_by_uuid(&self, uuid: Uuid) -> Option<Self::Entity>;
}

impl ServerboundTeleportToEntityPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundTeleportToEntityPacket#<init>(java.util.UUID)
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTeleportToEntityPacket#<init>(net.minecraft.network.FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, io::Error> {
        if buf.remaining() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "UUID too short",
            ));
        }
        let most_significant_bits = buf.get_i64();
        let least_significant_bits = buf.get_i64();
        Ok(Self {
            uuid: Uuid {
                most_significant_bits,
                least_significant_bits,
            },
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTeleportToEntityPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_i64(self.uuid.most_significant_bits);
        buf.put_i64(self.uuid.least_significant_bits);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTeleportToEntityPacket#type
    pub fn get_type(&self) -> PacketType {
        SERVERBOUND_TELEPORT_TO_ENTITY
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTeleportToEntityPacket#handle
    pub fn handle<L: ServerGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_teleport_to_entity_packet(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundTeleportToEntityPacket#getEntity
    pub fn get_entity<L: ServerLevelEntityLookup>(&self, level: &L) -> Option<L::Entity> {
        level.get_entity_by_uuid(self.uuid)
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        uuid: Option<Uuid>,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_teleport_to_entity_packet(
            &mut self,
            packet: &ServerboundTeleportToEntityPacket,
        ) {
            self.called = true;
            self.uuid = Some(packet.uuid());
        }
    }

    struct MockLevel;

    impl ServerLevelEntityLookup for MockLevel {
        type Entity = u32;
        fn get_entity_by_uuid(&self, uuid: Uuid) -> Option<Self::Entity> {
            if uuid.most_significant_bits == 1 && uuid.least_significant_bits == 2 {
                Some(42)
            } else {
                None
            }
        }
    }

    #[test]
    fn roundtrip_uuid_bits() {
        let uuid = Uuid {
            most_significant_bits: 0x0123_4567_89ab_cdef_u64 as i64,
            least_significant_bits: -1_i64,
        };
        let packet = ServerboundTeleportToEntityPacket::new(uuid);
        let mut buf = Vec::new();
        packet.write(&mut buf);
        assert_eq!(buf.len(), 16);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundTeleportToEntityPacket::read(&mut slice).unwrap();
        assert_eq!(decoded.uuid(), uuid);
    }

    #[test]
    fn read_short_buffer_returns_error() {
        let mut slice: &[u8] = &[0u8; 15];
        let err = ServerboundTeleportToEntityPacket::read(&mut slice);
        assert!(err.is_err());
    }

    #[test]
    fn get_entity_looks_up_by_uuid() {
        let packet = ServerboundTeleportToEntityPacket::new(Uuid {
            most_significant_bits: 1,
            least_significant_bits: 2,
        });
        assert_eq!(packet.get_entity(&MockLevel), Some(42));
        let missing = ServerboundTeleportToEntityPacket::new(Uuid {
            most_significant_bits: 9,
            least_significant_bits: 9,
        });
        assert_eq!(missing.get_entity(&MockLevel), None);
    }

    #[test]
    fn handle_dispatches_and_type_is_stable() {
        let uuid = Uuid {
            most_significant_bits: i64::MIN,
            least_significant_bits: i64::MAX,
        };
        let packet = ServerboundTeleportToEntityPacket::new(uuid);
        let mut listener = MockListener {
            called: false,
            uuid: None,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.uuid, Some(uuid));
        assert_eq!(packet.get_type(), SERVERBOUND_TELEPORT_TO_ENTITY);
    }
}
