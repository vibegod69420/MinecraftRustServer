use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    UnexpectedEof,
    InvalidVarInt,
    InvalidUtf8,
    InvalidLength(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_SELECT_KNOWN_PACKS: PacketType =
    PacketType("configuration/clientbound_select_known_packs");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack {
    namespace: String,
    id: String,
    version: String,
}

impl KnownPack {
    // JAVA: net.minecraft.server.packs.repository.KnownPack#<init>
    pub fn new(namespace: String, id: String, version: String) -> Self {
        Self {
            namespace,
            id,
            version,
        }
    }

    // JAVA: net.minecraft.server.packs.repository.KnownPack#namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    // JAVA: net.minecraft.server.packs.repository.KnownPack#id
    pub fn id(&self) -> &str {
        &self.id
    }

    // JAVA: net.minecraft.server.packs.repository.KnownPack#version
    pub fn version(&self) -> &str {
        &self.version
    }

    // JAVA: net.minecraft.server.packs.repository.KnownPack#STREAM_CODEC
    pub fn encode(&self, buf: &mut BytesMut) {
        write_utf(buf, &self.namespace);
        write_utf(buf, &self.id);
        write_utf(buf, &self.version);
    }

    // JAVA: net.minecraft.server.packs.repository.KnownPack#STREAM_CODEC
    pub fn decode(buf: &mut &[u8]) -> Result<Self, CodecError> {
        let namespace = read_utf(buf)?;
        let id = read_utf(buf)?;
        let version = read_utf(buf)?;
        Ok(Self {
            namespace,
            id,
            version,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSelectKnownPacks {
    known_packs: Vec<KnownPack>,
}

pub trait ClientConfigurationPacketListener {
    fn handle_select_known_packs(&mut self, packet: &ClientboundSelectKnownPacks);
}

impl ClientboundSelectKnownPacks {
    // JAVA: net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks#<init>
    pub fn new(known_packs: Vec<KnownPack>) -> Self {
        Self { known_packs }
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks#knownPacks
    pub fn known_packs(&self) -> &[KnownPack] {
        &self.known_packs
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks#type
    pub fn type_(&self) -> PacketType {
        CLIENTBOUND_SELECT_KNOWN_PACKS
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks#handle
    pub fn handle<L: ClientConfigurationPacketListener>(&self, listener: &mut L) {
        listener.handle_select_known_packs(self);
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks#STREAM_CODEC
    pub fn encode(&self, buf: &mut BytesMut) {
        write_varint(buf, self.known_packs.len() as i32);
        for pack in &self.known_packs {
            pack.encode(buf);
        }
    }

    // JAVA: net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks#STREAM_CODEC
    pub fn decode(buf: &mut &[u8]) -> Result<Self, CodecError> {
        let count = read_varint(buf)?;
        if count < 0 {
            return Err(CodecError::InvalidLength(count));
        }
        let count = count as usize;
        let mut known_packs = Vec::with_capacity(count);
        for _ in 0..count {
            known_packs.push(KnownPack::decode(buf)?);
        }
        Ok(Self { known_packs })
    }
}

fn write_varint(buf: &mut BytesMut, mut value: i32) {
    loop {
        let mut b = ((value as u32) & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            b |= 0x80;
        }
        buf.put_u8(b);
        if value == 0 {
            break;
        }
    }
}

fn read_varint(buf: &mut &[u8]) -> Result<i32, CodecError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(CodecError::UnexpectedEof);
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl(shift);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return Err(CodecError::InvalidVarInt);
        }
    }
}

fn write_utf(buf: &mut BytesMut, s: &str) {
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

fn read_utf(buf: &mut &[u8]) -> Result<String, CodecError> {
    let len = read_varint(buf)?;
    if len < 0 {
        return Err(CodecError::InvalidLength(len));
    }
    let len = len as usize;
    if buf.remaining() < len {
        return Err(CodecError::UnexpectedEof);
    }
    let bytes = &buf[..len];
    buf.advance(len);
    String::from_utf8(bytes.to_vec()).map_err(|_| CodecError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        received: Option<Vec<KnownPack>>,
    }

    impl ClientConfigurationPacketListener for MockListener {
        fn handle_select_known_packs(&mut self, packet: &ClientboundSelectKnownPacks) {
            self.received = Some(packet.known_packs().to_vec());
        }
    }

    #[test]
    fn type_is_select_known_packs() {
        let pkt = ClientboundSelectKnownPacks::new(vec![]);
        assert_eq!(pkt.type_(), CLIENTBOUND_SELECT_KNOWN_PACKS);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let packs = vec![KnownPack::new(
            "minecraft".into(),
            "core".into(),
            "1.21.11".into(),
        )];
        let pkt = ClientboundSelectKnownPacks::new(packs.clone());
        let mut listener = MockListener { received: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.received, Some(packs));
    }

    #[test]
    fn encode_decode_empty_list() {
        let pkt = ClientboundSelectKnownPacks::new(vec![]);
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSelectKnownPacks::decode(&mut slice).unwrap();
        assert_eq!(decoded, pkt);
        assert!(slice.is_empty());
    }

    #[test]
    fn encode_decode_multiple_packs() {
        let pkt = ClientboundSelectKnownPacks::new(vec![
            KnownPack::new("minecraft".into(), "core".into(), "1.21".into()),
            KnownPack::new("custom".into(), "pack".into(), "0.1.0".into()),
        ]);
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSelectKnownPacks::decode(&mut slice).unwrap();
        assert_eq!(decoded, pkt);
    }

    #[test]
    fn decode_negative_count_errors() {
        let data = [0xff, 0xff, 0xff, 0xff, 0x0f];
        let mut slice: &[u8] = &data;
        let err = ClientboundSelectKnownPacks::decode(&mut slice).unwrap_err();
        assert_eq!(err, CodecError::InvalidLength(-1));
    }

    #[test]
    fn encode_decode_unicode_pack_fields() {
        let pkt = ClientboundSelectKnownPacks::new(vec![KnownPack::new(
            "espace".into(),
            "identité".into(),
            "v1-β".into(),
        )]);
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSelectKnownPacks::decode(&mut slice).unwrap();
        assert_eq!(decoded.known_packs()[0].namespace(), "espace");
        assert_eq!(decoded.known_packs()[0].id(), "identité");
        assert_eq!(decoded.known_packs()[0].version(), "v1-β");
    }
}
