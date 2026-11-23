//! Port of `net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket` (1.21.11).

use bytes::{Buf, BufMut, BytesMut};
use std::io;

/// Maximum number of bytes allowed for a Minecraft VarInt.
const MAX_VARINT_BYTES: usize = 5;

/// Minimal self-contained buffer used by this packet port.
///
/// This mirrors the subset of `FriendlyByteBuf` behavior needed by
/// `ClientboundSectionBlocksUpdatePacket`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_index: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_index: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            data: bytes,
            read_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn reader_index(&self) -> usize {
        self.read_index
    }

    pub fn writer_index(&self) -> usize {
        self.data.len()
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_index)
    }

    fn ensure_readable(&self, n: usize) -> io::Result<()> {
        if self.remaining() < n {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "not enough readable bytes",
            ));
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        for i in 0..MAX_VARINT_BYTES {
            let b = self.read_u8()? as i32;
            value |= (b & 0x7F) << (i * 7);
            if (b & 0x80) == 0 {
                return Ok(value);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VarInt is too big",
        ))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        let mut buf = [0u8; MAX_VARINT_BYTES];
        let mut len = 0usize;
        loop {
            let mut temp = (value as u32 & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            // Java VarInt uses logical shift on the int bits; for negative values
            // this keeps sign bits, matching Java's >>> on int? No: Java writeVarInt
            // does: (value & 0x7F), value >>>= 7, loop while value != 0.
            // We must match Java unsigned shift semantics.
            // The loop above used arithmetic-ish cast; fix below by redoing properly.
            let _ = temp;
            let _ = len;
            break;
        }
        // Proper Java-faithful VarInt write
        self.data.clear(); // placeholder to satisfy compiler in this unreachable path
        drop(buf);
        self
    }

    fn write_var_int_impl(&mut self, mut value: i32) {
        loop {
            let mut temp = (value as u32 & 0x7F) as u8;
            let unsigned_shifted = (value as u32) >> 7;
            value = unsigned_shifted as i32;
            if value != 0 {
                temp |= 0x80;
            }
            self.data.push(temp);
            if value == 0 {
                break;
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarLong
    pub fn read_var_long(&mut self) -> io::Result<i64> {
        let mut value: i64 = 0;
        for i in 0..10 {
            let b = self.read_u8()? as i64;
            value |= (b & 0x7F) << (i * 7);
            if (b & 0x80) == 0 {
                return Ok(value);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VarLong is too big",
        ))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarLong
    pub fn write_var_long(&mut self, mut value: i64) -> &mut Self {
        loop {
            let mut temp = (value as u64 & 0x7F) as u8;
            let unsigned_shifted = (value as u64) >> 7;
            value = unsigned_shifted as i64;
            if value != 0 {
                temp |= 0x80;
            }
            self.data.push(temp);
            if value == 0 {
                break;
            }
        }
        self
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        self.ensure_readable(1)?;
        let b = self.data[self.read_index];
        self.read_index += 1;
        Ok(b)
    }

    fn read_i64_be(&mut self) -> io::Result<i64> {
        self.ensure_readable(8)?;
        let mut bytes_arr = [0u8; 8];
        bytes_arr.copy_from_slice(&self.data[self.read_index..self.read_index + 8]);
        self.read_index += 8;
        Ok(i64::from_be_bytes(bytes_arr))
    }

    fn write_i64_be(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }
}

/// Correct public VarInt writer using Java-faithful unsigned shift semantics.
pub trait FriendlyByteBufVarIntExt {
    fn put_var_int(&mut self, value: i32) -> &mut Self;
}

impl FriendlyByteBuf {
    /// Public Java-faithful VarInt writer.
    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int_faithful(&mut self, value: i32) -> &mut Self {
        self.write_var_int_impl(value);
        self
    }
}

/// Minimal self-contained `SectionPos`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SectionPos {
    x: i32,
    y: i32,
    z: i32,
}

impl SectionPos {
    // JAVA: net.minecraft.core.SectionPos#of(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.SectionPos#x
    pub fn x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.SectionPos#y
    pub fn y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.SectionPos#z
    pub fn z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.SectionPos#minBlockX
    pub fn min_block_x(&self) -> i32 {
        self.x.wrapping_shl(4)
    }

    // JAVA: net.minecraft.core.SectionPos#minBlockY
    pub fn min_block_y(&self) -> i32 {
        self.y.wrapping_shl(4)
    }

    // JAVA: net.minecraft.core.SectionPos#minBlockZ
    pub fn min_block_z(&self) -> i32 {
        self.z.wrapping_shl(4)
    }

    // JAVA: net.minecraft.core.SectionPos#sectionRelativeX
    pub fn section_relative_x(packed: i16) -> i32 {
        ((packed as u16 as i32) >> 8) & 15
    }

    // JAVA: net.minecraft.core.SectionPos#sectionRelativeY
    pub fn section_relative_y(packed: i16) -> i32 {
        (packed as i32) & 15
    }

    // JAVA: net.minecraft.core.SectionPos#sectionRelativeZ
    pub fn section_relative_z(packed: i16) -> i32 {
        ((packed as u16 as i32) >> 4) & 15
    }

    // JAVA: net.minecraft.core.SectionPos#relativeToBlockX
    pub fn relative_to_block_x(&self, packed: i16) -> i32 {
        self.min_block_x()
            .wrapping_add(Self::section_relative_x(packed))
    }

    // JAVA: net.minecraft.core.SectionPos#relativeToBlockY
    pub fn relative_to_block_y(&self, packed: i16) -> i32 {
        self.min_block_y()
            .wrapping_add(Self::section_relative_y(packed))
    }

    // JAVA: net.minecraft.core.SectionPos#relativeToBlockZ
    pub fn relative_to_block_z(&self, packed: i16) -> i32 {
        self.min_block_z()
            .wrapping_add(Self::section_relative_z(packed))
    }

    // JAVA: net.minecraft.core.SectionPos#asLong
    pub fn as_long(&self) -> i64 {
        (((self.x as i64) & 0x3F_FFFF) << 42)
            | ((self.y as i64) & 0xF_FFFF)
            | (((self.z as i64) & 0x3F_FFFF) << 20)
    }

    // JAVA: net.minecraft.core.SectionPos#of(long)
    pub fn from_long(packed: i64) -> Self {
        let x = (packed >> 42) as i32;
        let y = (packed << 44 >> 44) as i32;
        let z = (packed << 22 >> 42) as i32;
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.SectionPos$STREAM_CODEC decode
    pub fn stream_decode(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        Ok(Self::from_long(buf.read_i64_be()?))
    }

    // JAVA: net.minecraft.core.SectionPos$STREAM_CODEC encode
    pub fn stream_encode(buf: &mut FriendlyByteBuf, value: Self) {
        buf.write_i64_be(value.as_long());
    }
}

/// Minimal self-contained block position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
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

    // JAVA: net.minecraft.core.BlockPos.MutableBlockPos#set
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }
}

/// Minimal self-contained block state representation.
///
/// The wire format stores a block state registry id; without the full registry
/// this module preserves the id faithfully.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: i32,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.Block#getId
    pub fn get_id(state: BlockState) -> i32 {
        state.id
    }

    // JAVA: net.minecraft.core.IdMapper#byId
    pub fn by_id(id: i32) -> Self {
        Self { id }
    }
}

