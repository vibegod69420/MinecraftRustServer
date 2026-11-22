use std::marker::PhantomData;

use bytes::BytesMut;

/// Stub for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType<T> {
    pub id: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> PacketType<T> {
    pub const fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

/// Stub for `net.minecraft.network.codec.StreamCodec` unit codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamCodec<B, V> {
    value: V,
    _marker: PhantomData<fn(B)>,
}

impl<B, V: Copy> StreamCodec<B, V> {
    pub const fn unit(value: V) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub const fn value(&self) -> V {
        self.value
    }
}

/// Stub for `net.minecraft.network.protocol.configuration.ConfigurationPacketTypes`.
pub struct ConfigurationPacketTypes;

impl ConfigurationPacketTypes {
    pub const CLIENTBOUND_RESET_CHAT: PacketType<ClientboundResetChatPacket> =
        PacketType::new("configuration/clientbound_reset_chat");
}

/// Stub for `net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener`.
pub trait ClientConfigurationPacketListener {
    fn handle_reset_chat(&mut self, packet: &ClientboundResetChatPacket);
}

/// Port of `net.minecraft.network.protocol.configuration.ClientboundResetChatPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundResetChatPacket(());

impl ClientboundResetChatPacket {
    pub const INSTANCE: Self = Self(());

    pub const STREAM_CODEC: StreamCodec<BytesMut, Self> = StreamCodec::unit(Self::INSTANCE);

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundResetChatPacket#type
    pub fn get_type(&self) -> PacketType<Self> {
        ConfigurationPacketTypes::CLIENTBOUND_RESET_CHAT
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundResetChatPacket#handle
    pub fn handle(&self, listener: &mut impl ClientConfigurationPacketListener) {
        listener.handle_reset_chat(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockListener {
        calls: u32,
        last_was_instance: bool,
    }

    impl ClientConfigurationPacketListener for MockListener {
        fn handle_reset_chat(&mut self, packet: &ClientboundResetChatPacket) {
            self.calls = self.calls.wrapping_add(1);
            self.last_was_instance = *packet == ClientboundResetChatPacket::INSTANCE;
        }
    }

    #[test]
    fn instance_is_copy_and_eq() {
        let a = ClientboundResetChatPacket::INSTANCE;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn packet_type_matches_configuration_constant() {
        let packet = ClientboundResetChatPacket::INSTANCE;
        assert_eq!(
            packet.get_type(),
            ConfigurationPacketTypes::CLIENTBOUND_RESET_CHAT
        );
        assert_eq!(
            packet.get_type().id,
            "configuration/clientbound_reset_chat"
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener::default();
        ClientboundResetChatPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.calls, 1);
        assert!(listener.last_was_instance);
    }

    #[test]
    fn stream_codec_unit_holds_instance() {
        let codec = ClientboundResetChatPacket::STREAM_CODEC;
        assert_eq!(codec.value(), ClientboundResetChatPacket::INSTANCE);
    }

    #[test]
    fn repeated_handle_wraps_u32_counter_on_overflow() {
        let mut listener = MockListener {
            calls: u32::MAX,
            last_was_instance: false,
        };
        ClientboundResetChatPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.calls, 0);
        assert!(listener.last_was_instance);
    }
}
