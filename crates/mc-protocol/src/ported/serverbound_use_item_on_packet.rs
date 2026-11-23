use bytes::{Buf, BufMut};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PacketDecodeError {
    #[error("unexpected end of buffer")]
    UnexpectedEof,
    #[error("varint too big")]
    VarIntTooBig,
    #[error("invalid enum ordinal: {0}")]
    InvalidEnumOrdinal(i32),
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<B: Buf>(buf: &mut B) -> Result<i32, PacketDecodeError> {
    let mut value: i32 = 0;
    let mut position: i32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let b0 = buf.get_u8() as i8;
        value |= ((b0 & 127) as i32) << (position * 7);
        position += 1;
        if position > 5 {
            return Err(PacketDecodeError::VarIntTooBig);
        }
        if (b0 & -128i8) == 0 {
            break;
        }
    }
    Ok(value)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value & -128i32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 127) | 128) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
fn read_bool<B: Buf>(buf: &mut B) -> Result<bool, PacketDecodeError> {
    if !buf.has_remaining() {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    Ok(buf.get_u8() != 0)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
fn write_bool<B: BufMut>(buf: &mut B, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionHand {
    MainHand = 0,
    OffHand = 1,
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readEnum
fn read_hand<B: Buf>(buf: &mut B) -> Result<InteractionHand, PacketDecodeError> {
    let ordinal = read_var_int(buf)?;
    match ordinal {
        0 => Ok(InteractionHand::MainHand),
        1 => Ok(InteractionHand::OffHand),
        _ => Err(PacketDecodeError::InvalidEnumOrdinal(ordinal)),
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeEnum
fn write_hand<B: BufMut>(buf: &mut B, hand: InteractionHand) {
    write_var_int(buf, hand as i32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readDirection
fn read_direction<B: Buf>(buf: &mut B) -> Result<Direction, PacketDecodeError> {
    let id = read_var_int(buf)?;
    match id {
        0 => Ok(Direction::Down),
        1 => Ok(Direction::Up),
        2 => Ok(Direction::North),
        3 => Ok(Direction::South),
        4 => Ok(Direction::West),
        5 => Ok(Direction::East),
        _ => Err(PacketDecodeError::InvalidEnumOrdinal(id)),
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeDirection
fn write_direction<B: BufMut>(buf: &mut B, direction: Direction) {
    write_var_int(buf, direction as i32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

const PACKED_X_LENGTH: u32 = 26;
const PACKED_Z_LENGTH: u32 = 26;
const PACKED_Y_LENGTH: u32 = 12;
const PACKED_X_MASK: i64 = (1i64 << PACKED_X_LENGTH) - 1;
const PACKED_Y_MASK: i64 = (1i64 << PACKED_Y_LENGTH) - 1;
const PACKED_Z_MASK: i64 = (1i64 << PACKED_Z_LENGTH) - 1;
const Z_OFFSET: u32 = PACKED_Y_LENGTH;
const X_OFFSET: u32 = Z_OFFSET + PACKED_Z_LENGTH;

// JAVA: net.minecraft.core.BlockPos#asLong
fn pack_block_pos(pos: BlockPos) -> i64 {
    ((pos.x as i64 & PACKED_X_MASK) << X_OFFSET)
        | ((pos.z as i64 & PACKED_Z_MASK) << Z_OFFSET)
        | (pos.y as i64 & PACKED_Y_MASK)
}

// JAVA: net.minecraft.core.BlockPos#of
fn unpack_block_pos(packed: i64) -> BlockPos {
    let x = (packed << (64 - X_OFFSET - PACKED_X_LENGTH) >> (64 - PACKED_X_LENGTH)) as i32;
    let y = (packed << (64 - PACKED_Y_LENGTH) >> (64 - PACKED_Y_LENGTH)) as i32;
    let z = (packed << (64 - Z_OFFSET - PACKED_Z_LENGTH) >> (64 - PACKED_Z_LENGTH)) as i32;
    BlockPos { x, y, z }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBlockPos
fn read_block_pos<B: Buf>(buf: &mut B) -> Result<BlockPos, PacketDecodeError> {
    if buf.remaining() < 8 {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    Ok(unpack_block_pos(buf.get_i64()))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBlockPos
fn write_block_pos<B: BufMut>(buf: &mut B, pos: BlockPos) {
    buf.put_i64(pack_block_pos(pos));
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockHitResult {
    pub location: Vec3,
    pub direction: Direction,
    pub block_pos: BlockPos,
    pub inside: bool,
    pub world_border_hit: bool,
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBlockHitResult
fn read_block_hit_result<B: Buf>(buf: &mut B) -> Result<BlockHitResult, PacketDecodeError> {
    let block_pos = read_block_pos(buf)?;
    let direction = read_direction(buf)?;
    if buf.remaining() < 4 {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let fx = buf.get_f32();
    if buf.remaining() < 4 {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let fy = buf.get_f32();
    if buf.remaining() < 4 {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let fz = buf.get_f32();
    let inside = read_bool(buf)?;
    let world_border_hit = read_bool(buf)?;
    let location = Vec3 {
        x: (block_pos.x as f64) + (fx as f64),
        y: (block_pos.y as f64) + (fy as f64),
        z: (block_pos.z as f64) + (fz as f64),
    };
    Ok(BlockHitResult {
        location,
        direction,
        block_pos,
        inside,
        world_border_hit,
    })
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBlockHitResult
fn write_block_hit_result<B: BufMut>(buf: &mut B, hit: &BlockHitResult) {
    write_block_pos(buf, hit.block_pos);
    write_direction(buf, hit.direction);
    buf.put_f32((hit.location.x - (hit.block_pos.x as f64)) as f32);
    buf.put_f32((hit.location.y - (hit.block_pos.y as f64)) as f32);
    buf.put_f32((hit.location.z - (hit.block_pos.z as f64)) as f32);
    write_bool(buf, hit.inside);
    write_bool(buf, hit.world_border_hit);
}

pub trait ServerGamePacketListener {
    fn handle_use_item_on(&mut self, packet: &ServerboundUseItemOnPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundUseItemOnPacket {
    block_hit: BlockHitResult,
    hand: InteractionHand,
    sequence: i32,
}

impl ServerboundUseItemOnPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemOnPacket#<init>
    pub fn new(hand: InteractionHand, block_hit: BlockHitResult, sequence: i32) -> Self {
        Self {
            block_hit,
            hand,
            sequence,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemOnPacket#<init>
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, PacketDecodeError> {
        let hand = read_hand(buf)?;
        let block_hit = read_block_hit_result(buf)?;
        let sequence = read_var_int(buf)?;
        Ok(Self {
            block_hit,
            hand,
            sequence,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemOnPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_hand(buf, self.hand);
        write_block_hit_result(buf, &self.block_hit);
        write_var_int(buf, self.sequence);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemOnPacket#type
    pub fn type_name(&self) -> &'static str {
        "minecraft:use_item_on"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemOnPacket#handle
    pub fn handle<H: ServerGamePacketListener>(&self, handler: &mut H) {
        handler.handle_use_item_on(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemOnPacket#getHand
    pub fn get_hand(&self) -> InteractionHand {
        self.hand
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemOnPacket#getHitResult
    pub fn get_hit_result(&self) -> &BlockHitResult {
        &self.block_hit
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemOnPacket#getSequence
    pub fn get_sequence(&self) -> i32 {
        self.sequence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockHandler {
        called: bool,
        sequence: i32,
    }

    impl ServerGamePacketListener for MockHandler {
        fn handle_use_item_on(&mut self, packet: &ServerboundUseItemOnPacket) {
            self.called = true;
            self.sequence = packet.get_sequence();
        }
    }

    fn sample_hit() -> BlockHitResult {
        BlockHitResult {
            location: Vec3 {
                x: 10.5,
                y: 64.25,
                z: -3.75,
            },
            direction: Direction::Up,
            block_pos: BlockPos { x: 10, y: 64, z: -4 },
            inside: false,
            world_border_hit: true,
        }
    }

    #[test]
    fn roundtrip_normal() {
        let pkt = ServerboundUseItemOnPacket::new(InteractionHand::MainHand, sample_hit(), 7);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundUseItemOnPacket::read(&mut buf.freeze()).unwrap();
        assert_eq!(decoded.get_hand(), InteractionHand::MainHand);
        assert_eq!(decoded.get_sequence(), 7);
        assert_eq!(decoded.get_hit_result().direction, Direction::Up);
        assert_eq!(decoded.get_hit_result().block_pos, BlockPos { x: 10, y: 64, z: -4 });
        assert!(decoded.get_hit_result().world_border_hit);
        assert!(!decoded.get_hit_result().inside);
        assert_eq!(decoded.get_hit_result().location.x.to_bits(), 10.5f64.to_bits());
    }

    #[test]
    fn block_pos_sign_extension_and_packing() {
        let pos = BlockPos { x: -1, y: -1, z: -1 };
        let packed = pack_block_pos(pos);
        assert_eq!(unpack_block_pos(packed), pos);

        let pos2 = BlockPos { x: 1_000_000, y: 127, z: -500_000 };
        let packed2 = pack_block_pos(pos2);
        assert_eq!(unpack_block_pos(packed2), pos2);

        let hit = BlockHitResult {
            location: Vec3 { x: -1.0, y: -1.0, z: -1.0 },
            direction: Direction::Down,
            block_pos: pos,
            inside: true,
            world_border_hit: false,
        };
        let pkt = ServerboundUseItemOnPacket::new(InteractionHand::OffHand, hit, -5);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ServerboundUseItemOnPacket::read(&mut buf.freeze()).unwrap();
        assert_eq!(decoded.get_hit_result().block_pos, pos);
        assert_eq!(decoded.get_hand(), InteractionHand::OffHand);
        assert_eq!(decoded.get_sequence(), -5);
    }

    #[test]
    fn invalid_enum_and_eof() {
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, 99);
        assert_eq!(
            read_hand(&mut buf.freeze()),
            Err(PacketDecodeError::InvalidEnumOrdinal(99))
        );

        let mut empty = bytes::Bytes::new();
        assert_eq!(
            ServerboundUseItemOnPacket::read(&mut empty),
            Err(PacketDecodeError::UnexpectedEof)
        );
    }

    #[test]
    fn handle_and_type() {
        let pkt = ServerboundUseItemOnPacket::new(InteractionHand::MainHand, sample_hit(), 42);
        let mut handler = MockHandler { called: false, sequence: 0 };
        pkt.handle(&mut handler);
        assert!(handler.called);
        assert_eq!(handler.sequence, 42);
        assert_eq!(pkt.type_name(), "minecraft:use_item_on");
    }
}
