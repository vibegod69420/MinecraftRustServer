use std::fmt;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum PacketDecodeError {
    #[error("unexpected end of buffer")]
    Eof,
    #[error("varint too big")]
    VarIntTooBig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_byte(&mut self) -> Result<i8, PacketDecodeError> {
        if self.remaining() < 1 {
            return Err(PacketDecodeError::Eof);
        }
        let v = self.data[self.pos] as i8;
        self.pos += 1;
        Ok(v)
    }

    pub fn write_byte(&mut self, value: i8) -> &mut Self {
        self.data.push(value as u8);
        self
    }

    pub fn read_short(&mut self) -> Result<i16, PacketDecodeError> {
        if self.remaining() < 2 {
            return Err(PacketDecodeError::Eof);
        }
        let v = i16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    pub fn write_short(&mut self, value: i16) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn read_boolean(&mut self) -> Result<bool, PacketDecodeError> {
        Ok(self.read_byte()? != 0)
    }

    pub fn write_boolean(&mut self, value: bool) -> &mut Self {
        self.write_byte(if value { 1 } else { 0 });
        self
    }

    pub fn read_var_int(&mut self) -> Result<i32, PacketDecodeError> {
        let mut i: i32 = 0;
        let mut j: u32 = 0;
        loop {
            let b = self.read_byte()?;
            i |= ((b & 127) as i32).wrapping_shl(j.wrapping_mul(7) & 31);
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(PacketDecodeError::VarIntTooBig);
            }
            if (b & -128i8) == 0 {
                break;
            }
        }
        Ok(i)
    }

    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & -128i32) == 0 {
                self.write_byte(value as i8);
                return self;
            }
            self.write_byte(((value & 127) | 128) as i8);
            value = ((value as u32) >> 7) as i32;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ClientGamePacketListener {
    fn handle_move_entity(&mut self, packet: &ClientboundMoveEntityPacket);
}

pub trait Entity {}

pub trait Level {
    fn get_entity(&self, entity_id: i32) -> Option<&dyn Entity>;
}

