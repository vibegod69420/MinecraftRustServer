use bytes::{Buf, BufMut, BytesMut};
use std::fmt;

#[derive(Debug)]
pub struct DecodeError(pub String);

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DecodeError {}

fn read_var_int(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut val: i32 = 0;
    let mut j: i32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(DecodeError("VarInt too short".into()));
        }
        let b = buf.get_u8();
        val |= ((b & 0x7F) as i32).wrapping_shl(((j as u32).wrapping_mul(7)) & 31);
        j = j.wrapping_add(1);
        if j > 5 {
            return Err(DecodeError("VarInt too big".into()));
        }
        if (b & 0x80) == 0 {
            return Ok(val);
        }
    }
}

fn write_var_int(buf: &mut BytesMut, value: i32) {
    let mut v = value as u32;
    loop {
        if (v & !0x7Fu32) == 0 {
            buf.put_u8(v as u8);
            return;
        }
        buf.put_u8(((v as u8) & 0x7F) | 0x80);
        v >>= 7;
    }
}

/// Port of `ServerboundDebugSubscriptionRequestPacket`.
/// `subscriptions` stores registry IDs (VarInt wire encoding) in wire order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundDebugSubscriptionRequestPacket {
    pub subscriptions: Vec<i32>,
}

impl ServerboundDebugSubscriptionRequestPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundDebugSubscriptionRequestPacket#<init>
    pub fn new(subscriptions: Vec<i32>) -> Self {
        Self { subscriptions }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundDebugSubscriptionRequestPacket#subscriptions
    pub fn subscriptions(&self) -> &[i32] {
        &self.subscriptions
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundDebugSubscriptionRequestPacket#write
    pub fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, self.subscriptions.len() as i32);
        for &id in &self.subscriptions {
            write_var_int(buf, id);
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundDebugSubscriptionRequestPacket#read
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let len = read_var_int(buf)?;
        if len < 0 {
            return Err(DecodeError(format!("Negative collection length: {len}")));
        }
        let mut subscriptions = Vec::with_capacity(len as usize);
        for _ in 0..len {
            subscriptions.push(read_var_int(buf)?);
        }
        Ok(Self { subscriptions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let pkt = ServerboundDebugSubscriptionRequestPacket::new(vec![]);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundDebugSubscriptionRequestPacket::read(&mut slice).unwrap();
        assert_eq!(pkt, decoded);
        assert!(slice.is_empty());
    }

    #[test]
    fn roundtrip_with_ids() {
        let pkt = ServerboundDebugSubscriptionRequestPacket::new(vec![0, 1, 127, 128, 255, 16384]);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundDebugSubscriptionRequestPacket::read(&mut slice).unwrap();
        assert_eq!(pkt.subscriptions(), decoded.subscriptions());
    }

    #[test]
    fn roundtrip_negative_and_extreme_ids() {
        let pkt = ServerboundDebugSubscriptionRequestPacket::new(vec![-1, i32::MIN, i32::MAX]);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ServerboundDebugSubscriptionRequestPacket::read(&mut slice).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn var_int_roundtrip_edges() {
        for v in [0, 1, 127, 128, 255, 16383, 16384, -1, i32::MIN, i32::MAX] {
            let mut buf = BytesMut::new();
            write_var_int(&mut buf, v);
            let mut slice: &[u8] = &buf;
            assert_eq!(read_var_int(&mut slice).unwrap(), v);
            assert!(slice.is_empty());
        }
    }

    #[test]
    fn decode_negative_length_fails() {
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, -1);
        let mut slice: &[u8] = &buf;
        assert!(ServerboundDebugSubscriptionRequestPacket::read(&mut slice).is_err());
    }

    #[test]
    fn var_int_too_big_fails() {
        let data = [0x80u8, 0x80, 0x80, 0x80, 0x80, 0x01];
        let mut slice: &[u8] = &data;
        assert!(read_var_int(&mut slice).is_err());
    }
}
