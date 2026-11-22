/// Stub for `net.minecraft.network.ConnectionProtocol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Configuration,
}

/// Stub supertrait for `net.minecraft.network.protocol.common.ServerCommonPacketListener`.
pub trait ServerCommonPacketListener {}

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundFinishConfigurationPacket;

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundSelectKnownPacks;

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundAcceptCodeOfConductPacket;

/// Port of `net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener`.
pub trait ServerConfigurationPacketListener: ServerCommonPacketListener {
    // JAVA: net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener#protocol
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Configuration
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener#handleConfigurationFinished
    fn handle_configuration_finished(&mut self, packet: &ServerboundFinishConfigurationPacket);

    // JAVA: net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener#handleSelectKnownPacks
    fn handle_select_known_packs(&mut self, packet: &ServerboundSelectKnownPacks);

    // JAVA: net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener#handleAcceptCodeOfConduct
    fn handle_accept_code_of_conduct(&mut self, packet: &ServerboundAcceptCodeOfConductPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        finished: u32,
        selected_packs: u32,
        accepted_coc: i32,
    }

    impl ServerCommonPacketListener for MockListener {}

    impl ServerConfigurationPacketListener for MockListener {
        fn handle_configuration_finished(
            &mut self,
            _packet: &ServerboundFinishConfigurationPacket,
        ) {
            self.finished = self.finished.wrapping_add(1);
        }

        fn handle_select_known_packs(&mut self, _packet: &ServerboundSelectKnownPacks) {
            self.selected_packs = self.selected_packs.wrapping_add(1);
        }

        fn handle_accept_code_of_conduct(
            &mut self,
            _packet: &ServerboundAcceptCodeOfConductPacket,
        ) {
            self.accepted_coc = self.accepted_coc.wrapping_add(1);
        }
    }

    #[test]
    fn protocol_defaults_to_configuration() {
        let listener = MockListener::default();
        assert_eq!(listener.protocol(), ConnectionProtocol::Configuration);
    }

    #[test]
    fn handle_configuration_finished_dispatches() {
        let mut listener = MockListener::default();
        listener.handle_configuration_finished(&ServerboundFinishConfigurationPacket);
        assert_eq!(listener.finished, 1);
    }

    #[test]
    fn handle_select_known_packs_dispatches() {
        let mut listener = MockListener::default();
        listener.handle_select_known_packs(&ServerboundSelectKnownPacks);
        assert_eq!(listener.selected_packs, 1);
    }

    #[test]
    fn handle_accept_code_of_conduct_dispatches() {
        let mut listener = MockListener::default();
        listener.handle_accept_code_of_conduct(&ServerboundAcceptCodeOfConductPacket);
        assert_eq!(listener.accepted_coc, 1);
    }

    #[test]
    fn handler_counters_handle_overflow_and_negative_values() {
        let mut listener = MockListener {
            finished: u32::MAX,
            selected_packs: 0,
            accepted_coc: -1,
        };
        listener.handle_configuration_finished(&ServerboundFinishConfigurationPacket);
        listener.handle_accept_code_of_conduct(&ServerboundAcceptCodeOfConductPacket);
        assert_eq!(listener.finished, 0);
        assert_eq!(listener.accepted_coc, 0);
    }
}
