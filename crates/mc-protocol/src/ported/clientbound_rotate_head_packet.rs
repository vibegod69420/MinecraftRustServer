use bytes::{Buf, BufMut};

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<B: Buf>(buf: &mut B) -> i32 {
    let mut val: u32 = 0;
    let mut pos: u32 = 0;
    loop {
        let b = buf.get_u8();
        val |= ((b & 0x7F) as u32).wrapping_shl(pos.wrapping_mul(7));
        pos = pos.wrapping_add(1);
        if pos > 5 {
            panic!("VarInt too big");
        }
        if (b & 0x80) == 0 {
            break;
        }
    }
    val as i32
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value as u32 & !0x7F_u32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value as u8) & 0x7F) | 0x80);
        value = ((value as u32) >> 7) as i32;
    }
}

// JAVA: net.minecraft.util.Mth#unpackDegrees
pub fn unpack_degrees(angle: i8) -> f32 {
    (angle as f32 * 360.0_f32) / 256.0_f32
}

pub trait ClientGamePacketListener {
    fn handle_rotate_mob(&mut self, packet: &ClientboundRotateHeadPacket);
}

pub trait LevelAccess {
    type Entity;
    fn get_entity(&self, id: i32) -> Option<Self::Entity>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundRotateHeadPacket {
    entity_id: i32,
    y_head_rot: i8,
}

impl ClientboundRotateHeadPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundRotateHeadPacket#<init>(Entity, byte)
    pub fn new(entity_id: i32, y_head_rot: i8) -> Self {
        Self {
            entity_id,
            y_head_rot,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRotateHeadPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let entity_id = read_var_int(buf);
        let y_head_rot = buf.get_i8();
        Self {
            entity_id,
            y_head_rot,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRotateHeadPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_var_int(buf, self.entity_id);
        buf.put_i8(self.y_head_rot);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRotateHeadPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:rotate_head"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRotateHeadPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_rotate_mob(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRotateHeadPacket#getEntity
    pub fn get_entity<L: LevelAccess>(&self, level: &L) -> Option<L::Entity> {
        level.get_entity(self.entity_id)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRotateHeadPacket#getYHeadRot
    pub fn get_y_head_rot(&self) -> f32 {
        unpack_degrees(self.y_head_rot)
    }

    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    pub fn y_head_rot_raw(&self) -> i8 {
        self.y_head_rot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn write_read_roundtrip() {
        let packet = ClientboundRotateHeadPacket::new(42, 16);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundRotateHeadPacket::read(&mut reader);
        assert_eq!(packet, decoded);
    }

    #[test]
    fn negative_entity_id_roundtrip() {
        let packet = ClientboundRotateHeadPacket::new(-1, -128);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundRotateHeadPacket::read(&mut reader);
        assert_eq!(decoded.entity_id(), -1);
        assert_eq!(decoded.y_head_rot_raw(), -128);
    }

    #[test]
    fn unpack_degrees_values_and_float_bits() {
        assert_eq!(unpack_degrees(0).to_bits(), 0.0_f32.to_bits());
        let neg = unpack_degrees(-128);
        assert_eq!(neg.to_bits(), (-180.0_f32).to_bits());
        let pos = unpack_degrees(127);
        let expected = (127.0_f32 * 360.0_f32) / 256.0_f32;
        assert_eq!(pos.to_bits(), expected.to_bits());
    }

    #[test]
    fn get_y_head_rot_matches_unpack() {
        let packet = ClientboundRotateHeadPacket::new(7, 64);
        let expected = (64.0_f32 * 360.0_f32) / 256.0_f32;
        assert_eq!(packet.get_y_head_rot().to_bits(), expected.to_bits());
    }

    #[test]
    fn handle_calls_listener() {
        struct L {
            seen: Option<(i32, i8)>,
        }
        impl ClientGamePacketListener for L {
            fn handle_rotate_mob(&mut self, packet: &ClientboundRotateHeadPacket) {
                self.seen = Some((packet.entity_id(), packet.y_head_rot_raw()));
            }
        }
        let packet = ClientboundRotateHeadPacket::new(5, 10);
        let mut l = L { seen: None };
        packet.handle(&mut l);
        assert_eq!(l.seen, Some((5, 10)));
    }

    #[test]
    fn get_entity_uses_id() {
        struct L;
        impl LevelAccess for L {
            type Entity = i32;
            fn get_entity(&self, id: i32) -> Option<Self::Entity> {
                Some(id)
            }
        }
        let packet = ClientboundRotateHeadPacket::new(99, 0);
        assert_eq!(packet.get_entity(&L), Some(99));
    }
}
