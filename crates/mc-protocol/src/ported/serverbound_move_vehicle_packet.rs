use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    // JAVA: net.minecraft.world.phys.Vec3#<init>(double,double,double)
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.world.phys.Vec3#STREAM_CODEC
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            x: buf.get_f64(),
            y: buf.get_f64(),
            z: buf.get_f64(),
        }
    }

    // JAVA: net.minecraft.world.phys.Vec3#STREAM_CODEC
    pub fn write(&self, buf: &mut impl BufMut) {
        buf.put_f64(self.x);
        buf.put_f64(self.y);
        buf.put_f64(self.z);
    }
}

pub trait EntityLike {
    fn is_interpolating(&self) -> bool;
    fn interpolation_position(&self) -> Vec3;
    fn interpolation_y_rot(&self) -> f32;
    fn interpolation_x_rot(&self) -> f32;
    fn on_ground(&self) -> bool;
    fn position(&self) -> Vec3;
    fn y_rot(&self) -> f32;
    fn x_rot(&self) -> f32;
}

pub trait ServerGamePacketListener {
    fn handle_move_vehicle(&mut self, packet: &ServerboundMoveVehiclePacket);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundMoveVehiclePacket {
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
}

impl ServerboundMoveVehiclePacket {
    pub const PACKET_TYPE: &'static str = "minecraft:move_vehicle";

    // JAVA: net.minecraft.network.protocol.game.ServerboundMoveVehiclePacket#<init>
    pub fn new(position: Vec3, y_rot: f32, x_rot: f32, on_ground: bool) -> Self {
        Self {
            position,
            y_rot,
            x_rot,
            on_ground,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMoveVehiclePacket#fromEntity
    pub fn from_entity(entity: &impl EntityLike) -> Self {
        if entity.is_interpolating() {
            Self::new(
                entity.interpolation_position(),
                entity.interpolation_y_rot(),
                entity.interpolation_x_rot(),
                entity.on_ground(),
            )
        } else {
            Self::new(
                entity.position(),
                entity.y_rot(),
                entity.x_rot(),
                entity.on_ground(),
            )
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMoveVehiclePacket#STREAM_CODEC
    pub fn read(buf: &mut impl Buf) -> Self {
        let position = Vec3::read(buf);
        let y_rot = buf.get_f32();
        let x_rot = buf.get_f32();
        let on_ground = buf.get_u8() != 0;
        Self {
            position,
            y_rot,
            x_rot,
            on_ground,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMoveVehiclePacket#STREAM_CODEC
    pub fn write(&self, buf: &mut impl BufMut) {
        self.position.write(buf);
        buf.put_f32(self.y_rot);
        buf.put_f32(self.x_rot);
        buf.put_u8(if self.on_ground { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMoveVehiclePacket#type
    pub fn packet_type(&self) -> &'static str {
        Self::PACKET_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundMoveVehiclePacket#handle
    pub fn handle(&self, listener: &mut impl ServerGamePacketListener) {
        listener.handle_move_vehicle(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockEntity {
        interp: bool,
        ipos: Vec3,
        iy: f32,
        ix: f32,
        pos: Vec3,
        y: f32,
        x: f32,
        og: bool,
    }

    impl EntityLike for MockEntity {
        fn is_interpolating(&self) -> bool { self.interp }
        fn interpolation_position(&self) -> Vec3 { self.ipos }
        fn interpolation_y_rot(&self) -> f32 { self.iy }
        fn interpolation_x_rot(&self) -> f32 { self.ix }
        fn on_ground(&self) -> bool { self.og }
        fn position(&self) -> Vec3 { self.pos }
        fn y_rot(&self) -> f32 { self.y }
        fn x_rot(&self) -> f32 { self.x }
    }

    #[test]
    fn roundtrip_basic() {
        let pkt = ServerboundMoveVehiclePacket::new(
            Vec3::new(1.5, -64.0, 3.25),
            90.0f32,
            -45.0f32,
            true,
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundMoveVehiclePacket::read(&mut slice);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_float_bits() {
        let pkt = ServerboundMoveVehiclePacket::new(
            Vec3::new(f64::NAN, f64::INFINITY, f64::NEG_INFINITY),
            f32::NAN,
            f32::NEG_INFINITY,
            false,
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundMoveVehiclePacket::read(&mut slice);
        assert_eq!(decoded.position.x.to_bits(), f64::NAN.to_bits());
        assert_eq!(decoded.position.y.to_bits(), f64::INFINITY.to_bits());
        assert_eq!(decoded.position.z.to_bits(), f64::NEG_INFINITY.to_bits());
        assert_eq!(decoded.y_rot.to_bits(), f32::NAN.to_bits());
        assert_eq!(decoded.x_rot.to_bits(), f32::NEG_INFINITY.to_bits());
        assert!(!decoded.on_ground);
    }

    #[test]
    fn from_entity_interpolating() {
        let e = MockEntity {
            interp: true,
            ipos: Vec3::new(1.0, 2.0, 3.0),
            iy: 10.0,
            ix: 20.0,
            pos: Vec3::new(9.0, 9.0, 9.0),
            y: 90.0,
            x: 80.0,
            og: true,
        };
        let pkt = ServerboundMoveVehiclePacket::from_entity(&e);
        assert_eq!(pkt.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(pkt.y_rot.to_bits(), 10.0f32.to_bits());
        assert_eq!(pkt.x_rot.to_bits(), 20.0f32.to_bits());
        assert!(pkt.on_ground);
    }

    #[test]
    fn from_entity_not_interpolating() {
        let e = MockEntity {
            interp: false,
            ipos: Vec3::new(1.0, 2.0, 3.0),
            iy: 10.0,
            ix: 20.0,
            pos: Vec3::new(9.0, 8.0, 7.0),
            y: 90.0,
            x: 80.0,
            og: false,
        };
        let pkt = ServerboundMoveVehiclePacket::from_entity(&e);
        assert_eq!(pkt.position, Vec3::new(9.0, 8.0, 7.0));
        assert_eq!(pkt.y_rot.to_bits(), 90.0f32.to_bits());
        assert_eq!(pkt.x_rot.to_bits(), 80.0f32.to_bits());
        assert!(!pkt.on_ground);
    }

    #[test]
    fn packet_type_and_handle() {
        struct Listener { og: bool }
        impl ServerGamePacketListener for Listener {
            fn handle_move_vehicle(&mut self, packet: &ServerboundMoveVehiclePacket) {
                self.og = packet.on_ground;
            }
        }
        let pkt = ServerboundMoveVehiclePacket::new(Vec3::new(0.0, 0.0, 0.0), 0.0, 0.0, true);
        assert_eq!(pkt.packet_type(), "minecraft:move_vehicle");
        let mut l = Listener { og: false };
        pkt.handle(&mut l);
        assert!(l.og);
    }
}
