use std::collections::BTreeSet;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_TELEPORT_ENTITY: PacketType =
    PacketType("minecraft:clientbound_teleport_entity");

pub trait ClientGamePacketListener {
    fn handle_teleport_entity(&mut self, packet: &ClientboundTeleportEntityPacket);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    // JAVA: net.minecraft.world.phys.Vec3#<init>
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionMoveRotation {
    pub position: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
}

impl PositionMoveRotation {
    // JAVA: net.minecraft.world.entity.PositionMoveRotation#<init>
    pub fn new(position: Vec3, delta_movement: Vec3, y_rot: f32, x_rot: f32) -> Self {
        Self {
            position,
            delta_movement,
            y_rot,
            x_rot,
        }
    }

    // JAVA: net.minecraft.world.entity.PositionMoveRotation#read (stream codec)
    fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let position = buf.read_vec3()?;
        let delta_movement = buf.read_vec3()?;
        let y_rot = buf.read_float()?;
        let x_rot = buf.read_float()?;
        Ok(Self {
            position,
            delta_movement,
            y_rot,
            x_rot,
        })
    }

    // JAVA: net.minecraft.world.entity.PositionMoveRotation#write (stream codec)
    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_vec3(&self.position);
        buf.write_vec3(&self.delta_movement);
        buf.write_float(self.y_rot);
        buf.write_float(self.x_rot);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Relative {
    X = 0,
    Y = 1,
    Z = 2,
    YRot = 3,
    XRot = 4,
    DeltaX = 5,
    DeltaY = 6,
    DeltaZ = 7,
    RotateDelta = 8,
}

impl Relative {
    // JAVA: net.minecraft.world.entity.Relative#pack
    pub fn pack(set: &BTreeSet<Relative>) -> i32 {
        let mut mask: i32 = 0;
        for rel in set {
            let bit = 1i32 << ((*rel as u32) & 31);
            mask |= bit;
        }
        mask
    }

