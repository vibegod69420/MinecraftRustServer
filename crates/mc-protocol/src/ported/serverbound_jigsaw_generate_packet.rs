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

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_boolean(&mut self) -> bool {
        self.inner.get_u8() != 0
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

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_boolean(&mut self, value: bool) -> &mut Self {
        self.inner.put_u8(if value { 1 } else { 0 });
        self
    }
}

pub trait ServerGamePacketListener {
    fn handle_jigsaw_generate(&mut self, packet: &ServerboundJigsawGeneratePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundJigsawGeneratePacket {
    pos: BlockPos,
    levels: i32,
    keep_jigsaws: bool,
}

impl ServerboundJigsawGeneratePacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket#<init>(BlockPos, int, boolean)
    pub fn new(pos: BlockPos, levels: i32, keep_jigsaws: bool) -> Self {
        Self {
            pos,
            levels,
            keep_jigsaws,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut FriendlyByteBuf<B>) -> Self {
        let pos = buf.read_block_pos();
        let levels = buf.read_var_int();
        let keep_jigsaws = buf.read_boolean();
        Self {
            pos,
            levels,
            keep_jigsaws,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket#write
    pub fn write<B: BufMut>(&self, buf: &mut FriendlyByteBuf<B>) {
        buf.write_block_pos(self.pos);
        buf.write_var_int(self.levels);
        buf.write_boolean(self.keep_jigsaws);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_jigsaw_generate(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket#levels
    pub fn levels(&self) -> i32 {
        self.levels
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket#keepJigsaws
    pub fn keep_jigsaws(&self) -> bool {
        self.keep_jigsaws
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener(Option<(BlockPos, i32, bool)>);
    impl ServerGamePacketListener for MockListener {
        fn handle_jigsaw_generate(&mut self, packet: &ServerboundJigsawGeneratePacket) {
            self.0 = Some((packet.get_pos(), packet.levels(), packet.keep_jigsaws()));
        }
    }

    #[test]
    fn round_trip() {
        let pkt = ServerboundJigsawGeneratePacket::new(BlockPos { x: 7, y: 8, z: 9 }, 3, true);
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ServerboundJigsawGeneratePacket::read(&mut read_buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn keep_jigsaws_false_and_negative_levels() {
        let pkt = ServerboundJigsawGeneratePacket::new(
            BlockPos { x: -1, y: -2048, z: 0 },
            -5,
            false,
        );
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ServerboundJigsawGeneratePacket::read(&mut read_buf);
        assert_eq!(decoded.levels(), -5);
        assert!(!decoded.keep_jigsaws());
        assert_eq!(decoded.get_pos(), BlockPos { x: -1, y: -2048, z: 0 });
    }

    #[test]
    fn varint_extremes_for_levels() {
        for levels in [i32::MIN, i32::MAX, 0, 1, -1] {
            let pkt = ServerboundJigsawGeneratePacket::new(BlockPos { x: 0, y: 0, z: 0 }, levels, true);
            let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
            pkt.write(&mut buf);
            let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
            let decoded = ServerboundJigsawGeneratePacket::read(&mut read_buf);
            assert_eq!(decoded.levels(), levels);
        }
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ServerboundJigsawGeneratePacket::new(BlockPos { x: 1, y: 2, z: 3 }, 4, true);
        let mut listener = MockListener(None);
        pkt.handle(&mut listener);
        assert_eq!(listener.0, Some((BlockPos { x: 1, y: 2, z: 3 }, 4, true)));
    }
}
