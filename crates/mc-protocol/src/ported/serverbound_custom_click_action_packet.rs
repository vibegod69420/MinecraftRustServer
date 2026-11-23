//! Port of `net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket`.

use bytes::{Buf, BufMut};

pub const NBT_ACCUMULATOR_MAX_BYTES: i64 = 32_768;
pub const NBT_ACCUMULATOR_MAX_DEPTH: i32 = 16;
pub const UNTRUSTED_TAG_MAX_LENGTH: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub java_constant: &'static str,
}

pub const SERVERBOUND_CUSTOM_CLICK_ACTION_TYPE: PacketType = PacketType {
    java_constant: "CommonPacketTypes.SERVERBOUND_CUSTOM_CLICK_ACTION",
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PacketDecodeError {
    #[error("varint too large")]
    VarIntTooLarge,
    #[error("string too long: {actual} > {max}")]
    StringTooLong { actual: usize, max: usize },
    #[error("invalid utf8")]
    InvalidUtf8,
    #[error("invalid payload length: {0}")]
    InvalidPayloadLength(i32),
    #[error("payload too long: {actual} > {max}")]
    PayloadTooLong { actual: usize, max: usize },
    #[error("unexpected end of input")]
    UnexpectedEof,
}

/// Placeholder for `net.minecraft.nbt.Tag`.
/// Stores raw NBT bytes in this self-contained module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub raw_bytes: Vec<u8>,
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

pub trait ServerCommonPacketListener {
    fn handle_custom_click_action(&mut self, packet: &ServerboundCustomClickActionPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCustomClickActionPacket {
    id: Identifier,
    payload: Option<Tag>,
}

impl ServerboundCustomClickActionPacket {
    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket#<init>
    pub fn new(id: Identifier, payload: Option<Tag>) -> Self {
        Self { id, payload }
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket#id
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket#payload
    pub fn payload(&self) -> Option<&Tag> {
        self.payload.as_ref()
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket#type
    pub fn get_type(&self) -> PacketType {
        SERVERBOUND_CUSTOM_CLICK_ACTION_TYPE
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket#handle
    pub fn handle<L: ServerCommonPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_custom_click_action(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket#STREAM_CODEC
    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), PacketDecodeError> {
        write_utf8(buf, &self.id.to_string_value(), 32767)?;
        write_optional_tag(buf, self.payload.as_ref())
    }

    // JAVA: net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket#STREAM_CODEC
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, PacketDecodeError> {
        let id_str = read_utf8(buf, 32767)?;
        let id = Identifier::parse(&id_str);
        let payload = read_optional_tag(buf)?;
        Ok(Self::new(id, payload))
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

fn write_optional_tag<B: BufMut>(
    buf: &mut B,
    payload: Option<&Tag>,
) -> Result<(), PacketDecodeError> {
    let inner: &[u8] = match payload {
        None => &[0x00],
        Some(tag) => &tag.raw_bytes,
    };
    if inner.len() > UNTRUSTED_TAG_MAX_LENGTH {
        return Err(PacketDecodeError::PayloadTooLong {
            actual: inner.len(),
            max: UNTRUSTED_TAG_MAX_LENGTH,
        });
    }
    write_varint(buf, inner.len() as i32);
    buf.put_slice(inner);
    Ok(())
}

fn read_optional_tag<B: Buf>(buf: &mut B) -> Result<Option<Tag>, PacketDecodeError> {
    let len_i32 = read_varint(buf)?;
    if len_i32 < 0 {
        return Err(PacketDecodeError::InvalidPayloadLength(len_i32));
    }
    let len = len_i32 as usize;
    if len > UNTRUSTED_TAG_MAX_LENGTH {
        return Err(PacketDecodeError::PayloadTooLong {
            actual: len,
            max: UNTRUSTED_TAG_MAX_LENGTH,
        });
    }
    if buf.remaining() < len {
        return Err(PacketDecodeError::UnexpectedEof);
    }
    let mut raw = vec![0u8; len];
    buf.copy_to_slice(&mut raw);
    if raw.len() == 1 && raw[0] == 0x00 {
        Ok(None)
    } else {
        Ok(Some(Tag { raw_bytes: raw }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
        id: String,
    }

    impl ServerCommonPacketListener for MockListener {
        fn handle_custom_click_action(&mut self, packet: &ServerboundCustomClickActionPacket) {
            self.called = true;
            self.id = packet.id().to_string_value();
        }
    }

    #[test]
    fn roundtrip_no_payload() {
        let id = Identifier::parse("minecraft:test");
        let packet = ServerboundCustomClickActionPacket::new(id, None);
        let mut buf = BytesMut::new();
        packet.encode(&mut buf).unwrap();
        let mut read_buf = buf.freeze();
        let decoded = ServerboundCustomClickActionPacket::decode(&mut read_buf).unwrap();
        assert_eq!(decoded.id().to_string_value(), "minecraft:test");
        assert!(decoded.payload().is_none());
        assert_eq!(decoded.get_type(), SERVERBOUND_CUSTOM_CLICK_ACTION_TYPE);
    }

    #[test]
    fn roundtrip_with_payload_and_handle() {
        let id = Identifier::parse("custom:action");
        let payload = Some(Tag {
            raw_bytes: vec![10, 0, 0, 0],
        });
        let packet = ServerboundCustomClickActionPacket::new(id, payload);
        let mut buf = BytesMut::new();
        packet.encode(&mut buf).unwrap();
        let mut read_buf = buf.freeze();
        let decoded = ServerboundCustomClickActionPacket::decode(&mut read_buf).unwrap();
        assert_eq!(decoded.payload().unwrap().raw_bytes, vec![10, 0, 0, 0]);

        let mut listener = MockListener {
            called: false,
            id: String::new(),
        };
        decoded.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.id, "custom:action");
    }

    #[test]
    fn rejects_oversized_payload() {
        let id = Identifier::parse("minecraft:test");
        let payload = Some(Tag {
            raw_bytes: vec![0u8; UNTRUSTED_TAG_MAX_LENGTH + 1],
        });
        let packet = ServerboundCustomClickActionPacket::new(id, payload);
        let mut buf = BytesMut::new();
        let err = packet.encode(&mut buf).unwrap_err();
        assert!(matches!(err, PacketDecodeError::PayloadTooLong { .. }));
    }

    #[test]
    fn rejects_negative_payload_length() {
        let mut buf = BytesMut::new();
        write_utf8(&mut buf, "minecraft:test", 32767).unwrap();
        write_varint(&mut buf, -1);
        let mut read_buf = buf.freeze();
        let err = ServerboundCustomClickActionPacket::decode(&mut read_buf).unwrap_err();
        assert!(matches!(err, PacketDecodeError::InvalidPayloadLength(-1)));
    }

    #[test]
    fn constants_match_java_and_identifier_parse_default_namespace() {
        assert_eq!(NBT_ACCUMULATOR_MAX_BYTES, 32_768_i64);
        assert_eq!(NBT_ACCUMULATOR_MAX_DEPTH, 16_i32);
        assert_eq!(UNTRUSTED_TAG_MAX_LENGTH, 65_536_usize);

        let id = Identifier::parse("test_path");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "test_path");
    }
}
