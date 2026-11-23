use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionHand {
    MainHand,
    OffHand,
}

impl InteractionHand {
    // JAVA: net.minecraft.network.FriendlyByteBuf#readEnum
    fn read_from_buf(buf: &mut impl Buf) -> Self {
        let ordinal = read_var_int(buf);
        match ordinal {
            0 => InteractionHand::MainHand,
            1 => InteractionHand::OffHand,
            _ => InteractionHand::MainHand,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeEnum
    fn write_to_buf(self, buf: &mut impl BufMut) {
        let ordinal = match self {
            InteractionHand::MainHand => 0i32,
            InteractionHand::OffHand => 1i32,
        };
        write_var_int(buf, ordinal);
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32) << (shift & 31);
        if (b & 0x80) == 0 {
            return result;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return result;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut impl BufMut, value: i32) {
    let mut v = value as u32;
    loop {
        if (v & !0x7Fu32) == 0 {
            buf.put_u8(v as u8);
            return;
        }
        buf.put_u8(((v & 0x7F) as u8) | 0x80);
        v >>= 7;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundUseItemPacket {
    hand: InteractionHand,
    sequence: i32,
    y_rot: f32,
    x_rot: f32,
}

impl ServerboundUseItemPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#<init>(InteractionHand,int,float,float)
    pub fn new(hand: InteractionHand, sequence: i32, y_rot: f32, x_rot: f32) -> Self {
        Self {
            hand,
            sequence,
            y_rot,
            x_rot,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let hand = InteractionHand::read_from_buf(buf);
        let sequence = read_var_int(buf);
        let y_rot = buf.get_f32();
        let x_rot = buf.get_f32();
        Self {
            hand,
            sequence,
            y_rot,
            x_rot,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        self.hand.write_to_buf(buf);
        write_var_int(buf, self.sequence);
        buf.put_f32(self.y_rot);
        buf.put_f32(self.x_rot);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:use_item"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#handle
    pub fn handle<H: ServerGamePacketListener>(&self, handler: &mut H) {
        handler.handle_use_item(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#getHand
    pub fn get_hand(&self) -> InteractionHand {
        self.hand
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#getSequence
    pub fn get_sequence(&self) -> i32 {
        self.sequence
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#getYRot
    pub fn get_y_rot(&self) -> f32 {
        self.y_rot
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundUseItemPacket#getXRot
    pub fn get_x_rot(&self) -> f32 {
        self.x_rot
    }
}

pub trait ServerGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerGamePacketListener#handleUseItem
    fn handle_use_item(&mut self, packet: &ServerboundUseItemPacket);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_main_hand() {
        let pkt = ServerboundUseItemPacket::new(InteractionHand::MainHand, 42, 90.0, -45.0);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundUseItemPacket::read(&mut slice);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.packet_type(), "minecraft:use_item");
    }

    #[test]
    fn roundtrip_off_hand_negative_sequence() {
        let pkt = ServerboundUseItemPacket::new(InteractionHand::OffHand, -1, 0.0, 0.0);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundUseItemPacket::read(&mut slice);
        assert_eq!(decoded.get_hand(), InteractionHand::OffHand);
        assert_eq!(decoded.get_sequence(), -1);
    }

    #[test]
    fn float_bits_preserved_nan_and_infinity() {
        let y_rot = f32::from_bits(0x7FC0_0001);
        let x_rot = f32::NEG_INFINITY;
        let pkt = ServerboundUseItemPacket::new(InteractionHand::MainHand, 0, y_rot, x_rot);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundUseItemPacket::read(&mut slice);
        assert_eq!(decoded.get_y_rot().to_bits(), y_rot.to_bits());
        assert_eq!(decoded.get_x_rot().to_bits(), x_rot.to_bits());
    }

    #[test]
    fn negative_zero_and_extreme_sequence() {
        let neg_zero = f32::from_bits(0x8000_0000);
        let pkt = ServerboundUseItemPacket::new(InteractionHand::OffHand, i32::MIN, neg_zero, 360.0);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ServerboundUseItemPacket::read(&mut slice);
        assert_eq!(decoded.get_y_rot().to_bits(), neg_zero.to_bits());
        assert_eq!(decoded.get_x_rot(), 360.0);
        assert_eq!(decoded.get_sequence(), i32::MIN);
    }

    #[test]
    fn handle_dispatches() {
        struct H {
            called: bool,
        }
        impl ServerGamePacketListener for H {
            fn handle_use_item(&mut self, _packet: &ServerboundUseItemPacket) {
                self.called = true;
            }
        }
        let pkt = ServerboundUseItemPacket::new(InteractionHand::MainHand, 1, 0.0, 0.0);
        let mut h = H { called: false };
        pkt.handle(&mut h);
        assert!(h.called);
    }
}
