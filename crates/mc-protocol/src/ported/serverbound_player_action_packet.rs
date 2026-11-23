use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProtocolError {
    #[error("VarInt too big")]
    VarIntTooBig,
    #[error("buffer underflow: need {needed}, have {remaining}")]
    Underflow { needed: usize, remaining: usize },
    #[error("invalid enum ordinal: {0}")]
    InvalidEnumOrdinal(i32),
    #[error("invalid direction 3D data value: {0}")]
    InvalidDirectionDataValue(i32),
}

fn ensure_remaining<B: Buf>(buf: &B, needed: usize) -> Result<(), ProtocolError> {
    if buf.remaining() < needed {
        return Err(ProtocolError::Underflow {
            needed,
            remaining: buf.remaining(),
        });
    }
    Ok(())
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
pub fn read_var_int<B: Buf>(buf: &mut B) -> Result<i32, ProtocolError> {
    let mut result: i32 = 0;
    let mut j: u32 = 0;
    loop {
        ensure_remaining(buf, 1)?;
        let b0 = buf.get_u8() as i8;
        result |= ((b0 & 127) as i32) << (j.wrapping_mul(7));
        j = j.wrapping_add(1);
        if j > 5 {
            return Err(ProtocolError::VarIntTooBig);
        }
        if (b0 & -128i8) != -128i8 {
            break;
        }
    }
    Ok(result)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
pub fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value & -128) != 0 {
            buf.put_u8(((value & 127) | 128) as u8);
            value = ((value as u32) >> 7) as i32;
        } else {
            buf.put_u8(value as u8);
            return;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBlockPos
pub fn read_block_pos<B: Buf>(buf: &mut B) -> Result<BlockPos, ProtocolError> {
    ensure_remaining(buf, 8)?;
    let packed = buf.get_i64();
    Ok(BlockPos::of_long(packed))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBlockPos
pub fn write_block_pos<B: BufMut>(buf: &mut B, pos: BlockPos) {
    buf.put_i64(pos.as_long());
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readEnum
fn read_enum_action<B: Buf>(buf: &mut B) -> Result<ServerboundPlayerActionPacketAction, ProtocolError> {
    let ordinal = read_var_int(buf)?;
    ServerboundPlayerActionPacketAction::from_ordinal(ordinal)
        .ok_or(ProtocolError::InvalidEnumOrdinal(ordinal))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeEnum
fn write_enum_action<B: BufMut>(buf: &mut B, action: ServerboundPlayerActionPacketAction) {
    write_var_int(buf, action.ordinal());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    const PACKED_HORIZONTAL_LENGTH: i32 = 26;
    const PACKED_Y_LENGTH: i32 = 12;
    const PACKED_X_MASK: i64 = (1i64 << Self::PACKED_HORIZONTAL_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1i64 << Self::PACKED_Y_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1i64 << Self::PACKED_HORIZONTAL_LENGTH) - 1;
    const Z_OFFSET: i32 = Self::PACKED_Y_LENGTH;
    const X_OFFSET: i32 = Self::Z_OFFSET + Self::PACKED_HORIZONTAL_LENGTH;
    const TOTAL_BITS: i32 = 64;

    // JAVA: net.minecraft.core.BlockPos#BlockPos(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#immutable
    pub fn immutable(self) -> Self {
        self
    }

    // JAVA: net.minecraft.core.BlockPos#asLong
    pub fn as_long(self) -> i64 {
        let mut l: i64 = 0;
        l |= ((self.x as i64) & Self::PACKED_X_MASK) << Self::X_OFFSET;
        l |= ((self.z as i64) & Self::PACKED_Z_MASK) << Self::Z_OFFSET;
        l | ((self.y as i64) & Self::PACKED_Y_MASK)
    }

    // JAVA: net.minecraft.core.BlockPos#of
    pub fn of_long(packed_pos: i64) -> Self {
        let x = (packed_pos << (Self::TOTAL_BITS - Self::X_OFFSET - Self::PACKED_HORIZONTAL_LENGTH)
            >> (Self::TOTAL_BITS - Self::PACKED_HORIZONTAL_LENGTH)) as i32;
        let y = (packed_pos << (Self::TOTAL_BITS - Self::PACKED_Y_LENGTH)
            >> (Self::TOTAL_BITS - Self::PACKED_Y_LENGTH)) as i32;
        let z = (packed_pos << (Self::TOTAL_BITS - Self::Z_OFFSET - Self::PACKED_HORIZONTAL_LENGTH)
            >> (Self::TOTAL_BITS - Self::PACKED_HORIZONTAL_LENGTH)) as i32;
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.BlockPos#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.BlockPos#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    // JAVA: net.minecraft.core.Direction#get3DDataValue
    pub fn get_3d_data_value(self) -> i32 {
        match self {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
        }
    }

    // JAVA: net.minecraft.core.Direction#from3DDataValue
    pub fn from_3d_data_value(value: i32) -> Self {
        let by_3d_data = [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];
        let idx = mth_abs(value.wrapping_rem(by_3d_data.len() as i32)) as usize;
        by_3d_data[idx]
    }
}

// JAVA: net.minecraft.util.Mth#abs(int)
fn mth_abs(value: i32) -> i32 {
    if value >= 0 { value } else { value.wrapping_neg() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ServerboundPlayerActionPacketAction {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
    Stab,
}

impl ServerboundPlayerActionPacketAction {
    fn ordinal(self) -> i32 {
        match self {
            Self::StartDestroyBlock => 0,
            Self::AbortDestroyBlock => 1,
            Self::StopDestroyBlock => 2,
            Self::DropAllItems => 3,
            Self::DropItem => 4,
            Self::ReleaseUseItem => 5,
            Self::SwapItemWithOffhand => 6,
            Self::Stab => 7,
        }
    }

    fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::StartDestroyBlock),
            1 => Some(Self::AbortDestroyBlock),
            2 => Some(Self::StopDestroyBlock),
            3 => Some(Self::DropAllItems),
            4 => Some(Self::DropItem),
            5 => Some(Self::ReleaseUseItem),
            6 => Some(Self::SwapItemWithOffhand),
            7 => Some(Self::Stab),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundPlayerActionPacket {
    pos: BlockPos,
    direction: Direction,
    action: ServerboundPlayerActionPacketAction,
    sequence: i32,
}

impl ServerboundPlayerActionPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#<init>(ServerboundPlayerActionPacket.Action,BlockPos,Direction,int)
    pub fn new(
        action: ServerboundPlayerActionPacketAction,
        pos: BlockPos,
        direction: Direction,
        sequence: i32,
    ) -> Self {
        Self {
            action,
            pos: pos.immutable(),
            direction,
            sequence,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#<init>(ServerboundPlayerActionPacket.Action,BlockPos,Direction)
    pub fn new_without_sequence(
        action: ServerboundPlayerActionPacketAction,
        pos: BlockPos,
        direction: Direction,
    ) -> Self {
        Self::new(action, pos, direction, 0)
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, ProtocolError> {
        let action = read_enum_action(buf)?;
        let pos = read_block_pos(buf)?;
        ensure_remaining(buf, 1)?;
        let direction = Direction::from_3d_data_value(buf.get_u8() as i32);
        let sequence = read_var_int(buf)?;
        Ok(Self {
            action,
            pos,
            direction,
            sequence,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_enum_action(buf, self.action);
        write_block_pos(buf, self.pos);
        buf.put_u8(self.direction.get_3d_data_value() as u8);
        write_var_int(buf, self.sequence);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#type
    pub fn packet_type_name(&self) -> &'static str {
        "minecraft:player_action"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#handle
    pub fn handle<H: ServerGamePacketListener>(&self, handler: &mut H) {
        handler.handle_player_action(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#getPos
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#getDirection
    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#getAction
    pub fn get_action(&self) -> ServerboundPlayerActionPacketAction {
        self.action
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerActionPacket#getSequence
    pub fn get_sequence(&self) -> i32 {
        self.sequence
    }
}

pub trait ServerGamePacketListener {
    fn handle_player_action(&mut self, packet: &ServerboundPlayerActionPacket);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn action_round_trip_and_fields() {
        let pos = BlockPos::new(10, 64, -20);
        let packet = ServerboundPlayerActionPacket::new(
            ServerboundPlayerActionPacketAction::StartDestroyBlock,
            pos,
            Direction::North,
            42,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundPlayerActionPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(decoded.get_pos(), pos);
        assert_eq!(decoded.get_direction(), Direction::North);
        assert_eq!(decoded.get_action(), ServerboundPlayerActionPacketAction::StartDestroyBlock);
        assert_eq!(decoded.get_sequence(), 42);
        assert_eq!(decoded.packet_type_name(), "minecraft:player_action");
    }

    #[test]
    fn constructor_without_sequence_defaults_to_zero() {
        let packet = ServerboundPlayerActionPacket::new_without_sequence(
            ServerboundPlayerActionPacketAction::DropItem,
            BlockPos::new(0, 0, 0),
            Direction::Up,
        );
        assert_eq!(packet.get_sequence(), 0);
    }

    #[test]
    fn block_pos_packing_negative_and_extreme_values() {
        let positions = [
            BlockPos::new(-1, -1, -1),
            BlockPos::new(33_554_431, 2047, 33_554_431),
            BlockPos::new(-33_554_432, -2048, -33_554_432),
            BlockPos::new(12345, -64, -9876),
        ];
        for pos in positions {
            let packed = pos.as_long();
            let unpacked = BlockPos::of_long(packed);
            assert_eq!(unpacked, pos, "round-trip failed for {:?}", pos);
        }
    }

    #[test]
    fn var_int_round_trip_negative_and_overflow_edges() {
        let values = [0i32, 1, 127, 128, 255, -1, -2147483648, 2147483647];
        for v in values {
            let mut buf = BytesMut::new();
            write_var_int(&mut buf, v);
            let mut slice: &[u8] = &buf;
            let decoded = read_var_int(&mut slice).unwrap();
            assert_eq!(decoded, v, "varint round-trip failed for {v}");
        }
    }

    #[test]
    fn direction_from_3d_data_value_wraps_like_java() {
        assert_eq!(Direction::from_3d_data_value(0), Direction::Down);
        assert_eq!(Direction::from_3d_data_value(5), Direction::East);
        assert_eq!(Direction::from_3d_data_value(6), Direction::Down);
        assert_eq!(Direction::from_3d_data_value(-1), Direction::Up);
        assert_eq!(Direction::from_3d_data_value(-7), Direction::Up);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct MockListener {
            called: bool,
            action: Option<ServerboundPlayerActionPacketAction>,
        }
        impl ServerGamePacketListener for MockListener {
            fn handle_player_action(&mut self, packet: &ServerboundPlayerActionPacket) {
                self.called = true;
                self.action = Some(packet.get_action());
            }
        }
        let packet = ServerboundPlayerActionPacket::new_without_sequence(
            ServerboundPlayerActionPacketAction::SwapItemWithOffhand,
            BlockPos::new(1, 2, 3),
            Direction::West,
        );
        let mut listener = MockListener { called: false, action: None };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.action, Some(ServerboundPlayerActionPacketAction::SwapItemWithOffhand));
    }
}
