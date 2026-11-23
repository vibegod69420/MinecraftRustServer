use std::collections::BTreeSet;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

// JAVA: net.minecraft.network.protocol.game.GamePacketTypes#CLIENTBOUND_PLAYER_POSITION
pub const CLIENTBOUND_PLAYER_POSITION: PacketType =
    PacketType("minecraft:clientbound_player_position");

#[derive(Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    fn read_byte(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "read past end of buffer",
            ));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & !0x7f) == 0 {
                self.data.push(value as u8);
                return self;
            }
            self.data.push(((value & 0x7f) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_byte()?;
            result |= ((b & 0x7f) as i32).wrapping_shl(shift.wrapping_mul(7));
            shift += 1;
            if shift > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if (b & 0x80) == 0 {
                break;
            }
        }
        Ok(result)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInt
    pub fn write_int(&mut self, value: i32) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInt
    pub fn read_int(&mut self) -> io::Result<i32> {
        let mut bytes = [0u8; 4];
        for b in &mut bytes {
            *b = self.read_byte()?;
        }
        Ok(i32::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeDouble
    pub fn write_double(&mut self, value: f64) -> &mut Self {
        self.data.extend_from_slice(&value.to_bits().to_be_bytes());
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readDouble
    pub fn read_double(&mut self) -> io::Result<f64> {
        let mut bytes = [0u8; 8];
        for b in &mut bytes {
            *b = self.read_byte()?;
        }
        Ok(f64::from_bits(u64::from_be_bytes(bytes)))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeFloat
    pub fn write_float(&mut self, value: f32) -> &mut Self {
        self.data.extend_from_slice(&value.to_bits().to_be_bytes());
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readFloat
    pub fn read_float(&mut self) -> io::Result<f32> {
        let mut bytes = [0u8; 4];
        for b in &mut bytes {
            *b = self.read_byte()?;
        }
        Ok(f32::from_bits(u32::from_be_bytes(bytes)))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writerIndex
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readableBytes
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Relative {
    X,
    Y,
    Z,
    YRot,
    XRot,
    DeltaX,
    DeltaY,
    DeltaZ,
    RotateDelta,
}

impl Relative {
    // JAVA: net.minecraft.world.entity.Relative#ordinal
    fn ordinal(self) -> i32 {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
            Self::YRot => 3,
            Self::XRot => 4,
            Self::DeltaX => 5,
            Self::DeltaY => 6,
            Self::DeltaZ => 7,
            Self::RotateDelta => 8,
        }
    }

    // JAVA: net.minecraft.world.entity.Relative#values
    fn all() -> &'static [Relative] {
        &[
            Self::X,
            Self::Y,
            Self::Z,
            Self::YRot,
            Self::XRot,
            Self::DeltaX,
            Self::DeltaY,
            Self::DeltaZ,
            Self::RotateDelta,
        ]
    }

    // JAVA: net.minecraft.world.entity.Relative#pack
    pub fn pack(set: &BTreeSet<Relative>) -> i32 {
        let mut mask = 0i32;
        for r in set {
            mask |= 1i32.wrapping_shl(r.ordinal() as u32);
        }
        mask
    }

    // JAVA: net.minecraft.world.entity.Relative#unpack
    pub fn unpack(mask: i32) -> BTreeSet<Relative> {
        let mut set = BTreeSet::new();
        for &r in Self::all() {
            if (mask & 1i32.wrapping_shl(r.ordinal() as u32)) != 0 {
                set.insert(r);
            }
        }
        set
    }

    // JAVA: net.minecraft.world.entity.Relative.SET_STREAM_CODEC (decode)
    pub fn read_set(buf: &mut FriendlyByteBuf) -> io::Result<BTreeSet<Relative>> {
        Ok(Self::unpack(buf.read_int()?))
    }

    // JAVA: net.minecraft.world.entity.Relative.SET_STREAM_CODEC (encode)
    pub fn write_set(buf: &mut FriendlyByteBuf, set: &BTreeSet<Relative>) {
        buf.write_int(Self::pack(set));
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionMoveRotation {
    pub position: [f64; 3],
    pub delta_movement: [f64; 3],
    pub y_rot: f32,
    pub x_rot: f32,
}

impl PositionMoveRotation {
    // JAVA: net.minecraft.world.entity.PositionMoveRotation#<init>
    pub fn new(position: [f64; 3], delta_movement: [f64; 3], y_rot: f32, x_rot: f32) -> Self {
        Self {
            position,
            delta_movement,
            y_rot,
            x_rot,
        }
    }

    // JAVA: net.minecraft.world.entity.PositionMoveRotation.STREAM_CODEC (decode)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let position = [buf.read_double()?, buf.read_double()?, buf.read_double()?];
        let delta_movement = [buf.read_double()?, buf.read_double()?, buf.read_double()?];
        let y_rot = buf.read_float()?;
        let x_rot = buf.read_float()?;
        Ok(Self {
            position,
            delta_movement,
            y_rot,
            x_rot,
        })
    }

    // JAVA: net.minecraft.world.entity.PositionMoveRotation.STREAM_CODEC (encode)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        for &v in &self.position {
            buf.write_double(v);
        }
        for &v in &self.delta_movement {
            buf.write_double(v);
        }
        buf.write_float(self.y_rot);
        buf.write_float(self.x_rot);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundPlayerPositionPacket {
    id: i32,
    change: PositionMoveRotation,
    relatives: BTreeSet<Relative>,
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleMovePlayer
    fn handle_move_player(&mut self, packet: &ClientboundPlayerPositionPacket);
}

impl ClientboundPlayerPositionPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket#of
    pub fn of(id: i32, change: PositionMoveRotation, relatives: BTreeSet<Relative>) -> Self {
        Self::new(id, change, relatives)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket#<init>
    pub fn new(id: i32, change: PositionMoveRotation, relatives: BTreeSet<Relative>) -> Self {
        Self {
            id,
            change,
            relatives,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket#id
    pub fn id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket#change
    pub fn change(&self) -> &PositionMoveRotation {
        &self.change
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket#relatives
    pub fn relatives(&self) -> &BTreeSet<Relative> {
        &self.relatives
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_PLAYER_POSITION
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_move_player(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket.STREAM_CODEC (decode)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_var_int()?;
        let change = PositionMoveRotation::read(buf)?;
        let relatives = Relative::read_set(buf)?;
        Ok(Self::new(id, change, relatives))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket.STREAM_CODEC (encode)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        self.change.write(buf);
        Relative::write_set(buf, &self.relatives);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_packet() {
        let mut relatives = BTreeSet::new();
        relatives.insert(Relative::X);
        relatives.insert(Relative::YRot);
        let change =
            PositionMoveRotation::new([1.5, -2.0, 3.25], [0.0, -0.1, 0.5], 90.0, -45.0);
        let pkt = ClientboundPlayerPositionPacket::of(42, change, relatives.clone());

        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf {
            data: buf.into_inner(),
            read_pos: 0,
        };
        let decoded = ClientboundPlayerPositionPacket::read(&mut r).unwrap();

        assert_eq!(decoded.id(), 42);
        assert_eq!(decoded.relatives(), &relatives);
        assert_eq!(decoded.change().y_rot.to_bits(), 90.0f32.to_bits());
        assert_eq!(decoded.change().position[1].to_bits(), (-2.0f64).to_bits());
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn varint_negative_and_extremes() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        buf.write_var_int(0);
        let mut r = FriendlyByteBuf {
            data: buf.into_inner(),
            read_pos: 0,
        };
        assert_eq!(r.read_var_int().unwrap(), -1);
        assert_eq!(r.read_var_int().unwrap(), i32::MIN);
        assert_eq!(r.read_var_int().unwrap(), i32::MAX);
        assert_eq!(r.read_var_int().unwrap(), 0);
    }

    #[test]
    fn varint_too_big() {
        let mut r = FriendlyByteBuf {
            data: vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80],
            read_pos: 0,
        };
        assert!(r.read_var_int().is_err());
    }

    #[test]
    fn relative_set_bitmask_roundtrip() {
        let mut set = BTreeSet::new();
        for &rel in Relative::all() {
            set.insert(rel);
        }
        let packed = Relative::pack(&set);
        let unpacked = Relative::unpack(packed);
        assert_eq!(set, unpacked);
        assert_eq!(Relative::unpack(0), BTreeSet::new());
        assert_eq!(Relative::pack(&BTreeSet::from([Relative::RotateDelta])), 1 << 8);
    }

    #[test]
    fn float_and_double_bit_preservation() {
        let nan_f64 = 0x7ff8_0000_0000_0001u64;
        let neg_zero_f64 = 0x8000_0000_0000_0000u64;
        let nan_f32 = 0xffc0_0001u32;
        let mut buf = FriendlyByteBuf::new();
        buf.write_double(f64::from_bits(nan_f64));
        buf.write_double(f64::from_bits(neg_zero_f64));
        buf.write_float(f32::from_bits(nan_f32));
        let mut r = FriendlyByteBuf {
            data: buf.into_inner(),
            read_pos: 0,
        };
        assert_eq!(r.read_double().unwrap().to_bits(), nan_f64);
        assert_eq!(r.read_double().unwrap().to_bits(), neg_zero_f64);
        assert_eq!(r.read_float().unwrap().to_bits(), nan_f32);
    }
}
