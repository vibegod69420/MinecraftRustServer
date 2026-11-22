use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(pub &'static str);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    const PACKED_X_LENGTH: u32 = 26;
    const PACKED_Z_LENGTH: u32 = 26;
    const PACKED_Y_LENGTH: u32 = 12;
    const PACKED_X_MASK: i64 = (1i64 << Self::PACKED_X_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1i64 << Self::PACKED_Z_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1i64 << Self::PACKED_Y_LENGTH) - 1;
    const Z_OFFSET: u32 = Self::PACKED_Y_LENGTH;
    const X_OFFSET: u32 = Self::PACKED_Y_LENGTH + Self::PACKED_Z_LENGTH;

    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(&self) -> i64 {
        let x = ((self.x as i64) & Self::PACKED_X_MASK) << Self::X_OFFSET;
        let y = (self.y as i64) & Self::PACKED_Y_MASK;
        let z = ((self.z as i64) & Self::PACKED_Z_MASK) << Self::Z_OFFSET;
        x | z | y
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn of(packed: i64) -> Self {
        let x = (packed << (64 - Self::X_OFFSET - Self::PACKED_X_LENGTH)
            >> (64 - Self::PACKED_X_LENGTH)) as i32;
        let y = (packed << (64 - Self::PACKED_Y_LENGTH) >> (64 - Self::PACKED_Y_LENGTH)) as i32;
        let z = (packed << (64 - Self::Z_OFFSET - Self::PACKED_Z_LENGTH)
            >> (64 - Self::PACKED_Z_LENGTH)) as i32;
        Self { x, y, z }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBlockPos
    pub fn read(buf: &mut impl Buf) -> Result<Self, DecodeError> {
        if buf.remaining() < 8 {
            return Err(DecodeError("Not enough bytes for BlockPos"));
        }
        Ok(Self::of(buf.get_i64()))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBlockPos
    pub fn write(&self, buf: &mut impl BufMut) {
        buf.put_i64(self.as_long());
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
pub fn read_bool(buf: &mut impl Buf) -> Result<bool, DecodeError> {
    if !buf.has_remaining() {
        return Err(DecodeError("Not enough bytes for boolean"));
    }
    Ok(buf.get_u8() != 0)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
pub fn write_bool(buf: &mut impl BufMut, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPickItemFromBlockPacket {
    pub pos: BlockPos,
    pub include_data: bool,
}

pub trait ServerGamePacketListener {
    fn handle_pick_item_from_block(&mut self, packet: &ServerboundPickItemFromBlockPacket);
}

impl ServerboundPickItemFromBlockPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromBlockPacket#<init>
    pub fn new(pos: BlockPos, include_data: bool) -> Self {
        Self { pos, include_data }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromBlockPacket#STREAM_CODEC
    pub fn read(buf: &mut impl Buf) -> Result<Self, DecodeError> {
        let pos = BlockPos::read(buf)?;
        let include_data = read_bool(buf)?;
        Ok(Self::new(pos, include_data))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromBlockPacket#STREAM_CODEC
    pub fn write(&self, buf: &mut impl BufMut) {
        self.pos.write(buf);
        write_bool(buf, self.include_data);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromBlockPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:pick_item_from_block")
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPickItemFromBlockPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_pick_item_from_block(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BytesMut};

    #[test]
    fn block_pos_negative_roundtrip() {
        let pos = BlockPos {
            x: -12_345,
            y: -64,
            z: 99_999,
        };
        let packed = pos.as_long();
        assert_eq!(BlockPos::of(packed), pos);
    }

    #[test]
    fn bool_roundtrip() {
        for v in [false, true] {
            let mut buf = BytesMut::new();
            write_bool(&mut buf, v);
            let mut slice = &buf[..];
            assert_eq!(read_bool(&mut slice).unwrap(), v);
            assert!(!slice.has_remaining());
        }
    }

    #[test]
    fn packet_roundtrip() {
        let packet = ServerboundPickItemFromBlockPacket::new(
            BlockPos { x: 100, y: 72, z: -200 },
            true,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundPickItemFromBlockPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn packet_roundtrip_extreme_y() {
        let packet = ServerboundPickItemFromBlockPacket::new(
            BlockPos { x: 0, y: -2048, z: 0 },
            false,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundPickItemFromBlockPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, packet);
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener(Option<(BlockPos, bool)>);
        impl ServerGamePacketListener for MockListener {
            fn handle_pick_item_from_block(
                &mut self,
                packet: &ServerboundPickItemFromBlockPacket,
            ) {
                self.0 = Some((packet.pos, packet.include_data));
            }
        }
        let packet =
            ServerboundPickItemFromBlockPacket::new(BlockPos { x: 1, y: 2, z: 3 }, true);
        let mut listener = MockListener(None);
        packet.handle(&mut listener);
        assert_eq!(listener.0, Some((packet.pos, true)));
        assert_eq!(
            packet.packet_type(),
            PacketType("minecraft:pick_item_from_block")
        );
    }
}
