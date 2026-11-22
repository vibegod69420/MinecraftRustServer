//! Port of `net.minecraft.network.protocol.handshake.ServerHandshakePacketListener`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Handshaking,
}

pub trait ServerPacketListener {
    fn protocol(&self) -> ConnectionProtocol;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientIntentionPacket;

pub trait ServerHandshakePacketListener: ServerPacketListener {
    // JAVA: net.minecraft.network.protocol.handshake.ServerHandshakePacketListener#protocol
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Handshaking
    }

    // JAVA: net.minecraft.network.protocol.handshake.ServerHandshakePacketListener#handleIntention
    fn handle_intention(&mut self, packet: &ClientIntentionPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyListener {
        handled: bool,
        last_packet: Option<ClientIntentionPacket>,
    }

    impl ServerPacketListener for DummyListener {
        fn protocol(&self) -> ConnectionProtocol {
            <Self as ServerHandshakePacketListener>::protocol(self)
        }
    }

    impl ServerHandshakePacketListener for DummyListener {
        fn handle_intention(&mut self, packet: &ClientIntentionPacket) {
            self.handled = true;
            self.last_packet = Some(packet.clone());
        }
    }

    #[test]
    fn default_protocol_is_handshaking() {
        let listener = DummyListener {
            handled: false,
            last_packet: None,
        };
        assert_eq!(
            ServerHandshakePacketListener::protocol(&listener),
            ConnectionProtocol::Handshaking
        );
    }

    #[test]
    fn server_packet_listener_protocol_delegates() {
        let listener = DummyListener {
            handled: false,
            last_packet: None,
        };
        assert_eq!(
            ServerPacketListener::protocol(&listener),
            ConnectionProtocol::Handshaking
        );
    }

    #[test]
    fn handle_intention_receives_packet() {
        let mut listener = DummyListener {
            handled: false,
            last_packet: None,
        };
        let packet = ClientIntentionPacket;
        listener.handle_intention(&packet);
        assert!(listener.handled);
        assert_eq!(listener.last_packet, Some(ClientIntentionPacket));
    }
}
