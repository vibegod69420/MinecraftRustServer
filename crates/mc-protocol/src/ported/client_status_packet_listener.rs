#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

pub trait ClientboundPacketListener {}
pub trait ClientPongPacketListener {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientboundStatusResponsePacket {
    pub status_json: String,
}

pub trait ClientStatusPacketListener: ClientPongPacketListener + ClientboundPacketListener {
    // JAVA: net.minecraft.network.protocol.status.ClientStatusPacketListener#protocol
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Status
    }

    // JAVA: net.minecraft.network.protocol.status.ClientStatusPacketListener#handleStatusResponse
    fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        last_status: Option<String>,
    }

    impl ClientboundPacketListener for MockListener {}
    impl ClientPongPacketListener for MockListener {}
    impl ClientStatusPacketListener for MockListener {
        fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket) {
            self.last_status = Some(packet.status_json.clone());
        }
    }

    #[test]
    fn protocol_defaults_to_status() {
        let listener = MockListener { last_status: None };
        assert_eq!(listener.protocol(), ConnectionProtocol::Status);
    }

    #[test]
    fn handle_status_response_receives_packet() {
        let mut listener = MockListener { last_status: None };
        let packet = ClientboundStatusResponsePacket {
            status_json: "{\"version\":{\"name\":\"1.21.11\"}}".to_string(),
        };
        listener.handle_status_response(&packet);
        assert_eq!(
            listener.last_status.as_deref(),
            Some("{\"version\":{\"name\":\"1.21.11\"}}")
        );
    }

    #[test]
    fn handle_status_response_empty_json_edge_case() {
        let mut listener = MockListener { last_status: None };
        let packet = ClientboundStatusResponsePacket {
            status_json: String::new(),
        };
        listener.handle_status_response(&packet);
        assert_eq!(listener.last_status.as_deref(), Some(""));
    }

    #[test]
    fn protocol_enum_copy_and_distinct_variants() {
        let p = ConnectionProtocol::Status;
        let p2 = p;
        assert_eq!(p, p2);
        assert_ne!(ConnectionProtocol::Status, ConnectionProtocol::Play);
        assert_ne!(ConnectionProtocol::Status, ConnectionProtocol::Login);
    }
}
