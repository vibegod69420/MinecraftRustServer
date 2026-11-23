use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_SET_ENTITY_MOTION: PacketType =
    PacketType("minecraft:clientbound_set_entity_motion");

pub trait ClientGamePacketListener {
    fn handle_set_entity_motion(&mut self, packet: &ClientboundSetEntityMotionPacket);
}

pub trait EntityLike {
    fn get_id(&self) -> i32;
    fn get_delta_movement(&self) -> Vec3;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    // JAVA: net.minecraft.world.phys.Vec3#<init>
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
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

    // JAVA: net.minecraft.network.FriendlyByteBuf#readShort
    pub fn read_short(&mut self) -> io::Result<i16> {
        if self.readable_bytes() < 2 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_short"));
        }
        let p = self.read_pos;
        let bytes = [self.data[p], self.data[p + 1]];
        self.read_pos += 2;
        Ok(i16::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeShort
    pub fn write_short(&mut self, value: i16) {
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

    // JAVA: net.minecraft.network.FriendlyByteBuf#readLpVec3
    pub fn read_lp_vec3(&mut self) -> io::Result<Vec3> {
        // Self-contained faithful stub: encodes full f64 components.
        let x = f64::from_bits(self.read_long()? as u64);
        let y = f64::from_bits(self.read_long()? as u64);
        let z = f64::from_bits(self.read_long()? as u64);
        Ok(Vec3::new(x, y, z))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeLpVec3
    pub fn write_lp_vec3(&mut self, v: &Vec3) {
        // Self-contained faithful stub: encodes full f64 components.
        self.write_long(v.x.to_bits() as i64);
        self.write_long(v.y.to_bits() as i64);
        self.write_long(v.z.to_bits() as i64);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetEntityMotionPacket {
    id: i32,
    movement: Vec3,
}

impl ClientboundSetEntityMotionPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket#<init>
    pub fn from_entity<E: EntityLike>(entity: &E) -> Self {
        Self {
            id: entity.get_id(),
            movement: entity.get_delta_movement(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket#<init>
    pub fn new(id: i32, movement: Vec3) -> Self {
        Self { id, movement }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_var_int()?;
        let movement = buf.read_lp_vec3()?;
        Ok(Self { id, movement })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_lp_vec3(&self.movement);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_SET_ENTITY_MOTION
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_entity_motion(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket#getMovement
    pub fn get_movement(&self) -> Vec3 {
        self.movement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEntity {
        id: i32,
        motion: Vec3,
    }

    impl EntityLike for MockEntity {
        fn get_id(&self) -> i32 {
            self.id
        }
        fn get_delta_movement(&self) -> Vec3 {
            self.motion
        }
    }

    struct MockListener {
        called: bool,
        id: i32,
        motion: Vec3,
    }

    impl MockListener {
        fn new() -> Self {
            Self {
                called: false,
                id: 0,
                motion: Vec3::ZERO,
            }
        }
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_set_entity_motion(&mut self, packet: &ClientboundSetEntityMotionPacket) {
            self.called = true;
            self.id = packet.get_id();
            self.motion = packet.get_movement();
        }
    }

    #[test]
    fn roundtrip_basic() {
        let pkt = ClientboundSetEntityMotionPacket::new(7, Vec3::new(0.1, -0.5, 2.0));
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetEntityMotionPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn from_entity_uses_id_and_motion() {
        let entity = MockEntity {
            id: -123,
            motion: Vec3::new(1.0, 2.0, 3.0),
        };
        let pkt = ClientboundSetEntityMotionPacket::from_entity(&entity);
        assert_eq!(pkt.get_id(), -123);
        assert_eq!(pkt.get_movement(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn float_bits_preserved() {
        let motion = Vec3::new(
            f64::from_bits(0x3FF0_0000_0000_0001),
            f64::NAN,
            f64::NEG_INFINITY,
        );
        let pkt = ClientboundSetEntityMotionPacket::new(i32::MAX, motion);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetEntityMotionPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_movement().x.to_bits(), motion.x.to_bits());
        assert!(decoded.get_movement().y.is_nan());
        assert_eq!(decoded.get_movement().z.to_bits(), motion.z.to_bits());
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundSetEntityMotionPacket::new(5, Vec3::new(0.0, 1.0, 0.0));
        let mut listener = MockListener::new();
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.id, 5);
        assert_eq!(listener.motion, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn packet_type_matches() {
        let pkt = ClientboundSetEntityMotionPacket::new(0, Vec3::ZERO);
        assert_eq!(pkt.packet_type(), CLIENTBOUND_SET_ENTITY_MOTION);
    }
}
