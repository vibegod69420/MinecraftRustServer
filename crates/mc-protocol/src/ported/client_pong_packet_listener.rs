/// Minimal self-contained stub for `net.minecraft.network.PacketListener`.
pub trait PacketListener {}

/// Minimal self-contained stub for `net.minecraft.network.protocol.ping.ClientboundPongResponsePacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPongResponsePacket {
    pub time: i64,
}

impl ClientboundPongResponsePacket {
    pub fn new(time: i64) -> Self {
        Self { time }
    }
}

/// Port of `net.minecraft.network.protocol.ping.ClientPongPacketListener`.
pub trait ClientPongPacketListener: PacketListener {
    // JAVA: net.minecraft.network.protocol.ping.ClientPongPacketListener#handlePongResponse
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        received: Option<i64>,
    }

    impl PacketListener for TestListener {}

    impl ClientPongPacketListener for TestListener {
        fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket) {
            self.received = Some(packet.time);
        }
    }

    #[test]
    fn handle_pong_response_receives_packet() {
        let mut listener = TestListener { received: None };
        let packet = ClientboundPongResponsePacket::new(456);
        listener.handle_pong_response(&packet);
        assert_eq!(listener.received, Some(456));
    }

    #[test]
    fn handle_pong_response_preserves_negative_time() {
        let mut listener = TestListener { received: None };
        let packet = ClientboundPongResponsePacket::new(-1);
        listener.handle_pong_response(&packet);
        assert_eq!(listener.received, Some(-1));
    }

    #[test]
    fn packet_extreme_values() {
        let min_packet = ClientboundPongResponsePacket::new(i64::MIN);
        let max_packet = ClientboundPongResponsePacket::new(i64::MAX);
        assert_eq!(min_packet.time, i64::MIN);
        assert_eq!(max_packet.time, i64::MAX);
    }

    #[test]
    fn packet_copy_and_eq() {
        let a = ClientboundPongResponsePacket::new(0);
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, ClientboundPongResponsePacket::new(i64::MIN));
    }
}
