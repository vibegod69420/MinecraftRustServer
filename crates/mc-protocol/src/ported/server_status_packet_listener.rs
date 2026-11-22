#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

pub trait ServerPacketListener {}
pub trait ServerPingPacketListener {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundStatusRequestPacket;

pub trait ServerStatusPacketListener: ServerPacketListener + ServerPingPacketListener {
    // JAVA: net.minecraft.network.protocol.status.ServerStatusPacketListener#protocol
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Status
    }

    // JAVA: net.minecraft.network.protocol.status.ServerStatusPacketListener#handleStatusRequest
    fn handle_status_request(&mut self, packet: &ServerboundStatusRequestPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        request_count: u32,
    }

    impl ServerPacketListener for MockListener {}
    impl ServerPingPacketListener for MockListener {}
    impl ServerStatusPacketListener for MockListener {
        fn handle_status_request(&mut self, _packet: &ServerboundStatusRequestPacket) {
            self.request_count = self.request_count.wrapping_add(1);
        }
    }

    #[test]
    fn protocol_defaults_to_status() {
        let listener = MockListener { request_count: 0 };
        assert_eq!(listener.protocol(), ConnectionProtocol::Status);
    }

    #[test]
    fn handle_status_request_increments_count() {
        let mut listener = MockListener { request_count: 0 };
        let packet = ServerboundStatusRequestPacket;
        listener.handle_status_request(&packet);
        assert_eq!(listener.request_count, 1);
        listener.handle_status_request(&packet);
        assert_eq!(listener.request_count, 2);
    }

    #[test]
    fn handle_status_request_wrapping_edge_case() {
        let mut listener = MockListener {
            request_count: u32::MAX,
        };
        let packet = ServerboundStatusRequestPacket;
        listener.handle_status_request(&packet);
        assert_eq!(listener.request_count, 0);
    }

    #[test]
    fn protocol_enum_distinct_from_other_phases() {
        assert_ne!(ConnectionProtocol::Status, ConnectionProtocol::Handshaking);
        assert_ne!(ConnectionProtocol::Status, ConnectionProtocol::Configuration);
    }
}
