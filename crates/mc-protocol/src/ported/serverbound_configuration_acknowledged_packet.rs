use std::io::{self, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ServerGamePacketListener {
    fn handle_configuration_acknowledged(
        &mut self,
        packet: &ServerboundConfigurationAcknowledgedPacket,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundConfigurationAcknowledgedPacket {
    _private: (),
}

pub const INSTANCE: ServerboundConfigurationAcknowledgedPacket =
    ServerboundConfigurationAcknowledgedPacket { _private: () };

impl ServerboundConfigurationAcknowledgedPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket#<init>
    const fn new() -> Self {
        Self { _private: () }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:configuration_acknowledged")
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_configuration_acknowledged(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket#isTerminal
    pub fn is_terminal(&self) -> bool {
        true
    }

    /// Unit codec encode: writes no bytes.
    pub fn encode<W: Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    /// Unit codec decode: always returns the singleton instance.
    pub fn decode<R: Read>(_reader: &mut R) -> io::Result<Self> {
        Ok(INSTANCE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct MockListener {
        called: bool,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_configuration_acknowledged(
            &mut self,
            _packet: &ServerboundConfigurationAcknowledgedPacket,
        ) {
            self.called = true;
        }
    }

    #[test]
    fn singleton_type_and_terminal() {
        let packet = INSTANCE;
        assert_eq!(
            packet,
            ServerboundConfigurationAcknowledgedPacket::new()
        );
        assert_eq!(
            packet.get_type(),
            PacketType("minecraft:configuration_acknowledged")
        );
        assert!(packet.is_terminal());
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener { called: false };
        INSTANCE.handle(&mut listener);
        assert!(listener.called);
    }

    #[test]
    fn unit_codec_roundtrip_is_empty() {
        let mut buf = Vec::new();
        INSTANCE.encode(&mut buf).unwrap();
        assert!(buf.is_empty());
        let decoded =
            ServerboundConfigurationAcknowledgedPacket::decode(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(decoded, INSTANCE);
    }

    #[test]
    fn decode_ignores_input_for_unit_codec() {
        let input = [0xFFu8, 0x00, 0x7F];
        let decoded =
            ServerboundConfigurationAcknowledgedPacket::decode(&mut Cursor::new(&input)).unwrap();
        assert_eq!(decoded, INSTANCE);
        assert!(decoded.is_terminal());
    }
}
