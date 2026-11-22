/// Minimal self-contained stub for `net.minecraft.network.PacketListener`.
pub trait PacketListener {}

/// Minimal self-contained stub for `net.minecraft.network.protocol.ping.ServerboundPingRequestPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPingRequestPacket {
    pub time: i64,
}

impl ServerboundPingRequestPacket {
    pub fn new(time: i64) -> Self {
        Self { time }
    }
}

/// Port of `net.minecraft.network.protocol.ping.ServerPingPacketListener`.
pub trait ServerPingPacketListener: PacketListener {
    // JAVA: net.minecraft.network.protocol.ping.ServerPingPacketListener#handlePingRequest
    fn handle_ping_request(&mut self, packet: &ServerboundPingRequestPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        received: Option<i64>,
    }

    impl PacketListener for TestListener {}

    impl ServerPingPacketListener for TestListener {
        fn handle_ping_request(&mut self, packet: &ServerboundPingRequestPacket) {
            self.received = Some(packet.time);
        }
    }

    #[test]
    fn handle_ping_request_receives_packet() {
        let mut listener = TestListener { received: None };
        let packet = ServerboundPingRequestPacket::new(123);
        listener.handle_ping_request(&packet);
        assert_eq!(listener.received, Some(123));
    }

    #[test]
    fn handle_ping_request_preserves_negative_time() {
        let mut listener = TestListener { received: None };
        let packet = ServerboundPingRequestPacket::new(-987_654_321);
        listener.handle_ping_request(&packet);
        assert_eq!(listener.received, Some(-987_654_321));
    }

    #[test]
    fn packet_extreme_values() {
        let min_packet = ServerboundPingRequestPacket::new(i64::MIN);
        let max_packet = ServerboundPingRequestPacket::new(i64::MAX);
        assert_eq!(min_packet.time, i64::MIN);
        assert_eq!(max_packet.time, i64::MAX);
    }

    #[test]
    fn packet_copy_and_eq() {
        let a = ServerboundPingRequestPacket::new(42);
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, ServerboundPingRequestPacket::new(43));
    }
}
