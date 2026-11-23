//! Port of `net.minecraft.network.protocol.game.ServerboundMovePlayerPacket`.

use std::io;

const FLAG_ON_GROUND: i32 = 1;
const FLAG_HORIZONTAL_COLLISION: i32 = 2;

/// Minimal buffer for packet serialization, matching Minecraft's FriendlyByteBuf semantics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_double(&mut self) -> io::Result<f64> {
        let mut b = [0u8; 8];
        self.read_exact_into(&mut b)?;
        Ok(f64::from_be_bytes(b))
    }

    pub fn write_double(&mut self, value: f64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_float(&mut self) -> io::Result<f32> {
        let mut b = [0u8; 4];
        self.read_exact_into(&mut b)?;
        Ok(f32::from_be_bytes(b))
    }

    pub fn write_float(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    /// Reads an unsigned byte as i16 (0..=255), matching Java readUnsignedByte().
    pub fn read_unsigned_byte(&mut self) -> io::Result<i16> {
        let mut b = [0u8; 1];
        self.read_exact_into(&mut b)?;
        Ok(b[0] as i16)
    }

    /// Writes the low byte of an i32, matching Java writeByte(int).
    pub fn write_byte(&mut self, value: i32) {
        self.data.push((value & 0xFF) as u8);
    }

    fn read_exact_into(&mut self, dst: &mut [u8]) -> io::Result<()> {
        let end = self
            .pos
            .checked_add(dst.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "buffer overflow"))?;
        if end > self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "not enough data in FriendlyByteBuf",
            ));
        }
        dst.copy_from_slice(&self.data[self.pos..end]);
        self.pos = end;
        Ok(())
    }
}

/// Minimal Vec3 for packet construction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// Minimal packet type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketType {
    ServerboundMovePlayerPos,
    ServerboundMovePlayerPosRot,
    ServerboundMovePlayerRot,
    ServerboundMovePlayerStatusOnly,
}

/// Minimal server game packet listener for handle dispatch.
pub trait ServerGamePacketListener {
    fn handle_move_player(&mut self, packet: &ServerboundMovePlayerPacket);
}

// JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#packFlags
pub fn pack_flags(on_ground: bool, horizontal_collision: bool) -> i32 {
    let mut flags = 0i32;
    if on_ground {
        flags |= FLAG_ON_GROUND;
    }
    if horizontal_collision {
        flags |= FLAG_HORIZONTAL_COLLISION;
    }
    flags
}

// JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#unpackOnGround
pub fn unpack_on_ground(flags: i32) -> bool {
    (flags & FLAG_ON_GROUND) != 0
}

// JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#unpackHorizontalCollision
pub fn unpack_horizontal_collision(flags: i32) -> bool {
    (flags & FLAG_HORIZONTAL_COLLISION) != 0
}

/// Common data fields of `ServerboundMovePlayerPacket`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovePlayerPacketData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub has_pos: bool,
    pub has_rot: bool,
}

impl MovePlayerPacketData {
    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#<init>
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
        has_pos: bool,
        has_rot: bool,
    ) -> Self {
        Self {
            x,
            y,
            z,
            y_rot,
            x_rot,
            on_ground,
            horizontal_collision,
            has_pos,
            has_rot,
        }
    }
}

/// `ServerboundMovePlayerPacket.Pos`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    pub data: MovePlayerPacketData,
}

