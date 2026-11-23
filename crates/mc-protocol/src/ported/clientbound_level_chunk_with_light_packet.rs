//! Port of `net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket`.

use std::fmt;
use std::io;

/// Minimal decode error for self-contained packet codecs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl DecodeError {
    fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecodeError {}

impl From<io::Error> for DecodeError {
    fn from(e: io::Error) -> Self {
        Self::new(e.to_string())
    }
}

fn read_byte(buf: &mut &[u8]) -> Result<u8, DecodeError> {
    if buf.is_empty() {
        return Err(DecodeError::new("EOF reading byte"));
    }
    let b = buf[0];
    *buf = &buf[1..];
    Ok(b)
}

fn read_i32(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    if buf.len() < 4 {
        return Err(DecodeError::new("EOF reading int"));
    }
    let bytes = [buf[0], buf[1], buf[2], buf[3]];
    *buf = &buf[4..];
    Ok(i32::from_be_bytes(bytes))
}

fn write_i32(buf: &mut Vec<u8>, value: i32) {
    buf.extend_from_slice(&value.to_be_bytes());
}

/// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt()
fn read_varint(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    loop {
        let b = read_byte(buf)? as i8;
        i |= ((b & 127) as i32) << (j.wrapping_mul(7));
        j = j.wrapping_add(1);
        if j > 5 {
            return Err(DecodeError::new("VarInt too big"));
        }
        if (b & -128) != -128 {
            break;
        }
    }
    Ok(i)
}

/// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt(int)
fn write_varint(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut b = (value & 0x7F) as u8;
        let u = value as u32;
        value = (u >> 7) as i32;
        if value != 0 {
            b |= 0x80;
        }
        buf.push(b);
        if value == 0 {
            break;
        }
    }
}

fn read_long(buf: &mut &[u8]) -> Result<i64, DecodeError> {
    if buf.len() < 8 {
        return Err(DecodeError::new("EOF reading long"));
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buf[..8]);
    *buf = &buf[8..];
    Ok(i64::from_be_bytes(bytes))
}

fn write_long(buf: &mut Vec<u8>, value: i64) {
    buf.extend_from_slice(&value.to_be_bytes());
}

fn read_byte_array(buf: &mut &[u8]) -> Result<Vec<u8>, DecodeError> {
    let len = read_varint(buf)?;
    if len < 0 {
        return Err(DecodeError::new("Negative byte array length"));
    }
    let len = len as usize;
    if buf.len() < len {
        return Err(DecodeError::new("EOF reading byte array"));
    }
    let out = buf[..len].to_vec();
    *buf = &buf[len..];
    Ok(out)
}

fn write_byte_array(buf: &mut Vec<u8>, bytes: &[u8]) {
    write_varint(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

/// JAVA: net.minecraft.world.level.ChunkPos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

/// Minimal BitSet codec matching Java BitSet.valueOf/toLongArray wire used by FriendlyByteBuf.
/// JAVA: net.minecraft.network.FriendlyByteBuf#readBitSet() / writeBitSet(BitSet)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BitSet {
    pub words: Vec<i64>,
}

impl BitSet {
    pub fn new(words: Vec<i64>) -> Self {
        Self { words }
    }

    /// JAVA: net.minecraft.network.FriendlyByteBuf#readBitSet()
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let len = read_varint(buf)?;
        if len < 0 {
            return Err(DecodeError::new("Negative BitSet length"));
        }
        let mut words = Vec::with_capacity(len as usize);
        for _ in 0..len {
            words.push(read_long(buf)?);
        }
        Ok(Self { words })
    }

    /// JAVA: net.minecraft.network.FriendlyByteBuf#writeBitSet(BitSet)
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_varint(buf, self.words.len() as i32);
        for &w in &self.words {
            write_long(buf, w);
        }
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData
/// Self-contained representation preserving wire payload bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLevelChunkPacketData {
    pub heightmaps: Vec<u8>,
    pub buffer: Vec<u8>,
    pub block_entities_data: Vec<BlockEntityInfo>,
}

