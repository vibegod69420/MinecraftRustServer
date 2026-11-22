//! Port of `net.minecraft.network.protocol.cookie.ClientCookiePacketListener`.

/// Minimal self-contained stub for the unported `ClientboundPacketListener`.
pub trait ClientboundPacketListener {}

/// Minimal self-contained stub for the unported `ClientboundCookieRequestPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundCookieRequestPacket;

/// Port of `net.minecraft.network.protocol.cookie.ClientCookiePacketListener`.
pub trait ClientCookiePacketListener: ClientboundPacketListener {
    // JAVA: net.minecraft.network.protocol.cookie.ClientCookiePacketListener#handleRequestCookie
    fn handle_request_cookie(&mut self, packet: &ClientboundCookieRequestPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        calls: i32,
        last_packet: Option<ClientboundCookieRequestPacket>,
    }

    impl ClientboundPacketListener for MockListener {}

    impl ClientCookiePacketListener for MockListener {
        fn handle_request_cookie(&mut self, packet: &ClientboundCookieRequestPacket) {
            self.calls = self.calls.wrapping_add(1);
            self.last_packet = Some(*packet);
        }
    }

    #[test]
    fn dispatches_request_cookie() {
        let mut listener = MockListener::default();
        let packet = ClientboundCookieRequestPacket;
        listener.handle_request_cookie(&packet);
        assert_eq!(listener.calls, 1);
        assert_eq!(listener.last_packet, Some(packet));
    }

    #[test]
    fn dispatches_multiple_times_with_wrapping_count() {
        let mut listener = MockListener {
            calls: i32::MAX,
            last_packet: None,
        };
        listener.handle_request_cookie(&ClientboundCookieRequestPacket);
        assert_eq!(listener.calls, i32::MIN);
        listener.handle_request_cookie(&ClientboundCookieRequestPacket);
        assert_eq!(listener.calls, i32::MIN.wrapping_add(1));
    }

    #[test]
    fn packet_stub_has_expected_value_semantics() {
        let a = ClientboundCookieRequestPacket;
        let b = a;
        let c = ClientboundCookieRequestPacket::default();
        assert_eq!(a, b);
        assert_eq!(b, c);
        let _ = format!("{a:?}");
    }

    #[test]
    fn trait_object_dispatch_works() {
        let mut listener = MockListener::default();
        let l: &mut dyn ClientCookiePacketListener = &mut listener;
        l.handle_request_cookie(&ClientboundCookieRequestPacket);
        assert_eq!(listener.calls, 1);
    }
}
