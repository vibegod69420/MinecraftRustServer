use bytes::{Buf, BufMut};

/// Mirrors ClientboundStoreCookiePacket.PAYLOAD_STREAM_CODEC max size (5120 bytes).
pub const MAX_COOKIE_PAYLOAD_SIZE: usize = 5120;

#[derive(Debug, thiserror::Error)]
pub enum PacketDecodeError {
    #[error("unexpected end of buffer")]
    UnexpectedEof,
    #[error("varint too large")]
    VarIntTooLarge,
    #[error("invalid utf8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("string too long: {len} > {max}")]
    StringTooLong { len: usize, max: usize },
    #[error("byte array too long: {len} > {max}")]
    ByteArrayTooLong { len: usize, max: usize },
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),
}

// JAVA: net.minecraft.network.VarInt#read
fn read_varint(buf: &mut impl Buf) -> Result<i32, PacketDecodeError> {
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

// JAVA: net.minecraft.network.VarInt#write
fn write_varint(buf: &mut impl BufMut, mut value: i32) {
    loop {
        if (value & !0x7F) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

fn utf16_len(s: &str) -> usize {
    s.chars().map(|c| c.len_utf16()).sum()
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
fn read_utf(buf: &mut impl Buf, max_length: usize) -> Result<String, PacketDecodeError> {
    let byte_len_i32 = read_varint(buf)?;
    if byte_len_i32 < 0 {
        return Err(PacketDecodeError::StringTooLong {
            len: 0,
            max: max_length.saturating_mul(3),
        });
    }
    let byte_len = byte_len_i32 as usize;
    let max_bytes = max_length.saturating_mul(3);
    if byte_len > max_bytes {
        return Err(PacketDecodeError::StringTooLong {
            len: byte_len,
            max: max_bytes,
        });
    }
    if buf.remaining() < byte_len {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let mut bytes = vec![0u8; byte_len];
    buf.copy_to_slice(&mut bytes);
    let s = String::from_utf8(bytes)?;
    let char_len = utf16_len(&s);
    if char_len > max_length {
        return Err(PacketDecodeError::StringTooLong {
            len: char_len,
            max: max_length,
        });
    }
    Ok(s)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
fn write_utf(buf: &mut impl BufMut, s: &str, max_length: usize) {
    let char_len = utf16_len(s);
    if char_len > max_length {
        panic!("String too long ({} > {})", char_len, max_length);
    }
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

/// Minimal self-contained Identifier representation for packet encoding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#parse
    pub fn parse(s: &str) -> Result<Self, PacketDecodeError> {
        let (namespace, path) = match s.split_once(':') {
            Some((n, p)) => (n.to_owned(), p.to_owned()),
            None => ("minecraft".to_owned(), s.to_owned()),
        };
        if namespace.is_empty() || path.is_empty() {
            return Err(PacketDecodeError::InvalidIdentifier(s.to_owned()));
        }
        Ok(Self { namespace, path })
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier
fn read_identifier(buf: &mut impl Buf) -> Result<Identifier, PacketDecodeError> {
    let s = read_utf(buf, 32767)?;
    Identifier::parse(&s)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier
fn write_identifier(buf: &mut impl BufMut, id: &Identifier) {
    write_utf(buf, &id.to_string(), 32767);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readByteArray
fn read_byte_array(buf: &mut impl Buf, max_size: usize) -> Result<Vec<u8>, PacketDecodeError> {
    let len_i32 = read_varint(buf)?;
    if len_i32 < 0 {
        return Err(PacketDecodeError::ByteArrayTooLong {
            len: 0,
            max: max_size,
        });
    }
    let len = len_i32 as usize;
    if len > max_size {
        return Err(PacketDecodeError::ByteArrayTooLong { len, max: max_size });
    }
    if buf.remaining() < len {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let mut out = vec![0u8; len];
    buf.copy_to_slice(&mut out);
    Ok(out)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeByteArray
fn write_byte_array(buf: &mut impl BufMut, data: &[u8]) {
    write_varint(buf, data.len() as i32);
    buf.put_slice(data);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readNullable
fn read_nullable_byte_array(
    buf: &mut impl Buf,
    max_size: usize,
) -> Result<Option<Vec<u8>>, PacketDecodeError> {
    if !buf.has_remaining() {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let present = buf.get_u8() != 0;
    if present {
        Ok(Some(read_byte_array(buf, max_size)?))
    } else {
        Ok(None)
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeNullable
fn write_nullable_byte_array(buf: &mut impl BufMut, value: Option<&[u8]>) {
    match value {
        Some(data) => {
            buf.put_u8(1);
            write_byte_array(buf, data);
        }
        None => {
            buf.put_u8(0);
        }
    }
}

/// Minimal self-contained packet type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType<T>(std::marker::PhantomData<fn() -> T>);

impl<T> PacketType<T> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

pub trait ServerCookiePacketListener {
    fn handle_cookie_response(&mut self, packet: &ServerboundCookieResponsePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCookieResponsePacket {
    pub key: Identifier,
    pub payload: Option<Vec<u8>>,
}

pub const SERVERBOUND_COOKIE_RESPONSE_TYPE: PacketType<ServerboundCookieResponsePacket> =
    PacketType::new();

impl ServerboundCookieResponsePacket {
    // JAVA: net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket#<init>
    pub fn read(buf: &mut impl Buf) -> Result<Self, PacketDecodeError> {
        let key = read_identifier(buf)?;
        let payload = read_nullable_byte_array(buf, MAX_COOKIE_PAYLOAD_SIZE)?;
        Ok(Self { key, payload })
    }

    // JAVA: net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_identifier(buf, &self.key);
        write_nullable_byte_array(buf, self.payload.as_deref());
    }

    // JAVA: net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket#type
    pub fn get_type(&self) -> PacketType<Self> {
        SERVERBOUND_COOKIE_RESPONSE_TYPE
    }

    // JAVA: net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket#handle
    pub fn handle<L: ServerCookiePacketListener>(&self, listener: &mut L) {
        listener.handle_cookie_response(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BufMut, BytesMut};

    #[test]
    fn roundtrip_with_payload() {
        let packet = ServerboundCookieResponsePacket {
            key: Identifier::parse("minecraft:test").unwrap(),
            payload: Some(vec![1, 2, 3, 4, 5]),
        };
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ServerboundCookieResponsePacket::read(&mut reader).unwrap();
        assert_eq!(packet, decoded);
        assert!(!reader.has_remaining());
    }

    #[test]
    fn roundtrip_without_payload() {
        let packet = ServerboundCookieResponsePacket {
            key: Identifier::parse("custom:cookie").unwrap(),
            payload: None,
        };
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ServerboundCookieResponsePacket::read(&mut reader).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn payload_too_large_rejected() {
        let mut buf = BytesMut::new();
        write_identifier(&mut buf, &Identifier::parse("minecraft:test").unwrap());
        buf.put_u8(1);
        write_varint(&mut buf, (MAX_COOKIE_PAYLOAD_SIZE as i32).wrapping_add(1));
        let mut reader = buf.freeze();
        let err = ServerboundCookieResponsePacket::read(&mut reader).unwrap_err();
        assert!(matches!(err, PacketDecodeError::ByteArrayTooLong { .. }));
    }

    #[test]
    fn negative_payload_length_rejected() {
        let mut buf = BytesMut::new();
        write_identifier(&mut buf, &Identifier::parse("minecraft:test").unwrap());
        buf.put_u8(1);
        write_varint(&mut buf, -1);
        let mut reader = buf.freeze();
        let err = ServerboundCookieResponsePacket::read(&mut reader).unwrap_err();
        assert!(matches!(err, PacketDecodeError::ByteArrayTooLong { .. }));
    }

    #[test]
    fn max_payload_size_and_handle_dispatch() {
        let payload = vec![0xAB; MAX_COOKIE_PAYLOAD_SIZE];
        let packet = ServerboundCookieResponsePacket {
            key: Identifier::parse("minecraft:big").unwrap(),
            payload: Some(payload.clone()),
        };
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ServerboundCookieResponsePacket::read(&mut reader).unwrap();
        assert_eq!(decoded.payload.as_ref().unwrap().len(), MAX_COOKIE_PAYLOAD_SIZE);

        struct MockListener(Option<(Identifier, Option<Vec<u8>>)>);
        impl ServerCookiePacketListener for MockListener {
            fn handle_cookie_response(&mut self, packet: &ServerboundCookieResponsePacket) {
                self.0 = Some((packet.key.clone(), packet.payload.clone()));
            }
        }
        let mut listener = MockListener(None);
        decoded.handle(&mut listener);
        let (key, got_payload) = listener.0.unwrap();
        assert_eq!(key.to_string(), "minecraft:big");
        assert_eq!(got_payload, Some(payload));
        assert_eq!(decoded.get_type(), SERVERBOUND_COOKIE_RESPONSE_TYPE);
    }
}
