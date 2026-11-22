use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ClientCommonPacketListener {
    fn handle_resource_pack_pop(&mut self, packet: &ClientboundResourcePackPopPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResourcePackPopPacket {
    id: Option<Uuid>,
}

impl ClientboundResourcePackPopPacket {
    pub const TYPE: PacketType = PacketType("common/clientbound_resource_pack_pop");

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket#<init>(Optional)
    pub fn new(id: Option<Uuid>) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let present = buf.get_u8() != 0;
        let id = if present {
            Some(Uuid {
                most_significant_bits: buf.get_i64(),
                least_significant_bits: buf.get_i64(),
            })
        } else {
            None
        };
        Self { id }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket#write(FriendlyByteBuf)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        match self.id {
            Some(uuid) => {
                buf.put_u8(1);
                buf.put_i64(uuid.most_significant_bits);
                buf.put_i64(uuid.least_significant_bits);
            }
            None => {
                buf.put_u8(0);
            }
        }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket#type()
    pub fn packet_type(&self) -> PacketType {
        Self::TYPE
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket#handle(ClientCommonPacketListener)
    pub fn handle<L: ClientCommonPacketListener>(&self, listener: &mut L) {
        listener.handle_resource_pack_pop(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket#id()
    pub fn id(&self) -> Option<Uuid> {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        last: Option<ClientboundResourcePackPopPacket>,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_resource_pack_pop(&mut self, packet: &ClientboundResourcePackPopPacket) {
            self.last = Some(packet.clone());
        }
    }

    #[test]
    fn roundtrip_none() {
        let pkt = ClientboundResourcePackPopPacket::new(None);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundResourcePackPopPacket::read(&mut buf.freeze());
        assert_eq!(decoded.id(), None);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_some_uuid() {
        let uuid = Uuid {
            most_significant_bits: 0x0123_4567_89ab_cdef_u64 as i64,
            least_significant_bits: -1,
        };
        let pkt = ClientboundResourcePackPopPacket::new(Some(uuid));
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundResourcePackPopPacket::read(&mut buf.freeze());
        assert_eq!(decoded.id(), Some(uuid));
    }

    #[test]
    fn uuid_extreme_and_negative_values() {
        let uuid = Uuid {
            most_significant_bits: i64::MIN,
            least_significant_bits: i64::MAX,
        };
        let pkt = ClientboundResourcePackPopPacket::new(Some(uuid));
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundResourcePackPopPacket::read(&mut buf.freeze());
        let decoded_uuid = decoded.id().expect("uuid should be present");
        assert_eq!(decoded_uuid.most_significant_bits, i64::MIN);
        assert_eq!(decoded_uuid.least_significant_bits, i64::MAX);
    }

    #[test]
    fn handle_dispatches_and_type() {
        let pkt = ClientboundResourcePackPopPacket::new(None);
        let mut listener = MockListener { last: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.last, Some(pkt.clone()));
        assert_eq!(
            pkt.packet_type(),
            PacketType("common/clientbound_resource_pack_pop")
        );
    }
}
