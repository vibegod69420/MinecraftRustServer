use bytes::{Buf, BufMut, BytesMut};

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

    // JAVA: net.minecraft.network.codec.StreamCodec#Vec3_STREAM_CODEC_decode
    fn read<B: Buf>(buf: &mut B) -> Self {
        Self {
            x: buf.get_f64(),
            y: buf.get_f64(),
            z: buf.get_f64(),
        }
    }

    // JAVA: net.minecraft.network.codec.StreamCodec#Vec3_STREAM_CODEC_encode
    fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_f64(self.x);
        buf.put_f64(self.y);
        buf.put_f64(self.z);
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

    // JAVA: net.minecraft.world.entity.PositionMoveRotation#STREAM_CODEC_decode
    fn read<B: Buf>(buf: &mut B) -> Self {
        let position = Vec3::read(buf);
        let delta_movement = Vec3::read(buf);
        let y_rot = buf.get_f32();
        let x_rot = buf.get_f32();
        Self::new(position, delta_movement, y_rot, x_rot)
    }

    // JAVA: net.minecraft.world.entity.PositionMoveRotation#STREAM_CODEC_encode
    fn write<B: BufMut>(&self, buf: &mut B) {
        self.position.write(buf);
        self.delta_movement.write(buf);
        buf.put_f32(self.y_rot);
        buf.put_f32(self.x_rot);
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<B: Buf>(buf: &mut B) -> i32 {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8();
        result |= ((b & 0x7f) as i32) << shift;
        if (b & 0x80) == 0 {
            break;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            break;
        }
    }
    result
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<B: BufMut>(buf: &mut B, value: i32) {
    let mut val = value as u32;
    loop {
        if (val & !0x7f) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7f) as u8) | 0x80);
        val >>= 7;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
fn read_bool<B: Buf>(buf: &mut B) -> bool {
    buf.get_u8() != 0
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
fn write_bool<B: BufMut>(buf: &mut B, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub trait ClientGamePacketListener {
    fn handle_entity_position_sync(&mut self, packet: &ClientboundEntityPositionSyncPacket);
}

pub trait EntityAccess {
    fn get_id(&self) -> i32;
    fn tracking_position(&self) -> Vec3;
    fn get_delta_movement(&self) -> Vec3;
    fn get_y_rot(&self) -> f32;
    fn get_x_rot(&self) -> f32;
    fn on_ground(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundEntityPositionSyncPacket {
    pub id: i32,
    pub values: PositionMoveRotation,
    pub on_ground: bool,
}

impl ClientboundEntityPositionSyncPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket#<init>
    pub fn new(id: i32, values: PositionMoveRotation, on_ground: bool) -> Self {
        Self {
            id,
            values,
            on_ground,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket#of
    pub fn of(entity: &impl EntityAccess) -> Self {
        Self::new(
            entity.get_id(),
            PositionMoveRotation::new(
                entity.tracking_position(),
                entity.get_delta_movement(),
                entity.get_y_rot(),
                entity.get_x_rot(),
            ),
            entity.on_ground(),
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket#STREAM_CODEC_decode
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let id = read_var_int(buf);
        let values = PositionMoveRotation::read(buf);
        let on_ground = read_bool(buf);
        Self::new(id, values, on_ground)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket#STREAM_CODEC_encode
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_var_int(buf, self.id);
        self.values.write(buf);
        write_bool(buf, self.on_ground);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType {
            id: "minecraft:clientbound_entity_position_sync",
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_entity_position_sync(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEntity {
        id: i32,
        pos: Vec3,
        delta: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
    }

    impl EntityAccess for MockEntity {
        fn get_id(&self) -> i32 {
            self.id
        }
        fn tracking_position(&self) -> Vec3 {
            self.pos
        }
        fn get_delta_movement(&self) -> Vec3 {
            self.delta
        }
        fn get_y_rot(&self) -> f32 {
            self.y_rot
        }
        fn get_x_rot(&self) -> f32 {
            self.x_rot
        }
        fn on_ground(&self) -> bool {
            self.on_ground
        }
    }

    struct MockListener {
        received_id: Option<i32>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_entity_position_sync(&mut self, packet: &ClientboundEntityPositionSyncPacket) {
            self.received_id = Some(packet.id);
        }
    }

    #[test]
    fn round_trip_encode_decode() {
        let packet = ClientboundEntityPositionSyncPacket::new(
            42,
            PositionMoveRotation::new(
                Vec3::new(1.5, -64.0, 256.25),
                Vec3::new(0.1, -0.2, 0.0),
                90.0,
                -45.0,
            ),
            true,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut bytes = buf.freeze();
        let decoded = ClientboundEntityPositionSyncPacket::read(&mut bytes);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn of_constructs_from_entity() {
        let entity = MockEntity {
            id: -7,
            pos: Vec3::new(10.0, 20.0, 30.0),
            delta: Vec3::new(-1.0, 0.5, 2.0),
            y_rot: 180.0,
            x_rot: -90.0,
            on_ground: false,
        };
        let packet = ClientboundEntityPositionSyncPacket::of(&entity);
        assert_eq!(packet.id, -7);
        assert_eq!(packet.values.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(packet.values.delta_movement, Vec3::new(-1.0, 0.5, 2.0));
        assert_eq!(packet.values.y_rot, 180.0);
        assert_eq!(packet.values.x_rot, -90.0);
        assert!(!packet.on_ground);
    }

    #[test]
    fn var_int_negative_and_large_values() {
        let packet = ClientboundEntityPositionSyncPacket::new(
            -1,
            PositionMoveRotation::new(
                Vec3::new(f64::MIN_POSITIVE, f64::MAX, f64::NEG_INFINITY),
                Vec3::new(0.0, -0.0, f64::INFINITY),
                f32::NAN,
                -0.0,
            ),
            false,
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut bytes = buf.freeze();
        let decoded = ClientboundEntityPositionSyncPacket::read(&mut bytes);
        assert_eq!(decoded.id, -1);
        assert_eq!(decoded.values.position.x.to_bits(), f64::MIN_POSITIVE.to_bits());
        assert_eq!(decoded.values.position.y.to_bits(), f64::MAX.to_bits());
        assert_eq!(decoded.values.position.z.to_bits(), f64::NEG_INFINITY.to_bits());
        assert_eq!(decoded.values.delta_movement.x.to_bits(), 0.0f64.to_bits());
        assert_eq!(decoded.values.delta_movement.y.to_bits(), (-0.0f64).to_bits());
        assert_eq!(decoded.values.delta_movement.z.to_bits(), f64::INFINITY.to_bits());
        assert!(decoded.values.y_rot.is_nan());
        assert_eq!(decoded.values.x_rot.to_bits(), (-0.0f32).to_bits());
        assert!(!decoded.on_ground);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packet = ClientboundEntityPositionSyncPacket::new(
            123,
            PositionMoveRotation::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0), 0.0, 0.0),
            true,
        );
        let mut listener = MockListener { received_id: None };
        packet.handle(&mut listener);
        assert_eq!(listener.received_id, Some(123));
        assert_eq!(packet.packet_type().id, "minecraft:clientbound_entity_position_sync");
    }
}
