use bytes::{Buf, BufMut};

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

pub trait ClientGamePacketListener {
    fn handle_open_sign_editor(&mut self, packet: &ClientboundOpenSignEditorPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundOpenSignEditorPacket {
    pos: BlockPos,
    is_front_text: bool,
}

impl ClientboundOpenSignEditorPacket {
    pub const PACKET_TYPE: &'static str = "minecraft:open_sign_editor";

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket#<init>(BlockPos,boolean)
    pub fn new(pos: BlockPos, is_front_text: bool) -> Self {
        Self { pos, is_front_text }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let pos = BlockPos::from_long(buf.get_i64());
        let is_front_text = buf.get_u8() != 0;
        Self { pos, is_front_text }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        buf.put_i64(self.pos.as_long());
        buf.put_u8(if self.is_front_text { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket#type
    pub fn packet_type(&self) -> &'static str {
        Self::PACKET_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_open_sign_editor(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket#isFrontText
    pub fn is_front_text(&self) -> bool {
        self.is_front_text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn block_pos_roundtrip() {
        let cases = [
            (0, 0, 0),
            (1, 2, 3),
            (-1, -1, -1),
            (33_554_431, 2_047, 33_554_431),
            (-33_554_432, -2_048, -33_554_432),
        ];
        for (x, y, z) in cases {
            let p = BlockPos { x, y, z };
            assert_eq!(p, BlockPos::from_long(p.as_long()));
        }
    }

    #[test]
    fn block_pos_packed_minus_one() {
        let p = BlockPos::from_long(-1i64);
        assert_eq!(p, BlockPos { x: -1, y: -1, z: -1 });
        assert_eq!(p.as_long(), -1i64);
    }

    #[test]
    fn packet_roundtrip_front() {
        let pkt = ClientboundOpenSignEditorPacket::new(BlockPos { x: 10, y: 64, z: -20 }, true);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundOpenSignEditorPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
        assert!(decoded.is_front_text());
        assert_eq!(decoded.get_pos(), BlockPos { x: 10, y: 64, z: -20 });
    }

    #[test]
    fn packet_roundtrip_back() {
        let pkt = ClientboundOpenSignEditorPacket::new(BlockPos { x: -7, y: -3, z: 99 }, false);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundOpenSignEditorPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
        assert!(!decoded.is_front_text());
    }

    #[test]
    fn packet_type_and_handle() {
        struct Listener {
            called: bool,
            front: bool,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_open_sign_editor(&mut self, packet: &ClientboundOpenSignEditorPacket) {
                self.called = true;
                self.front = packet.is_front_text();
            }
        }
        let pkt = ClientboundOpenSignEditorPacket::new(BlockPos { x: 0, y: 0, z: 0 }, true);
        assert_eq!(pkt.packet_type(), "minecraft:open_sign_editor");
        let mut l = Listener { called: false, front: false };
        pkt.handle(&mut l);
        assert!(l.called);
        assert!(l.front);
    }
}
