use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_PLAYER_ROTATION: PacketType = PacketType {
        id: "clientbound_player_rotation",
    };
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleRotatePlayer
    fn handle_rotate_player(&mut self, packet: &ClientboundPlayerRotationPacket);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundPlayerRotationPacket {
    pub y_rot: f32,
    pub relative_y: bool,
    pub x_rot: f32,
    pub relative_x: bool,
}

impl ClientboundPlayerRotationPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket#<init>
    pub fn new(y_rot: f32, relative_y: bool, x_rot: f32, relative_x: bool) -> Self {
        Self {
            y_rot,
            relative_y,
            x_rot,
            relative_x,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket#type
    pub fn get_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_PLAYER_ROTATION
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_rotate_player(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket#STREAM_CODEC
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let y_rot = buf.get_f32();
        let relative_y = buf.get_u8() != 0;
        let x_rot = buf.get_f32();
        let relative_x = buf.get_u8() != 0;
        Self::new(y_rot, relative_y, x_rot, relative_x)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket#STREAM_CODEC
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_f32(self.y_rot);
        buf.put_u8(if self.relative_y { 1 } else { 0 });
        buf.put_f32(self.x_rot);
        buf.put_u8(if self.relative_x { 1 } else { 0 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};

    #[test]
    fn round_trip_normal_values() {
        let packet = ClientboundPlayerRotationPacket::new(90.0, true, -45.0, false);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        assert_eq!(buf.len(), 10); // f32 + bool + f32 + bool
        let decoded = ClientboundPlayerRotationPacket::read(&mut buf.freeze());
        assert_eq!(decoded, packet);
    }

    #[test]
    fn round_trip_float_edge_bits() {
        let cases: [f32; 7] = [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            -0.0,
            f32::MIN,
            f32::MAX,
            f32::from_bits(0x8000_0001),
        ];
        for &y in &cases {
            for &x in &cases {
                let packet = ClientboundPlayerRotationPacket::new(y, false, x, true);
                let mut buf = BytesMut::new();
                packet.write(&mut buf);
                let decoded = ClientboundPlayerRotationPacket::read(&mut buf.freeze());
                assert_eq!(decoded.y_rot.to_bits(), y.to_bits());
                assert_eq!(decoded.x_rot.to_bits(), x.to_bits());
                assert_eq!(decoded.relative_y, false);
                assert_eq!(decoded.relative_x, true);
            }
        }
    }

    #[test]
    fn boolean_decodes_nonzero_as_true() {
        let mut buf = BytesMut::new();
        buf.put_f32(-1.0);
        buf.put_u8(2);
        buf.put_f32(180.0);
        buf.put_u8(0xFF);
        let decoded = ClientboundPlayerRotationPacket::read(&mut buf.freeze());
        assert_eq!(decoded.y_rot.to_bits(), (-1.0f32).to_bits());
        assert!(decoded.relative_y);
        assert_eq!(decoded.x_rot.to_bits(), 180.0f32.to_bits());
        assert!(decoded.relative_x);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct TestListener {
            called: bool,
            y_rot: f32,
            relative_y: bool,
            x_rot: f32,
            relative_x: bool,
        }

        impl ClientGamePacketListener for TestListener {
            fn handle_rotate_player(&mut self, packet: &ClientboundPlayerRotationPacket) {
                self.called = true;
                self.y_rot = packet.y_rot;
                self.relative_y = packet.relative_y;
                self.x_rot = packet.x_rot;
                self.relative_x = packet.relative_x;
            }
        }

        let packet = ClientboundPlayerRotationPacket::new(1.5, true, -2.5, false);
        let mut listener = TestListener {
            called: false,
            y_rot: 0.0,
            relative_y: false,
            x_rot: 0.0,
            relative_x: false,
        };
        packet.handle(&mut listener);

        assert!(listener.called);
        assert_eq!(listener.y_rot.to_bits(), 1.5f32.to_bits());
        assert!(listener.relative_y);
        assert_eq!(listener.x_rot.to_bits(), (-2.5f32).to_bits());
        assert!(!listener.relative_x);
    }

    #[test]
    fn type_is_player_rotation() {
        let packet = ClientboundPlayerRotationPacket::new(0.0, false, 0.0, false);
        assert_eq!(
            packet.get_type(),
            GamePacketTypes::CLIENTBOUND_PLAYER_ROTATION
        );
        assert_eq!(packet.get_type().id, "clientbound_player_rotation");
    }
}
