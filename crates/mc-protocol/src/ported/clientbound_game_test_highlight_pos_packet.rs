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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundGameTestHighlightPosPacket {
    pub absolute_pos: BlockPos,
    pub relative_pos: BlockPos,
}

pub trait ClientGamePacketListener {
    fn handle_game_test_highlight_pos(&mut self, packet: &ClientboundGameTestHighlightPosPacket);
}

impl ClientboundGameTestHighlightPosPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundGameTestHighlightPosPacket#<init>
    pub fn new(absolute_pos: BlockPos, relative_pos: BlockPos) -> Self {
        Self {
            absolute_pos,
            relative_pos,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameTestHighlightPosPacket#STREAM_CODEC
    pub fn read(buf: &mut impl Buf) -> Result<Self, DecodeError> {
        let absolute_pos = BlockPos::read(buf)?;
        let relative_pos = BlockPos::read(buf)?;
        Ok(Self::new(absolute_pos, relative_pos))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameTestHighlightPosPacket#STREAM_CODEC
    pub fn write(&self, buf: &mut impl BufMut) {
        self.absolute_pos.write(buf);
        self.relative_pos.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameTestHighlightPosPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:game_test_highlight_pos")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundGameTestHighlightPosPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_game_test_highlight_pos(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BytesMut};

    #[test]
    fn block_pos_roundtrip_extremes() {
        let positions = [
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: -1, y: -1, z: -1 },
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
        ];
        for pos in positions {
            let packed = pos.as_long();
            assert_eq!(BlockPos::of(packed), pos);
        }
    }

    #[test]
    fn block_pos_negative_coordinates() {
        let pos = BlockPos {
            x: -123_456,
            y: -64,
            z: 987_654,
        };
        let mut buf = BytesMut::new();
        pos.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = BlockPos::read(&mut slice).unwrap();
        assert_eq!(decoded, pos);
        assert!(!slice.has_remaining());
    }

    #[test]
    fn packet_roundtrip() {
        let packet = ClientboundGameTestHighlightPosPacket::new(
            BlockPos { x: 10, y: 64, z: -20 },
            BlockPos { x: -3, y: 0, z: 7 },
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundGameTestHighlightPosPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, packet);
        assert!(!slice.has_remaining());
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener(Option<(BlockPos, BlockPos)>);
        impl ClientGamePacketListener for MockListener {
            fn handle_game_test_highlight_pos(
                &mut self,
                packet: &ClientboundGameTestHighlightPosPacket,
            ) {
                self.0 = Some((packet.absolute_pos, packet.relative_pos));
            }
        }
        let packet = ClientboundGameTestHighlightPosPacket::new(
            BlockPos { x: 1, y: 2, z: 3 },
            BlockPos { x: 4, y: 5, z: 6 },
        );
        let mut listener = MockListener(None);
        packet.handle(&mut listener);
        assert_eq!(listener.0, Some((packet.absolute_pos, packet.relative_pos)));
        assert_eq!(
            packet.packet_type(),
            PacketType("minecraft:game_test_highlight_pos")
        );
    }

    #[test]
    fn read_fails_on_short_buffer() {
        let mut slice = &[0u8, 1, 2][..];
        assert!(ClientboundGameTestHighlightPosPacket::read(&mut slice).is_err());
    }
}