impl ClientboundLevelChunkPacketData {
    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData#<init>(net.minecraft.network.RegistryFriendlyByteBuf, int, int)
    pub fn read(buf: &mut &[u8], _chunk_x: i32, _chunk_z: i32) -> Result<Self, DecodeError> {
        let heightmaps = read_byte_array(buf)?;
        let buffer = read_byte_array(buf)?;
        let block_entities_count = read_varint(buf)?;
        if block_entities_count < 0 {
            return Err(DecodeError::new("Negative block entity count"));
        }
        let mut block_entities_data = Vec::with_capacity(block_entities_count as usize);
        for _ in 0..block_entities_count {
            block_entities_data.push(BlockEntityInfo::read(buf)?);
        }
        Ok(Self {
            heightmaps,
            buffer,
            block_entities_data,
        })
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData#write(net.minecraft.network.RegistryFriendlyByteBuf)
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_byte_array(buf, &self.heightmaps);
        write_byte_array(buf, &self.buffer);
        write_varint(buf, self.block_entities_data.len() as i32);
        for be in &self.block_entities_data {
            be.write(buf);
        }
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData.BlockEntityInfo
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityInfo {
    pub packed_xz: u8,
    pub y: i16,
    pub type_id: i32,
    pub tag: Option<Vec<u8>>,
}

impl BlockEntityInfo {
    fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let packed_xz = read_byte(buf)?;
        if buf.len() < 2 {
            return Err(DecodeError::new("EOF reading block entity y"));
        }
        let y = i16::from_be_bytes([buf[0], buf[1]]);
        *buf = &buf[2..];
        let type_id = read_varint(buf)?;
        let tag = read_optional_nbt_bytes(buf)?;
        Ok(Self {
            packed_xz,
            y,
            type_id,
            tag,
        })
    }

    fn write(&self, buf: &mut Vec<u8>) {
        buf.push(self.packed_xz);
        buf.extend_from_slice(&self.y.to_be_bytes());
        write_varint(buf, self.type_id);
        write_optional_nbt_bytes(buf, self.tag.as_deref());
    }
}

fn read_optional_nbt_bytes(buf: &mut &[u8]) -> Result<Option<Vec<u8>>, DecodeError> {
    let tag_type = read_byte(buf)?;
    if tag_type == 0x00 {
        return Ok(None);
    }
    if buf.len() < 2 {
        return Err(DecodeError::new("EOF reading NBT name length"));
    }
    let name_len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
    *buf = &buf[2..];
    if buf.len() < name_len {
        return Err(DecodeError::new("EOF reading NBT name"));
    }
    let mut raw = Vec::with_capacity(1 + 2 + name_len);
    raw.push(tag_type);
    raw.extend_from_slice(&(name_len as u16).to_be_bytes());
    raw.extend_from_slice(&buf[..name_len]);
    *buf = &buf[name_len..];
    // For self-contained port, store the compound payload as opaque remaining bytes is unsafe.
    // Instead, only support end-tag/empty-compound style payload marker by reading until END in a
    // minimal way is non-trivial. We conservatively support tag_type==0 only for optional NBT.
    Err(DecodeError::new(
        "Non-empty optional NBT is not supported in this self-contained port; use None/empty tag",
    ))
}

fn write_optional_nbt_bytes(buf: &mut Vec<u8>, tag: Option<&[u8]>) {
    match tag {
        Some(bytes) if !bytes.is_empty() => buf.extend_from_slice(bytes),
        _ => buf.push(0x00),
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacketData
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLightUpdatePacketData {
    pub sky_y_mask: BitSet,
    pub block_y_mask: BitSet,
    pub empty_sky_y_mask: BitSet,
    pub empty_block_y_mask: BitSet,
    pub sky_updates: Vec<Vec<u8>>,
    pub block_updates: Vec<Vec<u8>>,
}

impl ClientboundLightUpdatePacketData {
    /// JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacketData#<init>(net.minecraft.network.FriendlyByteBuf, int, int)
    pub fn read(buf: &mut &[u8], _chunk_x: i32, _chunk_z: i32) -> Result<Self, DecodeError> {
        let sky_y_mask = BitSet::read(buf)?;
        let block_y_mask = BitSet::read(buf)?;
        let empty_sky_y_mask = BitSet::read(buf)?;
        let empty_block_y_mask = BitSet::read(buf)?;

        let sky_len = read_varint(buf)?;
        if sky_len < 0 {
            return Err(DecodeError::new("Negative sky updates length"));
        }
        let mut sky_updates = Vec::with_capacity(sky_len as usize);
        for _ in 0..sky_len {
            sky_updates.push(read_byte_array(buf)?);
        }

        let block_len = read_varint(buf)?;
        if block_len < 0 {
            return Err(DecodeError::new("Negative block updates length"));
        }
        let mut block_updates = Vec::with_capacity(block_len as usize);
        for _ in 0..block_len {
            block_updates.push(read_byte_array(buf)?);
        }

        Ok(Self {
            sky_y_mask,
            block_y_mask,
            empty_sky_y_mask,
            empty_block_y_mask,
            sky_updates,
            block_updates,
        })
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLightUpdatePacketData#write(net.minecraft.network.FriendlyByteBuf)
    pub fn write(&self, buf: &mut Vec<u8>) {
        self.sky_y_mask.write(buf);
        self.block_y_mask.write(buf);
        self.empty_sky_y_mask.write(buf);
        self.empty_block_y_mask.write(buf);

        write_varint(buf, self.sky_updates.len() as i32);
        for arr in &self.sky_updates {
            write_byte_array(buf, arr);
        }
        write_varint(buf, self.block_updates.len() as i32);
        for arr in &self.block_updates {
            write_byte_array(buf, arr);
        }
    }
}

/// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLevelChunkWithLightPacket {
    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#x
    pub x: i32,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#z
    pub z: i32,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#chunkData
    pub chunk_data: ClientboundLevelChunkPacketData,
    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#lightData
    pub light_data: ClientboundLightUpdatePacketData,
}

impl ClientboundLevelChunkWithLightPacket {
    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#<init>(net.minecraft.world.level.chunk.LevelChunk, net.minecraft.world.level.lighting.LevelLightEngine, java.util.BitSet, java.util.BitSet)
    pub fn new(
        x: i32,
        z: i32,
        chunk_data: ClientboundLevelChunkPacketData,
        light_data: ClientboundLightUpdatePacketData,
    ) -> Self {
        Self {
            x,
            z,
            chunk_data,
            light_data,
        }
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#<init>(net.minecraft.network.RegistryFriendlyByteBuf)
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let x = read_i32(buf)?;
        let z = read_i32(buf)?;
        let chunk_data = ClientboundLevelChunkPacketData::read(buf, x, z)?;
        let light_data = ClientboundLightUpdatePacketData::read(buf, x, z)?;
        Ok(Self {
            x,
            z,
            chunk_data,
            light_data,
        })
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#write(net.minecraft.network.RegistryFriendlyByteBuf)
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_i32(buf, self.x);
        write_i32(buf, self.z);
        self.chunk_data.write(buf);
        self.light_data.write(buf);
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#getX()
    pub fn get_x(&self) -> i32 {
        self.x
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#getZ()
    pub fn get_z(&self) -> i32 {
        self.z
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#getChunkData()
    pub fn get_chunk_data(&self) -> &ClientboundLevelChunkPacketData {
        &self.chunk_data
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#getLightData()
    pub fn get_light_data(&self) -> &ClientboundLightUpdatePacketData {
        &self.light_data
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#type()
    pub fn type_id(&self) -> &'static str {
        "minecraft:level_chunk_with_light"
    }

    /// JAVA: net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket#handle(net.minecraft.network.protocol.game.ClientGamePacketListener)
    pub fn handle<H: FnOnce(&Self)>(&self, handler: H) {
        handler(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packet() -> ClientboundLevelChunkWithLightPacket {
        ClientboundLevelChunkWithLightPacket::new(
            3,
            -5,
            ClientboundLevelChunkPacketData {
                heightmaps: vec![0x01, 0x02],
                buffer: vec![0xAA, 0xBB, 0xCC],
                block_entities_data: vec![BlockEntityInfo {
                    packed_xz: 0x12,
                    y: -64,
                    type_id: 7,
                    tag: None,
                }],
            },
            ClientboundLightUpdatePacketData {
                sky_y_mask: BitSet::new(vec![1, -1]),
                block_y_mask: BitSet::new(vec![]),
                empty_sky_y_mask: BitSet::new(vec![0]),
                empty_block_y_mask: BitSet::new(vec![i64::MIN]),
                sky_updates: vec![vec![0xFF; 2048]],
                block_updates: vec![],
            },
        )
    }

    #[test]
    fn roundtrip_chunk_with_light_packet() {
        let packet = sample_packet();
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundLevelChunkWithLightPacket::read(&mut slice).expect("read");
        assert!(slice.is_empty());
        assert_eq!(decoded, packet);
        assert_eq!(decoded.get_x(), 3);
        assert_eq!(decoded.get_z(), -5);
    }

    #[test]
    fn roundtrip_empty_masks_and_updates() {
        let packet = ClientboundLevelChunkWithLightPacket::new(
            0,
            0,
            ClientboundLevelChunkPacketData {
                heightmaps: vec![],
                buffer: vec![],
                block_entities_data: vec![],
            },
            ClientboundLightUpdatePacketData {
                sky_y_mask: BitSet::new(vec![]),
                block_y_mask: BitSet::new(vec![]),
                empty_sky_y_mask: BitSet::new(vec![]),
                empty_block_y_mask: BitSet::new(vec![]),
                sky_updates: vec![],
                block_updates: vec![],
            },
        );
        let mut buf = Vec::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundLevelChunkWithLightPacket::read(&mut slice).expect("read");
        assert_eq!(decoded, packet);
    }

    #[test]
    fn wire_starts_with_chunk_coords_big_endian() {
        let packet = sample_packet();
        let mut buf = Vec::new();
        packet.write(&mut buf);
        assert_eq!(&buf[0..4], &3i32.to_be_bytes());
        assert_eq!(&buf[4..8], &(-5i32).to_be_bytes());
    }

    #[test]
    fn bitset_roundtrip_negative_words() {
        let bs = BitSet::new(vec![-1i64, i64::MIN, i64::MAX, 0]);
        let mut buf = Vec::new();
        bs.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = BitSet::read(&mut slice).expect("read bitset");
        assert!(slice.is_empty());
        assert_eq!(decoded, bs);
    }

    #[test]
    fn decode_error_on_truncated_input() {
        let mut slice: &[u8] = &[0x00, 0x00, 0x00];
        let err = ClientboundLevelChunkWithLightPacket::read(&mut slice).unwrap_err();
        assert!(err.message.contains("EOF"));
    }
}
