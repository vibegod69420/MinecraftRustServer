//! Port of `net.minecraft.network.protocol.cookie.ServerCookiePacketListener`.

/// Minimal self-contained stub for the unported `ServerPacketListener`.
pub trait ServerPacketListener {}

/// Minimal self-contained stub for the unported `ServerboundCookieResponsePacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundCookieResponsePacket;

/// Port of `net.minecraft.network.protocol.cookie.ServerCookiePacketListener`.
pub trait ServerCookiePacketListener: ServerPacketListener {
    // JAVA: net.minecraft.network.protocol.cookie.ServerCookiePacketListener#handleCookieResponse
    fn handle_cookie_response(&mut self, packet: &ServerboundCookieResponsePacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        calls: i32,
        last_packet: Option<ServerboundCookieResponsePacket>,
    }

    impl ServerPacketListener for MockListener {}

    impl ServerCookiePacketListener for MockListener {
        fn handle_cookie_response(&mut self, packet: &ServerboundCookieResponsePacket) {
            self.calls = self.calls.wrapping_add(1);
            self.last_packet = Some(*packet);
        }
    }

    #[test]
    fn dispatches_cookie_response() {
        let mut listener = MockListener::default();
        let packet = ServerboundCookieResponsePacket;
        listener.handle_cookie_response(&packet);
        assert_eq!(listener.calls, 1);
        assert_eq!(listener.last_packet, Some(packet));
    }

    #[test]
    fn dispatches_multiple_times_with_wrapping_count() {
        let mut listener = MockListener {
            calls: i32::MAX,
            last_packet: None,
        };
        listener.handle_cookie_response(&ServerboundCookieResponsePacket);
        assert_eq!(listener.calls, i32::MIN);
        listener.handle_cookie_response(&ServerboundCookieResponsePacket);
        assert_eq!(listener.calls, i32::MIN.wrapping_add(1));
    }

    #[test]
    fn packet_stub_has_expected_value_semantics() {
        let a = ServerboundCookieResponsePacket;
        let b = a;
        let c = ServerboundCookieResponsePacket::default();
        assert_eq!(a, b);
        assert_eq!(b, c);
        let _ = format!("{a:?}");
    }

    #[test]
    fn trait_object_dispatch_works() {
        let mut listener = MockListener::default();
        let l: &mut dyn ServerCookiePacketListener = &mut listener;
        l.handle_cookie_response(&ServerboundCookieResponsePacket);
        assert_eq!(listener.calls, 1);
    }
}
