#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Login,
}

pub trait ClientCookiePacketListener {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientboundHelloPacket;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientboundLoginFinishedPacket;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientboundLoginDisconnectPacket;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientboundLoginCompressionPacket;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientboundCustomQueryPacket;

pub trait ClientLoginPacketListener: ClientCookiePacketListener {
    // JAVA: net.minecraft.network.protocol.login.ClientLoginPacketListener#protocol
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Login
    }

    // JAVA: net.minecraft.network.protocol.login.ClientLoginPacketListener#handleHello
    fn handle_hello(&mut self, packet: &ClientboundHelloPacket);

    // JAVA: net.minecraft.network.protocol.login.ClientLoginPacketListener#handleLoginFinished
    fn handle_login_finished(&mut self, packet: &ClientboundLoginFinishedPacket);

    // JAVA: net.minecraft.network.protocol.login.ClientLoginPacketListener#handleDisconnect
    fn handle_disconnect(&mut self, packet: &ClientboundLoginDisconnectPacket);

    // JAVA: net.minecraft.network.protocol.login.ClientLoginPacketListener#handleCompression
    fn handle_compression(&mut self, packet: &ClientboundLoginCompressionPacket);

    // JAVA: net.minecraft.network.protocol.login.ClientLoginPacketListener#handleCustomQuery
    fn handle_custom_query(&mut self, packet: &ClientboundCustomQueryPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        hello: u32,
        finished: u32,
        disconnect: u32,
        compression: i32,
        custom_query: i32,
    }

    impl ClientCookiePacketListener for MockListener {}

    impl ClientLoginPacketListener for MockListener {
        fn handle_hello(&mut self, _packet: &ClientboundHelloPacket) {
            self.hello = self.hello.wrapping_add(1);
        }

        fn handle_login_finished(&mut self, _packet: &ClientboundLoginFinishedPacket) {
            self.finished = self.finished.wrapping_add(1);
        }

        fn handle_disconnect(&mut self, _packet: &ClientboundLoginDisconnectPacket) {
            self.disconnect = self.disconnect.wrapping_add(1);
        }

        fn handle_compression(&mut self, _packet: &ClientboundLoginCompressionPacket) {
            self.compression = self.compression.wrapping_add(1);
        }

        fn handle_custom_query(&mut self, _packet: &ClientboundCustomQueryPacket) {
            self.custom_query = self.custom_query.wrapping_add(1);
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
        listener.handle_hello(&ClientboundHelloPacket);
        listener.handle_login_finished(&ClientboundLoginFinishedPacket);
        listener.handle_disconnect(&ClientboundLoginDisconnectPacket);
        listener.handle_compression(&ClientboundLoginCompressionPacket);
        listener.handle_custom_query(&ClientboundCustomQueryPacket);

        assert_eq!(listener.hello, 1);
        assert_eq!(listener.finished, 1);
        assert_eq!(listener.disconnect, 1);
        assert_eq!(listener.compression, 1);
        assert_eq!(listener.custom_query, 1);
    }

    #[test]
    fn unsigned_counter_wrapping_overflow() {
        let mut listener = MockListener::default();
        listener.hello = u32::MAX;
        listener.handle_hello(&ClientboundHelloPacket);
        assert_eq!(listener.hello, 0);
    }

    #[test]
    fn signed_counter_negative_wrapping() {
        let mut listener = MockListener {
            compression: i32::MIN,
            ..MockListener::default()
        };
        listener.handle_compression(&ClientboundLoginCompressionPacket);
        assert_eq!(listener.compression, i32::MIN.wrapping_add(1));
    }

    #[test]
    fn float_bits_edge_case_preserved() {
        let nan_bits = f64::NAN.to_bits();
        let exponent_mask = 0x7ff0_0000_0000_0000_u64;
        let mantissa_mask = 0x000f_ffff_ffff_ffff_u64;
        assert_eq!(nan_bits & exponent_mask, exponent_mask);
        assert_ne!(nan_bits & mantissa_mask, 0);
    }
}
