use bytes::{BufMut, BytesMut};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(&self) -> i64 {
        let x = (self.x as i64) & 0x3FF_FFFF;
        let z = (self.z as i64) & 0x3FF_FFFF;
        let y = (self.y as i64) & 0xFFF;
        (x << 38) | (z << 12) | y
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn of(packed: i64) -> Self {
        let x = (packed >> 38) as i32;
        let y = (packed << 52 >> 52) as i32;
        let z = (packed << 26 >> 38) as i32;
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#immutable
    pub fn immutable(&self) -> Self {
        *self
    }
}

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: BytesMut,
    rpos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: BytesMut::from(bytes),
            rpos: 0,
        }
    }

    pub fn into_inner(self) -> BytesMut {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.rpos)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInt
    pub fn read_int(&mut self) -> io::Result<i32> {
        if self.remaining() < 4 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_int"));
        }
        let v = i32::from_be_bytes([
            self.data[self.rpos],
            self.data[self.rpos + 1],
            self.data[self.rpos + 2],
            self.data[self.rpos + 3],
        ]);
        self.rpos += 4;
        Ok(v)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInt
    pub fn write_int(&mut self, value: i32) {
        self.data.put_i32(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> io::Result<i64> {
        if self.remaining() < 8 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_long"));
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.rpos..self.rpos + 8]);
        self.rpos += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, value: i64) {
        self.data.put_i64(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_boolean(&mut self) -> io::Result<bool> {
        if self.remaining() < 1 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_boolean"));
        }
        let v = self.data[self.rpos];
        self.rpos += 1;
        Ok(v != 0)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_boolean(&mut self, value: bool) {
        self.data.put_u8(if value { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBlockPos
    pub fn read_block_pos(&mut self) -> io::Result<BlockPos> {
        Ok(BlockPos::of(self.read_long()?))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBlockPos
    pub fn write_block_pos(&mut self, pos: BlockPos) {
        self.write_long(pos.as_long());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_LEVEL_EVENT: PacketType = PacketType;
}

pub trait ClientGamePacketListener {
    fn handle_level_event(&mut self, packet: &ClientboundLevelEventPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLevelEventPacket {
    type_: i32,
    pos: BlockPos,
    data: i32,
    global_event: bool,
}

impl ClientboundLevelEventPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#<init>
    pub fn new(type_: i32, pos: BlockPos, data: i32, global_event: bool) -> Self {
        Self {
            type_,
            pos: pos.immutable(),
            data,
            global_event,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let type_ = buf.read_int()?;
        let pos = buf.read_block_pos()?;
        let data = buf.read_int()?;
        let global_event = buf.read_boolean()?;
        Ok(Self {
            type_,
            pos,
            data,
            global_event,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.type_);
        buf.write_block_pos(self.pos);
        buf.write_int(self.data);
        buf.write_boolean(self.global_event);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#type
    pub fn packet_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_LEVEL_EVENT
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_level_event(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#isGlobalEvent
    pub fn is_global_event(&self) -> bool {
        self.global_event
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#getType
    pub fn get_type(&self) -> i32 {
        self.type_
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#getData
    pub fn get_data(&self) -> i32 {
        self.data
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelEventPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_basic() {
        let pkt = ClientboundLevelEventPacket::new(
            2001,
            BlockPos {
                x: 1,
                y: 64,
                z: -2,
            },
            42,
            false,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_inner();
        let mut read_buf = FriendlyByteBuf::from_bytes(&bytes);
        let decoded = ClientboundLevelEventPacket::from_buf(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_type(), 2001);
        assert_eq!(decoded.get_data(), 42);
        assert_eq!(
            decoded.get_pos(),
            BlockPos {
                x: 1,
                y: 64,
                z: -2
            }
        );
        assert!(!decoded.is_global_event());
    }

    #[test]
    fn roundtrip_negative_and_extreme_ints() {
        let pkt = ClientboundLevelEventPacket::new(
            i32::MIN,
            BlockPos {
                x: -1,
                y: -1,
                z: -1,
            },
            i32::MAX,
            true,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_inner();
        let mut read_buf = FriendlyByteBuf::from_bytes(&bytes);
        let decoded = ClientboundLevelEventPacket::from_buf(&mut read_buf).unwrap();
        assert_eq!(decoded.get_type(), i32::MIN);
        assert_eq!(decoded.get_data(), i32::MAX);
        assert!(decoded.is_global_event());
        assert_eq!(
            decoded.get_pos(),
            BlockPos {
                x: -1,
                y: -1,
                z: -1
            }
        );
    }

    #[test]
    fn block_pos_packing_edges() {
        let cases = [
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos {
                x: 33_554_431,
                y: 2047,
                z: 33_554_431,
            },
            BlockPos {
                x: -33_554_432,
                y: -2048,
                z: -33_554_432,
            },
            BlockPos {
                x: 12_345,
                y: -64,
                z: -54_321,
            },
        ];
        for pos in cases {
            let packed = pos.as_long();
            let unpacked = BlockPos::of(packed);
            assert_eq!(pos, unpacked, "failed for {:?}", pos);
        }
    }

    #[test]
    fn wire_format_matches_expected_layout() {
        let pos = BlockPos { x: 1, y: 2, z: 3 };
        let pkt = ClientboundLevelEventPacket::new(10, pos, 20, true);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_inner();
        assert_eq!(bytes.len(), 4 + 8 + 4 + 1);
        assert_eq!(&bytes[0..4], &10i32.to_be_bytes());
        assert_eq!(&bytes[4..12], &pos.as_long().to_be_bytes());
        assert_eq!(&bytes[12..16], &20i32.to_be_bytes());
        assert_eq!(bytes[16], 1);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct MockListener {
            called: bool,
            seen_type: i32,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_level_event(&mut self, packet: &ClientboundLevelEventPacket) {
                self.called = true;
                self.seen_type = packet.get_type();
            }
        }
        let pkt = ClientboundLevelEventPacket::new(1023, BlockPos { x: 0, y: 0, z: 0 }, 0, false);
        let mut listener = MockListener {
            called: false,
            seen_type: 0,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.seen_type, 1023);
    }
}
