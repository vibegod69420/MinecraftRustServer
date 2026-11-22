/// Stub for `net.minecraft.network.ConnectionProtocol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Configuration,
}

/// Stub supertrait for `net.minecraft.network.protocol.common.ClientCommonPacketListener`.
pub trait ClientCommonPacketListener {}

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundCodeOfConductPacket;

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ClientboundFinishConfigurationPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundFinishConfigurationPacket;

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundRegistryDataPacket;

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundUpdateEnabledFeaturesPacket;

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundSelectKnownPacks;

/// Placeholder for
/// `net.minecraft.network.protocol.configuration.ClientboundResetChatPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundResetChatPacket;

/// Port of `net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener`.
pub trait ClientConfigurationPacketListener: ClientCommonPacketListener {
    // JAVA: net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener#protocol
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Configuration
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener#handleCodeOfConduct
    fn handle_code_of_conduct(&mut self, packet: &ClientboundCodeOfConductPacket);

    // JAVA: net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener#handleConfigurationFinished
    fn handle_configuration_finished(&mut self, packet: &ClientboundFinishConfigurationPacket);

    // JAVA: net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener#handleRegistryData
    fn handle_registry_data(&mut self, packet: &ClientboundRegistryDataPacket);

    // JAVA: net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener#handleEnabledFeatures
    fn handle_enabled_features(&mut self, packet: &ClientboundUpdateEnabledFeaturesPacket);

    // JAVA: net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener#handleSelectKnownPacks
    fn handle_select_known_packs(&mut self, packet: &ClientboundSelectKnownPacks);

    // JAVA: net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener#handleResetChat
    fn handle_reset_chat(&mut self, packet: &ClientboundResetChatPacket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        code_of_conduct: u32,
        finished: u32,
        registry_data: u32,
        enabled_features: u32,
        selected_packs: i32,
        reset_chat: u32,
    }

    impl ClientCommonPacketListener for MockListener {}

    impl ClientConfigurationPacketListener for MockListener {
        fn handle_code_of_conduct(&mut self, _packet: &ClientboundCodeOfConductPacket) {
            self.code_of_conduct = self.code_of_conduct.wrapping_add(1);
        }

        fn handle_configuration_finished(
            &mut self,
            _packet: &ClientboundFinishConfigurationPacket,
        ) {
            self.finished = self.finished.wrapping_add(1);
        }

        fn handle_registry_data(&mut self, _packet: &ClientboundRegistryDataPacket) {
            self.registry_data = self.registry_data.wrapping_add(1);
        }

        fn handle_enabled_features(&mut self, _packet: &ClientboundUpdateEnabledFeaturesPacket) {
            self.enabled_features = self.enabled_features.wrapping_add(1);
        }

        fn handle_select_known_packs(&mut self, _packet: &ClientboundSelectKnownPacks) {
            self.selected_packs = self.selected_packs.wrapping_add(1);
        }

        fn handle_reset_chat(&mut self, _packet: &ClientboundResetChatPacket) {
            self.reset_chat = self.reset_chat.wrapping_add(1);
        }
    }

    #[test]
    fn protocol_defaults_to_configuration() {
        let listener = MockListener::default();
        assert_eq!(listener.protocol(), ConnectionProtocol::Configuration);
    }

    #[test]
    fn handlers_dispatch_to_correct_methods() {
        let mut listener = MockListener::default();
        listener.handle_code_of_conduct(&ClientboundCodeOfConductPacket);
        listener.handle_configuration_finished(&ClientboundFinishConfigurationPacket);
        listener.handle_registry_data(&ClientboundRegistryDataPacket);
        listener.handle_enabled_features(&ClientboundUpdateEnabledFeaturesPacket);
        listener.handle_select_known_packs(&ClientboundSelectKnownPacks);
        listener.handle_reset_chat(&ClientboundResetChatPacket);

        assert_eq!(listener.code_of_conduct, 1);
        assert_eq!(listener.finished, 1);
        assert_eq!(listener.registry_data, 1);
        assert_eq!(listener.enabled_features, 1);
        assert_eq!(listener.selected_packs, 1);
        assert_eq!(listener.reset_chat, 1);
    }

    #[test]
    fn handlers_handle_overflow_wrapping() {
        let mut listener = MockListener {
            code_of_conduct: u32::MAX,
            finished: u32::MAX,
            registry_data: u32::MAX,
            enabled_features: u32::MAX,
            selected_packs: i32::MAX,
            reset_chat: u32::MAX,
        };
        listener.handle_code_of_conduct(&ClientboundCodeOfConductPacket);
        listener.handle_configuration_finished(&ClientboundFinishConfigurationPacket);
        listener.handle_registry_data(&ClientboundRegistryDataPacket);
        listener.handle_enabled_features(&ClientboundUpdateEnabledFeaturesPacket);
        listener.handle_select_known_packs(&ClientboundSelectKnownPacks);
        listener.handle_reset_chat(&ClientboundResetChatPacket);

        assert_eq!(listener.code_of_conduct, 0);
        assert_eq!(listener.finished, 0);
        assert_eq!(listener.registry_data, 0);
        assert_eq!(listener.enabled_features, 0);
        assert_eq!(listener.selected_packs, i32::MIN);
        assert_eq!(listener.reset_chat, 0);
    }

    #[test]
    fn handlers_handle_negative_values_with_wrapping() {
        let mut listener = MockListener {
            selected_packs: -1,
            ..MockListener::default()
        };
        listener.handle_select_known_packs(&ClientboundSelectKnownPacks);
        assert_eq!(listener.selected_packs, 0);
    }

    #[test]
    fn placeholder_packets_are_copy_and_eq() {
        let a = ClientboundResetChatPacket;
        let b = a;
        assert_eq!(a, b);
        let c = ClientboundRegistryDataPacket;
        let d = c;
        assert_eq!(c, d);
    }
}
