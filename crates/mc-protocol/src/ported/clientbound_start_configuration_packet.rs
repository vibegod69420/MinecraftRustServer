#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub &'static str);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_START_CONFIGURATION: PacketType =
        PacketType("CLIENTBOUND_START_CONFIGURATION");
}

/// Placeholder for `net.minecraft.network.protocol.game.ClientGamePacketListener`.
pub trait ClientGamePacketListener {
    fn handle_configuration_start(&mut self, packet: &ClientboundStartConfigurationPacket);
}

/// Port of `net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientboundStartConfigurationPacket;

impl ClientboundStartConfigurationPacket {
    pub const INSTANCE: Self = Self;

    // JAVA: net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket#<init>
    const fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket#type
    pub fn type_(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_START_CONFIGURATION
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_configuration_start(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket#isTerminal
    pub fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        handled: bool,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_configuration_start(
            &mut self,
            _packet: &ClientboundStartConfigurationPacket,
        ) {
            self.handled = true;
        }
    }

    #[test]
    fn instance_is_singleton_value() {
        let a = ClientboundStartConfigurationPacket::INSTANCE;
        let b = ClientboundStartConfigurationPacket::INSTANCE;
        assert_eq!(a, b);
    }

    #[test]
    fn packet_type_is_start_configuration() {
        assert_eq!(
            ClientboundStartConfigurationPacket::INSTANCE.type_(),
            GamePacketTypes::CLIENTBOUND_START_CONFIGURATION
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener { handled: false };
        ClientboundStartConfigurationPacket::INSTANCE.handle(&mut listener);
        assert!(listener.handled);
    }

    #[test]
    fn is_terminal_returns_true() {
        assert!(ClientboundStartConfigurationPacket::INSTANCE.is_terminal());
    }

    #[test]
    fn copy_and_eq_semantics() {
        let p = ClientboundStartConfigurationPacket::INSTANCE;
        let copied = p;
        assert_eq!(p, copied);
        assert!(copied.is_terminal());
    }
}
