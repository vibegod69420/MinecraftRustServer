use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    ClientboundBlockEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBlockPos
    pub fn read(buf: &mut impl Buf) -> Self {
        let v = buf.get_i64();
        Self {
            x: (v >> 38) as i32,
            y: (v << 52 >> 52) as i32,
            z: (v << 26 >> 38) as i32,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBlockPos
    pub fn write(&self, buf: &mut impl BufMut) {
        let v = (((self.x as i64) & 0x3FF_FFFF) << 38)
            | (((self.z as i64) & 0x3FF_FFFF) << 12)
            | ((self.y as i64) & 0xFFF);
        buf.put_i64(v);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub registry_id: i32,
}

impl Block {
    // JAVA: net.minecraft.world.level.block.Block#<init>
    pub fn new(registry_id: i32) -> Self {
        Self { registry_id }
    }

    // JAVA: net.minecraft.network.codec.ByteBufCodecs#registry decode
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            registry_id: read_var_int(buf),
        }
    }

    // JAVA: net.minecraft.network.codec.ByteBufCodecs#registry encode
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.registry_id);
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8() as u32;
        result |= (b & 0x7F) << shift;
        if (b & 0x80) == 0 {
            return result as i32;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return result as i32;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut impl BufMut, value: i32) {
    let mut val = value as u32;
    loop {
        if (val & !0x7F) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7F) as u8) | 0x80);
        val >>= 7;
    }
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleBlockEvent
    fn handle_block_event(&mut self, packet: &ClientboundBlockEventPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundBlockEventPacket {
    pos: BlockPos,
    b0: i32,
    b1: i32,
    block: Block,
}

impl ClientboundBlockEventPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#<init>(BlockPos,Block,int,int)
    pub fn new(pos: BlockPos, block: Block, b0: i32, b1: i32) -> Self {
        Self { pos, b0, b1, block }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let pos = BlockPos::read(buf);
        let b0 = buf.get_u8() as i32;
        let b1 = buf.get_u8() as i32;
        let block = Block::read(buf);
        Self { pos, b0, b1, block }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        self.pos.write(buf);
        buf.put_u8(self.b0 as u8);
        buf.put_u8(self.b1 as u8);
        self.block.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ClientboundBlockEvent
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_block_event(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#getB0
    pub fn get_b0(&self) -> i32 {
        self.b0
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#getB1
    pub fn get_b1(&self) -> i32 {
        self.b1
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundBlockEventPacket#getBlock
    pub fn get_block(&self) -> Block {
        self.block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener {
        called: bool,
        b0: i32,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_block_event(&mut self, packet: &ClientboundBlockEventPacket) {
            self.called = true;
            self.b0 = packet.get_b0();
        }
    }

    #[test]
    fn round_trip_basic() {
        let packet = ClientboundBlockEventPacket::new(
            BlockPos::new(10, 64, -20),
            Block::new(5),
            1,
            2,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundBlockEventPacket::read(&mut reader);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn block_pos_negative_and_edge_values() {
        let pos = BlockPos::new(-33_554_432, -2048, 33_554_431);
        let mut buf = BytesMut::new();
        pos.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = BlockPos::read(&mut reader);
        assert_eq!(decoded, pos);
    }

    #[test]
    fn unsigned_byte_fields_preserve_high_bits() {
        let packet = ClientboundBlockEventPacket::new(
            BlockPos::new(0, 0, 0),
            Block::new(1),
            255,
            128,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundBlockEventPacket::read(&mut reader);
        assert_eq!(decoded.get_b0(), 255);
        assert_eq!(decoded.get_b1(), 128);
    }

    #[test]
    fn handle_calls_listener() {
        let packet = ClientboundBlockEventPacket::new(
            BlockPos::new(1, 2, 3),
            Block::new(9),
            7,
            8,
        );
        let mut listener = TestListener { called: false, b0: 0 };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.b0, 7);
        assert_eq!(packet.packet_type(), PacketType::ClientboundBlockEvent);
    }
}
