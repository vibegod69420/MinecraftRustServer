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
    pub const CLIENTBOUND_CHUNK_BATCH_START: PacketType<ClientboundChunkBatchStartPacket> =
        PacketType::new();
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleChunkBatchStart
    fn handle_chunk_batch_start(&mut self, packet: &ClientboundChunkBatchStartPacket);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundChunkBatchStartPacket {
    _private: (),
}

pub const INSTANCE: ClientboundChunkBatchStartPacket =
    ClientboundChunkBatchStartPacket { _private: () };
pub const STREAM_CODEC: StreamCodec<(), ClientboundChunkBatchStartPacket> =
    StreamCodec::unit(INSTANCE);

impl ClientboundChunkBatchStartPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundChunkBatchStartPacket#type
    pub fn type_(&self) -> PacketType<Self> {
        GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChunkBatchStartPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_chunk_batch_start(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        calls: i32,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_chunk_batch_start(&mut self, _packet: &ClientboundChunkBatchStartPacket) {
            self.calls = self.calls.wrapping_sub(1);
        }
    }

    #[test]
    fn singleton_instance_copy_and_eq() {
        let a = INSTANCE;
        let b = a;
        assert_eq!(a, b);
        assert_eq!(INSTANCE, INSTANCE);
    }

    #[test]
    fn packet_type_matches_game_packet_type_constant() {
        assert_eq!(
            INSTANCE.type_(),
            GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START
        );
    }

    #[test]
    fn handle_dispatches_with_negative_wrapping() {
        let mut listener = MockListener { calls: i32::MIN };
        INSTANCE.handle(&mut listener);
        assert_eq!(listener.calls, i32::MAX);
    }

    #[test]
    fn stream_codec_unit_is_constructible_and_eq() {
        let codec = StreamCodec::<(), ClientboundChunkBatchStartPacket>::unit(INSTANCE);
        assert_eq!(STREAM_CODEC, codec);
    }

    #[test]
    fn float_bits_and_negative_zero_edge_cases() {
        assert_eq!(
            std::mem::size_of::<ClientboundChunkBatchStartPacket>(),
            0
        );
        let neg_zero_f32 = (-0.0_f32).to_bits();
        let pos_zero_f32 = 0.0_f32.to_bits();
        assert_ne!(neg_zero_f32, pos_zero_f32);
        assert_eq!(u32::MAX.wrapping_add(1), 0);
    }
}
