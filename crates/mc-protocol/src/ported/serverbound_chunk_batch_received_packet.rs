use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PacketType(pub &'static str);

pub mod game_packet_types {
    use super::PacketType;

    pub const SERVERBOUND_CHUNK_BATCH_RECEIVED: PacketType =
        PacketType("serverbound_chunk_batch_received");
}

pub trait ServerGamePacketListener {
    fn handle_chunk_batch_received(&mut self, packet: &ServerboundChunkBatchReceivedPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundChunkBatchReceivedPacket {
    pub desired_chunks_per_tick: f32,
}

impl ServerboundChunkBatchReceivedPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket#<init>(float)
    pub fn new(desired_chunks_per_tick: f32) -> Self {
        Self {
            desired_chunks_per_tick,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            desired_chunks_per_tick: buf.get_f32(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        buf.put_f32(self.desired_chunks_per_tick);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket#type
    pub fn packet_type(&self) -> PacketType {
        game_packet_types::SERVERBOUND_CHUNK_BATCH_RECEIVED
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_chunk_batch_received(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_normal() {
        let pkt = ServerboundChunkBatchReceivedPacket::new(5.0);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        assert_eq!(buf.len(), 4);
        assert_eq!(&buf[..], &5.0f32.to_be_bytes());
        let decoded = ServerboundChunkBatchReceivedPacket::read(&mut buf);
        assert_eq!(
            pkt.desired_chunks_per_tick.to_bits(),
            decoded.desired_chunks_per_tick.to_bits()
        );
    }

    #[test]
    fn roundtrip_negative_and_zero() {
        for val in [0.0f32, -0.0f32, -1.5f32, -9999.0f32] {
            let pkt = ServerboundChunkBatchReceivedPacket::new(val);
            let mut buf = BytesMut::new();
            pkt.write(&mut buf);
            let decoded = ServerboundChunkBatchReceivedPacket::read(&mut buf);
            assert_eq!(val.to_bits(), decoded.desired_chunks_per_tick.to_bits());
        }
    }

    #[test]
    fn roundtrip_special_float_bits() {
        for bits in [
            0x7FC0_0001u32, // NaN payload
            0xFFC0_0001u32, // negative NaN payload
            0x7F80_0000u32, // +inf
            0xFF80_0000u32, // -inf
            0x0000_0001u32, // subnormal
        ] {
            let val = f32::from_bits(bits);
            let pkt = ServerboundChunkBatchReceivedPacket::new(val);
            let mut buf = BytesMut::new();
            pkt.write(&mut buf);
            let decoded = ServerboundChunkBatchReceivedPacket::read(&mut buf);
            assert_eq!(bits, decoded.desired_chunks_per_tick.to_bits());
        }
    }

    #[test]
    fn packet_type_and_handle() {
        struct MockListener {
            called: bool,
            value_bits: u32,
        }
        impl ServerGamePacketListener for MockListener {
            fn handle_chunk_batch_received(
                &mut self,
                packet: &ServerboundChunkBatchReceivedPacket,
            ) {
                self.called = true;
                self.value_bits = packet.desired_chunks_per_tick.to_bits();
            }
        }

        let pkt = ServerboundChunkBatchReceivedPacket::new(2.5);
        assert_eq!(
            pkt.packet_type(),
            game_packet_types::SERVERBOUND_CHUNK_BATCH_RECEIVED
        );

        let mut listener = MockListener {
            called: false,
            value_bits: 0,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.value_bits, 2.5f32.to_bits());
    }
}
