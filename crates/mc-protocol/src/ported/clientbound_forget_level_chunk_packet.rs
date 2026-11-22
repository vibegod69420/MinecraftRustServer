use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PacketType(pub &'static str);

pub mod game_packet_types {
    use super::PacketType;

    pub const CLIENTBOUND_FORGET_LEVEL_CHUNK: PacketType =
        PacketType("clientbound_forget_level_chunk");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#<init>(int, int)
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#asLong
    pub fn as_long(self) -> i64 {
        ((self.x as i64) & 0xFFFF_FFFF_i64) | (((self.z as i64) & 0xFFFF_FFFF_i64) << 32)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#<init>(long)
    pub fn from_long(packed: i64) -> Self {
        Self {
            x: packed as i32,
            z: (packed >> 32) as i32,
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readChunkPos
pub fn read_chunk_pos(buf: &mut impl Buf) -> ChunkPos {
    ChunkPos::from_long(buf.get_i64())
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeChunkPos
pub fn write_chunk_pos(buf: &mut impl BufMut, pos: ChunkPos) {
    buf.put_i64(pos.as_long());
}

pub trait ClientGamePacketListener {
    fn handle_forget_level_chunk(&mut self, packet: &ClientboundForgetLevelChunkPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundForgetLevelChunkPacket {
    pub pos: ChunkPos,
}

impl ClientboundForgetLevelChunkPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket#<init>(ChunkPos)
    pub fn new(pos: ChunkPos) -> Self {
        Self { pos }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            pos: read_chunk_pos(buf),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_chunk_pos(buf, self.pos);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket#type
    pub fn packet_type(&self) -> PacketType {
        game_packet_types::CLIENTBOUND_FORGET_LEVEL_CHUNK
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_forget_level_chunk(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn chunk_pos_long_roundtrip() {
        let positions = [
            ChunkPos::new(0, 0),
            ChunkPos::new(1, -1),
            ChunkPos::new(-1, 1),
            ChunkPos::new(i32::MIN, i32::MAX),
            ChunkPos::new(i32::MAX, i32::MIN),
        ];
        for pos in positions {
            let packed = pos.as_long();
            let unpacked = ChunkPos::from_long(packed);
            assert_eq!(pos, unpacked);
        }
    }

    #[test]
    fn packet_roundtrip() {
        let pkt = ClientboundForgetLevelChunkPacket::new(ChunkPos::new(10, -20));
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        assert_eq!(buf.len(), 8);
        let decoded = ClientboundForgetLevelChunkPacket::read(&mut buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn packet_negative_and_extreme_coords() {
        let pkt = ClientboundForgetLevelChunkPacket::new(ChunkPos::new(i32::MIN, -1));
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundForgetLevelChunkPacket::read(&mut buf);
        assert_eq!(pkt.pos, decoded.pos);
    }

    #[test]
    fn packet_type_and_handle() {
        struct MockListener {
            seen: Option<ChunkPos>,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_forget_level_chunk(&mut self, packet: &ClientboundForgetLevelChunkPacket) {
                self.seen = Some(packet.pos);
            }
        }

        let pkt = ClientboundForgetLevelChunkPacket::new(ChunkPos::new(3, 4));
        assert_eq!(
            pkt.packet_type(),
            game_packet_types::CLIENTBOUND_FORGET_LEVEL_CHUNK
        );
        let mut listener = MockListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen, Some(ChunkPos::new(3, 4)));
    }
}
