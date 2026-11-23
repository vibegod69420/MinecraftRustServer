use bytes::{Buf, BufMut};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PacketDecodeError {
    #[error("unexpected end of buffer")]
    UnexpectedEof,
    #[error("varint too big")]
    VarIntTooBig,
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<B: Buf>(buf: &mut B) -> Result<i32, PacketDecodeError> {
    let mut value: i32 = 0;
    let mut position: i32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let b0 = buf.get_u8() as i8;
        value |= ((b0 & 127) as i32) << (position * 7);
        position += 1;
        if position > 5 {
            return Err(PacketDecodeError::VarIntTooBig);
        }
        if (b0 & -128i8) == 0 {
            break;
        }
    }
    Ok(value)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value & -128i32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 127) | 128) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

pub trait ClientGamePacketListener {
    fn handle_set_experience(&mut self, packet: &ClientboundSetExperiencePacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetExperiencePacket {
    experience_progress: f32,
    total_experience: i32,
    experience_level: i32,
}

impl ClientboundSetExperiencePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetExperiencePacket#<init>
    pub fn new(experience_progress: f32, total_experience: i32, experience_level: i32) -> Self {
        Self {
            experience_progress,
            total_experience,
            experience_level,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetExperiencePacket#<init>
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, PacketDecodeError> {
        if buf.remaining() < 4 {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let experience_progress = buf.get_f32();
        let experience_level = read_var_int(buf)?;
        let total_experience = read_var_int(buf)?;
        Ok(Self {
            experience_progress,
            total_experience,
            experience_level,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetExperiencePacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_f32(self.experience_progress);
        write_var_int(buf, self.experience_level);
        write_var_int(buf, self.total_experience);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetExperiencePacket#type
    pub fn type_name(&self) -> &'static str {
        "minecraft:set_experience"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetExperiencePacket#handle
    pub fn handle<H: ClientGamePacketListener>(&self, handler: &mut H) {
        handler.handle_set_experience(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetExperiencePacket#getExperienceProgress
    pub fn get_experience_progress(&self) -> f32 {
        self.experience_progress
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetExperiencePacket#getTotalExperience
    pub fn get_total_experience(&self) -> i32 {
        self.total_experience
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetExperiencePacket#getExperienceLevel
    pub fn get_experience_level(&self) -> i32 {
        self.experience_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockHandler {
        called: bool,
        level: i32,
    }

    impl ClientGamePacketListener for MockHandler {
        fn handle_set_experience(&mut self, packet: &ClientboundSetExperiencePacket) {
            self.called = true;
            self.level = packet.get_experience_level();
        }
    }

    #[test]
    fn roundtrip_normal() {
        let pkt = ClientboundSetExperiencePacket::new(0.5, 1234, 30);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundSetExperiencePacket::read(&mut buf.freeze()).unwrap();
        assert_eq!(decoded.get_experience_progress().to_bits(), 0.5f32.to_bits());
        assert_eq!(decoded.get_total_experience(), 1234);
        assert_eq!(decoded.get_experience_level(), 30);
    }

    #[test]
    fn roundtrip_float_bits_and_negatives() {
        let pkt = ClientboundSetExperiencePacket::new(f32::NAN, -1, -100);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundSetExperiencePacket::read(&mut buf.freeze()).unwrap();
        assert_eq!(decoded.get_experience_progress().to_bits(), f32::NAN.to_bits());
        assert_eq!(decoded.get_total_experience(), -1);
        assert_eq!(decoded.get_experience_level(), -100);

        let pkt2 = ClientboundSetExperiencePacket::new(-0.0, i32::MAX, i32::MIN);
        let mut buf2 = BytesMut::new();
        pkt2.write(&mut buf2);
        let decoded2 = ClientboundSetExperiencePacket::read(&mut buf2.freeze()).unwrap();
        assert_eq!(decoded2.get_experience_progress().to_bits(), (-0.0f32).to_bits());
        assert_eq!(decoded2.get_total_experience(), i32::MAX);
        assert_eq!(decoded2.get_experience_level(), i32::MIN);
    }

    #[test]
    fn wire_order_is_progress_level_total() {
        let pkt = ClientboundSetExperiencePacket::new(1.0, 999, 7);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let bytes = buf.freeze();
        // float 1.0 = 3f800000, varint 7 = 07, varint 999 = e707
        assert_eq!(&bytes[..4], &[0x3f, 0x80, 0x00, 0x00]);
        assert_eq!(bytes[4], 0x07);
        assert_eq!(&bytes[5..], &[0xe7, 0x07]);
    }

    #[test]
    fn decode_eof_handle_and_type() {
        let mut empty = bytes::Bytes::new();
        assert_eq!(
            ClientboundSetExperiencePacket::read(&mut empty),
            Err(PacketDecodeError::UnexpectedEof)
        );

        let pkt = ClientboundSetExperiencePacket::new(0.0, 0, 5);
        let mut handler = MockHandler { called: false, level: 0 };
        pkt.handle(&mut handler);
        assert!(handler.called);
        assert_eq!(handler.level, 5);
        assert_eq!(pkt.type_name(), "minecraft:set_experience");
    }
}