impl Pos {
    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Pos#<init>
    pub fn from_vec(pos: Vec3, on_ground: bool, horizontal_collision: bool) -> Self {
        Self::new(pos.x, pos.y, pos.z, on_ground, horizontal_collision)
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Pos#<init>
    pub fn new(x: f64, y: f64, z: f64, on_ground: bool, horizontal_collision: bool) -> Self {
        Self {
            data: MovePlayerPacketData::new(
                x,
                y,
                z,
                0.0,
                0.0,
                on_ground,
                horizontal_collision,
                true,
                false,
            ),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Pos#read
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let x = buf.read_double()?;
        let y = buf.read_double()?;
        let z = buf.read_double()?;
        let flags = buf.read_unsigned_byte()? as i32;
        let on_ground = unpack_on_ground(flags);
        let horizontal_collision = unpack_horizontal_collision(flags);
        Ok(Self::new(x, y, z, on_ground, horizontal_collision))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Pos#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_double(self.data.x);
        buf.write_double(self.data.y);
        buf.write_double(self.data.z);
        buf.write_byte(pack_flags(self.data.on_ground, self.data.horizontal_collision));
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Pos#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ServerboundMovePlayerPos
    }
}

/// `ServerboundMovePlayerPacket.PosRot`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosRot {
    pub data: MovePlayerPacketData,
}

impl PosRot {
    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$PosRot#<init>
    pub fn from_vec(
        pos: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    ) -> Self {
        Self::new(
            pos.x,
            pos.y,
            pos.z,
            y_rot,
            x_rot,
            on_ground,
            horizontal_collision,
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$PosRot#<init>
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    ) -> Self {
        Self {
            data: MovePlayerPacketData::new(
                x,
                y,
                z,
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
                true,
                true,
            ),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$PosRot#read
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let x = buf.read_double()?;
        let y = buf.read_double()?;
        let z = buf.read_double()?;
        let y_rot = buf.read_float()?;
        let x_rot = buf.read_float()?;
        let flags = buf.read_unsigned_byte()? as i32;
        let on_ground = unpack_on_ground(flags);
        let horizontal_collision = unpack_horizontal_collision(flags);
        Ok(Self::new(
            x,
            y,
            z,
            y_rot,
            x_rot,
            on_ground,
            horizontal_collision,
        ))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$PosRot#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_double(self.data.x);
        buf.write_double(self.data.y);
        buf.write_double(self.data.z);
        buf.write_float(self.data.y_rot);
        buf.write_float(self.data.x_rot);
        buf.write_byte(pack_flags(self.data.on_ground, self.data.horizontal_collision));
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$PosRot#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ServerboundMovePlayerPosRot
    }
}

/// `ServerboundMovePlayerPacket.Rot`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rot {
    pub data: MovePlayerPacketData,
}

impl Rot {
    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Rot#<init>
    pub fn new(y_rot: f32, x_rot: f32, on_ground: bool, horizontal_collision: bool) -> Self {
        Self {
            data: MovePlayerPacketData::new(
                0.0,
                0.0,
                0.0,
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
                false,
                true,
            ),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Rot#read
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let y_rot = buf.read_float()?;
        let x_rot = buf.read_float()?;
        let flags = buf.read_unsigned_byte()? as i32;
        let on_ground = unpack_on_ground(flags);
        let horizontal_collision = unpack_horizontal_collision(flags);
        Ok(Self::new(y_rot, x_rot, on_ground, horizontal_collision))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Rot#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_float(self.data.y_rot);
        buf.write_float(self.data.x_rot);
        buf.write_byte(pack_flags(self.data.on_ground, self.data.horizontal_collision));
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$Rot#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ServerboundMovePlayerRot
    }
}

/// `ServerboundMovePlayerPacket.StatusOnly`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusOnly {
    pub data: MovePlayerPacketData,
}

impl StatusOnly {
    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$StatusOnly#<init>
    pub fn new(on_ground: bool, horizontal_collision: bool) -> Self {
        Self {
            data: MovePlayerPacketData::new(
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                on_ground,
                horizontal_collision,
                false,
                false,
            ),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$StatusOnly#read
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let flags = buf.read_unsigned_byte()? as i32;
        let on_ground = unpack_on_ground(flags);
        let horizontal_collision = unpack_horizontal_collision(flags);
        Ok(Self::new(on_ground, horizontal_collision))
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$StatusOnly#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_byte(pack_flags(self.data.on_ground, self.data.horizontal_collision));
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$StatusOnly#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ServerboundMovePlayerStatusOnly
    }
}

/// Port of `net.minecraft.network.protocol.game.ServerboundMovePlayerPacket`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerboundMovePlayerPacket {
    Pos(Pos),
    PosRot(PosRot),
    Rot(Rot),
    StatusOnly(StatusOnly),
}

impl ServerboundMovePlayerPacket {
    pub fn data(&self) -> &MovePlayerPacketData {
        match self {
            Self::Pos(p) => &p.data,
            Self::PosRot(p) => &p.data,
            Self::Rot(p) => &p.data,
            Self::StatusOnly(p) => &p.data,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#type
    pub fn packet_type(&self) -> PacketType {
        match self {
            Self::Pos(p) => p.packet_type(),
            Self::PosRot(p) => p.packet_type(),
            Self::Rot(p) => p.packet_type(),
            Self::StatusOnly(p) => p.packet_type(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_move_player(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#getX
    pub fn get_x(&self, default: f64) -> f64 {
        let d = self.data();
        if d.has_pos {
            d.x
        } else {
            default
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#getY
    pub fn get_y(&self, default: f64) -> f64 {
        let d = self.data();
        if d.has_pos {
            d.y
        } else {
            default
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#getZ
    pub fn get_z(&self, default: f64) -> f64 {
        let d = self.data();
        if d.has_pos {
            d.z
        } else {
            default
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#getYRot
    pub fn get_y_rot(&self, default: f32) -> f32 {
        let d = self.data();
        if d.has_rot {
            d.y_rot
        } else {
            default
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#getXRot
    pub fn get_x_rot(&self, default: f32) -> f32 {
        let d = self.data();
        if d.has_rot {
            d.x_rot
        } else {
            default
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#isOnGround
    pub fn is_on_ground(&self) -> bool {
        self.data().on_ground
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#horizontalCollision
    pub fn horizontal_collision(&self) -> bool {
        self.data().horizontal_collision
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#hasPosition
    pub fn has_position(&self) -> bool {
        self.data().has_pos
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#hasRotation
    pub fn has_rotation(&self) -> bool {
        self.data().has_rot
    }

    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        match self {
            Self::Pos(p) => p.write(buf),
            Self::PosRot(p) => p.write(buf),
            Self::Rot(p) => p.write(buf),
            Self::StatusOnly(p) => p.write(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_flags_all_combinations() {
        assert_eq!(pack_flags(false, false), 0);
        assert_eq!(pack_flags(true, false), 1);
        assert_eq!(pack_flags(false, true), 2);
        assert_eq!(pack_flags(true, true), 3);

        assert!(unpack_on_ground(1));
        assert!(!unpack_on_ground(2));
        assert!(unpack_horizontal_collision(2));
        assert!(!unpack_horizontal_collision(1));

        // Extra/high bits and negative values should be handled by Java int bit semantics.
        assert!(unpack_on_ground(0xFF));
        assert!(unpack_horizontal_collision(0xFE));
        assert!(!unpack_on_ground(0xFE));
        assert!(unpack_on_ground(-1));
        assert!(unpack_horizontal_collision(-1));
        assert!(!unpack_on_ground(-2));
    }

    #[test]
    fn pos_round_trip_preserves_bits() {
        let pkt = Pos::new(-1.5e308, f64::MIN_POSITIVE, f64::NAN, true, false);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf).unwrap();

        let decoded = Pos::read(&mut FriendlyByteBuf::from_bytes(buf.into_bytes())).unwrap();
        assert_eq!(decoded.data.x.to_bits(), pkt.data.x.to_bits());
        assert_eq!(decoded.data.y.to_bits(), pkt.data.y.to_bits());
        assert_eq!(decoded.data.z.to_bits(), pkt.data.z.to_bits());
        assert!(decoded.data.on_ground);
        assert!(!decoded.data.horizontal_collision);
        assert!(decoded.data.has_pos);
        assert!(!decoded.data.has_rot);
        assert_eq!(decoded.packet_type(), PacketType::ServerboundMovePlayerPos);
    }

    #[test]
    fn pos_rot_round_trip_preserves_float_and_double_bits() {
        let y_rot = f32::from_bits(0xFFC0_0001); // NaN payload
        let x_rot = f32::NEG_INFINITY;
        let pkt = PosRot::new(
            f64::NEG_INFINITY,
            -0.0,
            123.456,
            y_rot,
            x_rot,
            false,
            true,
        );
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf).unwrap();

        let decoded = PosRot::read(&mut FriendlyByteBuf::from_bytes(buf.into_bytes())).unwrap();
        assert_eq!(decoded.data.x.to_bits(), f64::NEG_INFINITY.to_bits());
        assert_eq!(decoded.data.y.to_bits(), (-0.0f64).to_bits());
        assert_eq!(decoded.data.z.to_bits(), 123.456f64.to_bits());
        assert_eq!(decoded.data.y_rot.to_bits(), y_rot.to_bits());
        assert_eq!(decoded.data.x_rot.to_bits(), x_rot.to_bits());
        assert!(!decoded.data.on_ground);
        assert!(decoded.data.horizontal_collision);
        assert!(decoded.data.has_pos);
        assert!(decoded.data.has_rot);
        assert_eq!(
            decoded.packet_type(),
            PacketType::ServerboundMovePlayerPosRot
        );
    }

    #[test]
    fn rot_and_status_only_round_trip() {
        let rot = Rot::new(180.0, -90.0, true, true);
        let mut buf = FriendlyByteBuf::new();
        rot.write(&mut buf).unwrap();
        let decoded_rot = Rot::read(&mut FriendlyByteBuf::from_bytes(buf.into_bytes())).unwrap();
        assert_eq!(decoded_rot.data.y_rot.to_bits(), 180.0f32.to_bits());
        assert_eq!(decoded_rot.data.x_rot.to_bits(), (-90.0f32).to_bits());
        assert!(decoded_rot.data.on_ground);
        assert!(decoded_rot.data.horizontal_collision);
        assert!(!decoded_rot.data.has_pos);
        assert!(decoded_rot.data.has_rot);
        assert_eq!(decoded_rot.packet_type(), PacketType::ServerboundMovePlayerRot);

        let status = StatusOnly::new(false, true);
        let mut buf2 = FriendlyByteBuf::new();
        status.write(&mut buf2).unwrap();
        let decoded_status =
            StatusOnly::read(&mut FriendlyByteBuf::from_bytes(buf2.into_bytes())).unwrap();
        assert!(!decoded_status.data.on_ground);
        assert!(decoded_status.data.horizontal_collision);
        assert!(!decoded_status.data.has_pos);
        assert!(!decoded_status.data.has_rot);
        assert_eq!(
            decoded_status.packet_type(),
            PacketType::ServerboundMovePlayerStatusOnly
        );
    }

    #[test]
    fn getters_return_defaults_when_absent_and_handle_dispatches() {
        struct MockListener {
            called: bool,
            has_pos: bool,
            on_ground: bool,
        }
        impl ServerGamePacketListener for MockListener {
            fn handle_move_player(&mut self, packet: &ServerboundMovePlayerPacket) {
                self.called = true;
                self.has_pos = packet.has_position();
                self.on_ground = packet.is_on_ground();
            }
        }

        let rot_pkt = ServerboundMovePlayerPacket::Rot(Rot::new(1.0, 2.0, false, false));
        assert_eq!(rot_pkt.get_x(99.0), 99.0);
        assert_eq!(rot_pkt.get_y(-1.0), -1.0);
        assert!(rot_pkt.get_z(f64::NAN).is_nan());
        assert_eq!(rot_pkt.get_y_rot(0.0).to_bits(), 1.0f32.to_bits());
        assert_eq!(rot_pkt.get_x_rot(0.0).to_bits(), 2.0f32.to_bits());
        assert!(!rot_pkt.has_position());
        assert!(rot_pkt.has_rotation());

        let pos_pkt = ServerboundMovePlayerPacket::Pos(Pos::from_vec(
            Vec3::new(1.0, 2.0, 3.0),
            true,
            false,
        ));
        assert_eq!(pos_pkt.get_x(0.0), 1.0);
        assert_eq!(pos_pkt.get_y(0.0), 2.0);
        assert_eq!(pos_pkt.get_z(0.0), 3.0);
        assert_eq!(pos_pkt.get_y_rot(42.0).to_bits(), 42.0f32.to_bits());
        assert_eq!(pos_pkt.get_x_rot(-7.0).to_bits(), (-7.0f32).to_bits());
        assert!(pos_pkt.has_position());
        assert!(!pos_pkt.has_rotation());

        let mut listener = MockListener {
            called: false,
            has_pos: true,
            on_ground: false,
        };
        let status_pkt = ServerboundMovePlayerPacket::StatusOnly(StatusOnly::new(true, false));
        status_pkt.handle(&mut listener);
        assert!(listener.called);
        assert!(!listener.has_pos);
        assert!(listener.on_ground);
    }

    #[test]
    fn enum_write_dispatch_and_packet_types() {
        let packets = [
            ServerboundMovePlayerPacket::Pos(Pos::new(1.0, 2.0, 3.0, true, false)),
            ServerboundMovePlayerPacket::PosRot(PosRot::new(1.0, 2.0, 3.0, 4.0, 5.0, false, true)),
            ServerboundMovePlayerPacket::Rot(Rot::new(4.0, 5.0, true, false)),
            ServerboundMovePlayerPacket::StatusOnly(StatusOnly::new(false, true)),
        ];
        let expected = [
            PacketType::ServerboundMovePlayerPos,
            PacketType::ServerboundMovePlayerPosRot,
            PacketType::ServerboundMovePlayerRot,
            PacketType::ServerboundMovePlayerStatusOnly,
        ];

        for (pkt, ty) in packets.iter().zip(expected.iter()) {
            assert_eq!(pkt.packet_type(), *ty);
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf).unwrap();
            assert!(!buf.bytes().is_empty());
        }
    }
}
