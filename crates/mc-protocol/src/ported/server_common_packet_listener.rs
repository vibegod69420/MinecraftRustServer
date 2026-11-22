//! Port of `net.minecraft.network.protocol.common.ServerCommonPacketListener`.

/// Placeholder for `net.minecraft.network.protocol.cookie.ServerCookiePacketListener`.
pub trait ServerCookiePacketListener {}

/// Placeholder for `net.minecraft.network.protocol.common.ServerboundKeepAlivePacket`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundKeepAlivePacket {
    pub id: i64,
}

/// Placeholder for `net.minecraft.network.protocol.common.ServerboundPongPacket`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundPongPacket {
    pub id: i32,
}

/// Placeholder for `net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundCustomPayloadPacket;

/// Placeholder for `net.minecraft.network.protocol.common.ServerboundResourcePackPacket`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundResourcePackPacket;

/// Placeholder for `net.minecraft.network.protocol.common.ServerboundClientInformationPacket`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundClientInformationPacket;

/// Placeholder for `net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerboundCustomClickActionPacket;

/// Port of `net.minecraft.network.protocol.common.ServerCommonPacketListener`.
pub trait ServerCommonPacketListener: ServerCookiePacketListener {
    // JAVA: net.minecraft.network.protocol.common.ServerCommonPacketListener#handleKeepAlive
    fn handle_keep_alive(&mut self, packet: &ServerboundKeepAlivePacket);

    // JAVA: net.minecraft.network.protocol.common.ServerCommonPacketListener#handlePong
    fn handle_pong(&mut self, packet: &ServerboundPongPacket);

    // JAVA: net.minecraft.network.protocol.common.ServerCommonPacketListener#handleCustomPayload
    fn handle_custom_payload(&mut self, packet: &ServerboundCustomPayloadPacket);

    // JAVA: net.minecraft.network.protocol.common.ServerCommonPacketListener#handleResourcePackResponse
    fn handle_resource_pack_response(&mut self, packet: &ServerboundResourcePackPacket);

    // JAVA: net.minecraft.network.protocol.common.ServerCommonPacketListener#handleClientInformation
    fn handle_client_information(&mut self, packet: &ServerboundClientInformationPacket);

    // JAVA: net.minecraft.network.protocol.common.ServerCommonPacketListener#handleCustomClickAction
    fn handle_custom_click_action(&mut self, packet: &ServerboundCustomClickActionPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        keep_alive_ids: Vec<i64>,
        pong_ids: Vec<i32>,
        custom_payload_count: u8,
        resource_pack_count: u8,
        client_info_count: u8,
        custom_click_count: u8,
    }

    impl ServerCookiePacketListener for MockListener {}

    impl ServerCommonPacketListener for MockListener {
        fn handle_keep_alive(&mut self, packet: &ServerboundKeepAlivePacket) {
            self.keep_alive_ids.push(packet.id);
        }

        fn handle_pong(&mut self, packet: &ServerboundPongPacket) {
            self.pong_ids.push(packet.id);
        }

        fn handle_custom_payload(&mut self, _packet: &ServerboundCustomPayloadPacket) {
            self.custom_payload_count = self.custom_payload_count.wrapping_add(1);
        }

        fn handle_resource_pack_response(&mut self, _packet: &ServerboundResourcePackPacket) {
            self.resource_pack_count = self.resource_pack_count.wrapping_add(1);
        }

        fn handle_client_information(&mut self, _packet: &ServerboundClientInformationPacket) {
            self.client_info_count = self.client_info_count.wrapping_add(1);
        }

        fn handle_custom_click_action(&mut self, _packet: &ServerboundCustomClickActionPacket) {
            self.custom_click_count = self.custom_click_count.wrapping_add(1);
        }
    }

    #[test]
    fn dispatch_keep_alive_negative_and_extreme() {
        let mut listener = MockListener::default();
        listener.handle_keep_alive(&ServerboundKeepAlivePacket { id: -1 });
        listener.handle_keep_alive(&ServerboundKeepAlivePacket { id: i64::MIN });
        listener.handle_keep_alive(&ServerboundKeepAlivePacket { id: i64::MAX });
        assert_eq!(listener.keep_alive_ids, vec![-1, i64::MIN, i64::MAX]);
    }

    #[test]
    fn dispatch_pong_negative_and_extreme() {
        let mut listener = MockListener::default();
        listener.handle_pong(&ServerboundPongPacket { id: -42 });
        listener.handle_pong(&ServerboundPongPacket { id: i32::MIN });
        listener.handle_pong(&ServerboundPongPacket { id: i32::MAX });
        assert_eq!(listener.pong_ids, vec![-42, i32::MIN, i32::MAX]);
    }

    #[test]
    fn dispatch_other_packets_with_wrapping_overflow() {
        let mut listener = MockListener::default();
        listener.custom_payload_count = u8::MAX;
        listener.handle_custom_payload(&ServerboundCustomPayloadPacket);
        assert_eq!(listener.custom_payload_count, 0);

        listener.resource_pack_count = u8::MAX;
        listener.handle_resource_pack_response(&ServerboundResourcePackPacket);
        assert_eq!(listener.resource_pack_count, 0);

        listener.handle_client_information(&ServerboundClientInformationPacket);
        listener.handle_custom_click_action(&ServerboundCustomClickActionPacket);
        assert_eq!(listener.client_info_count, 1);
        assert_eq!(listener.custom_click_count, 1);
    }

    #[test]
    fn trait_is_object_safe() {
        fn accepts(_listener: &mut dyn ServerCommonPacketListener) {}
        let mut listener = MockListener::default();
        accepts(&mut listener);
    }

    #[test]
    fn float_bits_edge_case_sanity() {
        assert_eq!(f32::NAN.to_bits() & 0x7fc0_0000, 0x7fc0_0000);
        assert_eq!(f64::NEG_INFINITY.to_bits(), 0xfff0_0000_0000_0000);
        assert_ne!(0.0f32.to_bits(), (-0.0f32).to_bits());
    }
}
