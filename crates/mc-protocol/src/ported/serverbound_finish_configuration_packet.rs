use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    UnexpectedEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const SERVERBOUND_FINISH_CONFIGURATION: PacketType =
    PacketType("configuration/serverbound_finish_configuration");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundFinishConfigurationPacket(());

pub const INSTANCE: ServerboundFinishConfigurationPacket = ServerboundFinishConfigurationPacket(());

pub trait ServerConfigurationPacketListener {
    fn handle_configuration_finished(&mut self, packet: &ServerboundFinishConfigurationPacket);
}

impl ServerboundFinishConfigurationPacket {
    // JAVA: net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket#type
    pub fn type_(&self) -> PacketType {
        SERVERBOUND_FINISH_CONFIGURATION
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket#handle
    pub fn handle<L: ServerConfigurationPacketListener>(&self, listener: &mut L) {
        listener.handle_configuration_finished(self);
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket#isTerminal
    pub fn is_terminal(&self) -> bool {
        true
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket#STREAM_CODEC
    pub fn encode(&self, _buf: &mut BytesMut) {}

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket#STREAM_CODEC
    pub fn decode(_buf: &mut &[u8]) -> Result<Self, CodecError> {
        Ok(INSTANCE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
    }

    impl ServerConfigurationPacketListener for MockListener {
        fn handle_configuration_finished(
            &mut self,
            _packet: &ServerboundFinishConfigurationPacket,
        ) {
            self.called = true;
        }
    }

    #[test]
    fn type_is_finish_configuration() {
        assert_eq!(INSTANCE.type_(), SERVERBOUND_FINISH_CONFIGURATION);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener { called: false };
        INSTANCE.handle(&mut listener);
        assert!(listener.called);
    }

    #[test]
    fn is_terminal_is_true() {
        assert!(INSTANCE.is_terminal());
    }

    #[test]
    fn encode_decode_roundtrip_empty() {
        let mut buf = BytesMut::new();
        INSTANCE.encode(&mut buf);
        assert!(buf.is_empty());
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundFinishConfigurationPacket::decode(&mut slice).unwrap();
        assert_eq!(decoded, INSTANCE);
        assert!(slice.is_empty());
    }

    #[test]
    fn decode_ignores_remaining_input() {
        let data = [0x01u8, 0x80, 0xff];
        let mut slice: &[u8] = &data;
        let decoded = ServerboundFinishConfigurationPacket::decode(&mut slice).unwrap();
        assert_eq!(decoded, INSTANCE);
        assert_eq!(slice.len(), 3);
    }

    #[test]
    fn instance_is_copy_and_eq() {
        let a = INSTANCE;
        let b = a;
        assert_eq!(a, b);
        let _ = format!("{:?}", a);
    }
}
