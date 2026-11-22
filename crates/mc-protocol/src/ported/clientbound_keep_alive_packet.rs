use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ClientCommonPacketListener {
    fn handle_keep_alive(&mut self, packet: &ClientboundKeepAlivePacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundKeepAlivePacket {
    id: i64,
}

impl ClientboundKeepAlivePacket {
    pub const TYPE: PacketType = PacketType("common/clientbound_keep_alive");

    // JAVA: net.minecraft.network.protocol.common.ClientboundKeepAlivePacket#<init>(long)
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundKeepAlivePacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        Self { id: buf.get_i64() }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundKeepAlivePacket#write(FriendlyByteBuf)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_i64(self.id);
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundKeepAlivePacket#type()
    pub fn packet_type(&self) -> PacketType {
        Self::TYPE
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundKeepAlivePacket#handle(ClientCommonPacketListener)
    pub fn handle<L: ClientCommonPacketListener>(&self, listener: &mut L) {
        listener.handle_keep_alive(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundKeepAlivePacket#getId()
    pub fn get_id(&self) -> i64 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        last_id: Option<i64>,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_keep_alive(&mut self, packet: &ClientboundKeepAlivePacket) {
            self.last_id = Some(packet.get_id());
        }
    }

    #[test]
    fn roundtrip_positive() {
        let pkt = ClientboundKeepAlivePacket::new(123_456_789);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundKeepAlivePacket::read(&mut buf.freeze());
        assert_eq!(decoded.get_id(), 123_456_789);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_negative_and_extremes() {
        for id in [i64::MIN, -1, 0, 1, i64::MAX] {
            let pkt = ClientboundKeepAlivePacket::new(id);
            let mut buf = BytesMut::new();
            pkt.write(&mut buf);
            let decoded = ClientboundKeepAlivePacket::read(&mut buf.freeze());
            assert_eq!(decoded.get_id(), id);
        }
    }

    #[test]
    fn float_bits_preserved() {
        let bits = f64::NAN.to_bits() as i64;
        let pkt = ClientboundKeepAlivePacket::new(bits);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundKeepAlivePacket::read(&mut buf.freeze());
        assert_eq!(decoded.get_id(), bits);
        assert_eq!(
            f64::from_bits(decoded.get_id() as u64).to_bits(),
            f64::NAN.to_bits()
        );
    }

    #[test]
    fn handle_dispatches_and_type() {
        let pkt = ClientboundKeepAlivePacket::new(-42);
        let mut listener = MockListener { last_id: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.last_id, Some(-42));
        assert_eq!(
            pkt.packet_type(),
            PacketType("common/clientbound_keep_alive")
        );
    }
}
