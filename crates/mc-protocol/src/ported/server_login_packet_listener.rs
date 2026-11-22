#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Login,
}

pub trait ServerCookiePacketListener {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundHelloPacket;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundKeyPacket;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundCustomQueryAnswerPacket;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundLoginAcknowledgedPacket;

pub trait ServerLoginPacketListener: ServerCookiePacketListener {
    // JAVA: net.minecraft.network.protocol.login.ServerLoginPacketListener#protocol
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Login
    }

    // JAVA: net.minecraft.network.protocol.login.ServerLoginPacketListener#handleHello
    fn handle_hello(&mut self, packet: &ServerboundHelloPacket);

    // JAVA: net.minecraft.network.protocol.login.ServerLoginPacketListener#handleKey
    fn handle_key(&mut self, packet: &ServerboundKeyPacket);

    // JAVA: net.minecraft.network.protocol.login.ServerLoginPacketListener#handleCustomQueryPacket
    fn handle_custom_query_packet(&mut self, packet: &ServerboundCustomQueryAnswerPacket);

    // JAVA: net.minecraft.network.protocol.login.ServerLoginPacketListener#handleLoginAcknowledgement
    fn handle_login_acknowledgement(&mut self, packet: &ServerboundLoginAcknowledgedPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        hello: u32,
        key: u32,
        custom_query: u32,
        ack: i32,
    }

    impl ServerCookiePacketListener for MockListener {}

    impl ServerLoginPacketListener for MockListener {
        fn handle_hello(&mut self, _packet: &ServerboundHelloPacket) {
            self.hello = self.hello.wrapping_add(1);
        }

        fn handle_key(&mut self, _packet: &ServerboundKeyPacket) {
            self.key = self.key.wrapping_add(1);
        }

        fn handle_custom_query_packet(&mut self, _packet: &ServerboundCustomQueryAnswerPacket) {
            self.custom_query = self.custom_query.wrapping_add(1);
        }

        fn handle_login_acknowledgement(&mut self, _packet: &ServerboundLoginAcknowledgedPacket) {
            self.ack = self.ack.wrapping_add(1);
        }
    }

    #[test]
    fn protocol_returns_login() {
        let listener = MockListener::default();
        assert_eq!(listener.protocol(), ConnectionProtocol::Login);
    }

    #[test]
    fn handlers_dispatch_to_expected_methods() {
        let mut listener = MockListener::default();
        listener.handle_hello(&ServerboundHelloPacket);
        listener.handle_key(&ServerboundKeyPacket);
        listener.handle_custom_query_packet(&ServerboundCustomQueryAnswerPacket);
        listener.handle_login_acknowledgement(&ServerboundLoginAcknowledgedPacket);

        assert_eq!(listener.hello, 1);
        assert_eq!(listener.key, 1);
        assert_eq!(listener.custom_query, 1);
        assert_eq!(listener.ack, 1);
    }

    #[test]
    fn handler_counter_wrapping_overflow() {
        let mut listener = MockListener::default();
        listener.hello = u32::MAX;
        listener.handle_hello(&ServerboundHelloPacket);
        assert_eq!(listener.hello, 0);
    }

    #[test]
    fn handler_counter_negative_wrapping() {
        let mut listener = MockListener {
            ack: i32::MIN,
            ..MockListener::default()
        };
        listener.handle_login_acknowledgement(&ServerboundLoginAcknowledgedPacket);
        assert_eq!(listener.ack, i32::MIN.wrapping_add(1));
    }

    #[test]
    fn float_bits_edge_case_preserved() {
        let neg_zero = (-0.0_f32).to_bits();
        let pos_zero = 0.0_f32.to_bits();
        assert_eq!(neg_zero, 0x8000_0000);
        assert_eq!(pos_zero, 0x0000_0000);
        assert_ne!(neg_zero, pos_zero);
    }
}
