use bytes::{BufMut, BytesMut};
use std::fmt;

const FLAG_IGNORE_ENTITIES: i32 = 1;
const FLAG_SHOW_AIR: i32 = 2;
const FLAG_SHOW_BOUNDING_BOX: i32 = 4;
const FLAG_STRICT: i32 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketError {
    Eof,
    VarTooLong,
    InvalidEnum(i32),
    InvalidUtf,
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eof => write!(f, "unexpected end of buffer"),
            Self::VarTooLong => write!(f, "varint/varlong too long"),
            Self::InvalidEnum(ord) => write!(f, "invalid enum ordinal: {ord}"),
            Self::InvalidUtf => write!(f, "invalid utf-8 string"),
        }
    }
}

impl std::error::Error for PacketError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType {
    pub id: &'static str,
}

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const SERVERBOUND_SET_STRUCTURE_BLOCK: PacketType = PacketType {
        id: "serverbound_set_structure_block",
    };
}

pub trait ProtocolEnum: Sized + Copy {
    fn ordinal(self) -> i32;
    fn from_ordinal(ord: i32) -> Result<Self, PacketError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdateType {
    UpdateData,
    SaveArea,
    LoadArea,
    ScanArea,
}

impl ProtocolEnum for UpdateType {
    fn ordinal(self) -> i32 {
        match self {
            Self::UpdateData => 0,
            Self::SaveArea => 1,
            Self::LoadArea => 2,
            Self::ScanArea => 3,
        }
    }

