use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readFloat
fn read_f32(buf: &mut &[u8]) -> Result<f32, DecodeError> {
    if buf.len() < 4 {
        return Err(DecodeError {
            message: "Not enough bytes for f32".to_string(),
        });
    }
    let bytes: [u8; 4] = buf[..4].try_into().unwrap();
    *buf = &buf[4..];
    Ok(f32::from_be_bytes(bytes))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeFloat
fn write_f32(buf: &mut Vec<u8>, value: f32) {
    buf.extend_from_slice(&value.to_be_bytes());
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readDouble
fn read_f64(buf: &mut &[u8]) -> Result<f64, DecodeError> {
    if buf.len() < 8 {
        return Err(DecodeError {
            message: "Not enough bytes for f64".to_string(),
        });
    }
    let bytes: [u8; 8] = buf[..8].try_into().unwrap();
    *buf = &buf[8..];
    Ok(f64::from_be_bytes(bytes))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeDouble
fn write_f64(buf: &mut Vec<u8>, value: f64) {
    buf.extend_from_slice(&value.to_be_bytes());
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

    // JAVA: net.minecraft.world.phys.Vec3#STREAM_CODEC
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let x = read_f64(buf)?;
        let y = read_f64(buf)?;
        let z = read_f64(buf)?;
        Ok(Self { x, y, z })
    }

    // JAVA: net.minecraft.world.phys.Vec3#STREAM_CODEC
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_f64(buf, self.x);
        write_f64(buf, self.y);
        write_f64(buf, self.z);
    }
}

pub trait EntityLike {
    fn position(&self) -> Vec3;
    fn y_rot(&self) -> f32;
    fn x_rot(&self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundMoveVehiclePacket {
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
}

impl ClientboundMoveVehiclePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket#<init>
    pub fn new(position: Vec3, y_rot: f32, x_rot: f32) -> Self {
        Self {
            position,
            y_rot,
            x_rot,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket#fromEntity
    pub fn from_entity<E: EntityLike>(entity: &E) -> Self {
        Self::new(entity.position(), entity.y_rot(), entity.x_rot())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket#STREAM_CODEC
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let position = Vec3::read(buf)?;
        let y_rot = read_f32(buf)?;
        let x_rot = read_f32(buf)?;
        Ok(Self {
            position,
            y_rot,
            x_rot,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket#STREAM_CODEC
    pub fn write(&self, buf: &mut Vec<u8>) {
        self.position.write(buf);
        write_f32(buf, self.y_rot);
        write_f32(buf, self.x_rot);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket#type
    pub fn packet_type() -> &'static str {
        "minecraft:move_vehicle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEntity {
        pos: Vec3,
        y_rot: f32,
        x_rot: f32,
    }

    impl EntityLike for MockEntity {
        fn position(&self) -> Vec3 {
            self.pos
        }

        fn y_rot(&self) -> f32 {
            self.y_rot
        }

        fn x_rot(&self) -> f32 {
            self.x_rot
        }
    }

    #[test]
    fn round_trip() {
        let pkt = ClientboundMoveVehiclePacket::new(
            Vec3::new(1.5, -64.0, 3.25),
            90.0,
            -45.0,
        );
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundMoveVehiclePacket::read(&mut slice).unwrap();
        assert_eq!(decoded, pkt);
        assert!(slice.is_empty());
    }

    #[test]
    fn from_entity() {
        let e = MockEntity {
            pos: Vec3::new(10.0, 20.0, 30.0),
            y_rot: 180.0,
            x_rot: 0.0,
        };
        let pkt = ClientboundMoveVehiclePacket::from_entity(&e);
        assert_eq!(pkt.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(pkt.y_rot.to_bits(), 180.0f32.to_bits());
        assert_eq!(pkt.x_rot.to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn float_and_double_bits_preserved() {
        let y_rot = f32::from_bits(0x7FC0_0001); // NaN payload
        let x_rot = -0.0f32;
        let pos = Vec3::new(-0.0f64, f64::from_bits(0x7FF8_0000_0000_0001), 1.0);
        let pkt = ClientboundMoveVehiclePacket::new(pos, y_rot, x_rot);
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundMoveVehiclePacket::read(&mut slice).unwrap();
        assert_eq!(decoded.y_rot.to_bits(), y_rot.to_bits());
        assert_eq!(decoded.x_rot.to_bits(), x_rot.to_bits());
        assert_eq!(decoded.position.x.to_bits(), pos.x.to_bits());
        assert_eq!(decoded.position.y.to_bits(), pos.y.to_bits());
    }

    #[test]
    fn negative_and_extreme_values() {
        let pkt = ClientboundMoveVehiclePacket::new(
            Vec3::new(-1.0e308, f64::MIN_POSITIVE, f64::INFINITY),
            f32::NEG_INFINITY,
            f32::MAX,
        );
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundMoveVehiclePacket::read(&mut slice).unwrap();
        assert_eq!(decoded.position.x.to_bits(), (-1.0e308f64).to_bits());
        assert_eq!(decoded.position.y.to_bits(), f64::MIN_POSITIVE.to_bits());
        assert_eq!(decoded.position.z.to_bits(), f64::INFINITY.to_bits());
        assert_eq!(decoded.y_rot.to_bits(), f32::NEG_INFINITY.to_bits());
        assert_eq!(decoded.x_rot.to_bits(), f32::MAX.to_bits());
    }

    #[test]
    fn short_buffer_errors() {
        let mut slice: &[u8] = &[0u8; 10];
        assert!(ClientboundMoveVehiclePacket::read(&mut slice).is_err());
    }
}
