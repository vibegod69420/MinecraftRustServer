use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ServerCommonPacketListener {
    fn handle_keep_alive(&mut self, packet: &ServerboundKeepAlivePacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundKeepAlivePacket {
    id: i64,
}

impl ServerboundKeepAlivePacket {
    pub const TYPE: PacketType = PacketType("common/serverbound_keep_alive");

    // JAVA: net.minecraft.network.protocol.common.ServerboundKeepAlivePacket#<init>(long)
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundKeepAlivePacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        Self { id: buf.get_i64() }
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundKeepAlivePacket#write(FriendlyByteBuf)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_i64(self.id);
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundKeepAlivePacket#type()
    pub fn packet_type(&self) -> PacketType {
        Self::TYPE
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundKeepAlivePacket#handle(ServerCommonPacketListener)
    pub fn handle<L: ServerCommonPacketListener>(&self, listener: &mut L) {
        listener.handle_keep_alive(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundKeepAlivePacket#getId()
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

    impl ServerCommonPacketListener for MockListener {
        fn handle_keep_alive(&mut self, packet: &ServerboundKeepAlivePacket) {
            self.last_id = Some(packet.get_id());
        }
    }

    #[test]
    fn roundtrip_positive() {
        let pkt = ServerboundKeepAlivePacket::new(987_654_321);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundKeepAlivePacket::read(&mut buf.freeze());
        assert_eq!(decoded.get_id(), 987_654_321);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_negative_and_extremes() {
        for id in [i64::MIN, -1, 0, 1, i64::MAX] {
            let pkt = ServerboundKeepAlivePacket::new(id);
            let mut buf = BytesMut::new();
            pkt.write(&mut buf);
            let decoded = ServerboundKeepAlivePacket::read(&mut buf.freeze());
            assert_eq!(decoded.get_id(), id);
        }
    }

    #[test]
    fn float_bits_preserved() {
        let bits = (-f64::INFINITY).to_bits() as i64;
        let pkt = ServerboundKeepAlivePacket::new(bits);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundKeepAlivePacket::read(&mut buf.freeze());
        assert_eq!(decoded.get_id(), bits);
        assert_eq!(
            f64::from_bits(decoded.get_id() as u64).to_bits(),
            (-f64::INFINITY).to_bits()
        );
    }

    #[test]
    fn handle_dispatches_and_type() {
        let pkt = ServerboundKeepAlivePacket::new(i64::MIN);
        let mut listener = MockListener { last_id: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.last_id, Some(i64::MIN));
        assert_eq!(
            pkt.packet_type(),
            PacketType("common/serverbound_keep_alive")
        );
    }
}