    fn from_ordinal(ord: i32) -> Result<Self, PacketError> {
        match ord {
            0 => Ok(Self::UpdateData),
            1 => Ok(Self::SaveArea),
            2 => Ok(Self::LoadArea),
            3 => Ok(Self::ScanArea),
            _ => Err(PacketError::InvalidEnum(ord)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructureMode {
    Save,
    Load,
    Corner,
    Data,
}

impl ProtocolEnum for StructureMode {
    fn ordinal(self) -> i32 {
        match self {
            Self::Save => 0,
            Self::Load => 1,
            Self::Corner => 2,
            Self::Data => 3,
        }
    }

    fn from_ordinal(ord: i32) -> Result<Self, PacketError> {
        match ord {
            0 => Ok(Self::Save),
            1 => Ok(Self::Load),
            2 => Ok(Self::Corner),
            3 => Ok(Self::Data),
            _ => Err(PacketError::InvalidEnum(ord)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mirror {
    None,
    LeftRight,
    FrontBack,
}

impl ProtocolEnum for Mirror {
    fn ordinal(self) -> i32 {
        match self {
            Self::None => 0,
            Self::LeftRight => 1,
            Self::FrontBack => 2,
        }
    }

    fn from_ordinal(ord: i32) -> Result<Self, PacketError> {
        match ord {
            0 => Ok(Self::None),
            1 => Ok(Self::LeftRight),
            2 => Ok(Self::FrontBack),
            _ => Err(PacketError::InvalidEnum(ord)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl ProtocolEnum for Rotation {
    fn ordinal(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Clockwise90 => 1,
            Self::Clockwise180 => 2,
            Self::Counterclockwise90 => 3,
        }
    }

    fn from_ordinal(ord: i32) -> Result<Self, PacketError> {
        match ord {
            0 => Ok(Self::None),
            1 => Ok(Self::Clockwise90),
            2 => Ok(Self::Clockwise180),
            3 => Ok(Self::Counterclockwise90),
            _ => Err(PacketError::InvalidEnum(ord)),
        }
    }
}

// JAVA: net.minecraft.util.Mth#clamp
pub fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

// JAVA: net.minecraft.util.Mth#clamp
pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: BytesMut,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: impl Into<BytesMut>) -> Self {
        Self {
            data: data.into(),
            read_pos: 0,
        }
    }

    pub fn into_inner(self) -> BytesMut {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }

    fn ensure_read(&self, n: usize) -> Result<(), PacketError> {
        if self.remaining() < n {
            Err(PacketError::Eof)
        } else {
            Ok(())
        }
    }

    pub fn read_byte(&mut self) -> Result<i8, PacketError> {
        self.ensure_read(1)?;
        let b = self.data[self.read_pos] as i8;
        self.read_pos += 1;
        Ok(b)
    }

    pub fn write_byte(&mut self, value: i8) {
        self.data.put_i8(value);
    }

    pub fn read_long(&mut self) -> Result<i64, PacketError> {
        self.ensure_read(8)?;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 8]);
        self.read_pos += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn write_long(&mut self, value: i64) {
        self.data.put_i64(value);
    }

    pub fn read_float(&mut self) -> Result<f32, PacketError> {
        self.ensure_read(4)?;
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 4]);
        self.read_pos += 4;
        Ok(f32::from_be_bytes(bytes))
    }

    pub fn write_float(&mut self, value: f32) {
        self.data.put_f32(value);
    }

    pub fn read_var_int(&mut self) -> Result<i32, PacketError> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;
        loop {
            let b = self.read_byte()? as u8;
            value |= ((b & 0x7F) as i32) << position;
            if (b & 0x80) == 0 {
                break;
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(PacketError::VarTooLong);
            }
        }
        Ok(value)
    }

    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value as u32 & !0x7Fu32) == 0 {
                self.write_byte(value as i8);
                return;
            }
            self.write_byte(((value as u8 & 0x7F) | 0x80) as i8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    pub fn read_var_long(&mut self) -> Result<i64, PacketError> {
        let mut value: i64 = 0;
        let mut position: i32 = 0;
        loop {
            let b = self.read_byte()? as u8;
            value |= ((b & 0x7F) as i64) << position;
            if (b & 0x80) == 0 {
                break;
            }
            position = position.wrapping_add(7);
            if position >= 64 {
                return Err(PacketError::VarTooLong);
            }
        }
        Ok(value)
    }

    pub fn write_var_long(&mut self, mut value: i64) {
        loop {
            if (value as u64 & !0x7Fu64) == 0 {
                self.write_byte(value as i8);
                return;
            }
            self.write_byte(((value as u8 & 0x7F) | 0x80) as i8);
            value = ((value as u64) >> 7) as i64;
        }
    }

    pub fn read_block_pos(&mut self) -> Result<BlockPos, PacketError> {
        let v = self.read_long()?;
        let x = (v >> 38) as i32;
        let y = (v << 52 >> 52) as i32;
        let z = (v << 26 >> 38) as i32;
        Ok(BlockPos { x, y, z })
    }

    pub fn write_block_pos(&mut self, pos: BlockPos) {
        let v = (((pos.x as i64) & 0x3FF_FFFF_i64) << 38)
            | (((pos.z as i64) & 0x3FF_FFFF_i64) << 12)
            | ((pos.y as i64) & 0xFFF_i64);
        self.write_long(v);
    }

    pub fn read_utf(&mut self) -> Result<String, PacketError> {
        self.read_utf_max(32767)
    }

    pub fn read_utf_max(&mut self, max_length: i32) -> Result<String, PacketError> {
        let max_bytes = (max_length as usize).saturating_mul(3);
        let byte_len = self.read_var_int()?;
        if byte_len < 0 || (byte_len as usize) > max_bytes {
            return Err(PacketError::InvalidUtf);
        }
        let byte_len = byte_len as usize;
        self.ensure_read(byte_len)?;
        let bytes = self.data[self.read_pos..self.read_pos + byte_len].to_vec();
        self.read_pos += byte_len;
        let s = String::from_utf8(bytes).map_err(|_| PacketError::InvalidUtf)?;
        if s.encode_utf16().count() > max_length as usize {
            return Err(PacketError::InvalidUtf);
        }
        Ok(s)
    }

    pub fn write_utf(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    pub fn read_enum<T: ProtocolEnum>(&mut self) -> Result<T, PacketError> {
        let ord = self.read_var_int()?;
        T::from_ordinal(ord)
    }

    pub fn write_enum<T: ProtocolEnum>(&mut self, value: T) {
        self.write_var_int(value.ordinal());
    }
}

pub trait ServerGamePacketListener {
    fn handle_set_structure_block(&mut self, packet: &ServerboundSetStructureBlockPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundSetStructureBlockPacket {
    pub pos: BlockPos,
    pub update_type: UpdateType,
    pub mode: StructureMode,
    pub name: String,
    pub offset: BlockPos,
    pub size: Vec3i,
    pub mirror: Mirror,
    pub rotation: Rotation,
    pub data: String,
    pub ignore_entities: bool,
    pub strict: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
    pub integrity: f32,
    pub seed: i64,
}

impl ServerboundSetStructureBlockPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#<init>
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pos: BlockPos,
        update_type: UpdateType,
        mode: StructureMode,
        name: String,
        offset: BlockPos,
        size: Vec3i,
        mirror: Mirror,
        rotation: Rotation,
        data: String,
        ignore_entities: bool,
        strict: bool,
        show_air: bool,
        show_bounding_box: bool,
        integrity: f32,
        seed: i64,
    ) -> Self {
        Self {
            pos,
            update_type,
            mode,
            name,
            offset,
            size,
            mirror,
            rotation,
            data,
            ignore_entities,
            strict,
            show_air,
            show_bounding_box,
            integrity,
            seed,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, PacketError> {
        let pos = buf.read_block_pos()?;
        let update_type = buf.read_enum::<UpdateType>()?;
        let mode = buf.read_enum::<StructureMode>()?;
        let name = buf.read_utf()?;
        let offset = BlockPos::new(
            clamp_i32(buf.read_byte()? as i32, -48, 48),
            clamp_i32(buf.read_byte()? as i32, -48, 48),
            clamp_i32(buf.read_byte()? as i32, -48, 48),
        );
        let size = Vec3i::new(
            clamp_i32(buf.read_byte()? as i32, 0, 48),
            clamp_i32(buf.read_byte()? as i32, 0, 48),
            clamp_i32(buf.read_byte()? as i32, 0, 48),
        );
        let mirror = buf.read_enum::<Mirror>()?;
        let rotation = buf.read_enum::<Rotation>()?;
        let data = buf.read_utf_max(128)?;
        let integrity = clamp_f32(buf.read_float()?, 0.0, 1.0);
        let seed = buf.read_var_long()?;
        let flags = buf.read_byte()? as i32;
        let ignore_entities = (flags & FLAG_IGNORE_ENTITIES) != 0;
        let strict = (flags & FLAG_STRICT) != 0;
        let show_air = (flags & FLAG_SHOW_AIR) != 0;
        let show_bounding_box = (flags & FLAG_SHOW_BOUNDING_BOX) != 0;
        Ok(Self {
            pos,
            update_type,
            mode,
            name,
            offset,
            size,
            mirror,
            rotation,
            data,
            ignore_entities,
            strict,
            show_air,
            show_bounding_box,
            integrity,
            seed,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(self.pos);
        buf.write_enum(self.update_type);
        buf.write_enum(self.mode);
        buf.write_utf(&self.name);
        buf.write_byte(self.offset.x as i8);
        buf.write_byte(self.offset.y as i8);
        buf.write_byte(self.offset.z as i8);
        buf.write_byte(self.size.x as i8);
        buf.write_byte(self.size.y as i8);
        buf.write_byte(self.size.z as i8);
        buf.write_enum(self.mirror);
        buf.write_enum(self.rotation);
        buf.write_utf(&self.data);
        buf.write_float(self.integrity);
        buf.write_var_long(self.seed);
        let mut flags: i32 = 0;
        if self.ignore_entities {
            flags |= FLAG_IGNORE_ENTITIES;
        }
        if self.show_air {
            flags |= FLAG_SHOW_AIR;
        }
        if self.show_bounding_box {
            flags |= FLAG_SHOW_BOUNDING_BOX;
        }
        if self.strict {
            flags |= FLAG_STRICT;
        }
        buf.write_byte(flags as i8);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#type
    pub fn get_type(&self) -> PacketType {
        GamePacketTypes::SERVERBOUND_SET_STRUCTURE_BLOCK
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_set_structure_block(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getUpdateType
    pub fn get_update_type(&self) -> UpdateType {
        self.update_type
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getMode
    pub fn get_mode(&self) -> StructureMode {
        self.mode
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getName
    pub fn get_name(&self) -> &str {
        &self.name
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getOffset
    pub fn get_offset(&self) -> BlockPos {
        self.offset
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getSize
    pub fn get_size(&self) -> Vec3i {
        self.size
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getMirror
    pub fn get_mirror(&self) -> Mirror {
        self.mirror
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getRotation
    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getData
    pub fn get_data(&self) -> &str {
        &self.data
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#isIgnoreEntities
    pub fn is_ignore_entities(&self) -> bool {
        self.ignore_entities
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#isStrict
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#isShowAir
    pub fn is_show_air(&self) -> bool {
        self.show_air
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#isShowBoundingBox
    pub fn is_show_bounding_box(&self) -> bool {
        self.show_bounding_box
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getIntegrity
    pub fn get_integrity(&self) -> f32 {
        self.integrity
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#getSeed
    pub fn get_seed(&self) -> i64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packet() -> ServerboundSetStructureBlockPacket {
        ServerboundSetStructureBlockPacket::new(
            BlockPos::new(1, 2, 3),
            UpdateType::SaveArea,
            StructureMode::Load,
            "test".to_string(),
            BlockPos::new(-10, 5, 48),
            Vec3i::new(0, 24, 48),
            Mirror::LeftRight,
            Rotation::Clockwise90,
            "meta".to_string(),
            true,
            false,
            true,
            false,
            0.5,
            12345,
        )
    }

    #[test]
    fn round_trip_write_read() {
        let pkt = sample_packet();
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ServerboundSetStructureBlockPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn read_clamps_offset_size_integrity_and_handles_negative_seed() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_block_pos(BlockPos::new(0, 0, 0));
        buf.write_enum(UpdateType::UpdateData);
        buf.write_enum(StructureMode::Save);
        buf.write_utf("n");
        buf.write_byte(-100);
        buf.write_byte(100);
        buf.write_byte(-48);
        buf.write_byte(-1);
        buf.write_byte(49);
        buf.write_byte(20);
        buf.write_enum(Mirror::None);
        buf.write_enum(Rotation::None);
        buf.write_utf("d");
        buf.write_float(2.5);
        buf.write_var_long(-1);
        buf.write_byte(0);

        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let pkt = ServerboundSetStructureBlockPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt.get_offset(), BlockPos::new(-48, 48, -48));
        assert_eq!(pkt.get_size(), Vec3i::new(0, 48, 20));
        assert_eq!(pkt.get_integrity().to_bits(), 1.0f32.to_bits());
        assert_eq!(pkt.get_seed(), -1);
    }

    #[test]
    fn float_bits_flags_and_extreme_seed() {
        let pkt = ServerboundSetStructureBlockPacket::new(
            BlockPos::new(-1, -2048, 100_000),
            UpdateType::ScanArea,
            StructureMode::Data,
            String::new(),
            BlockPos::new(0, 0, 0),
            Vec3i::new(1, 1, 1),
            Mirror::FrontBack,
            Rotation::Counterclockwise90,
            String::new(),
            true,
            true,
            true,
            true,
            f32::from_bits(0x3F00_0000),
            i64::MIN,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_inner();
        assert_eq!(*bytes.last().unwrap() as i8, 15);

        let mut read_buf = FriendlyByteBuf::from_bytes(bytes);
        let decoded = ServerboundSetStructureBlockPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_integrity().to_bits(), 0x3F00_0000);
        assert_eq!(decoded.get_seed(), i64::MIN);
        assert!(decoded.is_ignore_entities());
        assert!(decoded.is_strict());
        assert!(decoded.is_show_air());
        assert!(decoded.is_show_bounding_box());
    }

    #[test]
    fn block_pos_round_trip_edge_coordinates() {
        let positions = [
            BlockPos::new(0, 0, 0),
            BlockPos::new(-1, -1, -1),
            BlockPos::new(33_554_431, 2047, 33_554_431),
            BlockPos::new(-33_554_432, -2048, -33_554_432),
        ];
        let mut buf = FriendlyByteBuf::new();
        for &p in &positions {
            buf.write_block_pos(p);
        }
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        for &p in &positions {
            assert_eq!(read_buf.read_block_pos().unwrap(), p);
        }
    }

    #[test]
    fn flag_decoding_mixed_bits() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_block_pos(BlockPos::new(7, 8, 9));
        buf.write_enum(UpdateType::LoadArea);
        buf.write_enum(StructureMode::Corner);
        buf.write_utf("");
        buf.write_byte(0);
        buf.write_byte(0);
        buf.write_byte(0);
        buf.write_byte(1);
        buf.write_byte(2);
        buf.write_byte(3);
        buf.write_enum(Mirror::None);
        buf.write_enum(Rotation::Clockwise180);
        buf.write_utf("");
        buf.write_float(-1.0);
        buf.write_var_long(0);
        buf.write_byte(0b0101);

        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let pkt = ServerboundSetStructureBlockPacket::read(&mut read_buf).unwrap();
        assert!(pkt.is_ignore_entities());
        assert!(!pkt.is_show_air());
        assert!(pkt.is_show_bounding_box());
        assert!(!pkt.is_strict());
        assert_eq!(pkt.get_integrity().to_bits(), 0.0f32.to_bits());
        assert_eq!(pkt.get_type(), GamePacketTypes::SERVERBOUND_SET_STRUCTURE_BLOCK);
    }
}