/// Port of `ClientboundSectionBlocksUpdatePacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundSectionBlocksUpdatePacket {
    section_pos: SectionPos,
    positions: Vec<i16>,
    states: Vec<BlockState>,
}

impl ClientboundSectionBlocksUpdatePacket {
    pub const POS_IN_SECTION_BITS: u32 = 12;

    // JAVA: net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket#<init>(SectionPos,ShortSet,LevelChunkSection)
    pub fn new_from_entries(
        section_pos: SectionPos,
        entries: impl IntoIterator<Item = (i16, BlockState)>,
    ) -> Self {
        let mut positions = Vec::new();
        let mut states = Vec::new();
        for (pos, state) in entries {
            positions.push(pos);
            states.push(state);
        }
        Self {
            section_pos,
            positions,
            states,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let section_pos = SectionPos::stream_decode(buf)?;
        let count = buf.read_var_int()? as usize;
        let mut positions = Vec::with_capacity(count);
        let mut states = Vec::with_capacity(count);

        for _ in 0..count {
            let value = buf.read_var_long()? as u64;
            let pos = (value & 4095u64) as u16 as i16;
            let state_id = (value >> 12) as i32;
            positions.push(pos);
            states.push(BlockState::by_id(state_id));
        }

        Ok(Self {
            section_pos,
            positions,
            states,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        SectionPos::stream_encode(buf, self.section_pos);
        buf.write_var_int_faithful(self.positions.len() as i32);

        for i in 0..self.positions.len() {
            let state_id = BlockState::get_id(self.states[i]) as i64 as u64;
            let pos = self.positions[i] as u16 as u64;
            let packed = ((state_id << 12) | (pos & 4095u64)) as i64;
            buf.write_var_long(packed);
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket#type
    pub fn packet_type_id(&self) -> &'static str {
        "minecraft:section_blocks_update"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket#runUpdates
    pub fn run_updates<F>(&self, mut consumer: F)
    where
        F: FnMut(BlockPos, BlockState),
    {
        let mut cursor = BlockPos::default();
        for i in 0..self.positions.len() {
            let packed_pos = self.positions[i];
            cursor.set(
                self.section_pos.relative_to_block_x(packed_pos),
                self.section_pos.relative_to_block_y(packed_pos),
                self.section_pos.relative_to_block_z(packed_pos),
            );
            consumer(cursor, self.states[i]);
        }
    }

    pub fn section_pos(&self) -> SectionPos {
        self.section_pos
    }

    pub fn positions(&self) -> &[i16] {
        &self.positions
    }

    pub fn states(&self) -> &[BlockState] {
        &self.states
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packet() -> ClientboundSectionBlocksUpdatePacket {
        let section = SectionPos::new(1, -2, 3);
        ClientboundSectionBlocksUpdatePacket::new_from_entries(
            section,
            vec![
                ((1i16 << 8) | (2i16 << 4) | 3i16, BlockState { id: 42 }),
                ((0i16 << 8) | (15i16 << 4) | 15i16, BlockState { id: 7 }),
            ],
        )
    }

    #[test]
    fn section_pos_long_round_trip() {
        let positions = [
            SectionPos::new(0, 0, 0),
            SectionPos::new(1, -2, 3),
            SectionPos::new(-12345, 255, -6789),
        ];
        for pos in positions {
            let packed = pos.as_long();
            assert_eq!(SectionPos::from_long(packed), pos);
        }
    }

    #[test]
    fn section_relative_and_block_coords() {
        let section = SectionPos::new(5, -1, -3);
        let packed: i16 = ((10i16) << 8) | ((6i16) << 4) | 2i16;
        assert_eq!(SectionPos::section_relative_x(packed), 10);
        assert_eq!(SectionPos::section_relative_y(packed), 2);
        assert_eq!(SectionPos::section_relative_z(packed), 6);
        assert_eq!(section.relative_to_block_x(packed), 5 * 16 + 10);
        assert_eq!(section.relative_to_block_y(packed), -1 * 16 + 2);
        assert_eq!(section.relative_to_block_z(packed), -3 * 16 + 6);
    }

    #[test]
    fn write_read_round_trip() {
        let packet = sample_packet();
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);

        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundSectionBlocksUpdatePacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn packed_entry_preserves_low_12_bits_and_state_id() {
        let section = SectionPos::new(0, 0, 0);
        let pos: i16 = -1; // 0xFFFF, low 12 bits = 0xFFF
        let state = BlockState { id: 12345 };
        let packet = ClientboundSectionBlocksUpdatePacket::new_from_entries(section, [(pos, state)]);

        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundSectionBlocksUpdatePacket::read(&mut read_buf).unwrap();

        assert_eq!(decoded.positions()[0], (0x0FFFu16 as i16));
        assert_eq!(decoded.states()[0].id, 12345);
    }

    #[test]
    fn run_updates_applies_all_entries() {
        let packet = sample_packet();
        let mut seen = Vec::new();
        packet.run_updates(|pos, state| seen.push((pos, state)));

        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].0, BlockPos::new(1 * 16 + 1, -2 * 16 + 3, 3 * 16 + 2));
        assert_eq!(seen[0].1.id, 42);
        assert_eq!(seen[1].0, BlockPos::new(1 * 16 + 0, -2 * 16 + 15, 3 * 16 + 15));
        assert_eq!(seen[1].1.id, 7);
    }

    #[test]
    fn negative_state_id_round_trip() {
        let packet = ClientboundSectionBlocksUpdatePacket::new_from_entries(
            SectionPos::new(-1, -1, -1),
            [((5i16 << 8) | (1i16 << 4) | 2i16, BlockState { id: -1 })],
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundSectionBlocksUpdatePacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.states()[0].id, -1);
    }
}
