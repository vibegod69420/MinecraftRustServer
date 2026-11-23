use bytes::{Buf, BufMut};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PacketDecodeError {
    #[error("unexpected end of buffer")]
    UnexpectedEof,
    #[error("varlong too big")]
    VarLongTooBig,
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarLong
fn read_var_long<B: Buf>(buf: &mut B) -> Result<i64, PacketDecodeError> {
    let mut value: i64 = 0;
    let mut position: i32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let b0 = buf.get_u8() as i8;
        value |= ((b0 & 127) as i64) << (position * 7);
        position += 1;
        if position > 10 {
            return Err(PacketDecodeError::VarLongTooBig);
        }
        if (b0 & -128i8) == 0 {
            break;
        }
    }
    Ok(value)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarLong
fn write_var_long<B: BufMut>(buf: &mut B, mut value: i64) {
    loop {
        if (value & -128i64) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 127) | 128) as u8);
        value = ((value as u64) >> 7) as i64;
    }
}

pub trait ClientGamePacketListener {
    fn handle_set_border_lerp_size(&mut self, packet: &ClientboundSetBorderLerpSizePacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetBorderLerpSizePacket {
    old_size: f64,
    new_size: f64,
    lerp_time: i64,
}

impl ClientboundSetBorderLerpSizePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket#<init>
    pub fn new(old_size: f64, new_size: f64, lerp_time: i64) -> Self {
        Self {
            old_size,
            new_size,
            lerp_time,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket#<init>
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, PacketDecodeError> {
        if buf.remaining() < 8 {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let old_size = buf.get_f64();
        if buf.remaining() < 8 {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let new_size = buf.get_f64();
        let lerp_time = read_var_long(buf)?;
        Ok(Self {
            old_size,
            new_size,
            lerp_time,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        buf.put_f64(self.old_size);
        buf.put_f64(self.new_size);
        write_var_long(buf, self.lerp_time);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket#type
    pub fn type_name(&self) -> &'static str {
        "minecraft:set_border_lerp_size"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket#handle
    pub fn handle<H: ClientGamePacketListener>(&self, handler: &mut H) {
        handler.handle_set_border_lerp_size(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket#getOldSize
    pub fn get_old_size(&self) -> f64 {
        self.old_size
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket#getNewSize
    pub fn get_new_size(&self) -> f64 {
        self.new_size
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket#getLerpTime
    pub fn get_lerp_time(&self) -> i64 {
        self.lerp_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockHandler {
        called: bool,
        lerp_time: i64,
    }

    impl ClientGamePacketListener for MockHandler {
        fn handle_set_border_lerp_size(&mut self, packet: &ClientboundSetBorderLerpSizePacket) {
            self.called = true;
            self.lerp_time = packet.get_lerp_time();
        }
    }

    #[test]
    fn roundtrip_normal() {
        let pkt = ClientboundSetBorderLerpSizePacket::new(100.0, 200.0, 5000);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundSetBorderLerpSizePacket::read(&mut buf.freeze()).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_old_size().to_bits(), 100.0f64.to_bits());
        assert_eq!(decoded.get_new_size().to_bits(), 200.0f64.to_bits());
        assert_eq!(decoded.get_lerp_time(), 5000);
    }

    #[test]
    fn roundtrip_float_bits_and_negative_time() {
        let pkt = ClientboundSetBorderLerpSizePacket::new(f64::NAN, -0.0, -1);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let decoded = ClientboundSetBorderLerpSizePacket::read(&mut buf.freeze()).unwrap();
        assert_eq!(decoded.get_old_size().to_bits(), f64::NAN.to_bits());
        assert_eq!(decoded.get_new_size().to_bits(), (-0.0f64).to_bits());
        assert_eq!(decoded.get_lerp_time(), -1);

        let pkt2 = ClientboundSetBorderLerpSizePacket::new(f64::INFINITY, f64::NEG_INFINITY, i64::MIN);
        let mut buf2 = BytesMut::new();
        pkt2.write(&mut buf2);
        let decoded2 = ClientboundSetBorderLerpSizePacket::read(&mut buf2.freeze()).unwrap();
        assert_eq!(decoded2.get_old_size(), f64::INFINITY);
        assert_eq!(decoded2.get_new_size(), f64::NEG_INFINITY);
        assert_eq!(decoded2.get_lerp_time(), i64::MIN);
    }

    #[test]
    fn varlong_encoding_and_errors() {
        let mut buf = BytesMut::new();
        write_var_long(&mut buf, -1);
        assert_eq!(buf.len(), 10);
        let mut read_buf = buf.freeze();
        assert_eq!(read_var_long(&mut read_buf).unwrap(), -1);

        let mut empty = bytes::Bytes::new();
        assert_eq!(
            ClientboundSetBorderLerpSizePacket::read(&mut empty),
            Err(PacketDecodeError::UnexpectedEof)
        );
    }

    #[test]
    fn handle_and_type() {
        let pkt = ClientboundSetBorderLerpSizePacket::new(1.0, 2.0, 3);
        let mut handler = MockHandler {
            called: false,
            lerp_time: 0,
        };
        pkt.handle(&mut handler);
        assert!(handler.called);
        assert_eq!(handler.lerp_time, 3);
        assert_eq!(pkt.type_name(), "minecraft:set_border_lerp_size");
    }
}
