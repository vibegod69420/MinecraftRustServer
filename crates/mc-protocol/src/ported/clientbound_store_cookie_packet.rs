//! Port of `net.minecraft.network.protocol.common.ClientboundStoreCookiePacket`.

use bytes::{Buf, BufMut};

pub const MAX_PAYLOAD_SIZE: usize = 5120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub java_constant: &'static str,
}

pub const CLIENTBOUND_STORE_COOKIE_TYPE: PacketType = PacketType {
    java_constant: "CommonPacketTypes.CLIENTBOUND_STORE_COOKIE",
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PacketDecodeError {
    #[error("varint too large")]
    VarIntTooLarge,
    #[error("string too long: {actual} > {max}")]
    StringTooLong { actual: usize, max: usize },
    #[error("invalid utf8")]
    InvalidUtf8,
    #[error("byte array too long: {actual} > {max}")]
    ByteArrayTooLong { actual: usize, max: usize },
    #[error("unexpected end of input")]
    UnexpectedEof,
}

/// Minimal `net.minecraft.resources.Identifier` representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#<init>
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    // JAVA: net.minecraft.resources.Identifier#parse
    pub fn parse(value: &str) -> Self {
        match value.split_once(':') {
            Some((ns, path)) => Self::new(ns, path),
            None => Self::new("minecraft", value),
        }
    }

    // JAVA: net.minecraft.resources.Identifier#toString
    pub fn to_string_value(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

pub trait ClientCommonPacketListener {
    fn handle_store_cookie(&mut self, packet: &ClientboundStoreCookiePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundStoreCookiePacket {
    key: Identifier,
    payload: Vec<u8>,
}

impl ClientboundStoreCookiePacket {
    // JAVA: net.minecraft.network.protocol.common.ClientboundStoreCookiePacket#<init>
    pub fn new(key: Identifier, payload: Vec<u8>) -> Self {
        Self { key, payload }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundStoreCookiePacket#<init>
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, PacketDecodeError> {
        let key = read_identifier(buf)?;
        let payload = read_byte_array(buf, MAX_PAYLOAD_SIZE)?;
        Ok(Self { key, payload })
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundStoreCookiePacket#write
    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), PacketDecodeError> {
        write_identifier(buf, &self.key)?;
        write_byte_array(buf, &self.payload, MAX_PAYLOAD_SIZE)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundStoreCookiePacket#key
    pub fn key(&self) -> &Identifier {
        &self.key
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundStoreCookiePacket#payload
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundStoreCookiePacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_STORE_COOKIE_TYPE
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundStoreCookiePacket#handle
    pub fn handle<L: ClientCommonPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_store_cookie(self);
    }
}

fn read_varint<B: Buf>(buf: &mut B) -> Result<i32, PacketDecodeError> {
    let mut result: i32 = 0;
    for shift in [0u32, 7, 14, 21, 28] {
        if !buf.has_remaining() {
            return Err(PacketDecodeError::UnexpectedEof);
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32) << shift;
        if (b & 0x80) == 0 {
            return Ok(result);
        }
    }
    Err(PacketDecodeError::VarIntTooLarge)
}

fn write_varint<B: BufMut>(buf: &mut B, value: i32) {
    let mut val = value as u32;
    loop {
        if (val & !0x7Fu32) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7F) as u8) | 0x80);
        val >>= 7;
    }
}

fn read_utf8<B: Buf>(buf: &mut B, max_chars: usize) -> Result<String, PacketDecodeError> {
    let max_bytes = max_chars.saturating_mul(3);
    let len_i32 = read_varint(buf)?;
    if len_i32 < 0 {
        return Err(PacketDecodeError::StringTooLong {
            actual: usize::MAX,
            max: max_bytes,
        });
    }
    let len = len_i32 as usize;
    if len > max_bytes {
        return Err(PacketDecodeError::StringTooLong {
            actual: len,
            max: max_bytes,
        });
    }
    if buf.remaining() < len {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let mut bytes = vec![0u8; len];
    buf.copy_to_slice(&mut bytes);
    let s = String::from_utf8(bytes).map_err(|_| PacketDecodeError::InvalidUtf8)?;
    if s.chars().count() > max_chars {
        return Err(PacketDecodeError::StringTooLong {
            actual: s.chars().count(),
            max: max_chars,
        });
    }
    Ok(s)
}

fn write_utf8<B: BufMut>(
    buf: &mut B,
    value: &str,
    max_chars: usize,
) -> Result<(), PacketDecodeError> {
    let char_count = value.chars().count();
    if char_count > max_chars {
        return Err(PacketDecodeError::StringTooLong {
            actual: char_count,
            max: max_chars,
        });
    }
    let bytes = value.as_bytes();
    let max_bytes = max_chars.saturating_mul(3);
    if bytes.len() > max_bytes {
        return Err(PacketDecodeError::StringTooLong {
            actual: bytes.len(),
            max: max_bytes,
        });
    }
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
    Ok(())
}

fn read_identifier<B: Buf>(buf: &mut B) -> Result<Identifier, PacketDecodeError> {
    let s = read_utf8(buf, 32767)?;
    Ok(Identifier::parse(&s))
}

fn write_identifier<B: BufMut>(buf: &mut B, id: &Identifier) -> Result<(), PacketDecodeError> {
    write_utf8(buf, &id.to_string_value(), 32767)
}

fn read_byte_array<B: Buf>(buf: &mut B, max_len: usize) -> Result<Vec<u8>, PacketDecodeError> {
    let len_i32 = read_varint(buf)?;
    if len_i32 < 0 || (len_i32 as usize) > max_len {
        return Err(PacketDecodeError::ByteArrayTooLong {
            actual: if len_i32 < 0 { 0 } else { len_i32 as usize },
            max: max_len,
        });
    }
    let len = len_i32 as usize;
    if buf.remaining() < len {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let mut out = vec![0u8; len];
    buf.copy_to_slice(&mut out);
    Ok(out)
}

fn write_byte_array<B: BufMut>(
    buf: &mut B,
    data: &[u8],
    max_len: usize,
) -> Result<(), PacketDecodeError> {
    if data.len() > max_len {
        return Err(PacketDecodeError::ByteArrayTooLong {
            actual: data.len(),
            max: max_len,
        });
    }
    write_varint(buf, data.len() as i32);
    buf.put_slice(data);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
        key: String,
        payload_len: usize,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_store_cookie(&mut self, packet: &ClientboundStoreCookiePacket) {
            self.called = true;
            self.key = packet.key().to_string_value();
            self.payload_len = packet.payload().len();
        }
    }

    #[test]
    fn roundtrip_store_cookie() {
        let key = Identifier::parse("minecraft:test_cookie");
        let payload = vec![1u8, 2, 3, 4, 5];
        let packet = ClientboundStoreCookiePacket::new(key, payload.clone());
        let mut buf = BytesMut::new();
        packet.encode(&mut buf).unwrap();
        let mut read_buf = buf.freeze();
        let decoded = ClientboundStoreCookiePacket::decode(&mut read_buf).unwrap();
        assert_eq!(decoded.key().to_string_value(), "minecraft:test_cookie");
        assert_eq!(decoded.payload(), payload.as_slice());
        assert_eq!(decoded.get_type(), CLIENTBOUND_STORE_COOKIE_TYPE);
    }

    #[test]
    fn handle_dispatches() {
        let packet = ClientboundStoreCookiePacket::new(
            Identifier::parse("ns:cookie"),
            vec![9u8; 3],
        );
        let mut listener = MockListener {
            called: false,
            key: String::new(),
            payload_len: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.key, "ns:cookie");
        assert_eq!(listener.payload_len, 3);
    }

    #[test]
    fn rejects_oversized_payload() {
        let packet = ClientboundStoreCookiePacket::new(
            Identifier::parse("minecraft:x"),
            vec![0u8; MAX_PAYLOAD_SIZE + 1],
        );
        let mut buf = BytesMut::new();
        let err = packet.encode(&mut buf).unwrap_err();
        assert!(matches!(err, PacketDecodeError::ByteArrayTooLong { .. }));
    }

    #[test]
    fn rejects_negative_or_too_large_payload_length_on_decode() {
        let mut buf = BytesMut::new();
        write_identifier(&mut buf, &Identifier::parse("minecraft:x")).unwrap();
        write_varint(&mut buf, -1);
        let mut read_buf = buf.freeze();
        let err = ClientboundStoreCookiePacket::decode(&mut read_buf).unwrap_err();
        assert!(matches!(err, PacketDecodeError::ByteArrayTooLong { .. }));

        let mut buf2 = BytesMut::new();
        write_identifier(&mut buf2, &Identifier::parse("minecraft:x")).unwrap();
        write_varint(&mut buf2, (MAX_PAYLOAD_SIZE as i32).wrapping_add(1));
        let mut read_buf2 = buf2.freeze();
        let err2 = ClientboundStoreCookiePacket::decode(&mut read_buf2).unwrap_err();
        assert!(matches!(err2, PacketDecodeError::ByteArrayTooLong { .. }));
    }

    #[test]
    fn empty_and_max_payload_boundary() {
        let empty = ClientboundStoreCookiePacket::new(
            Identifier::parse("minecraft:empty"),
            Vec::new(),
        );
        let mut buf = BytesMut::new();
        empty.encode(&mut buf).unwrap();
        let mut read_buf = buf.freeze();
        let decoded_empty = ClientboundStoreCookiePacket::decode(&mut read_buf).unwrap();
        assert!(decoded_empty.payload().is_empty());

        let max_payload = vec![0xABu8; MAX_PAYLOAD_SIZE];
        let packet = ClientboundStoreCookiePacket::new(
            Identifier::parse("minecraft:max"),
            max_payload.clone(),
        );
        let mut buf2 = BytesMut::new();
        packet.encode(&mut buf2).unwrap();
        let mut read_buf2 = buf2.freeze();
        let decoded = ClientboundStoreCookiePacket::decode(&mut read_buf2).unwrap();
        assert_eq!(decoded.payload().len(), MAX_PAYLOAD_SIZE);
        assert_eq!(decoded.payload()[0], 0xAB);
    }
}
