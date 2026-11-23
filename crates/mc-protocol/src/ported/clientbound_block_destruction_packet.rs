use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    const PACKED_X_LENGTH: i32 = 26;
    const PACKED_Z_LENGTH: i32 = 26;
    const PACKED_Y_LENGTH: i32 = 12;
    const PACKED_X_MASK: i64 = (1i64 << Self::PACKED_X_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1i64 << Self::PACKED_Y_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1i64 << Self::PACKED_Z_LENGTH) - 1;
    const Z_OFFSET: i32 = Self::PACKED_Y_LENGTH;
    const X_OFFSET: i32 = Self::Z_OFFSET + Self::PACKED_Z_LENGTH;

    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(&self) -> i64 {
        ((self.x as i64 & Self::PACKED_X_MASK) << Self::X_OFFSET)
            | ((self.z as i64 & Self::PACKED_Z_MASK) << Self::Z_OFFSET)
            | (self.y as i64 & Self::PACKED_Y_MASK)
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn of(packed: i64) -> Self {
        Self {
            x: Self::get_x(packed),
            y: Self::get_y(packed),
            z: Self::get_z(packed),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#getX(long)
    pub fn get_x(packed: i64) -> i32 {
        (packed << (64 - Self::X_OFFSET - Self::PACKED_X_LENGTH)
            >> (64 - Self::PACKED_X_LENGTH)) as i32
    }

    // JAVA: net.minecraft.core.BlockPos#getY(long)
    pub fn get_y(packed: i64) -> i32 {
        (packed << (64 - Self::PACKED_Y_LENGTH) >> (64 - Self::PACKED_Y_LENGTH)) as i32
    }

    // JAVA: net.minecraft.core.BlockPos#getZ(long)
    pub fn get_z(packed: i64) -> i32 {
        (packed << (64 - Self::Z_OFFSET - Self::PACKED_Z_LENGTH)
            >> (64 - Self::PACKED_Z_LENGTH)) as i32
    }
}

#[derive(Debug)]
pub struct FriendlyByteBuf<B> {
    pub inner: B,
}

impl<B: Buf> FriendlyByteBuf<B> {
    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b = self.inner.get_u8() as i8;
            i |= ((b & 127) as i32) << ((j.wrapping_mul(7)) & 31);
            j = j.wrapping_add(1);
            if j > 5 {
                panic!("VarInt too big");
            }
            if (b & -128i8) != -128i8 {
                break;
            }
        }
        i
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBlockPos
    pub fn read_block_pos(&mut self) -> BlockPos {
        BlockPos::of(self.inner.get_i64())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUnsignedByte
    pub fn read_unsigned_byte(&mut self) -> i32 {
        self.inner.get_u8() as i32
    }
}

impl<B: BufMut> FriendlyByteBuf<B> {
    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & !0x7F) == 0 {
                self.inner.put_u8(value as u8);
                return self;
            }
            self.inner.put_u8(((value & 0x7F) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBlockPos
    pub fn write_block_pos(&mut self, pos: BlockPos) -> &mut Self {
        self.inner.put_i64(pos.as_long());
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte
    pub fn write_byte(&mut self, value: i32) -> &mut Self {
        self.inner.put_u8(value as u8);
        self
    }
}

pub trait ClientGamePacketListener {
    fn handle_block_destruction(&mut self, packet: &ClientboundBlockDestructionPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundBlockDestructionPacket {
    id: i32,
    pos: BlockPos,
    progress: i32,
}

impl ClientboundBlockDestructionPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket#<init>(int, BlockPos, int)
    pub fn new(id: i32, pos: BlockPos, progress: i32) -> Self {
        Self { id, pos, progress }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut FriendlyByteBuf<B>) -> Self {
        let id = buf.read_var_int();
        let pos = buf.read_block_pos();
        let progress = buf.read_unsigned_byte();
        Self { id, pos, progress }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut FriendlyByteBuf<B>) {
        buf.write_var_int(self.id);
        buf.write_block_pos(self.pos);
        buf.write_byte(self.progress);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_block_destruction(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket#getProgress
    pub fn get_progress(&self) -> i32 {
        self.progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener(Option<(i32, BlockPos, i32)>);
    impl ClientGamePacketListener for MockListener {
        fn handle_block_destruction(&mut self, packet: &ClientboundBlockDestructionPacket) {
            self.0 = Some((packet.get_id(), packet.get_pos(), packet.get_progress()));
        }
    }

    #[test]
    fn round_trip() {
        let pkt = ClientboundBlockDestructionPacket::new(42, BlockPos { x: 1, y: 64, z: -3 }, 7);
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ClientboundBlockDestructionPacket::read(&mut read_buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn negative_and_boundary_values() {
        let pkt = ClientboundBlockDestructionPacket::new(
            -1,
            BlockPos { x: -33_554_432, y: -2048, z: 33_554_431 },
            255,
        );
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ClientboundBlockDestructionPacket::read(&mut read_buf);
        assert_eq!(decoded.get_id(), -1);
        assert_eq!(decoded.get_pos(), BlockPos { x: -33_554_432, y: -2048, z: 33_554_431 });
        assert_eq!(decoded.get_progress(), 255);
    }

    #[test]
    fn varint_extremes_round_trip() {
        for v in [0, 1, -1, i32::MAX, i32::MIN, 127, 128, 255, -2147483648] {
            let pkt = ClientboundBlockDestructionPacket::new(v, BlockPos { x: 0, y: 0, z: 0 }, 0);
            let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
            pkt.write(&mut buf);
            let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
            let decoded = ClientboundBlockDestructionPacket::read(&mut read_buf);
            assert_eq!(decoded.get_id(), v);
        }
    }

    #[test]
    fn block_pos_packing_round_trip() {
        let positions = [
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 12345, y: -64, z: -9876 },
            BlockPos { x: -1, y: -1, z: -1 },
            BlockPos { x: 33_554_431, y: 2047, z: 33_554_431 },
        ];
        for pos in positions {
            assert_eq!(BlockPos::of(pos.as_long()), pos);
        }
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundBlockDestructionPacket::new(5, BlockPos { x: 1, y: 2, z: 3 }, 9);
        let mut listener = MockListener(None);
        pkt.handle(&mut listener);
        assert_eq!(listener.0, Some((5, BlockPos { x: 1, y: 2, z: 3 }, 9)));
    }
}