// JAVA: net.minecraft.util.Mth#unpackDegrees
pub fn unpack_degrees(angle: i8) -> f32 {
    (angle as f32) * 360.0f32 / 256.0f32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveEntityPos {
    pub entity_id: i32,
    pub xa: i16,
    pub ya: i16,
    pub za: i16,
    pub on_ground: bool,
}

impl MoveEntityPos {
    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Pos#<init>
    pub fn new(entity_id: i32, xa: i16, ya: i16, za: i16, on_ground: bool) -> Self {
        Self {
            entity_id,
            xa,
            ya,
            za,
            on_ground,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Pos#read
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, PacketDecodeError> {
        let entity_id = buf.read_var_int()?;
        let xa = buf.read_short()?;
        let ya = buf.read_short()?;
        let za = buf.read_short()?;
        let on_ground = buf.read_boolean()?;
        Ok(Self {
            entity_id,
            xa,
            ya,
            za,
            on_ground,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Pos#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        buf.write_short(self.xa);
        buf.write_short(self.ya);
        buf.write_short(self.za);
        buf.write_boolean(self.on_ground);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Pos#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:clientbound_move_entity_pos")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveEntityPosRot {
    pub entity_id: i32,
    pub xa: i16,
    pub ya: i16,
    pub za: i16,
    pub y_rot: i8,
    pub x_rot: i8,
    pub on_ground: bool,
}

impl MoveEntityPosRot {
    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.PosRot#<init>
    pub fn new(
        entity_id: i32,
        xa: i16,
        ya: i16,
        za: i16,
        y_rot: i8,
        x_rot: i8,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            xa,
            ya,
            za,
            y_rot,
            x_rot,
            on_ground,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.PosRot#read
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, PacketDecodeError> {
        let entity_id = buf.read_var_int()?;
        let xa = buf.read_short()?;
        let ya = buf.read_short()?;
        let za = buf.read_short()?;
        let y_rot = buf.read_byte()?;
        let x_rot = buf.read_byte()?;
        let on_ground = buf.read_boolean()?;
        Ok(Self {
            entity_id,
            xa,
            ya,
            za,
            y_rot,
            x_rot,
            on_ground,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.PosRot#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        buf.write_short(self.xa);
        buf.write_short(self.ya);
        buf.write_short(self.za);
        buf.write_byte(self.y_rot);
        buf.write_byte(self.x_rot);
        buf.write_boolean(self.on_ground);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.PosRot#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:clientbound_move_entity_pos_rot")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveEntityRot {
    pub entity_id: i32,
    pub y_rot: i8,
    pub x_rot: i8,
    pub on_ground: bool,
}

impl MoveEntityRot {
    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Rot#<init>
    pub fn new(entity_id: i32, y_rot: i8, x_rot: i8, on_ground: bool) -> Self {
        Self {
            entity_id,
            y_rot,
            x_rot,
            on_ground,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Rot#read
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, PacketDecodeError> {
        let entity_id = buf.read_var_int()?;
        let y_rot = buf.read_byte()?;
        let x_rot = buf.read_byte()?;
        let on_ground = buf.read_boolean()?;
        Ok(Self {
            entity_id,
            y_rot,
            x_rot,
            on_ground,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Rot#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        buf.write_byte(self.y_rot);
        buf.write_byte(self.x_rot);
        buf.write_boolean(self.on_ground);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Rot#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:clientbound_move_entity_rot")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientboundMoveEntityPacket {
    Pos(MoveEntityPos),
    PosRot(MoveEntityPosRot),
    Rot(MoveEntityRot),
}

impl ClientboundMoveEntityPacket {
    fn entity_id(&self) -> i32 {
        match self {
            Self::Pos(p) => p.entity_id,
            Self::PosRot(p) => p.entity_id,
            Self::Rot(p) => p.entity_id,
        }
    }

    fn y_rot_raw(&self) -> i8 {
        match self {
            Self::Pos(_) => 0,
            Self::PosRot(p) => p.y_rot,
            Self::Rot(p) => p.y_rot,
        }
    }

    fn x_rot_raw(&self) -> i8 {
        match self {
            Self::Pos(_) => 0,
            Self::PosRot(p) => p.x_rot,
            Self::Rot(p) => p.x_rot,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#type
    pub fn packet_type(&self) -> PacketType {
        match self {
            Self::Pos(p) => p.packet_type(),
            Self::PosRot(p) => p.packet_type(),
            Self::Rot(p) => p.packet_type(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_move_entity(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#toString
    pub fn to_string_java(&self) -> String {
        format!("Entity_{:?}", self)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#getEntity
    pub fn get_entity<'a>(&self, level: &'a dyn Level) -> Option<&'a dyn Entity> {
        level.get_entity(self.entity_id())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#getXa
    pub fn get_xa(&self) -> i16 {
        match self {
            Self::Pos(p) => p.xa,
            Self::PosRot(p) => p.xa,
            Self::Rot(_) => 0,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#getYa
    pub fn get_ya(&self) -> i16 {
        match self {
            Self::Pos(p) => p.ya,
            Self::PosRot(p) => p.ya,
            Self::Rot(_) => 0,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#getZa
    pub fn get_za(&self) -> i16 {
        match self {
            Self::Pos(p) => p.za,
            Self::PosRot(p) => p.za,
            Self::Rot(_) => 0,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#getYRot
    pub fn get_y_rot(&self) -> f32 {
        unpack_degrees(self.y_rot_raw())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#getXRot
    pub fn get_x_rot(&self) -> f32 {
        unpack_degrees(self.x_rot_raw())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#hasRotation
    pub fn has_rotation(&self) -> bool {
        matches!(self, Self::PosRot(_) | Self::Rot(_))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#hasPosition
    pub fn has_position(&self) -> bool {
        matches!(self, Self::Pos(_) | Self::PosRot(_))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveEntityPacket#isOnGround
    pub fn is_on_ground(&self) -> bool {
        match self {
            Self::Pos(p) => p.on_ground,
            Self::PosRot(p) => p.on_ground,
            Self::Rot(p) => p.on_ground,
        }
    }
}

impl fmt::Display for ClientboundMoveEntityPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_java())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_round_trip_negative() {
        let pkt = MoveEntityPos::new(-1, i16::MIN, -123, i16::MAX, true);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = MoveEntityPos::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);

        let ep = ClientboundMoveEntityPacket::Pos(decoded);
        assert_eq!(ep.get_xa(), i16::MIN);
        assert_eq!(ep.get_ya(), -123);
        assert_eq!(ep.get_za(), i16::MAX);
        assert!(ep.has_position());
        assert!(!ep.has_rotation());
        assert!(ep.is_on_ground());
        assert_eq!(ep.packet_type(), PacketType("minecraft:clientbound_move_entity_pos"));
    }

    #[test]
    fn pos_rot_round_trip() {
        let pkt = MoveEntityPosRot::new(42, 100, -200, 300, -128i8, 127i8, false);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf);
        let decoded = MoveEntityPosRot::read(&mut FriendlyByteBuf::new(buf.into_inner())).unwrap();
        assert_eq!(pkt, decoded);

        let ep = ClientboundMoveEntityPacket::PosRot(decoded);
        assert!(ep.has_position());
        assert!(ep.has_rotation());
        assert!(!ep.is_on_ground());
        assert_eq!(ep.get_y_rot().to_bits(), unpack_degrees(-128i8).to_bits());
        assert_eq!(ep.get_x_rot().to_bits(), unpack_degrees(127i8).to_bits());
    }

    #[test]
    fn rot_round_trip_and_defaults() {
        let pkt = MoveEntityRot::new(i32::MAX, 64, -64, true);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf);
        let decoded = MoveEntityRot::read(&mut FriendlyByteBuf::new(buf.into_inner())).unwrap();
        assert_eq!(pkt, decoded);

        let ep = ClientboundMoveEntityPacket::Rot(decoded);
        assert_eq!(ep.get_xa(), 0);
        assert_eq!(ep.get_ya(), 0);
        assert_eq!(ep.get_za(), 0);
        assert!(!ep.has_position());
        assert!(ep.has_rotation());
        assert!(ep.is_on_ground());
    }

    #[test]
    fn unpack_degrees_float_bits() {
        assert_eq!(unpack_degrees(0).to_bits(), 0.0f32.to_bits());
        let expected_1 = (1.0f32 * 360.0f32 / 256.0f32).to_bits();
        assert_eq!(unpack_degrees(1).to_bits(), expected_1);
        let expected_neg128 = (-128.0f32 * 360.0f32 / 256.0f32).to_bits();
        assert_eq!(unpack_degrees(-128i8).to_bits(), expected_neg128);
        let expected_127 = (127.0f32 * 360.0f32 / 256.0f32).to_bits();
        assert_eq!(unpack_degrees(127i8).to_bits(), expected_127);
    }

    #[test]
    fn varint_negative_and_large() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(-1);
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        buf.write_var_int(0);

        let mut r = FriendlyByteBuf::new(buf.into_inner());
        assert_eq!(r.read_var_int().unwrap(), -1);
        assert_eq!(r.read_var_int().unwrap(), i32::MIN);
        assert_eq!(r.read_var_int().unwrap(), i32::MAX);
        assert_eq!(r.read_var_int().unwrap(), 0);
    }
}
