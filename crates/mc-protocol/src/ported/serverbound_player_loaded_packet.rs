use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType<T: ?Sized> {
    pub id: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> PacketType<T> {
    pub const fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServerboundPlayerLoadedPacket;

pub trait ServerGamePacketListener {
    fn handle_accept_player_load(&mut self, packet: &ServerboundPlayerLoadedPacket);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ByteBuf;

pub struct StreamCodec<B, V> {
    value: V,
    _phantom: PhantomData<fn(&mut B)>,
}

impl<B, V: Copy> StreamCodec<B, V> {
    pub const fn unit(value: V) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    pub fn decode(&self, _buf: &mut B) -> V {
        self.value
    }

    pub fn encode(&self, _buf: &mut B, _value: &V) {}
}

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const SERVERBOUND_PLAYER_LOADED: PacketType<ServerboundPlayerLoadedPacket> =
        PacketType::new("serverbound_player_loaded");
}

pub static STREAM_CODEC: StreamCodec<ByteBuf, ServerboundPlayerLoadedPacket> =
    StreamCodec::unit(ServerboundPlayerLoadedPacket);

impl ServerboundPlayerLoadedPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket#type
    pub fn type_(&self) -> PacketType<ServerboundPlayerLoadedPacket> {
        GamePacketTypes::SERVERBOUND_PLAYER_LOADED
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_accept_player_load(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        seen_id: Option<&'static str>,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_accept_player_load(&mut self, packet: &ServerboundPlayerLoadedPacket) {
            self.called = true;
            self.seen_id = Some(packet.type_().id);
        }
    }

    #[test]
    fn type_returns_player_loaded() {
        let pkt = ServerboundPlayerLoadedPacket;
        assert_eq!(
            pkt.type_(),
            GamePacketTypes::SERVERBOUND_PLAYER_LOADED
        );
        assert_eq!(pkt.type_().id, "serverbound_player_loaded");
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let pkt = ServerboundPlayerLoadedPacket;
        let mut listener = MockListener {
            called: false,
            seen_id: None,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.seen_id, Some("serverbound_player_loaded"));
    }

    #[test]
    fn stream_codec_decode_returns_unit_value() {
        let mut buf = ByteBuf;
        let decoded = STREAM_CODEC.decode(&mut buf);
        assert_eq!(decoded, ServerboundPlayerLoadedPacket);
    }

    #[test]
    fn stream_codec_encode_is_noop_for_unit_packet() {
        let mut buf = ByteBuf::default();
        STREAM_CODEC.encode(&mut buf, &ServerboundPlayerLoadedPacket);
        let decoded = STREAM_CODEC.decode(&mut buf);
        assert_eq!(decoded, ServerboundPlayerLoadedPacket);
    }

    #[test]
    fn default_copy_and_equality() {
        let a = ServerboundPlayerLoadedPacket;
        let b = ServerboundPlayerLoadedPacket::default();
        let c = a;
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
