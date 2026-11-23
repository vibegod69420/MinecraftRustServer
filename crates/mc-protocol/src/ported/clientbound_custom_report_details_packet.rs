//! Port of `net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket`.

use bytes::{Buf, BufMut};
use std::collections::HashMap;

pub const MAX_DETAIL_KEY_LENGTH: usize = 128;
pub const MAX_DETAIL_VALUE_LENGTH: usize = 4096;
pub const MAX_DETAIL_COUNT: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub java_constant: &'static str,
}

pub const CLIENTBOUND_CUSTOM_REPORT_DETAILS_TYPE: PacketType = PacketType {
    java_constant: "CommonPacketTypes.CLIENTBOUND_CUSTOM_REPORT_DETAILS",
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PacketDecodeError {
    #[error("varint too large")]
    VarIntTooLarge,
    #[error("string too long: {actual} > {max}")]
    StringTooLong { actual: usize, max: usize },
    #[error("invalid utf8")]
    InvalidUtf8,
    #[error("map too large: {actual} > {max}")]
    MapTooLarge { actual: usize, max: usize },
    #[error("unexpected end of input")]
    UnexpectedEof,
}

pub trait ClientCommonPacketListener {
    fn handle_custom_report_details(&mut self, packet: &ClientboundCustomReportDetailsPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomReportDetailsPacket {
    details: HashMap<String, String>,
}

impl ClientboundCustomReportDetailsPacket {
    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket#<init>
    pub fn new(details: HashMap<String, String>) -> Self {
        Self { details }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket#details
    pub fn details(&self) -> &HashMap<String, String> {
        &self.details
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_CUSTOM_REPORT_DETAILS_TYPE
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket#handle
    pub fn handle<L: ClientCommonPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_custom_report_details(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket#STREAM_CODEC
    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), PacketDecodeError> {
        write_details_map(buf, &self.details)
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket#STREAM_CODEC
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, PacketDecodeError> {
        let details = read_details_map(buf)?;
        Ok(Self::new(details))
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

fn read_details_map<B: Buf>(buf: &mut B) -> Result<HashMap<String, String>, PacketDecodeError> {
    let count = read_varint(buf)?;
    if count < 0 || (count as usize) > MAX_DETAIL_COUNT {
        return Err(PacketDecodeError::MapTooLarge {
            actual: if count < 0 { 0 } else { count as usize },
            max: MAX_DETAIL_COUNT,
        });
    }
    let mut map = HashMap::with_capacity(count as usize);
    for _ in 0..count {
        let key = read_utf8(buf, MAX_DETAIL_KEY_LENGTH)?;
        let value = read_utf8(buf, MAX_DETAIL_VALUE_LENGTH)?;
        map.insert(key, value);
    }
    Ok(map)
}

fn write_details_map<B: BufMut>(
    buf: &mut B,
    map: &HashMap<String, String>,
) -> Result<(), PacketDecodeError> {
    if map.len() > MAX_DETAIL_COUNT {
        return Err(PacketDecodeError::MapTooLarge {
            actual: map.len(),
            max: MAX_DETAIL_COUNT,
        });
    }
    write_varint(buf, map.len() as i32);
    for (k, v) in map {
        write_utf8(buf, k, MAX_DETAIL_KEY_LENGTH)?;
        write_utf8(buf, v, MAX_DETAIL_VALUE_LENGTH)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
        count: usize,
    }

    impl ClientCommonPacketListener for MockListener {
        fn handle_custom_report_details(&mut self, packet: &ClientboundCustomReportDetailsPacket) {
            self.called = true;
            self.count = packet.details().len();
        }
    }

    #[test]
    fn roundtrip_empty_details() {
        let packet = ClientboundCustomReportDetailsPacket::new(HashMap::new());
        let mut buf = BytesMut::new();
        packet.encode(&mut buf).unwrap();
        let mut read_buf = buf.freeze();
        let decoded = ClientboundCustomReportDetailsPacket::decode(&mut read_buf).unwrap();
        assert_eq!(decoded.details().len(), 0);
        assert_eq!(packet.get_type(), CLIENTBOUND_CUSTOM_REPORT_DETAILS_TYPE);
    }

    #[test]
    fn roundtrip_and_handle() {
        let mut details = HashMap::new();
        details.insert("k".to_string(), "v".to_string());
        let packet = ClientboundCustomReportDetailsPacket::new(details);
        let mut buf = BytesMut::new();
        packet.encode(&mut buf).unwrap();
        let mut read_buf = buf.freeze();
        let decoded = ClientboundCustomReportDetailsPacket::decode(&mut read_buf).unwrap();
        assert_eq!(decoded.details().get("k"), Some(&"v".to_string()));

        let mut listener = MockListener {
            called: false,
            count: 0,
        };
        decoded.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.count, 1);
    }

    #[test]
    fn rejects_too_many_details() {
        let mut map = HashMap::new();
        for i in 0..(MAX_DETAIL_COUNT + 1) {
            map.insert(format!("k{i}"), "v".to_string());
        }
        let packet = ClientboundCustomReportDetailsPacket::new(map);
        let mut buf = BytesMut::new();
        let err = packet.encode(&mut buf).unwrap_err();
        assert!(matches!(err, PacketDecodeError::MapTooLarge { .. }));
    }

    #[test]
    fn rejects_negative_count_and_too_large_count() {
        let mut buf = BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F][..]);
        let err = ClientboundCustomReportDetailsPacket::decode(&mut buf).unwrap_err();
        assert!(matches!(err, PacketDecodeError::MapTooLarge { .. }));

        let mut buf2 = BytesMut::new();
        write_varint(&mut buf2, 33);
        let mut read_buf2 = buf2.freeze();
        let err2 = ClientboundCustomReportDetailsPacket::decode(&mut read_buf2).unwrap_err();
        assert!(matches!(err2, PacketDecodeError::MapTooLarge { .. }));
    }

    #[test]
    fn rejects_oversized_key_and_value() {
        let mut details = HashMap::new();
        details.insert("a".repeat(MAX_DETAIL_KEY_LENGTH + 1), "v".to_string());
        let packet = ClientboundCustomReportDetailsPacket::new(details);
        let mut buf = BytesMut::new();
        assert!(matches!(
            packet.encode(&mut buf).unwrap_err(),
            PacketDecodeError::StringTooLong { .. }
        ));

        let mut details2 = HashMap::new();
        details2.insert("k".to_string(), "b".repeat(MAX_DETAIL_VALUE_LENGTH + 1));
        let packet2 = ClientboundCustomReportDetailsPacket::new(details2);
        let mut buf2 = BytesMut::new();
        assert!(matches!(
            packet2.encode(&mut buf2).unwrap_err(),
            PacketDecodeError::StringTooLong { .. }
        ));
    }

    #[test]
    fn varint_negative_roundtrip_and_boundary_lengths() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, -1);
        let mut read_buf = buf.freeze();
        assert_eq!(read_varint(&mut read_buf).unwrap(), -1);

        let mut details = HashMap::new();
        details.insert("k".repeat(MAX_DETAIL_KEY_LENGTH), "v".repeat(MAX_DETAIL_VALUE_LENGTH));
        let packet = ClientboundCustomReportDetailsPacket::new(details);
        let mut buf2 = BytesMut::new();
        packet.encode(&mut buf2).unwrap();
        let mut read_buf2 = buf2.freeze();
        let decoded = ClientboundCustomReportDetailsPacket::decode(&mut read_buf2).unwrap();
        assert_eq!(decoded.details().len(), 1);
    }
}
