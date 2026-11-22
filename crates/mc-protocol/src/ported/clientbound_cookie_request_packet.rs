use bytes::{Buf, BufMut};

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
        panic!(
            "String too long ({} > {})",
            char_len, max_length
        );
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

/// Minimal self-contained packet type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType<T>(std::marker::PhantomData<fn() -> T>);

impl<T> PacketType<T> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

pub trait ClientCookiePacketListener {
    fn handle_request_cookie(&mut self, packet: &ClientboundCookieRequestPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCookieRequestPacket {
    pub key: Identifier,
}

pub const CLIENTBOUND_COOKIE_REQUEST_TYPE: PacketType<ClientboundCookieRequestPacket> =
    PacketType::new();

impl ClientboundCookieRequestPacket {
    // JAVA: net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket#<init>
    pub fn read(buf: &mut impl Buf) -> Result<Self, PacketDecodeError> {
        Ok(Self {
            key: read_identifier(buf)?,
        })
    }

    // JAVA: net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_identifier(buf, &self.key);
    }

    // JAVA: net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket#type
    pub fn get_type(&self) -> PacketType<Self> {
        CLIENTBOUND_COOKIE_REQUEST_TYPE
    }

    // JAVA: net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket#handle
    pub fn handle<L: ClientCookiePacketListener>(&self, listener: &mut L) {
        listener.handle_request_cookie(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BufMut, BytesMut};

    #[test]
    fn roundtrip_simple() {
        let packet = ClientboundCookieRequestPacket {
            key: Identifier {
                namespace: "minecraft".to_owned(),
                path: "test".to_owned(),
            },
        };
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundCookieRequestPacket::read(&mut reader).unwrap();
        assert_eq!(packet, decoded);
        assert!(!reader.has_remaining());
    }

    #[test]
    fn roundtrip_with_namespace_path() {
        let packet = ClientboundCookieRequestPacket {
            key: Identifier::parse("custom:my/cookie_key").unwrap(),
        };
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundCookieRequestPacket::read(&mut reader).unwrap();
        assert_eq!(decoded.key.namespace, "custom");
        assert_eq!(decoded.key.path, "my/cookie_key");
    }

    #[test]
    fn varint_roundtrip_negative_and_extremes() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, -1);
        write_varint(&mut buf, i32::MIN);
        write_varint(&mut buf, i32::MAX);
        let mut reader = buf.freeze();
        assert_eq!(read_varint(&mut reader).unwrap(), -1);
        assert_eq!(read_varint(&mut reader).unwrap(), i32::MIN);
        assert_eq!(read_varint(&mut reader).unwrap(), i32::MAX);
    }

    #[test]
    fn decode_unexpected_eof() {
        let mut reader = bytes::Bytes::from_static(&[]);
        let err = ClientboundCookieRequestPacket::read(&mut reader).unwrap_err();
        assert!(matches!(err, PacketDecodeError::UnexpectedEof));
    }

    #[test]
    fn decode_invalid_varint_too_large() {
        let mut reader = bytes::Bytes::from_static(&[0x80, 0x80, 0x80, 0x80, 0x80]);
        let err = read_varint(&mut reader).unwrap_err();
        assert!(matches!(err, PacketDecodeError::VarIntTooLarge));
    }

    #[test]
    fn handle_dispatches_and_type() {
        struct MockListener(Option<Identifier>);
        impl ClientCookiePacketListener for MockListener {
            fn handle_request_cookie(&mut self, packet: &ClientboundCookieRequestPacket) {
                self.0 = Some(packet.key.clone());
            }
        }
        let packet = ClientboundCookieRequestPacket {
            key: Identifier::parse("minecraft:test").unwrap(),
        };
        let mut listener = MockListener(None);
        packet.handle(&mut listener);
        assert_eq!(listener.0.unwrap().to_string(), "minecraft:test");
        assert_eq!(packet.get_type(), CLIENTBOUND_COOKIE_REQUEST_TYPE);
    }
}