    // JAVA: net.minecraft.world.entity.Relative#unpack
    pub fn unpack(mask: i32) -> BTreeSet<Relative> {
        let mut set = BTreeSet::new();
        let all = [
            Relative::X,
            Relative::Y,
            Relative::Z,
            Relative::YRot,
            Relative::XRot,
            Relative::DeltaX,
            Relative::DeltaY,
            Relative::DeltaZ,
            Relative::RotateDelta,
        ];
        for rel in all {
            let bit = 1i32 << ((rel as u32) & 31);
            if (mask & bit) != 0 {
                set.insert(rel);
            }
        }
        set
    }
}

#[derive(Debug, Clone, Default)]
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

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readableBytes
    pub fn readable_bytes(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writerIndex/getBytes
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    pub fn read_byte(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_byte"));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte
    pub fn write_byte(&mut self, value: u8) {
        self.data.push(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInt
    pub fn read_int(&mut self) -> io::Result<i32> {
        if self.readable_bytes() < 4 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_int"));
        }
        let p = self.read_pos;
        let bytes = [
            self.data[p],
            self.data[p + 1],
            self.data[p + 2],
            self.data[p + 3],
        ];
        self.read_pos += 4;
        Ok(i32::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInt
    pub fn write_int(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLong
    pub fn read_long(&mut self) -> io::Result<i64> {
        if self.readable_bytes() < 8 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_long"));
        }
        let p = self.read_pos;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[p..p + 8]);
        self.read_pos += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLong
    pub fn write_long(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readFloat
    pub fn read_float(&mut self) -> io::Result<f32> {
        if self.readable_bytes() < 4 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_float"));
        }
        let p = self.read_pos;
        let bytes = [
            self.data[p],
            self.data[p + 1],
            self.data[p + 2],
            self.data[p + 3],
        ];
        self.read_pos += 4;
        Ok(f32::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeFloat
    pub fn write_float(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readDouble
    pub fn read_double(&mut self) -> io::Result<f64> {
        if self.readable_bytes() < 8 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_double"));
        }
        let p = self.read_pos;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[p..p + 8]);
        self.read_pos += 8;
        Ok(f64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeDouble
    pub fn write_double(&mut self, value: f64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_byte()? != 0)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
    pub fn write_bool(&mut self, value: bool) {
        self.write_byte(if value { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let byte = self.read_byte()? as i32;
            value |= (byte & 0x7F) << (position & 31);
            if (byte & 0x80) == 0 {
                return Ok(value);
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.write_byte(val as u8);
                return;
            }
            self.write_byte(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVec3
    pub fn read_vec3(&mut self) -> io::Result<Vec3> {
        let x = self.read_double()?;
        let y = self.read_double()?;
        let z = self.read_double()?;
        Ok(Vec3::new(x, y, z))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVec3
    pub fn write_vec3(&mut self, v: &Vec3) {
        self.write_double(v.x);
        self.write_double(v.y);
        self.write_double(v.z);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundTeleportEntityPacket {
    id: i32,
    change: PositionMoveRotation,
    relatives: BTreeSet<Relative>,
    on_ground: bool,
}

impl ClientboundTeleportEntityPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#<init>
    pub fn new(
        id: i32,
        change: PositionMoveRotation,
        relatives: BTreeSet<Relative>,
        on_ground: bool,
    ) -> Self {
        Self {
            id,
            change,
            relatives,
            on_ground,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#teleport
    pub fn teleport(
        id: i32,
        change: PositionMoveRotation,
        relatives: BTreeSet<Relative>,
        on_ground: bool,
    ) -> Self {
        Self::new(id, change, relatives, on_ground)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#read (stream codec)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_var_int()?;
        let change = PositionMoveRotation::read(buf)?;
        let rel_mask = buf.read_int()?;
        let relatives = Relative::unpack(rel_mask);
        let on_ground = buf.read_bool()?;
        Ok(Self {
            id,
            change,
            relatives,
            on_ground,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#write (stream codec)
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        self.change.write(buf);
        buf.write_int(Relative::pack(&self.relatives));
        buf.write_bool(self.on_ground);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_TELEPORT_ENTITY
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_teleport_entity(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#id
    pub fn id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#change
    pub fn change(&self) -> &PositionMoveRotation {
        &self.change
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#relatives
    pub fn relatives(&self) -> &BTreeSet<Relative> {
        &self.relatives
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket#onGround
    pub fn on_ground(&self) -> bool {
        self.on_ground
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        id: i32,
        on_ground: bool,
    }

    impl MockListener {
        fn new() -> Self {
            Self {
                called: false,
                id: 0,
                on_ground: false,
            }
        }
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_teleport_entity(&mut self, packet: &ClientboundTeleportEntityPacket) {
            self.called = true;
            self.id = packet.id();
            self.on_ground = packet.on_ground();
        }
    }

    fn sample_packet() -> ClientboundTeleportEntityPacket {
        let change = PositionMoveRotation::new(
            Vec3::new(1.5, -64.0, 3.25),
            Vec3::new(0.0, 0.1, -0.2),
            90.0f32,
            -45.0f32,
        );
        let mut rel = BTreeSet::new();
        rel.insert(Relative::X);
        rel.insert(Relative::YRot);
        rel.insert(Relative::DeltaZ);
        ClientboundTeleportEntityPacket::teleport(42, change, rel, true)
    }

    #[test]
    fn roundtrip_basic() {
        let pkt = sample_packet();
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundTeleportEntityPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn negative_and_extreme_values() {
        let change = PositionMoveRotation::new(
            Vec3::new(f64::MIN_POSITIVE, -0.0, f64::from_bits(0xBFF0_0000_0000_0000)),
            Vec3::new(f64::INFINITY, f64::NEG_INFINITY, f64::NAN),
            f32::from_bits(0xBF80_0000),
            f32::from_bits(0x7FC0_0001),
        );
        let pkt = ClientboundTeleportEntityPacket::new(i32::MIN, change, BTreeSet::new(), false);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundTeleportEntityPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.id(), i32::MIN);
        assert!(!decoded.on_ground());
        assert_eq!(decoded.change().y_rot.to_bits(), 0xBF80_0000);
        assert_eq!(decoded.change().x_rot.to_bits(), 0x7FC0_0001);
        assert_eq!(decoded.change().position.x.to_bits(), f64::MIN_POSITIVE.to_bits());
        assert_eq!(
            decoded.change().position.z.to_bits(),
            0xBFF0_0000_0000_0000
        );
        assert!(decoded.change().delta_movement.z.is_nan());
    }

    #[test]
    fn relatives_pack_unpack_all_flags() {
        let mut all = BTreeSet::new();
        for r in [
            Relative::X,
            Relative::Y,
            Relative::Z,
            Relative::YRot,
            Relative::XRot,
            Relative::DeltaX,
            Relative::DeltaY,
            Relative::DeltaZ,
            Relative::RotateDelta,
        ] {
            all.insert(r);
        }
        let mask = Relative::pack(&all);
        let unpacked = Relative::unpack(mask);
        assert_eq!(all, unpacked);
        assert_eq!(Relative::unpack(0), BTreeSet::new());
    }

    #[test]
    fn handle_dispatches() {
        let pkt = sample_packet();
        let mut listener = MockListener::new();
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.id, 42);
        assert!(listener.on_ground);
    }

    #[test]
    fn packet_type_matches() {
        let pkt = sample_packet();
        assert_eq!(pkt.packet_type(), CLIENTBOUND_TELEPORT_ENTITY);
    }
}
