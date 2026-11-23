use bytes::{Buf, BufMut};

fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut val: i32 = 0;
    for i in 0..5 {
        let b = buf.get_u8() as i32;
        val |= (b & 0x7F) << (i * 7);
        if (b & 0x80) == 0 {
            return val;
        }
    }
    val
}

fn write_var_int(buf: &mut impl BufMut, value: i32) {
    let mut v = value as u32;
    loop {
        if (v & !0x7F) == 0 {
            buf.put_u8(v as u8);
            return;
        }
        buf.put_u8(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    const X_OFFSET: i32 = Self::PACKED_Y_LENGTH + Self::PACKED_Z_LENGTH;

    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(&self) -> i64 {
        let mut l: i64 = 0;
        l |= ((self.x as i64) & Self::PACKED_X_MASK) << Self::X_OFFSET;
        l |= ((self.z as i64) & Self::PACKED_Z_MASK) << Self::Z_OFFSET;
        l | ((self.y as i64) & Self::PACKED_Y_MASK)
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn from_long(packed: i64) -> Self {
        let x = (packed << (64 - Self::X_OFFSET - Self::PACKED_X_LENGTH)
            >> (64 - Self::PACKED_X_LENGTH)) as i32;
        let y = (packed << (64 - Self::PACKED_Y_LENGTH) >> (64 - Self::PACKED_Y_LENGTH)) as i32;
        let z = (packed << (64 - Self::Z_OFFSET - Self::PACKED_Z_LENGTH)
            >> (64 - Self::PACKED_Z_LENGTH)) as i32;
        Self { x, y, z }
    }
}

pub trait ServerGamePacketListener {
    fn handle_block_entity_tag_query(&mut self, packet: &ServerboundBlockEntityTagQueryPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundBlockEntityTagQueryPacket {
    transaction_id: i32,
    pos: BlockPos,
}

impl ServerboundBlockEntityTagQueryPacket {
    pub const PACKET_TYPE: &'static str = "minecraft:block_entity_tag_query";

    // JAVA: net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket#<init>(int,BlockPos)
    pub fn new(transaction_id: i32, pos: BlockPos) -> Self {
        Self { transaction_id, pos }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let transaction_id = read_var_int(buf);
        let pos = BlockPos::from_long(buf.get_i64());
        Self { transaction_id, pos }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.transaction_id);
        buf.put_i64(self.pos.as_long());
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket#type
    pub fn packet_type(&self) -> &'static str {
        Self::PACKET_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket#handle
    pub fn handle(&self, listener: &mut impl ServerGamePacketListener) {
        listener.handle_block_entity_tag_query(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket#getTransactionId
    pub fn get_transaction_id(&self) -> i32 {
        self.transaction_id
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_basic() {
        let pkt = ServerboundBlockEntityTagQueryPacket::new(42, BlockPos { x: 1, y: 64, z: -2 });
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundBlockEntityTagQueryPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_transaction_id(), 42);
        assert_eq!(decoded.get_pos(), BlockPos { x: 1, y: 64, z: -2 });
    }

    #[test]
    fn roundtrip_negative_and_extreme() {
        let pkt = ServerboundBlockEntityTagQueryPacket::new(
            -1,
            BlockPos { x: -33_554_432, y: -2_048, z: 33_554_431 },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundBlockEntityTagQueryPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn var_int_encoding_size() {
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, 0);
        assert_eq!(buf.len(), 1);
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, 128);
        assert_eq!(buf.len(), 2);
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, -1);
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn packet_type_and_handle() {
        struct Listener {
            tx: i32,
        }
        impl ServerGamePacketListener for Listener {
            fn handle_block_entity_tag_query(&mut self, packet: &ServerboundBlockEntityTagQueryPacket) {
                self.tx = packet.get_transaction_id();
            }
        }
        let pkt = ServerboundBlockEntityTagQueryPacket::new(7, BlockPos { x: 0, y: 0, z: 0 });
        assert_eq!(pkt.packet_type(), "minecraft:block_entity_tag_query");
        let mut l = Listener { tx: 0 };
        pkt.handle(&mut l);
        assert_eq!(l.tx, 7);
    }
}
