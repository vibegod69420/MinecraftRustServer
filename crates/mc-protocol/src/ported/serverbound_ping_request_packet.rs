use bytes::{Buf, BufMut};

/// Minimal local stand-in for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

/// Minimal local stand-in for `net.minecraft.network.protocol.ping.PingPacketTypes`.
pub struct PingPacketTypes;

impl PingPacketTypes {
    pub const SERVERBOUND_PING_REQUEST: PacketType = PacketType {
        id: "minecraft:ping/serverbound_ping_request",
    };
}

/// Minimal local stand-in for `net.minecraft.network.protocol.ping.ServerPingPacketListener`.
pub trait ServerPingPacketListener {
    fn handle_ping_request(&mut self, packet: &ServerboundPingRequestPacket);
}

/// Port of `net.minecraft.network.protocol.ping.ServerboundPingRequestPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPingRequestPacket {
    time: i64,
}

impl ServerboundPingRequestPacket {
    // JAVA: net.minecraft.network.protocol.ping.ServerboundPingRequestPacket#<init>(long)
    pub fn new(time: i64) -> Self {
        Self { time }
    }

    // JAVA: net.minecraft.network.protocol.ping.ServerboundPingRequestPacket#<init>(ByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            time: buf.get_i64(),
        }
    }

    // JAVA: net.minecraft.network.protocol.ping.ServerboundPingRequestPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        buf.put_i64(self.time);
    }

    // JAVA: net.minecraft.network.protocol.ping.ServerboundPingRequestPacket#type
    pub fn packet_type(&self) -> PacketType {
        PingPacketTypes::SERVERBOUND_PING_REQUEST
    }

    // JAVA: net.minecraft.network.protocol.ping.ServerboundPingRequestPacket#handle
    pub fn handle(&self, listener: &mut impl ServerPingPacketListener) {
        listener.handle_ping_request(self);
    }

    // JAVA: net.minecraft.network.protocol.ping.ServerboundPingRequestPacket#getTime
    pub fn get_time(&self) -> i64 {
        self.time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Bytes, BytesMut};

    #[test]
    fn roundtrip_time() {
        let packet = ServerboundPingRequestPacket::new(123_456_789);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ServerboundPingRequestPacket::read(&mut reader);
        assert_eq!(packet, decoded);
        assert_eq!(decoded.get_time(), 123_456_789);
    }

    #[test]
    fn negative_and_extreme_values() {
        for time in [0_i64, -1, i64::MIN, i64::MAX, -123_456_789] {
            let packet = ServerboundPingRequestPacket::new(time);
            let mut buf = BytesMut::new();
            packet.write(&mut buf);
            assert_eq!(buf.len(), 8);
            let mut reader = Bytes::from(buf);
            let decoded = ServerboundPingRequestPacket::read(&mut reader);
            assert_eq!(decoded.get_time(), time);
        }
    }

    #[test]
    fn wire_bytes_are_big_endian() {
        let packet = ServerboundPingRequestPacket::new(0x0102_0304_0506_0708_i64);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        assert_eq!(&buf[..], &[1, 2, 3, 4, 5, 6, 7, 8]);

        let mut reader = Bytes::copy_from_slice(&[0xFF, 0, 0, 0, 0, 0, 0, 1]);
        let decoded = ServerboundPingRequestPacket::read(&mut reader);
        assert_eq!(decoded.get_time(), -0x00FF_FFFF_FFFF_FFFF_i64);
    }

    #[test]
    fn packet_type_matches_ping_request() {
        let packet = ServerboundPingRequestPacket::new(0);
        assert_eq!(
            packet.packet_type(),
            PingPacketTypes::SERVERBOUND_PING_REQUEST
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct MockListener {
            received: Option<i64>,
        }

        impl ServerPingPacketListener for MockListener {
            fn handle_ping_request(&mut self, packet: &ServerboundPingRequestPacket) {
                self.received = Some(packet.get_time());
            }
        }

        let packet = ServerboundPingRequestPacket::new(42);
        let mut listener = MockListener { received: None };
        packet.handle(&mut listener);
        assert_eq!(listener.received, Some(42));
    }
}
