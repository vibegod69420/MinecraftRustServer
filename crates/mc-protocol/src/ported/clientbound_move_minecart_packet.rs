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

#[derive(Debug, Clone, PartialEq)]
pub struct MinecartStep {
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub weight: f32,
}

impl MinecartStep {
    // JAVA: net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior$MinecartStep#<init>
    pub fn new(position: Vec3, movement: Vec3, y_rot: f32, x_rot: f32, weight: f32) -> Self {
        Self {
            position,
            movement,
            y_rot,
            x_rot,
            weight,
        }
    }

    // JAVA: net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior$MinecartStep#STREAM_CODEC
    pub fn read(buf: &mut impl Buf) -> Self {
        let position = Vec3::read(buf);
        let movement = Vec3::read(buf);
        let y_rot = read_rotation_byte(buf);
        let x_rot = read_rotation_byte(buf);
        let weight = buf.get_f32();
        Self {
            position,
            movement,
            y_rot,
            x_rot,
            weight,
        }
    }

    // JAVA: net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior$MinecartStep#STREAM_CODEC
    pub fn write(&self, buf: &mut impl BufMut) {
        self.position.write(buf);
        self.movement.write(buf);
        write_rotation_byte(buf, self.y_rot);
        write_rotation_byte(buf, self.x_rot);
        buf.put_f32(self.weight);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundMoveMinecartPacket {
    entity_id: i32,
    lerp_steps: Vec<MinecartStep>,
}

impl ClientboundMoveMinecartPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket#<init>(int,List)
    pub fn new(entity_id: i32, lerp_steps: Vec<MinecartStep>) -> Self {
        Self {
            entity_id,
            lerp_steps,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let entity_id = read_varint(buf);
        let lerp_steps = read_list(buf, |b| MinecartStep::read(b));
        Self {
            entity_id,
            lerp_steps,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_varint(buf, self.entity_id);
        write_list(buf, &self.lerp_steps, |b, step| step.write(b));
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket#entityId
    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket#lerpSteps
    pub fn lerp_steps(&self) -> &[MinecartStep] {
        &self.lerp_steps
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_varint(buf: &mut impl Buf) -> i32 {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32) << shift;
        if (b & 0x80) == 0 {
            break;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            panic!("VarInt too big");
        }
    }
    result
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_varint(buf: &mut impl BufMut, mut value: i32) {
    loop {
        let mut b = (value as u32 & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            b |= 0x80;
        }
        buf.put_u8(b);
        if value == 0 {
            break;
        }
    }
}

// JAVA: net.minecraft.network.codec.ByteBufCodecs#ROTATION_BYTE
fn read_rotation_byte(buf: &mut impl Buf) -> f32 {
    (buf.get_i8() as f32) * 360.0f32 / 256.0f32
}

// JAVA: net.minecraft.network.codec.ByteBufCodecs#ROTATION_BYTE
fn write_rotation_byte(buf: &mut impl BufMut, val: f32) {
    let v = (val * 256.0f32 / 360.0f32).floor() as i32;
    buf.put_i8(v as i8);
}

// JAVA: net.minecraft.network.codec.ByteBufCodecs#list
fn read_list<T, B: Buf>(buf: &mut B, reader: impl Fn(&mut B) -> T) -> Vec<T> {
    let len = read_varint(buf) as usize;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        out.push(reader(buf));
    }
    out
}

// JAVA: net.minecraft.network.codec.ByteBufCodecs#list
fn write_list<T, B: BufMut>(buf: &mut B, list: &[T], writer: impl Fn(&mut B, &T)) {
    write_varint(buf, list.len() as i32);
    for item in list {
        writer(buf, item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_empty_steps() {
        let pkt = ClientboundMoveMinecartPacket::new(42, vec![]);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundMoveMinecartPacket::read(&mut reader);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.entity_id(), 42);
        assert!(decoded.lerp_steps().is_empty());
    }

    #[test]
    fn roundtrip_single_step_preserves_float_bits() {
        let step = MinecartStep::new(
            Vec3::new(1.5, -2.25, 3.125),
            Vec3::new(0.0, -1.0, f64::from_bits(0xBFF0_0000_0000_0001)),
            90.0,
            -45.0,
            f32::from_bits(0x3F8C_CCCD),
        );
        let pkt = ClientboundMoveMinecartPacket::new(7, vec![step.clone()]);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundMoveMinecartPacket::read(&mut reader);
        assert_eq!(decoded.entity_id(), 7);
        assert_eq!(decoded.lerp_steps().len(), 1);
        let ds = &decoded.lerp_steps()[0];
        assert_eq!(ds.position.x.to_bits(), step.position.x.to_bits());
        assert_eq!(ds.position.y.to_bits(), step.position.y.to_bits());
        assert_eq!(ds.position.z.to_bits(), step.position.z.to_bits());
        assert_eq!(ds.movement.x.to_bits(), step.movement.x.to_bits());
        assert_eq!(ds.movement.y.to_bits(), step.movement.y.to_bits());
        assert_eq!(ds.movement.z.to_bits(), step.movement.z.to_bits());
        assert_eq!(ds.weight.to_bits(), step.weight.to_bits());
    }

    #[test]
    fn roundtrip_negative_entity_id_and_multiple_steps() {
        let steps = vec![
            MinecartStep::new(
                Vec3::new(0.0, 64.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                0.0,
                0.0,
                1.0,
            ),
            MinecartStep::new(
                Vec3::new(1.0, 64.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                180.0,
                -90.0,
                0.5,
            ),
        ];
        let pkt = ClientboundMoveMinecartPacket::new(-1, steps);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundMoveMinecartPacket::read(&mut reader);
        assert_eq!(decoded.entity_id(), -1);
        assert_eq!(decoded.lerp_steps().len(), 2);
        assert_eq!(decoded.lerp_steps()[1].weight.to_bits(), 0.5f32.to_bits());
    }

    #[test]
    fn rotation_byte_quantization() {
        let mut buf = BytesMut::new();
        write_rotation_byte(&mut buf, 90.0);
        write_rotation_byte(&mut buf, -180.0);
        write_rotation_byte(&mut buf, 360.0);
        let mut reader = buf.freeze();
        assert_eq!(read_rotation_byte(&mut reader).to_bits(), 90.0f32.to_bits());
        assert_eq!(
            read_rotation_byte(&mut reader).to_bits(),
            (-180.0f32).to_bits()
        );
        assert_eq!(read_rotation_byte(&mut reader).to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn varint_roundtrip_negative_and_min() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, -1);
        write_varint(&mut buf, i32::MIN);
        write_varint(&mut buf, i32::MAX);
        let mut reader = buf.freeze();
        assert_eq!(read_varint(&mut reader), -1);
        assert_eq!(read_varint(&mut reader), i32::MIN);
        assert_eq!(read_varint(&mut reader), i32::MAX);
    }
}
