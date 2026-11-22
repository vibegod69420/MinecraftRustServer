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
    pub const SERVERBOUND_ACCEPT_CODE_OF_CONDUCT: PacketType<
        ServerboundAcceptCodeOfConductPacket,
    > = PacketType::new("configuration/serverbound_accept_code_of_conduct");
}

/// Stub for `net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener`.
pub trait ServerConfigurationPacketListener {
    fn handle_accept_code_of_conduct(&mut self, packet: &ServerboundAcceptCodeOfConductPacket);
}

/// Port of `net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundAcceptCodeOfConductPacket(());

impl ServerboundAcceptCodeOfConductPacket {
    pub const INSTANCE: Self = Self(());

    pub const STREAM_CODEC: StreamCodec<BytesMut, Self> = StreamCodec::unit(Self::INSTANCE);

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket#type
    pub fn get_type(&self) -> PacketType<Self> {
        ConfigurationPacketTypes::SERVERBOUND_ACCEPT_CODE_OF_CONDUCT
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket#handle
    pub fn handle(&self, listener: &mut impl ServerConfigurationPacketListener) {
        listener.handle_accept_code_of_conduct(self);
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

    impl ServerConfigurationPacketListener for MockListener {
        fn handle_accept_code_of_conduct(
            &mut self,
            packet: &ServerboundAcceptCodeOfConductPacket,
        ) {
            self.calls = self.calls.wrapping_add(1);
            self.last_was_instance = *packet == ServerboundAcceptCodeOfConductPacket::INSTANCE;
        }
    }

    #[test]
    fn instance_is_copy_and_eq() {
        let a = ServerboundAcceptCodeOfConductPacket::INSTANCE;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn packet_type_matches_configuration_constant() {
        let packet = ServerboundAcceptCodeOfConductPacket::INSTANCE;
        assert_eq!(
            packet.get_type(),
            ConfigurationPacketTypes::SERVERBOUND_ACCEPT_CODE_OF_CONDUCT
        );
        assert_eq!(
            packet.get_type().id,
            "configuration/serverbound_accept_code_of_conduct"
        );
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener::default();
        ServerboundAcceptCodeOfConductPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.calls, 1);
        assert!(listener.last_was_instance);
    }

    #[test]
    fn stream_codec_unit_holds_instance() {
        let codec = ServerboundAcceptCodeOfConductPacket::STREAM_CODEC;
        assert_eq!(
            codec.value(),
            ServerboundAcceptCodeOfConductPacket::INSTANCE
        );
    }

    #[test]
    fn repeated_handle_wraps_u32_counter_on_overflow() {
        let mut listener = MockListener {
            calls: u32::MAX,
            last_was_instance: false,
        };
        ServerboundAcceptCodeOfConductPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.calls, 0);
        assert!(listener.last_was_instance);
    }
}
