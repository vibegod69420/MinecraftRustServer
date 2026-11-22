use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketType<T>(PhantomData<T>);

impl<T> PacketType<T> {
    // JAVA: net.minecraft.network.protocol.PacketType#<init>
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamCodec<B, V>(PhantomData<(B, V)>);

impl<B, V: Copy> StreamCodec<B, V> {
    // JAVA: net.minecraft.network.codec.StreamCodec#unit
    pub const fn unit(_value: V) -> Self {
        Self(PhantomData)
    }
}

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const SERVERBOUND_CLIENT_TICK_END: PacketType<ServerboundClientTickEndPacket> =
        PacketType::new();
}

pub trait ServerGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerGamePacketListener#handleClientTickEnd
    fn handle_client_tick_end(&mut self, packet: &ServerboundClientTickEndPacket);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundClientTickEndPacket;

pub const INSTANCE: ServerboundClientTickEndPacket = ServerboundClientTickEndPacket;
pub const STREAM_CODEC: StreamCodec<(), ServerboundClientTickEndPacket> =
    StreamCodec::unit(INSTANCE);

impl ServerboundClientTickEndPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundClientTickEndPacket#type
    pub fn type_(&self) -> PacketType<Self> {
        GamePacketTypes::SERVERBOUND_CLIENT_TICK_END
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundClientTickEndPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_client_tick_end(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        calls: u32,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_client_tick_end(&mut self, _packet: &ServerboundClientTickEndPacket) {
            self.calls = self.calls.wrapping_add(1);
        }
    }

    #[test]
    fn singleton_instance_copy_and_eq() {
        let a = INSTANCE;
        let b = a;
        assert_eq!(a, b);
        assert_eq!(INSTANCE, ServerboundClientTickEndPacket);
    }

    #[test]
    fn packet_type_matches_game_packet_type_constant() {
        assert_eq!(
            INSTANCE.type_(),
            GamePacketTypes::SERVERBOUND_CLIENT_TICK_END
        );
    }

    #[test]
    fn handle_dispatches_with_wrapping_call_count() {
        let mut listener = MockListener { calls: u32::MAX };
        INSTANCE.handle(&mut listener);
        INSTANCE.handle(&mut listener);
        assert_eq!(listener.calls, 1);
    }

    #[test]
    fn stream_codec_unit_is_constructible_and_eq() {
        let codec = StreamCodec::<(), ServerboundClientTickEndPacket>::unit(INSTANCE);
        assert_eq!(STREAM_CODEC, codec);
    }

    #[test]
    fn float_bits_and_negative_zero_edge_cases() {
        let neg_zero_f32 = (-0.0_f32).to_bits();
        let pos_zero_f32 = 0.0_f32.to_bits();
        assert_ne!(neg_zero_f32, pos_zero_f32);

        let neg_zero_f64 = (-0.0_f64).to_bits();
        let pos_zero_f64 = 0.0_f64.to_bits();
        assert_ne!(neg_zero_f64, pos_zero_f64);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
    }
}
