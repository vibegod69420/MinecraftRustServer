//! Port of `net.minecraft.network.protocol.common.ClientboundDisconnectPacket`.

use std::marker::PhantomData;

/// Placeholder for `io.netty.buffer.ByteBuf`.
pub type ByteBuf = bytes::BytesMut;

/// Placeholder for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType<T: ?Sized> {
    pub id: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> PacketType<T> {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

/// Placeholder for `net.minecraft.network.chat.Component`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Placeholder for `net.minecraft.network.codec.StreamCodec`.
#[derive(Debug, Clone, Copy)]
pub struct StreamCodec<B: ?Sized, V> {
    _marker: PhantomData<fn(&B) -> V>,
}

impl<B: ?Sized, V> StreamCodec<B, V> {
    pub const fn placeholder() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Placeholder for `net.minecraft.network.chat.ComponentSerialization`.
pub struct ComponentSerialization;

impl ComponentSerialization {
    pub const TRUSTED_CONTEXT_FREE_STREAM_CODEC: StreamCodec<ByteBuf, Component> =
        StreamCodec::placeholder();
}

/// Placeholder for `net.minecraft.network.protocol.common.ClientCommonPacketListener`.
pub trait ClientCommonPacketListener {
    fn handle_disconnect(&mut self, packet: &ClientboundDisconnectPacket);
}

/// Port of `net.minecraft.network.protocol.common.ClientboundDisconnectPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDisconnectPacket {
    reason: Component,
}

/// Placeholder for `net.minecraft.network.protocol.common.CommonPacketTypes`.
pub struct CommonPacketTypes;

impl CommonPacketTypes {
    pub const CLIENTBOUND_DISCONNECT: PacketType<ClientboundDisconnectPacket> = PacketType::new(1);
}

impl ClientboundDisconnectPacket {
    pub const STREAM_CODEC: StreamCodec<ByteBuf, Self> = StreamCodec::placeholder();

    // JAVA: net.minecraft.network.protocol.common.ClientboundDisconnectPacket#<init>
    pub fn new(reason: Component) -> Self {
        Self { reason }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundDisconnectPacket#reason
    pub fn reason(&self) -> &Component {
        &self.reason
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundDisconnectPacket#type
    pub fn packet_type(&self) -> PacketType<Self> {
        CommonPacketTypes::CLIENTBOUND_DISCONNECT
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundDisconnectPacket#handle
    pub fn handle<L: ClientCommonPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_disconnect(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        reasons: Vec<String>,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_disconnect(&mut self, packet: &ClientboundDisconnectPacket) {
            self.reasons.push(packet.reason().text.clone());
        }
    }

    #[test]
    fn new_and_reason_access() {
        let packet = ClientboundDisconnectPacket::new(Component::literal("bye"));
        assert_eq!(packet.reason().text, "bye");
    }

    #[test]
    fn empty_and_negative_text_edge_cases() {
        let empty = ClientboundDisconnectPacket::new(Component::literal(""));
        assert_eq!(empty.reason().text, "");

        let negative = ClientboundDisconnectPacket::new(Component::literal("-1"));
        assert_eq!(negative.reason().text, "-1");
    }

    #[test]
    fn packet_type_and_handle_dispatch() {
        let packet = ClientboundDisconnectPacket::new(Component::literal("kicked"));
        assert_eq!(
            packet.packet_type(),
            CommonPacketTypes::CLIENTBOUND_DISCONNECT
        );
        assert_eq!(packet.packet_type().id, 1);

        let mut listener = MockListener {
            reasons: Vec::new(),
        };
        packet.handle(&mut listener);
        assert_eq!(listener.reasons, vec!["kicked"]);
    }

    #[test]
    fn clone_and_equality() {
        let a = ClientboundDisconnectPacket::new(Component::literal("reason"));
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn float_bits_edge_case_sanity() {
        assert_eq!((-0.0f64).to_bits(), 0x8000_0000_0000_0000);
        assert_eq!(f32::INFINITY.to_bits(), 0x7f80_0000);
        assert_eq!(f32::NEG_INFINITY.to_bits(), 0xff80_0000);
    }
}
