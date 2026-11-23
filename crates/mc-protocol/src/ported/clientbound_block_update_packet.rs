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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub id: i32,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.Block#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.world.level.block.Block#stateById
    pub fn state_by_id(id: i32) -> Self {
        Self { id }
    }
}

pub trait BlockGetter {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
}

pub trait ClientGamePacketListener {
    fn handle_block_update(&mut self, packet: &ClientboundBlockUpdatePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundBlockUpdatePacket {
    pos: BlockPos,
    block_state: BlockState,
}

impl ClientboundBlockUpdatePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket#<init>(BlockPos, BlockState)
    pub fn new(pos: BlockPos, block_state: BlockState) -> Self {
        Self { pos, block_state }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket#<init>(BlockGetter, BlockPos)
    pub fn from_block_getter<G: BlockGetter>(getter: &G, pos: BlockPos) -> Self {
        Self::new(pos, getter.get_block_state(pos))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket#STREAM_CODEC
    pub fn read<B: Buf>(buf: &mut FriendlyByteBuf<B>) -> Self {
        let pos = buf.read_block_pos();
        let block_state = BlockState::state_by_id(buf.read_var_int());
        Self { pos, block_state }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket#STREAM_CODEC
    pub fn write<B: BufMut>(&self, buf: &mut FriendlyByteBuf<B>) {
        buf.write_block_pos(self.pos);
        buf.write_var_int(self.block_state.get_id());
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_block_update(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket#getBlockState
    pub fn get_block_state(&self) -> BlockState {
        self.block_state
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockGetter(BlockState);
    impl BlockGetter for MockGetter {
        fn get_block_state(&self, _pos: BlockPos) -> BlockState {
            self.0
        }
    }

    struct MockListener(Option<(BlockPos, i32)>);
    impl ClientGamePacketListener for MockListener {
        fn handle_block_update(&mut self, packet: &ClientboundBlockUpdatePacket) {
            self.0 = Some((packet.get_pos(), packet.get_block_state().get_id()));
        }
    }

    #[test]
    fn round_trip() {
        let pkt = ClientboundBlockUpdatePacket::new(
            BlockPos { x: 10, y: 64, z: -20 },
            BlockState { id: 42 },
        );
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ClientboundBlockUpdatePacket::read(&mut read_buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn from_block_getter() {
        let getter = MockGetter(BlockState { id: 7 });
        let pos = BlockPos { x: 1, y: 2, z: 3 };
        let pkt = ClientboundBlockUpdatePacket::from_block_getter(&getter, pos);
        assert_eq!(pkt.get_pos(), pos);
        assert_eq!(pkt.get_block_state().get_id(), 7);
    }

    #[test]
    fn negative_coords_and_state_id() {
        let pkt = ClientboundBlockUpdatePacket::new(
            BlockPos { x: -100, y: -2048, z: 33_554_431 },
            BlockState { id: -1 },
        );
        let mut buf = FriendlyByteBuf { inner: BytesMut::new() };
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf { inner: buf.inner.freeze() };
        let decoded = ClientboundBlockUpdatePacket::read(&mut read_buf);
        assert_eq!(decoded, pkt);
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundBlockUpdatePacket::new(
            BlockPos { x: 5, y: 6, z: 7 },
            BlockState { id: 99 },
        );
        let mut listener = MockListener(None);
        pkt.handle(&mut listener);
        assert_eq!(listener.0, Some((BlockPos { x: 5, y: 6, z: 7 }, 99)));
    }
}
