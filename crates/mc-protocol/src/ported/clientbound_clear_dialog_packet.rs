//! Port of `net.minecraft.network.protocol.common.ClientboundClearDialogPacket`.

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

/// Placeholder for `net.minecraft.network.codec.StreamCodec`.
#[derive(Debug, Clone, Copy)]
pub struct StreamCodec<B: ?Sized, V> {
    pub value: V,
    _marker: PhantomData<fn(&B)>,
}

impl<B: ?Sized, V> StreamCodec<B, V> {
    pub const fn unit(value: V) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

/// Placeholder for `net.minecraft.network.protocol.common.ClientCommonPacketListener`.
pub trait ClientCommonPacketListener {
    fn handle_clear_dialog(&mut self, packet: &ClientboundClearDialogPacket);
}

/// Port of `net.minecraft.network.protocol.common.ClientboundClearDialogPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundClearDialogPacket {
    _private: (),
}

/// Placeholder for `net.minecraft.network.protocol.common.CommonPacketTypes`.
pub struct CommonPacketTypes;

impl CommonPacketTypes {
    pub const CLIENTBOUND_CLEAR_DIALOG: PacketType<ClientboundClearDialogPacket> =
        PacketType::new(0);
}

impl ClientboundClearDialogPacket {
    pub const INSTANCE: Self = Self { _private: () };

    pub const STREAM_CODEC: StreamCodec<ByteBuf, Self> = StreamCodec::unit(Self::INSTANCE);

    // JAVA: net.minecraft.network.protocol.common.ClientboundClearDialogPacket#type
    pub fn packet_type(&self) -> PacketType<Self> {
        CommonPacketTypes::CLIENTBOUND_CLEAR_DIALOG
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundClearDialogPacket#handle
    pub fn handle<L: ClientCommonPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_clear_dialog(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        clear_count: u32,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_clear_dialog(&mut self, _packet: &ClientboundClearDialogPacket) {
            self.clear_count = self.clear_count.wrapping_add(1);
        }
    }

    #[test]
    fn instance_is_copy_and_eq() {
        let a = ClientboundClearDialogPacket::INSTANCE;
        let b = a;
        assert_eq!(a, b);
        assert_eq!(a, ClientboundClearDialogPacket::INSTANCE);
    }

    #[test]
    fn packet_type_matches_constant() {
        let packet = ClientboundClearDialogPacket::INSTANCE;
        assert_eq!(
            packet.packet_type(),
            CommonPacketTypes::CLIENTBOUND_CLEAR_DIALOG
        );
        assert_eq!(packet.packet_type().id, 0);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener { clear_count: 0 };
        ClientboundClearDialogPacket::INSTANCE.handle(&mut listener);
        ClientboundClearDialogPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.clear_count, 2);
    }

    #[test]
    fn handle_wrapping_overflow_counter() {
        let mut listener = MockListener {
            clear_count: u32::MAX,
        };
        ClientboundClearDialogPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.clear_count, 0);
    }

    #[test]
    fn stream_codec_unit_holds_instance() {
        let codec = ClientboundClearDialogPacket::STREAM_CODEC;
        assert_eq!(codec.value, ClientboundClearDialogPacket::INSTANCE);
    }

    #[test]
    fn float_bits_edge_case_sanity() {
        assert_eq!((-0.0f32).to_bits(), 0x8000_0000);
        assert_eq!(
            f64::NAN.to_bits() & 0x7ff0_0000_0000_0000,
            0x7ff0_0000_0000_0000
        );
        assert_eq!(f32::INFINITY.to_bits(), 0x7f80_0000);
    }
}
