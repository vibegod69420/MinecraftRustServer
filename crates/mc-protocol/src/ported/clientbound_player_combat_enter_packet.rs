use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub u32);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_PLAYER_COMBAT_ENTER: PacketType = PacketType(0x39);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPlayerCombatEnterPacket(());

impl ClientboundPlayerCombatEnterPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket#<init>
    const fn new() -> Self {
        Self(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket#type
    pub fn packet_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_ENTER
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_player_combat_enter(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket#STREAM_CODEC
    pub fn encode<B: BufMut>(&self, _buf: &mut B) {}

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket#STREAM_CODEC
    pub fn decode<B: Buf>(_buf: &mut B) -> Self {
        INSTANCE
    }
}

// JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket#INSTANCE
pub const INSTANCE: ClientboundPlayerCombatEnterPacket = ClientboundPlayerCombatEnterPacket::new();

pub trait ClientGamePacketListener {
    fn handle_player_combat_enter(&mut self, packet: &ClientboundPlayerCombatEnterPacket);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener {
        called: bool,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_player_combat_enter(&mut self, _packet: &ClientboundPlayerCombatEnterPacket) {
            self.called = true;
        }
    }

    #[test]
    fn instance_and_type() {
        assert_eq!(INSTANCE, ClientboundPlayerCombatEnterPacket::decode(&mut &[][..]));
        assert_eq!(
            INSTANCE.packet_type(),
            GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_ENTER
        );
    }

    #[test]
    fn encode_writes_nothing() {
        let mut buf = BytesMut::new();
        INSTANCE.encode(&mut buf);
        assert!(buf.is_empty());
    }

    #[test]
    fn decode_from_non_empty_buffer_preserves_bytes() {
        let bits = f32::NEG_INFINITY.to_bits();
        let bytes = bits.to_le_bytes();
        let mut buf: &[u8] = &bytes;
        let pkt = ClientboundPlayerCombatEnterPacket::decode(&mut buf);
        assert_eq!(pkt, INSTANCE);
        assert_eq!(buf.len(), bytes.len());
    }

    #[test]
    fn handle_dispatches() {
        let mut listener = TestListener { called: false };
        INSTANCE.handle(&mut listener);
        assert!(listener.called);
    }

    #[test]
    fn negative_and_overflow_edge_values_in_buffer() {
        let neg: i8 = -128;
        let overflow_byte = (i16::MAX).wrapping_add(1) as u8;
        let data = [neg as u8, overflow_byte, 0xFF];
        let mut buf: &[u8] = &data;
        let pkt = ClientboundPlayerCombatEnterPacket::decode(&mut buf);
        assert_eq!(pkt, INSTANCE);
        assert_eq!(buf, &data[..]);
    }
}
